use tempfile::TempDir;

#[test]
fn broadcast_to_multiple_roles_is_summarized_by_supervisor() {
    let project = "demo";
    let provider = "claude";
    let role_a = "backend";
    let role_b = "frontend";
    let agent_a = "backend1";
    let agent_b = "frontend1";

    let _tmp = TempDir::new().unwrap();
    let _ = std::fs::create_dir_all(format!("./logs/{project}"));

    // Simule un broadcast: deux events routed avec même broadcast_id
    let bid = "b-integ";
    let _ = crate::logging::ndjson::emit_routed_event(project, role_a, agent_a, provider, Some(bid), Some("m-1"));
    std::thread::sleep(std::time::Duration::from_millis(5));
    let _ = crate::logging::ndjson::emit_routed_event(project, role_b, agent_b, provider, Some(bid), Some("m-2"));

    // Supervisor agrège sur les rôles
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    let event_lines = sub.aggregate_tail(vec![role_a.to_string(), role_b.to_string()], Some("routed".to_string()), 100).unwrap();
    
    // Convert lines to NdjsonEvent for summary computation
    let events: Vec<crate::logging::events::NdjsonEvent> = event_lines.iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    
    let summary = crate::supervisor::metrics::compute_routed_metrics_from_events(events).expect("summary");

    assert!(summary.total >= 2, "Should have at least 2 routed events, got {}", summary.total);
    assert!(summary.per_role.get(role_a).cloned().unwrap_or(0) >= 1);
    assert!(summary.per_role.get(role_b).cloned().unwrap_or(0) >= 1);
    assert!(summary.unique_broadcasts >= 1);
}


