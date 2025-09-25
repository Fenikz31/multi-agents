//! Unit tests for supervisor module (M7-02)

#[cfg(test)]
mod tests {
    use super::*;

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
}


