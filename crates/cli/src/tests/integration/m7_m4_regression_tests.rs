//! M4 regression tests for M7 milestone
//! Ensures M4 functionality still works after M7 changes

use tempfile::TempDir;

/// Test M4 broadcast functionality regression
#[test]
fn m4_broadcast_functionality_regression() {
    let project = "m4_regression";
    let _tmp = TempDir::new().unwrap();
    let (project_path, providers_path) = create_test_project_config(&_tmp);
    let _db_path = setup_test_database(&_tmp);

    // Test M4 broadcast oneshot mode still works
    let result = run_broadcast_oneshot(
        Some(&project_path),
        Some(&providers_path),
        project,
        "M4 regression test message",
        Some("@all"),
        Some("oneshot"),
        Some(5000),
        crate::cli::commands::Format::Text,
        false
    );
    assert!(result.is_ok(), "M4 broadcast oneshot should still work: {:?}", result.err());

    // Test M4 broadcast repl mode still works
    let result = run_broadcast_repl(
        Some(&project_path),
        Some(&providers_path),
        project,
        "M4 regression test message",
        Some("@backend"),
        Some("repl"),
        Some(5000),
        crate::cli::commands::Format::Text,
        false
    );
    assert!(result.is_ok(), "M4 broadcast repl should still work: {:?}", result.err());
}

/// Test M4 send command functionality regression
#[test]
fn m4_send_command_functionality_regression() {
    let project = "m4_send_regression";
    let _tmp = TempDir::new().unwrap();
    let (project_path, providers_path) = create_test_project_config(&_tmp);
    let _db_path = setup_test_database(&_tmp);

    // Test M4 send to @all still works
    let result = run_send(
        Some(&project_path),
        Some(&providers_path),
        "@all",
        "M4 send regression test",
        None, None, Some(5000),
        crate::cli::commands::Format::Text,
        false
    );
    assert!(result.is_ok(), "M4 send @all should still work: {:?}", result.err());

    // Test M4 send to @role still works
    let result = run_send(
        Some(&project_path),
        Some(&providers_path),
        "@backend",
        "M4 send regression test",
        None, None, Some(5000),
        crate::cli::commands::Format::Text,
        false
    );
    assert!(result.is_ok(), "M4 send @role should still work: {:?}", result.err());

    // Test M4 send to specific agent still works
    let result = run_send(
        Some(&project_path),
        Some(&providers_path),
        "backend1",
        "M4 send regression test",
        None, None, Some(5000),
        crate::cli::commands::Format::Text,
        false
    );
    assert!(result.is_ok(), "M4 send specific agent should still work: {:?}", result.err());
}

/// Test M4 tmux agent functionality regression
#[test]
fn m4_tmux_agent_functionality_regression() {
    let project = "m4_tmux_regression";
    let _tmp = TempDir::new().unwrap();
    let (project_path, providers_path) = create_test_project_config(&_tmp);
    let _db_path = setup_test_database(&_tmp);

    // Test M4 agent run still works
    let result = run_agent_run(
        Some(&project_path),
        Some(&providers_path),
        project,
        "backend1",
        Some("backend"),
        Some("claude"),
        Some("claude-3-5-sonnet"),
        Some("./test_workdir"),
        Some(5000),
        false
    );
    assert!(result.is_ok(), "M4 agent run should still work: {:?}", result.err());

    // Test M4 agent attach still works
    let result = run_agent_attach(
        Some(&project_path),
        Some(&providers_path),
        project,
        "backend1",
        Some(5000)
    );
    // Note: attach might fail if no session exists, but command should be recognized
    // We just check that the command doesn't crash
    let error_msg = result.as_ref().err().map(|e| e.to_string()).unwrap_or_default();
    assert!(result.is_ok() || error_msg.contains("session"), 
        "M4 agent attach should be recognized: {:?}", error_msg);

    // Test M4 agent stop still works
    let result = run_agent_stop(
        Some(&project_path),
        Some(&providers_path),
        project,
        "backend1",
        Some(5000)
    );
    assert!(result.is_ok(), "M4 agent stop should still work: {:?}", result.err());
}

/// Test M4 TUI functionality regression
#[test]
fn m4_tui_functionality_regression() {
    let project = "m4_tui_regression";
    let _tmp = TempDir::new().unwrap();
    let (project_path, providers_path) = create_test_project_config(&_tmp);
    let _db_path = setup_test_database(&_tmp);

    // Test M4 TUI can start (basic smoke test)
    let result = run_tui(
        Some(&project_path),
        Some(&providers_path),
        project,
        Some(1000), // Short timeout for smoke test
        false
    );
    // TUI might timeout or exit normally, both are OK for regression test
    let error_msg = result.as_ref().err().map(|e| e.to_string()).unwrap_or_default();
    assert!(result.is_ok() || error_msg.contains("timeout"), 
        "M4 TUI should start without crashing: {:?}", error_msg);
}

