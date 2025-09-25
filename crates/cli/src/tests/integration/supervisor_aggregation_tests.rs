use tempfile::TempDir;

#[test]
fn supervisor_aggregate_tail_merges_roles_sorted_and_filtered() {
    let project = "demo";
    let role_backend = "backend";
    let role_frontend = "frontend";
    let agent_b = "backend1";
    let agent_f = "frontend1";
    let provider = "claude";

    let _tmp = TempDir::new().unwrap();
    let _ = std::fs::create_dir_all(format!("./logs/{project}"));

    // Write routed events in both roles
    let _ = crate::logging::ndjson::emit_routed_event(project, role_backend, agent_b, provider, Some("b-1"), Some("m-1"));
    std::thread::sleep(std::time::Duration::from_millis(5));
    let _ = crate::logging::ndjson::emit_routed_event(project, role_frontend, agent_f, provider, Some("b-2"), Some("m-2"));

    // Also write a stdout_line we expect to be filtered out
    let _ = crate::logging::ndjson::emit_stdout_line_event(project, role_backend, agent_b, provider, "noise");

    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    let lines = sub.aggregate_tail(vec![role_backend.to_string(), role_frontend.to_string()], Some("routed".to_string()), 100)
        .expect("aggregate_tail failed");

    assert!(lines.len() >= 2, "should contain at least 2 routed lines across roles");

    // Ensure sorted by ts ascending (RFC3339 string order works)
    let ts_of = |line: &str| -> String {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        v.get("ts").and_then(|t| t.as_str()).unwrap().to_string()
    };
    for i in 1..lines.len() {
        let prev = ts_of(&lines[i-1]);
        let curr = ts_of(&lines[i]);
        assert!(prev <= curr, "lines not sorted by ts: {} > {}", prev, curr);
    }

    // Ensure all are routed events
    for l in &lines {
        assert!(l.contains("\"event\":\"routed\""), "non-routed line leaked: {}", l);
    }
}


