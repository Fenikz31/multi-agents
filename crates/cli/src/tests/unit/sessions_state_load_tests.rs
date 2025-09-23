//! Unit tests for SessionsState loading from SQLite with filters

use crate::tui::state::view_state::SessionsState;
use db::open_or_create_db;
use rusqlite::params;
use std::fs;

fn temp_db_path() -> String {
    let pid = std::process::id();
    format!("/tmp/multi-agents-test-sessions-{}-{}.sqlite3", pid, rand::random::<u64>())
}

#[test]
fn test_sessions_state_load_from_db_and_filter() {
    let path = temp_db_path();
    let conn = open_or_create_db(&path).expect("open db");

    // seed minimal data: projects, agents, sessions
    conn.execute(
        "INSERT INTO projects (id, name, created_at) VALUES (?1, ?2, datetime('now'))",
        params!["proj-1", "Demo"],
    ).expect("insert project");
    conn.execute(
        "INSERT INTO agents (id, project_id, role, provider, model, allowed_tools_json, system_prompt, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))",
        params!["agent-1", "proj-1", "backend", "claude", "3.5", "[]", "prompt"],
    ).expect("insert agent");
    conn.execute(
        "INSERT INTO sessions (id, project_id, agent_id, provider, provider_session_id, created_at, last_activity, status, metadata, expires_at, type) VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'), ?6, NULL, NULL, ?7)",
        params!["sess-1", "proj-1", "agent-1", "claude", Option::<String>::None, "active", "repl"],
    ).expect("insert session");

    // Load with filter by project
    let mut state = SessionsState::new();
    state.load_from_db_with_filters(&path, Some("proj-1".into()), None).expect("load sessions");
    assert_eq!(state.sessions.len(), 1);

    // Apply agent filter string
    state.filter = "backend".into();
    let filtered = state.get_filtered_sessions();
    assert_eq!(filtered.len(), 1);

    // Cleanup temp db
    let _ = fs::remove_file(&path);
}


