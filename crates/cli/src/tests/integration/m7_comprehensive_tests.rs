//! Comprehensive integration tests for M7 milestone (Routing and Supervisor)
//! Tests supervisor + routing + logging integration with performance validation

use tempfile::TempDir;
use std::time::{Duration, Instant};
use crate::logging::events::NdjsonEvent;

/// Test comprehensive M7 integration: supervisor + routing + logging
#[test]
fn m7_comprehensive_integration_supervisor_routing_logging() {
    let project = "m7_comprehensive";
    let provider = "claude";
    let roles = vec!["backend", "frontend", "devops"];
    let agents_per_role = 2;

    let _tmp = TempDir::new().unwrap();
    let _ = std::fs::create_dir_all(format!("./logs/{project}"));

    // 1. Generate routed events across multiple roles and agents
    let mut broadcast_ids = Vec::new();
    for i in 0..3 {
        let broadcast_id = format!("b-comp-{}", i);
        broadcast_ids.push(broadcast_id.clone());
        
        for role in &roles {
            for agent_idx in 0..agents_per_role {
                let agent = format!("{}{}", role, agent_idx + 1);
                let message_id = format!("m-{}-{}-{}", i, role, agent_idx);
                let _ = crate::logging::ndjson::emit_routed_event(
                    project, role, &agent, provider, Some(&broadcast_id), Some(&message_id)
                );
                std::thread::sleep(Duration::from_millis(2)); // Ensure different timestamps
            }
        }
    }

    // 2. Test supervisor subscription and aggregation
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    let event_lines = sub.aggregate_tail(
        roles.iter().map(|r| r.to_string()).collect(),
        Some("routed".to_string()),
        100
    ).expect("aggregation failed");

    // Convert lines to NdjsonEvent for metrics computation
    let events: Vec<NdjsonEvent> = event_lines.iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // 3. Validate comprehensive metrics
    let metrics = crate::supervisor::metrics::compute_routed_metrics_from_events(events).unwrap();
    
    // Expected: 3 broadcasts × 3 roles × 2 agents = 18 total events
    assert_eq!(metrics.total, 18, "Total routed events mismatch");
    assert_eq!(metrics.unique_broadcasts, 3, "Unique broadcast IDs mismatch");
    
    // Each role should have 6 events (3 broadcasts × 2 agents)
    for role in &roles {
        assert_eq!(
            *metrics.per_role.get(*role).unwrap_or(&0),
            6,
            "Events per role mismatch for {}",
            role
        );
    }

    // 4. Validate top roles (should be all equal, so order may vary)
    assert_eq!(metrics.top_roles.len(), 3, "Top roles count mismatch");
    for (role, count) in &metrics.top_roles {
        assert_eq!(*count, 6, "Top role count mismatch for {}", role);
    }

    // 5. Validate p95 latency per broadcast
    assert_eq!(metrics.p95_latency_per_broadcast.len(), 3, "P95 latency count mismatch");
    for broadcast_id in &broadcast_ids {
        assert!(metrics.p95_latency_per_broadcast.contains_key(broadcast_id), 
            "Missing p95 latency for broadcast {}", broadcast_id);
    }
}

