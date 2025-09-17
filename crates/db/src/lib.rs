use rusqlite::{Connection, params, OptionalExtension};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid input: {0}")]
    InvalidInput(String),
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
    // v2: extend sessions table for M3
    if !migration_applied(conn, 2)? {
        apply_v2(conn)?;
        record_migration(conn, 2)?;
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

fn apply_v2(conn: &Connection) -> Result<(), DbError> {
    conn.execute_batch(
        r#"
        -- Add new columns to sessions table for M3 session resume functionality
        ALTER TABLE sessions ADD COLUMN last_activity TEXT;
        ALTER TABLE sessions ADD COLUMN status TEXT DEFAULT 'active';
        ALTER TABLE sessions ADD COLUMN metadata TEXT;
        ALTER TABLE sessions ADD COLUMN expires_at TEXT;
        
        -- Create indexes for performance
        CREATE INDEX IF NOT EXISTS idx_sessions_project_status_created ON sessions(project_id, status, created_at);
        CREATE INDEX IF NOT EXISTS idx_sessions_provider_session_id ON sessions(provider_session_id);
        CREATE INDEX IF NOT EXISTS idx_sessions_last_activity ON sessions(last_activity);
        "#,
    )?;
    Ok(())
}

// ---------- Repositories ----------

pub struct Project { pub id: String, pub name: String }
pub struct Agent {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub role: String,
    pub provider: String,
    pub model: String,
    pub allowed_tools: Vec<String>,
    pub system_prompt: String,
}

pub enum IdOrName<'a> { Id(&'a str), Name(&'a str) }

pub fn insert_project(conn: &Connection, name: &str) -> Result<Project, DbError> {
    if name.trim().is_empty() { return Err(DbError::InvalidInput("project name empty".into())); }
    let id = uuid();
    conn.execute(
        "INSERT INTO projects(id, name, created_at) VALUES (?1, ?2, ?3)",
        params![id, name, now_iso8601_utc()],
    )?;
    Ok(Project { id, name: name.to_string() })
}

pub fn find_project_id(conn: &Connection, by: IdOrName<'_>) -> Result<Option<String>, DbError> {
    let mut stmt = match by {
        IdOrName::Id(_) => conn.prepare("SELECT id FROM projects WHERE id=?1 LIMIT 1")?,
        IdOrName::Name(_) => conn.prepare("SELECT id FROM projects WHERE name=?1 LIMIT 1")?,
    };
    let val = match by {
        IdOrName::Id(v) | IdOrName::Name(v) => v,
    };
    let id: Option<String> = stmt.query_row(params![val], |r| r.get(0)).optional()?;
    Ok(id)
}

pub fn to_json_text(values: &[String]) -> String { json!(values).to_string() }
pub fn from_json_text(s: &str) -> Result<Vec<String>, DbError> {
    let v: Vec<String> = serde_json::from_str(s).map_err(|e| DbError::InvalidInput(e.to_string()))?;
    Ok(v)
}

pub fn insert_agent(
    conn: &Connection,
    project_id: &str,
    name: &str,
    role: &str,
    provider: &str,
    model: &str,
    allowed_tools: &[String],
    system_prompt: &str,
) -> Result<Agent, DbError> {
    if name.trim().is_empty() { return Err(DbError::InvalidInput("agent name empty".into())); }
    if role.trim().is_empty() { return Err(DbError::InvalidInput("agent role empty".into())); }
    let id = uuid();
    let tools = to_json_text(allowed_tools);
    conn.execute(
        "INSERT INTO agents(id, project_id, name, role, provider, model, allowed_tools_json, system_prompt, created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![id, project_id, name, role, provider, model, tools, system_prompt, now_iso8601_utc()],
    )?;
    Ok(Agent { id, project_id: project_id.into(), name: name.into(), role: role.into(), provider: provider.into(), model: model.into(), allowed_tools: allowed_tools.to_vec(), system_prompt: system_prompt.into() })
}

fn uuid() -> String { format!("{:x}{:x}", rand_u128(), rand_u128()) }

fn rand_u128() -> u128 { use std::time::{SystemTime, UNIX_EPOCH}; SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() }

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

    #[test]
    fn project_and_agent_crud() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();

        let p = insert_project(&conn, "demo").unwrap();
        assert_eq!(find_project_id(&conn, IdOrName::Name("demo")).unwrap().as_deref(), Some(p.id.as_str()));

        let a = insert_agent(&conn, &p.id, "backend", "backend", "gemini", "g-1.5", &vec!["Edit".into()], "sp").unwrap();
        assert_eq!(a.name, "backend");
        assert_eq!(from_json_text(&to_json_text(&a.allowed_tools)).unwrap(), vec!["Edit".to_string()]);

        // Uniqueness on project_id+name
        let dup = insert_agent(&conn, &p.id, "backend", "backend", "gemini", "g-1.5", &vec!["Edit".into()], "sp");
        assert!(dup.is_err());
    }

    #[test]
    fn migration_v2_extends_sessions_table() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Check that migration v2 was applied
        let v2_applied = migration_applied(&conn, 2).unwrap();
        assert!(v2_applied, "migration v2 should be applied");
        
        // Check that new columns exist
        let mut stmt = conn.prepare("PRAGMA table_info(sessions)").unwrap();
        let columns: Vec<(i64, String, String, i64, Option<String>, i64)> = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?))
        }).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
        
        let column_names: Vec<String> = columns.iter().map(|(_, name, _, _, _, _)| name.clone()).collect();
        assert!(column_names.contains(&"last_activity".to_string()), "last_activity column should exist");
        assert!(column_names.contains(&"status".to_string()), "status column should exist");
        assert!(column_names.contains(&"metadata".to_string()), "metadata column should exist");
        assert!(column_names.contains(&"expires_at".to_string()), "expires_at column should exist");
        
        // Check that indexes were created
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_sessions_%'").unwrap();
        let indexes: Vec<String> = stmt.query_map([], |row| Ok(row.get(0)?)).unwrap().collect::<Result<Vec<_>, _>>().unwrap();
        
        assert!(indexes.contains(&"idx_sessions_project_status_created".to_string()), "composite index should exist");
        assert!(indexes.contains(&"idx_sessions_provider_session_id".to_string()), "provider_session_id index should exist");
        assert!(indexes.contains(&"idx_sessions_last_activity".to_string()), "last_activity index should exist");
    }

    #[test]
    fn sessions_table_with_new_columns() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "backend", "backend", "gemini", "g-1.5", &vec!["Edit".into()], "sp").unwrap();
        
        // Insert session with new columns
        let session_id = uuid();
        let now = now_iso8601_utc();
        conn.execute(
            "INSERT INTO sessions(id, project_id, agent_id, provider, provider_session_id, created_at, last_activity, status, metadata, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![session_id, p.id, a.id, "gemini", "provider_123", now, now, "active", r#"{"test": "data"}"#, now],
        ).unwrap();
        
        // Verify the session was inserted with all columns
        let mut stmt = conn.prepare("SELECT last_activity, status, metadata, expires_at FROM sessions WHERE id = ?1").unwrap();
        let (last_activity, status, metadata, expires_at): (String, String, String, String) = stmt.query_row(params![session_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        }).unwrap();
        
        assert_eq!(last_activity, now);
        assert_eq!(status, "active");
        assert_eq!(metadata, r#"{"test": "data"}"#);
        assert_eq!(expires_at, now);
    }
}
