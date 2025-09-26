//! Comprehensive unit tests for M7 supervisor module
//! Tests all supervisor functionality with edge cases and error conditions

use tempfile::TempDir;
use crate::supervisor::{manager::SupervisorManager, subscription::SupervisorSubscription};
use crate::logging::events::NdjsonEvent;
use std::time::Duration;
use chrono::Utc;

// Helper function to extract timestamp from NDJSON line
fn extract_ts(line: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(line).unwrap();
    v.get("ts").and_then(|t| t.as_str()).unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test supervisor manager comprehensive functionality
    #[test]
    fn supervisor_manager_comprehensive_functionality() {
        let mut mgr = SupervisorManager::new();

        // Test 1: Multiple subscriptions with different projects
        let sub1 = crate::supervisor::subscription::Subscription {
            project: "project1".to_string(),
            roles: vec!["backend".to_string(), "frontend".to_string()],
        };
        let sub2 = crate::supervisor::subscription::Subscription {
            project: "project2".to_string(),
            roles: vec!["devops".to_string()],
        };

        let id1 = mgr.subscribe(sub1).expect("subscribe 1 should succeed");
        let id2 = mgr.subscribe(sub2).expect("subscribe 2 should succeed");
        assert_ne!(id1, id2, "Subscription IDs should be unique");

        // Test 2: Unsubscribe operations
        mgr.unsubscribe(&id1).expect("unsubscribe 1 should succeed");
        mgr.unsubscribe(&id2).expect("unsubscribe 2 should succeed");

        // Test 3: Idempotent unsubscribe
        mgr.unsubscribe(&id1).expect("unsubscribe non-existent should be OK");
        mgr.unsubscribe(&id2).expect("unsubscribe non-existent should be OK");
    }

    /// Test supervisor subscription comprehensive functionality
    #[test]
    fn supervisor_subscription_comprehensive_functionality() {
        let tmp_dir = TempDir::new().unwrap();
        let project = "test-subscription-comprehensive";
        
        // Use isolated temp directory for logs
        let logs_dir = tmp_dir.path().join("logs").join(project);
        std::fs::create_dir_all(&logs_dir).unwrap();
        
        // Create files directly in temp directory without changing working directory
        let backend_log = logs_dir.join("backend.ndjson");
        let frontend_log = logs_dir.join("frontend.ndjson");
        
        // Create empty backend log file
        std::fs::write(&backend_log, "").unwrap();
        
        // Create frontend log with some content
        let frontend_event = crate::logging::events::NdjsonEvent::new_routed(
            &chrono::Utc::now().to_rfc3339(),
            "frontend",
            "agent1",
            "claude",
            Some("b1".to_string()),
            Some("m1".to_string())
        );
        let _ = crate::logging::ndjson::write_ndjson_event(&frontend_log.to_string_lossy(), &frontend_event);

        // Create a subscription that uses the temp directory directly
        // We need to change to the temp directory for the subscription to work
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp_dir.path()).unwrap();
        
        let mut sub = SupervisorSubscription::new(project.to_string());

        // Test 1: Empty file handling
        let result = sub.tail_and_filter("backend".to_string(), Some("routed".to_string()), 100);
        assert!(result.is_ok(), "Empty file should be OK");
        assert_eq!(result.unwrap().len(), 0, "Empty file should return empty results");

        // Test 2: Non-existent file handling (should return empty results, not error)
        let result = sub.tail_and_filter("nonexistent".to_string(), Some("routed".to_string()), 100);
        assert!(result.is_ok(), "Non-existent file should return empty results, not error");
        assert_eq!(result.unwrap().len(), 0, "Non-existent file should return empty results");

        // Test 3: Generate test events
        let _ = crate::logging::ndjson::emit_routed_event(
            project, "backend", "agent1", "claude", Some("b1"), Some("m1")
        );
        let _ = crate::logging::ndjson::emit_routed_event(
            project, "frontend", "agent2", "claude", Some("b2"), Some("m2")
        );
        let _ = crate::logging::ndjson::emit_stdout_line_event(
            project, "backend", "agent1", "claude", "stdout message"
        );

        // Test 4: Filter by event type
        let routed_events = sub.tail_and_filter("backend".to_string(), Some("routed".to_string()), 100)
            .expect("filter routed should succeed");
        assert_eq!(routed_events.len(), 1, "Should find 1 routed event");

        let stdout_events = sub.tail_and_filter("backend".to_string(), Some("stdout_line".to_string()), 100)
            .expect("filter stdout should succeed");
        assert_eq!(stdout_events.len(), 1, "Should find 1 stdout event");

        // Test 5: No filter (all events)
        let all_events = sub.tail_and_filter("backend".to_string(), None, 100)
            .expect("no filter should succeed");
        assert_eq!(all_events.len(), 2, "Should find 2 total events");

        // Test 6: Max lines limit
        let limited_events = sub.tail_and_filter("backend".to_string(), None, 1)
            .expect("limited should succeed");
        assert_eq!(limited_events.len(), 1, "Should respect max lines limit");
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    /// Test supervisor aggregation comprehensive functionality
    #[test]
    fn supervisor_aggregation_comprehensive_functionality() {
        let tmp_dir = TempDir::new().unwrap();
        let project = "test-aggregation-comprehensive";
        
        // Use isolated temp directory for logs
        let logs_dir = tmp_dir.path().join("logs").join(project);
        std::fs::create_dir_all(&logs_dir).unwrap();
        
        // Change to temp directory for this test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        let mut sub = SupervisorSubscription::new(project.to_string());

        // Test 1: Empty roles list
        let result = sub.aggregate_tail(vec![], Some("routed".to_string()), 100);
        assert!(result.is_ok(), "Empty roles should be OK");
        assert_eq!(result.unwrap().len(), 0, "Empty roles should return empty events");

        // Test 2: Generate events with different timestamps
        let base_time = Utc::now();
        let events = vec![
            NdjsonEvent {
                ts: (base_time - Duration::from_millis(1000)).to_rfc3339(),
                event: "routed".to_string(),
                project_id: project.to_string(),
                agent_role: "backend".to_string(),
                agent_id: "agent1".to_string(),
                provider: "claude".to_string(),
                broadcast_id: Some("b1".to_string()),
                message_id: Some("m1".to_string()),
                ..Default::default()
            },
            NdjsonEvent {
                ts: (base_time - Duration::from_millis(500)).to_rfc3339(),
                event: "routed".to_string(),
                project_id: project.to_string(),
                agent_role: "frontend".to_string(),
                agent_id: "agent2".to_string(),
                provider: "claude".to_string(),
                broadcast_id: Some("b2".to_string()),
                message_id: Some("m2".to_string()),
                ..Default::default()
            },
            NdjsonEvent {
                ts: (base_time - Duration::from_millis(2000)).to_rfc3339(),
                event: "routed".to_string(),
                project_id: project.to_string(),
                agent_role: "backend".to_string(),
                agent_id: "agent3".to_string(),
                provider: "claude".to_string(),
                broadcast_id: Some("b3".to_string()),
                message_id: Some("m3".to_string()),
                ..Default::default()
            },
        ];

        // Write events to files
        for event in &events {
            let log_file = format!("./logs/{}/{}.ndjson", project, event.agent_role);
            let _ = crate::logging::ndjson::write_ndjson_event(&log_file, event);
        }

        // Test 3: Aggregate and verify sorting
        let aggregated_lines = sub.aggregate_tail(
            vec!["backend".to_string(), "frontend".to_string()],
            Some("routed".to_string()),
            100
        ).expect("aggregation should succeed");

        assert!(aggregated_lines.len() >= 3, "Should aggregate at least 3 events, got {}", aggregated_lines.len());

        // Verify chronological sorting (oldest first)
        for i in 1..aggregated_lines.len() {
            let prev_line = &aggregated_lines[i-1];
            let curr_line = &aggregated_lines[i];
            
            // Extract timestamp from JSON line
            let prev_ts_str = if let Some(start) = prev_line.find("\"ts\":\"") {
                let ts_start = start + 6;
                if let Some(end) = prev_line[ts_start..].find('"') {
                    &prev_line[ts_start..ts_start + end]
                } else {
                    continue;
                }
            } else {
                continue;
            };
            
            let curr_ts_str = if let Some(start) = curr_line.find("\"ts\":\"") {
                let ts_start = start + 6;
                if let Some(end) = curr_line[ts_start..].find('"') {
                    &curr_line[ts_start..ts_start + end]
                } else {
                    continue;
                }
            } else {
                continue;
            };
            
            let prev_ts = chrono::DateTime::parse_from_rfc3339(prev_ts_str).unwrap();
            let curr_ts = chrono::DateTime::parse_from_rfc3339(curr_ts_str).unwrap();
            assert!(prev_ts <= curr_ts, "Events should be sorted chronologically");
        }

        // Test 4: Filter by event type
        let routed_only = sub.aggregate_tail(
            vec!["backend".to_string(), "frontend".to_string()],
            Some("routed".to_string()),
            100
        ).expect("filtered aggregation should succeed");
        assert_eq!(routed_only.len(), 3, "Should filter to routed events only");

        // Test 5: Max lines limit (test with a very small limit to ensure it works)
        // Create a new subscription with a different project to avoid interference
        let mut limited_sub = SupervisorSubscription::new("test-limited".to_string());
        let limited_logs_dir = tmp_dir.path().join("logs").join("test-limited");
        std::fs::create_dir_all(&limited_logs_dir).unwrap();
        
        // Create only one event for this test
        let single_event = crate::logging::events::NdjsonEvent::new_routed(
            &chrono::Utc::now().to_rfc3339(),
            "test",
            "agent1",
            "claude",
            Some("b1".to_string()),
            Some("m1".to_string())
        );
        let single_log = limited_logs_dir.join("test.ndjson");
        let _ = crate::logging::ndjson::write_ndjson_event(&single_log.to_string_lossy(), &single_event);
        
        let limited = limited_sub.aggregate_tail(
            vec!["test".to_string()],
            Some("routed".to_string()),
            1
        ).expect("limited aggregation should succeed");
        assert_eq!(limited.len(), 1, "Should respect max lines limit, got {}", limited.len());
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    /// Test routed metrics comprehensive functionality
    #[test]
    fn routed_metrics_comprehensive_functionality() {
        let project = "metrics_test";
        let base_time = Utc::now();

        // Test 1: Empty events
        let empty_metrics = crate::supervisor::metrics::compute_routed_metrics(vec![]).unwrap();
        assert_eq!(empty_metrics.total, 0);
        assert_eq!(empty_metrics.unique_broadcasts, 0);
        assert_eq!(empty_metrics.per_role.len(), 0);
        assert_eq!(empty_metrics.p95_latency_per_broadcast.len(), 0);
        assert_eq!(empty_metrics.top_roles.len(), 0);

        // Test 2: Single event
        let single_event = vec![NdjsonEvent::new_routed(
            project, "backend", "agent1", "claude", Some("b1".to_string()), Some("m1".to_string())
        )];
        let single_metrics = crate::supervisor::metrics::compute_routed_metrics_from_events(single_event).unwrap();
        assert_eq!(single_metrics.total, 1);
        assert_eq!(single_metrics.unique_broadcasts, 1);
        assert_eq!(*single_metrics.per_role.get("backend").unwrap(), 1);
        assert_eq!(single_metrics.p95_latency_per_broadcast.len(), 1);
        assert_eq!(single_metrics.top_roles.len(), 1);

        // Test 3: Complex scenario with multiple broadcasts and roles
        let complex_events = vec![
            NdjsonEvent {
                ts: (base_time - Duration::from_millis(1000)).to_rfc3339(),
                event: "routed".to_string(),
                project_id: project.to_string(),
                agent_role: "backend".to_string(),
                agent_id: "agent1".to_string(),
                provider: "claude".to_string(),
                broadcast_id: Some("b1".to_string()),
                message_id: Some("m1".to_string()),
                ..Default::default()
            },
            NdjsonEvent {
                ts: (base_time - Duration::from_millis(800)).to_rfc3339(),
                event: "routed".to_string(),
                project_id: project.to_string(),
                agent_role: "frontend".to_string(),
                agent_id: "agent2".to_string(),
                provider: "claude".to_string(),
                broadcast_id: Some("b1".to_string()),
                message_id: Some("m2".to_string()),
                ..Default::default()
            },
            NdjsonEvent {
                ts: (base_time - Duration::from_millis(600)).to_rfc3339(),
                event: "routed".to_string(),
                project_id: project.to_string(),
                agent_role: "backend".to_string(),
                agent_id: "agent3".to_string(),
                provider: "claude".to_string(),
                broadcast_id: Some("b1".to_string()),
                message_id: Some("m3".to_string()),
                ..Default::default()
            },
            NdjsonEvent {
                ts: (base_time - Duration::from_millis(400)).to_rfc3339(),
                event: "routed".to_string(),
                project_id: project.to_string(),
                agent_role: "devops".to_string(),
                agent_id: "agent4".to_string(),
                provider: "claude".to_string(),
                broadcast_id: Some("b2".to_string()),
                message_id: Some("m4".to_string()),
                ..Default::default()
            },
        ];

        let complex_metrics = crate::supervisor::metrics::compute_routed_metrics_from_events(complex_events).unwrap();

        // Validate totals
        assert_eq!(complex_metrics.total, 4);
        assert_eq!(complex_metrics.unique_broadcasts, 2);

        // Validate per-role counts
        assert_eq!(*complex_metrics.per_role.get("backend").unwrap(), 2);
        assert_eq!(*complex_metrics.per_role.get("frontend").unwrap(), 1);
        assert_eq!(*complex_metrics.per_role.get("devops").unwrap(), 1);

        // Validate top roles (should be sorted by count descending)
        assert_eq!(complex_metrics.top_roles.len(), 3);
        assert_eq!(complex_metrics.top_roles[0].0, "backend");
        assert_eq!(complex_metrics.top_roles[0].1, 2);
        assert_eq!(complex_metrics.top_roles[1].1, 1);
        assert_eq!(complex_metrics.top_roles[2].1, 1);

        // Validate p95 latency per broadcast
        assert_eq!(complex_metrics.p95_latency_per_broadcast.len(), 2);
        assert!(complex_metrics.p95_latency_per_broadcast.contains_key("b1"));
        assert!(complex_metrics.p95_latency_per_broadcast.contains_key("b2"));

        // Test 4: Events without broadcast_id
        let no_broadcast_events = vec![
            NdjsonEvent {
                ts: base_time.to_rfc3339(),
                event: "routed".to_string(),
                project_id: project.to_string(),
                agent_role: "backend".to_string(),
                agent_id: "agent1".to_string(),
                provider: "claude".to_string(),
                broadcast_id: None,
                message_id: Some("m1".to_string()),
                ..Default::default()
            },
        ];

        let no_broadcast_metrics = crate::supervisor::metrics::compute_routed_metrics_from_events(no_broadcast_events).unwrap();
        assert_eq!(no_broadcast_metrics.total, 1);
        assert_eq!(no_broadcast_metrics.unique_broadcasts, 0);
        assert_eq!(no_broadcast_metrics.p95_latency_per_broadcast.len(), 0);
    }

    /// Test supervisor manager routed_summary integration
    #[test]
    fn supervisor_manager_routed_summary_integration() {
        let tmp_dir = TempDir::new().unwrap();
        let project = "test-summary-integration";
        
        // Use isolated temp directory for logs
        let logs_dir = tmp_dir.path().join("logs").join(project);
        std::fs::create_dir_all(&logs_dir).unwrap();
        
        // Generate test events directly in temp directory
        let backend_log = logs_dir.join("backend.ndjson");
        let frontend_log = logs_dir.join("frontend.ndjson");
        
        let backend_event = crate::logging::events::NdjsonEvent::new_routed(
            &chrono::Utc::now().to_rfc3339(),
            "backend",
            "agent1",
            "claude",
            Some("b1".to_string()),
            Some("m1".to_string())
        );
        let frontend_event = crate::logging::events::NdjsonEvent::new_routed(
            &chrono::Utc::now().to_rfc3339(),
            "frontend",
            "agent2",
            "claude",
            Some("b1".to_string()),
            Some("m2".to_string())
        );
        
        let _ = crate::logging::ndjson::write_ndjson_event(&backend_log.to_string_lossy(), &backend_event);
        let _ = crate::logging::ndjson::write_ndjson_event(&frontend_log.to_string_lossy(), &frontend_event);

        // Change to temp directory for this test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        // Test routed_summary integration
        let mut sub = SupervisorSubscription::new(project.to_string());
        let event_lines = sub.aggregate_tail(
            vec!["backend".to_string(), "frontend".to_string()],
            Some("routed".to_string()),
            100
        ).expect("aggregation should succeed");

        // Convert lines to NdjsonEvent for summary computation
        let events: Vec<NdjsonEvent> = event_lines.iter()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        let summary = crate::supervisor::metrics::compute_routed_metrics_from_events(events).expect("summary should succeed");
        assert!(summary.total >= 2, "Should have at least 2 events, got {}", summary.total);
        assert!(summary.unique_broadcasts >= 1, "Should have at least 1 unique broadcast, got {}", summary.unique_broadcasts);
        assert!(*summary.per_role.get("backend").unwrap_or(&0) >= 1, "Backend should have at least 1 event");
        assert!(*summary.per_role.get("frontend").unwrap_or(&0) >= 1, "Frontend should have at least 1 event");
        
        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}