/// Test M7 performance under multi-agent load
#[test]
fn m7_performance_multi_agent_load() {
    let project = "m7_perf";
    let provider = "claude";
    let roles = vec!["backend", "frontend", "devops", "supervisor"];
    let agents_per_role = 5; // 20 total agents
    let broadcasts_count = 10;

    let _tmp = TempDir::new().unwrap();
    let _ = std::fs::create_dir_all(format!("./logs/{project}"));

    let start = Instant::now();

    // Generate high load: 10 broadcasts × 4 roles × 5 agents = 200 events
    for i in 0..broadcasts_count {
        let broadcast_id = format!("b-perf-{}", i);
        
        for role in &roles {
            for agent_idx in 0..agents_per_role {
                let agent = format!("{}{}", role, agent_idx + 1);
                let message_id = format!("m-{}-{}-{}", i, role, agent_idx);
                let _ = crate::logging::ndjson::emit_routed_event(
                    project, role, &agent, provider, Some(&broadcast_id), Some(&message_id)
                );
            }
        }
    }

    let generation_time = start.elapsed();
    println!("Generated 200 events in {:?}", generation_time);

    // Test supervisor aggregation performance
    let agg_start = Instant::now();
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    let event_lines = sub.aggregate_tail(
        roles.iter().map(|r| r.to_string()).collect(),
        Some("routed".to_string()),
        200
    ).expect("aggregation failed");
    let agg_time = agg_start.elapsed();

    // Convert lines to NdjsonEvent for metrics computation
    let events: Vec<NdjsonEvent> = event_lines.iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // Test metrics computation performance
    let metrics_start = Instant::now();
    let metrics = crate::supervisor::metrics::compute_routed_metrics_from_events(events).unwrap();
    let metrics_time = metrics_start.elapsed();

    println!("Aggregation time: {:?}", agg_time);
    println!("Metrics computation time: {:?}", metrics_time);

    // Performance assertions
    assert!(generation_time < Duration::from_millis(1000), "Event generation too slow: {:?}", generation_time);
    assert!(agg_time < Duration::from_millis(500), "Aggregation too slow: {:?}", agg_time);
    assert!(metrics_time < Duration::from_millis(100), "Metrics computation too slow: {:?}", metrics_time);

    // Validate results
    assert_eq!(metrics.total, 200, "Total events mismatch");
    assert_eq!(metrics.unique_broadcasts, broadcasts_count, "Unique broadcasts mismatch");
    assert_eq!(metrics.per_role.len(), roles.len(), "Roles count mismatch");
}

/// Test M7 error handling and edge cases
#[test]
fn m7_error_handling_and_edge_cases() {
    let project = "m7_edge";
    let _tmp = TempDir::new().unwrap();

    // Test 1: Empty project directory
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new("nonexistent".to_string());
    let result = sub.aggregate_tail(vec!["backend".to_string()], Some("routed".to_string()), 100);
    assert!(result.is_err(), "Should error on nonexistent project");

    // Test 2: Empty roles list
    let _ = std::fs::create_dir_all(format!("./logs/{project}"));
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    let result = sub.aggregate_tail(vec![], Some("routed".to_string()), 100);
    assert!(result.is_ok(), "Empty roles should be OK");
    assert_eq!(result.unwrap().len(), 0, "Empty roles should return empty events");

    // Test 3: Invalid event filter
    let result = sub.aggregate_tail(vec!["backend".to_string()], Some("nonexistent".to_string()), 100);
    assert!(result.is_ok(), "Invalid filter should be OK");
    assert_eq!(result.unwrap().len(), 0, "Invalid filter should return empty events");

    // Test 4: Zero max_lines
    let result = sub.aggregate_tail(vec!["backend".to_string()], Some("routed".to_string()), 0);
    assert!(result.is_ok(), "Zero max_lines should be OK");
    assert_eq!(result.unwrap().len(), 0, "Zero max_lines should return empty events");
}

/// Test M7 supervisor manager edge cases
#[test]
fn m7_supervisor_manager_edge_cases() {
    let mut mgr = crate::supervisor::manager::SupervisorManager::new();

    // Test 1: Multiple subscriptions
    let sub1 = crate::supervisor::subscription::Subscription {
        project: "proj1".to_string(),
        roles: vec!["backend".to_string()],
    };
    let sub2 = crate::supervisor::subscription::Subscription {
        project: "proj2".to_string(),
        roles: vec!["frontend".to_string()],
    };

    let id1 = mgr.subscribe(sub1).expect("subscribe 1 ok");
    let id2 = mgr.subscribe(sub2).expect("subscribe 2 ok");
    assert_ne!(id1, id2, "Subscription IDs should be unique");

    // Test 2: Unsubscribe non-existent ID
    mgr.unsubscribe("nonexistent").expect("unsubscribe non-existent should be OK");

    // Test 3: Unsubscribe same ID multiple times
    mgr.unsubscribe(&id1).expect("unsubscribe 1 ok");
    mgr.unsubscribe(&id1).expect("unsubscribe 1 again should be OK");
    mgr.unsubscribe(&id2).expect("unsubscribe 2 ok");
}
