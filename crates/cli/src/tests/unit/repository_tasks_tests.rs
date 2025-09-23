//! Unit tests for TaskRepository
//! 
//! These tests create a temporary SQLite DB, seed minimal data
//! (projects, tasks) and verify queries from TaskRepository.

use crate::repository::{RepositoryManager, task_repository::TaskRepository};
use db::{open_or_create_db};
use rusqlite::params;

fn setup_temp_db() -> rusqlite::Connection {
    // Use a unique in-memory DB per test run
    let conn = rusqlite::Connection::open_in_memory().expect("open in memory");
    // Apply schema via db helper (migrations)
    // Note: open_or_create_db also applies migrations, but uses file path.
    // For in-memory, run the same migration logic manually by calling helpers indirectly:
    // We simulate by creating the schema similar to apply_v1 for required tables.
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT
        );
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            assignee_agent_id TEXT,
            created_at TEXT NOT NULL
        );
        "#,
    ).expect("create schema");
    conn
}

#[test]
fn test_task_repository_list_by_project_returns_rows() {
    let conn = setup_temp_db();

    // Seed one project
    conn.execute(
        "INSERT INTO projects (id, name, created_at) VALUES (?1, ?2, ?3)",
        params!["proj-1", "Test Project", "2025-01-17T10:00:00Z"],
    ).expect("insert project");

    // Seed a few tasks
    conn.execute(
        "INSERT INTO tasks (id, project_id, title, status, assignee_agent_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params!["task-1", "proj-1", "Setup Kanban", "todo", Option::<String>::None, "2025-01-17T10:00:01Z"],
    ).expect("insert task-1");
    conn.execute(
        "INSERT INTO tasks (id, project_id, title, status, assignee_agent_id, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params!["task-2", "proj-1", "Implement load", "doing", Some("agent-1".to_string()), "2025-01-17T10:00:02Z"],
    ).expect("insert task-2");

    let manager = RepositoryManager::new(conn);
    let rows = manager.tasks.list_by_project("proj-1").expect("list_by_project");

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].project_id, "proj-1");
    assert!(rows.iter().any(|r| r.id == "task-1"));
    assert!(rows.iter().any(|r| r.id == "task-2"));
}

#[test]
fn test_task_repository_list_by_project_empty_for_unknown_project() {
    let conn = setup_temp_db();
    // No seed for projects or tasks
    let manager = RepositoryManager::new(conn);
    let rows = manager.tasks.list_by_project("unknown").expect("list_by_project empty");
    assert!(rows.is_empty());
}


