//! Integration tests for supervisor subscription (M7-02)


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

#[test]
fn supervisor_subscription_detects_routed_event() {
    // Use isolated temp directory for this test
    let tmp = tempfile::tempdir().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    
    let project = "demo";
    let role = "backend";
    let agent = "backend1";
    let provider = "claude";
    
    // Create logs directory structure
    let _ = std::fs::create_dir_all(format!("./logs/{project}"));
    
    // Emit routed event
    let _ = crate::logging::ndjson::emit_routed_event(
        project, role, agent, provider, Some("b-xyz"), Some("m-abc")
    );

    // Test subscription
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    let events = sub.tail_and_filter(role.to_string(), Some("routed".to_string()), 100).expect("subscription failed");
    assert!(events.iter().any(|line| line.contains("\"event\":\"routed\"")));
    assert!(events.iter().any(|line| line.contains("b-xyz")));
    assert!(events.iter().any(|line| line.contains("m-abc")));
    
    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
}


