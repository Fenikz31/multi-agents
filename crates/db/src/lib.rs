use rusqlite::{Connection, params};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub fn now_iso8601_utc() -> String {
    let now = time::OffsetDateTime::now_utc();
    now.format(&time::format_description::well_known::Rfc3339).unwrap()
}

pub fn open_or_create_db(path: &str) -> Result<Connection, DbError> {
    let db_path = std::path::Path::new(path);
    if let Some(parent) = db_path.parent() { std::fs::create_dir_all(parent)?; }
    let conn = Connection::open(db_path)?;
    // PRAGMAs
    conn.pragma_update(None, "foreign_keys", &1i64)?;
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "busy_timeout", &3000i64)?;
    apply_pending_migrations(&conn)?;
    Ok(conn)
}

fn apply_pending_migrations(conn: &Connection) -> Result<(), DbError> {
    // migrations table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL)",
        [],
    )?;
    // v1: initial schema
    if !migration_applied(conn, 1)? {
        apply_v1(conn)?;
        record_migration(conn, 1)?;
    }
    Ok(())
}

fn migration_applied(conn: &Connection, v: i64) -> Result<bool, DbError> {
    let mut stmt = conn.prepare("SELECT 1 FROM migrations WHERE version = ?1 LIMIT 1")?;
    let exists = stmt.exists(params![v])?;
    Ok(exists)
}

fn record_migration(conn: &Connection, v: i64) -> Result<(), DbError> {
    conn.execute(
        "INSERT INTO migrations(version, applied_at) VALUES (?1, ?2)",
        params![v, now_iso8601_utc()],
    )?;
    Ok(())
}

fn apply_v1(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        r#"
        PRAGMA foreign_keys=ON;
        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_projects_name ON projects(name);

        CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            role TEXT NOT NULL,
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            allowed_tools_json TEXT NOT NULL,
            system_prompt TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE(project_id, name)
        );
        CREATE INDEX IF NOT EXISTS idx_agents_project_role ON agents(project_id, role);

        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
            provider TEXT NOT NULL,
            provider_session_id TEXT,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_sessions_project_created ON sessions(project_id, created_at);

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            sender TEXT NOT NULL,
            content TEXT NOT NULL,
            broadcast_id TEXT,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_messages_session_created ON messages(session_id, created_at);

        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            assignee_agent_id TEXT,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_tasks_project_status_created ON tasks(project_id, status, created_at);
        "#,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_v1_creates_schema() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        // Check a few tables
        for t in ["projects", "agents", "sessions", "messages", "tasks"] {
            let exists = table_exists(&conn, t).unwrap();
            assert!(exists, "table {} should exist", t);
        }
        // Check FK pragma is ON
        let v: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0)).unwrap();
        assert_eq!(v, 1);
    }

    fn table_exists(conn: &Connection, name: &str) -> Result<bool, DbError> {
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")?;
        Ok(stmt.exists(params![name])?)
    }
}
