//! Integration tests for supervisor subscription (M7-02)

use tempfile::TempDir;

#[test]
fn supervisor_subscription_smoke() {
    // Expect supervisor module provides basic subscription surface
    let mut mgr = crate::supervisor::manager::SupervisorManager::new();
    let sub = crate::supervisor::subscription::Subscription {
        project: "demo".to_string(),
        roles: vec!["backend".to_string()],
    };
    let id = mgr.subscribe(sub).expect("subscribe ok");
    assert!(!id.is_empty());
}


