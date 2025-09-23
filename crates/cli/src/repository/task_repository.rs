//! Task repository implementation
//! 
//! Provides read operations for Kanban tasks from SQLite.

use std::error::Error;
use std::sync::{Arc, Mutex};
use rusqlite::{Connection, params};

/// Simple Task DTO aligned with DB schema
#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: String,
    pub assignee_agent_id: Option<String>,
    pub created_at: String,
}

/// Repository to read tasks for a project
pub struct TaskRepository {
    conn: Arc<Mutex<Connection>>,
}

impl TaskRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self { Self { conn } }

    /// List tasks by project id ordered by created_at
    pub fn list_by_project(&self, project_id: &str) -> Result<Vec<TaskRow>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, title, status, assignee_agent_id, created_at
             FROM tasks WHERE project_id = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(TaskRow {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                status: row.get(3)?,
                assignee_agent_id: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        let mut tasks = Vec::new();
        for r in rows { tasks.push(r?); }
        Ok(tasks)
    }
}