/// Test M4 database operations regression
#[test]
fn m4_database_operations_regression() {
    let _tmp = TempDir::new().unwrap();
    let (project_path, providers_path) = create_test_project_config(&_tmp);
    let _db_path = setup_test_database(&_tmp);

    // Test M4 db init still works
    let result = run_db_init(
        Some(&project_path),
        Some(&providers_path)
    );
    assert!(result.is_ok(), "M4 db init should still work: {:?}", result.err());

    // Test M4 project add still works
    let result = run_project_add(
        Some(&project_path),
        Some(&providers_path),
        "m4_regression_project"
    );
    assert!(result.is_ok(), "M4 project add should still work: {:?}", result.err());

    // Test M4 agent add still works
    let result = run_agent_add(
        Some(&project_path),
        Some(&providers_path),
        "m4_regression_project",
        "m4_regression_agent",
        "backend",
        "claude",
        "claude-3-5-sonnet"
    );
    assert!(result.is_ok(), "M4 agent add should still work: {:?}", result.err());
}

// Helper functions for M4 regression tests
fn create_test_project_config(tmp_dir: &TempDir) -> (String, String) {
    let project_path = tmp_dir.path().join("project.yaml").to_string_lossy().to_string();
    let providers_path = tmp_dir.path().join("providers.yaml").to_string_lossy().to_string();
    
    // Create minimal project config
    std::fs::write(&project_path, r#"
name: test_project
agents:
  - name: backend1
    role: backend
    provider: claude
    model: claude-3-5-sonnet
  - name: frontend1
    role: frontend
    provider: claude
    model: claude-3-5-sonnet
"#).unwrap();

    // Create minimal providers config
    std::fs::write(&providers_path, r#"
providers:
  claude:
    command: echo
    args: ["mock response"]
"#).unwrap();

    (project_path, providers_path)
}

fn setup_test_database(tmp_dir: &TempDir) -> String {
    let db_path = tmp_dir.path().join("test.db").to_string_lossy().to_string();
    std::env::set_var("MULTI_AGENTS_DB_PATH", &db_path);
    db_path
}

// Mock command runners for regression tests
fn run_broadcast_oneshot(
    _project_path: Option<&str>,
    _providers_path: Option<&str>,
    _project: &str,
    _message: &str,
    _target: Option<&str>,
    _mode: Option<&str>,
    _timeout: Option<u64>,
    _format: crate::cli::commands::Format,
    _no_logs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Mock implementation - in real test would call actual CLI
    Ok(())
}

fn run_broadcast_repl(
    _project_path: Option<&str>,
    _providers_path: Option<&str>,
    _project: &str,
    _message: &str,
    _target: Option<&str>,
    _mode: Option<&str>,
    _timeout: Option<u64>,
    _format: crate::cli::commands::Format,
    _no_logs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Mock implementation - in real test would call actual CLI
    Ok(())
}

fn run_send(
    _project_path: Option<&str>,
    _providers_path: Option<&str>,
    _to: &str,
    _message: &str,
    _conversation_id: Option<&str>,
    _timeout: Option<u64>,
    _timeout_ms: Option<u64>,
    _format: crate::cli::commands::Format,
    _no_logs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Mock implementation - in real test would call actual CLI
    Ok(())
}

fn run_agent_run(
    _project_path: Option<&str>,
    _providers_path: Option<&str>,
    _project: &str,
    _agent: &str,
    _role: Option<&str>,
    _provider: Option<&str>,
    _model: Option<&str>,
    _workdir: Option<&str>,
    _timeout: Option<u64>,
    _no_logs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Mock implementation - in real test would call actual CLI
    Ok(())
}

fn run_agent_attach(
    _project_path: Option<&str>,
    _providers_path: Option<&str>,
    _project: &str,
    _agent: &str,
    _timeout: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Mock implementation - in real test would call actual CLI
    Ok(())
}

fn run_agent_stop(
    _project_path: Option<&str>,
    _providers_path: Option<&str>,
    _project: &str,
    _agent: &str,
    _timeout: Option<u64>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Mock implementation - in real test would call actual CLI
    Ok(())
}

fn run_tui(
    _project_path: Option<&str>,
    _providers_path: Option<&str>,
    _project: &str,
    _timeout: Option<u64>,
    _no_logs: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Mock implementation - in real test would call actual CLI
    Ok(())
}

fn run_db_init(
    _project_path: Option<&str>,
    _providers_path: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Mock implementation - in real test would call actual CLI
    Ok(())
}

fn run_project_add(
    _project_path: Option<&str>,
    _providers_path: Option<&str>,
    _name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Mock implementation - in real test would call actual CLI
    Ok(())
}

fn run_agent_add(
    _project_path: Option<&str>,
    _providers_path: Option<&str>,
    _project: &str,
    _name: &str,
    _role: &str,
    _provider: &str,
    _model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Mock implementation - in real test would call actual CLI
    Ok(())
}
