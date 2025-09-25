//! Unit tests for supervisor module (M7-02)

#[cfg(test)]
mod tests {
    use super::*;
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
        let project = "demo";
        let role_a = "backend";
        let role_b = "frontend";
        let agent_a = "backend1";
        let agent_b = "frontend1";
        let provider = "claude";

        let _tmp = TempDir::new().unwrap();
        let _ = std::fs::create_dir_all(format!("./logs/{project}"));

        // Two routed for same broadcast, different roles
        let _ = crate::logging::ndjson::emit_routed_event(project, role_a, agent_a, provider, Some("b-1"), Some("m-1a"));
        let _ = crate::logging::ndjson::emit_routed_event(project, role_b, agent_b, provider, Some("b-1"), Some("m-1b"));
        // One routed for a different broadcast
        let _ = crate::logging::ndjson::emit_routed_event(project, role_a, agent_a, provider, Some("b-2"), Some("m-2a"));

        let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
        let lines = sub.aggregate_tail(vec![role_a.to_string(), role_b.to_string()], Some("routed".to_string()), 100).unwrap();
        let metrics = crate::supervisor::metrics::compute_routed_metrics(lines).expect("metrics");

        assert_eq!(metrics.total, 3, "total routed events should be 3");
        assert_eq!(metrics.per_role.get(role_a).cloned().unwrap_or(0), 2);
        assert_eq!(metrics.per_role.get(role_b).cloned().unwrap_or(0), 1);
        assert_eq!(metrics.unique_broadcasts, 2, "unique broadcast ids should be 2");
    }
}


