//! Unit tests for supervisor module (M7-02)

#[cfg(test)]
mod tests {
    
    use tempfile::TempDir;

    #[test]
    fn supervisor_manager_can_be_created() {
        // API expectation: SupervisorManager::new() exists
        let _mgr = crate::supervisor::manager::SupervisorManager::new();
    }

    #[test]
    fn supervisor_can_subscribe_and_unsubscribe() {
        // API expectation: subscribe/unsubscribe return Result and are idempotent
        let mut mgr = crate::supervisor::manager::SupervisorManager::new();
        let sub = crate::supervisor::subscription::Subscription {
            project: "demo".to_string(),
            roles: vec!["backend".to_string(), "frontend".to_string()],
        };

        let id = mgr.subscribe(sub).expect("subscribe ok");
        assert!(!id.is_empty());

        // Unsubscribe should succeed
        mgr.unsubscribe(&id).expect("unsubscribe ok");

        // Unsubscribe again should be Ok (idempotent)
        mgr.unsubscribe(&id).expect("unsubscribe idempotent ok");
    }

    #[test]
    fn compute_routed_metrics_counts_total_per_role_and_unique_broadcasts() {
        let tmp_dir = TempDir::new().unwrap();
        let project = "test-metrics-demo";
        let role_a = "backend";
        let role_b = "frontend";
        let agent_a = "backend1";
        let agent_b = "frontend1";
        let provider = "claude";

        // Use isolated temp directory for logs
        let logs_dir = tmp_dir.path().join("logs").join(project);
        std::fs::create_dir_all(&logs_dir).unwrap();
        
        // Change to temp directory for this test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        // Two routed for same broadcast, different roles
        let _ = crate::logging::ndjson::emit_routed_event(project, role_a, agent_a, provider, Some("b-1"), Some("m-1a"));
        let _ = crate::logging::ndjson::emit_routed_event(project, role_b, agent_b, provider, Some("b-1"), Some("m-1b"));
        // One routed for a different broadcast
        let _ = crate::logging::ndjson::emit_routed_event(project, role_a, agent_a, provider, Some("b-2"), Some("m-2a"));

        let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
        let lines = sub.aggregate_tail(vec![role_a.to_string(), role_b.to_string()], Some("routed".to_string()), 100).unwrap();
        let metrics = crate::supervisor::metrics::compute_routed_metrics(lines).expect("metrics");

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        assert_eq!(metrics.total, 3, "total routed events should be 3");
        assert_eq!(metrics.per_role.get(role_a).cloned().unwrap_or(0), 2);
        assert_eq!(metrics.per_role.get(role_b).cloned().unwrap_or(0), 1);
        assert_eq!(metrics.unique_broadcasts, 2, "unique broadcast ids should be 2");
    }

    #[test]
    fn compute_routed_metrics_provides_p95_latency_per_broadcast_and_top_roles() {
        let tmp_dir = TempDir::new().unwrap();
        let project = "test-latency-demo";
        let role_a = "backend";
        let role_b = "frontend";
        let agent_a = "backend1";
        let agent_b = "frontend1";
        let provider = "claude";

        // Use isolated temp directory for logs
        let logs_dir = tmp_dir.path().join("logs").join(project);
        std::fs::create_dir_all(&logs_dir).unwrap();
        
        // Change to temp directory for this test
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp_dir.path()).unwrap();

        // Same broadcast b-1 with two routed events separated by a small delay
        let _ = crate::logging::ndjson::emit_routed_event(project, role_a, agent_a, provider, Some("b-1"), Some("m-1a"));
        std::thread::sleep(std::time::Duration::from_millis(5));
        let _ = crate::logging::ndjson::emit_routed_event(project, role_b, agent_b, provider, Some("b-1"), Some("m-1b"));

        // Another broadcast with single event (latency ~0)
        let _ = crate::logging::ndjson::emit_routed_event(project, role_a, agent_a, provider, Some("b-2"), Some("m-2a"));

        let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
        let lines = sub.aggregate_tail(vec![role_a.to_string(), role_b.to_string()], Some("routed".to_string()), 100).unwrap();
        let metrics = crate::supervisor::metrics::compute_routed_metrics(lines).expect("metrics");

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        // p95 latency per broadcast should exist and be non-zero for b-1
        let p95 = metrics.p95_latency_per_broadcast.get("b-1").cloned().unwrap_or(0);
        assert!(p95 >= 1, "expected non-zero latency for broadcast b-1");
        // b-2 has single event; latency should be 0
        let p95_b2 = metrics.p95_latency_per_broadcast.get("b-2").cloned().unwrap_or(999);
        assert_eq!(p95_b2, 0, "expected zero latency for single-event broadcast b-2");

        // top roles should list backend above frontend since backend has 2 events vs 1
        let top_roles = &metrics.top_roles;
        assert!(!top_roles.is_empty());
        assert_eq!(top_roles[0].0, role_a);
        assert!(top_roles[0].1 >= 1);
    }
}


