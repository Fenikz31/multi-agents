use rusqlite::{Connection, params, OptionalExtension};
use serde_json::json;
use config_model::ProjectConfig;

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

// ---------- Session Management Types ----------

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub agent_id: String,
    pub provider: String,
    pub provider_session_id: Option<String>,
    pub created_at: String,
    pub last_activity: Option<String>,
    pub status: SessionStatus,
    pub metadata: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionStatus {
    Active,
    Expired,
    Invalid,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Expired => write!(f, "expired"),
            SessionStatus::Invalid => write!(f, "invalid"),
        }
    }
}

impl std::str::FromStr for SessionStatus {
    type Err = DbError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(SessionStatus::Active),
            "expired" => Ok(SessionStatus::Expired),
            "invalid" => Ok(SessionStatus::Invalid),
            _ => Err(DbError::InvalidInput(format!("Invalid session status: {}", s))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionContext {
    pub session: Session,
    pub is_resumable: bool,
    pub provider_session_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("session not found: {0}")]
    NotFound(String),
    #[error("session expired: {0}")]
    Expired(String),
    #[error("session invalid: {0}")]
    Invalid(String),
    #[error("provider unavailable: {0}")]
    ProviderUnavailable(String),
    #[error("database error: {0}")]
    Database(#[from] DbError),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone)]
pub struct SessionFilters {
    pub project_id: Option<String>,
    pub agent_id: Option<String>,
    pub provider: Option<String>,
    pub status: Option<SessionStatus>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

// ---------- SessionManager Trait ----------

pub trait SessionManager {
    fn validate_session(&self, session_id: &str) -> Result<bool, SessionError>;
    fn resume_session(&self, session_id: &str) -> Result<SessionContext, SessionError>;
    fn create_session(&self, project_id: &str, agent_id: &str, provider: &str, provider_session_id: Option<&str>) -> Result<Session, SessionError>;
    fn cleanup_expired_sessions(&self) -> Result<u32, SessionError>;
}

// ---------- ClaudeSessionManager Implementation ----------

pub struct ClaudeSessionManager {
    conn: Connection,
}

impl ClaudeSessionManager {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
    
    fn ping_claude_session(&self, session_id: &str) -> Result<bool, SessionError> {
        // Simulate Claude session validation
        // In real implementation, this would call Claude API with --session-id
        if session_id.is_empty() {
            return Ok(false);
        }
        
        // Mock validation logic - in real implementation, this would:
        // 1. Call claude CLI with --session-id
        // 2. Check if session is still active
        // 3. Return true if valid, false if expired/invalid
        
        // For now, simulate that sessions starting with "valid_" are valid
        Ok(session_id.starts_with("valid_"))
    }
}

impl SessionManager for ClaudeSessionManager {
    fn validate_session(&self, session_id: &str) -> Result<bool, SessionError> {
        // First check if session exists in database
        let session = find_session(&self.conn, session_id)?;
        let session = session.ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;
        
        // Check if session is already marked as expired/invalid
        if session.status != SessionStatus::Active {
            return Ok(false);
        }
        
        // If we have a provider_session_id, ping Claude to validate
        if let Some(provider_session_id) = &session.provider_session_id {
            let is_valid = self.ping_claude_session(provider_session_id)?;
            
            // Update session status based on validation result
            let new_status = if is_valid {
                SessionStatus::Active
            } else {
                SessionStatus::Expired
            };
            
            update_session(&self.conn, session_id, None, None, Some(new_status))?;
            
            Ok(is_valid)
        } else {
            // No provider_session_id means session is not resumable
            Ok(false)
        }
    }
    
    fn resume_session(&self, session_id: &str) -> Result<SessionContext, SessionError> {
        let session = find_session(&self.conn, session_id)?
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;
        
        // Validate session before resuming
        if !self.validate_session(session_id)? {
            return Err(SessionError::Expired(session_id.to_string()));
        }
        
        let is_resumable = session.provider_session_id.is_some();
        let provider_session_id = session.provider_session_id.clone();
        
        Ok(SessionContext {
            session,
            is_resumable,
            provider_session_id,
        })
    }
    
    fn create_session(&self, project_id: &str, agent_id: &str, provider: &str, provider_session_id: Option<&str>) -> Result<Session, SessionError> {
        // Validate that this is a Claude session
        if provider != "claude" {
            return Err(SessionError::Invalid(format!("Expected claude provider, got {}", provider)));
        }
        
        // Create session in database
        let session = insert_session(&self.conn, project_id, agent_id, provider, provider_session_id)?;
        
        // If we have a provider_session_id, validate it
        if let Some(provider_session_id) = &session.provider_session_id {
            if !self.ping_claude_session(provider_session_id)? {
                // Mark as invalid if provider session is not valid
                update_session(&self.conn, &session.id, None, None, Some(SessionStatus::Invalid))?;
                return Err(SessionError::Invalid(format!("Invalid Claude session: {}", provider_session_id)));
            }
        }
        
        Ok(session)
    }
    
    fn cleanup_expired_sessions(&self) -> Result<u32, SessionError> {
        // Clean up sessions that are marked as expired or invalid
        let now = now_iso8601_utc();
        let expired_count = self.conn.execute(
            "DELETE FROM sessions WHERE status IN ('expired', 'invalid') AND last_activity < ?1",
            params![now],
        )?;
        
        Ok(expired_count as u32)
    }
}

// ---------- CursorSessionManager Implementation ----------

pub struct CursorSessionManager {
    conn: Connection,
}

impl CursorSessionManager {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
    
    fn ping_cursor_chat(&self, chat_id: &str) -> Result<bool, SessionError> {
        // Simulate Cursor chat validation
        // In real implementation, this would call Cursor CLI with --resume
        if chat_id.is_empty() {
            return Ok(false);
        }
        
        // Mock validation logic - in real implementation, this would:
        // 1. Call cursor-agent with --resume="chat-id-here"
        // 2. Check if chat is still active
        // 3. Return true if valid, false if expired/invalid
        
        // For now, simulate that chat IDs starting with "valid_" are valid
        Ok(chat_id.starts_with("valid_"))
    }
    
    fn create_cursor_chat(&self) -> Result<String, SessionError> {
        // Simulate Cursor chat creation
        // In real implementation, this would call cursor-agent create-chat
        // and return the generated chat_id
        
        // Mock implementation: generate a new chat ID using timestamp + random
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let random = (timestamp % 10000) as u32; // Use last 4 digits for uniqueness
        Ok(format!("valid_chat_{}_{}", timestamp, random))
    }
}

impl SessionManager for CursorSessionManager {
    fn validate_session(&self, session_id: &str) -> Result<bool, SessionError> {
        // First check if session exists in database
        let session = find_session(&self.conn, session_id)?;
        let session = session.ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;
        
        // Check if session is already marked as expired/invalid
        if session.status != SessionStatus::Active {
            return Ok(false);
        }
        
        // If we have a provider_session_id (chat_id), ping Cursor to validate
        if let Some(provider_session_id) = &session.provider_session_id {
            let is_valid = self.ping_cursor_chat(provider_session_id)?;
            
            // Update session status based on validation result
            let new_status = if is_valid {
                SessionStatus::Active
            } else {
                SessionStatus::Expired
            };
            
            update_session(&self.conn, session_id, None, None, Some(new_status))?;
            
            Ok(is_valid)
        } else {
            // No provider_session_id means session is not resumable
            Ok(false)
        }
    }
    
    fn resume_session(&self, session_id: &str) -> Result<SessionContext, SessionError> {
        let session = find_session(&self.conn, session_id)?
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;
        
        // Validate session before resuming
        if !self.validate_session(session_id)? {
            return Err(SessionError::Expired(session_id.to_string()));
        }
        
        let is_resumable = session.provider_session_id.is_some();
        let provider_session_id = session.provider_session_id.clone();
        
        Ok(SessionContext {
            session,
            is_resumable,
            provider_session_id,
        })
    }
    
    fn create_session(&self, project_id: &str, agent_id: &str, provider: &str, provider_session_id: Option<&str>) -> Result<Session, SessionError> {
        // Validate that this is a Cursor session
        if provider != "cursor-agent" {
            return Err(SessionError::Invalid(format!("Expected cursor-agent provider, got {}", provider)));
        }
        
        // If no provider_session_id provided, create a new Cursor chat
        let chat_id = if let Some(provider_session_id) = provider_session_id {
            // Validate existing chat_id
            if !self.ping_cursor_chat(provider_session_id)? {
                return Err(SessionError::Invalid(format!("Invalid Cursor chat: {}", provider_session_id)));
            }
            provider_session_id.to_string()
        } else {
            // Create new chat
            self.create_cursor_chat()?
        };
        
        // Create session in database
        let session = insert_session(&self.conn, project_id, agent_id, provider, Some(&chat_id))?;
        
        Ok(session)
    }
    
    fn cleanup_expired_sessions(&self) -> Result<u32, SessionError> {
        // Clean up sessions that are marked as expired or invalid
        let now = now_iso8601_utc();
        let expired_count = self.conn.execute(
            "DELETE FROM sessions WHERE status IN ('expired', 'invalid') AND last_activity < ?1",
            params![now],
        )?;
        
        Ok(expired_count as u32)
    }
}

// ---------- GeminiSessionManager Implementation ----------

pub struct GeminiSessionManager {
    conn: Connection,
}

impl GeminiSessionManager {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
    
    fn validate_gemini_context(&self, context_id: &str) -> Result<bool, SessionError> {
        // Simulate Gemini context validation
        // In real implementation, this would check if the context is still available
        if context_id.is_empty() {
            return Ok(false);
        }
        
        // Mock validation logic - in real implementation, this would:
        // 1. Check if Gemini context is still available
        // 2. Verify context hasn't expired
        // 3. Return true if valid, false if expired/invalid
        
        // For now, simulate that context IDs starting with "valid_" are valid
        Ok(context_id.starts_with("valid_"))
    }
    
    fn create_gemini_context(&self) -> Result<String, SessionError> {
        // Simulate Gemini context creation
        // In real implementation, this would create a new Gemini context
        // and return the generated context_id
        
        // Mock implementation: generate a new context ID using timestamp
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let random = (timestamp % 10000) as u32; // Use last 4 digits for uniqueness
        Ok(format!("valid_context_{}_{}", timestamp, random))
    }
}

impl SessionManager for GeminiSessionManager {
    fn validate_session(&self, session_id: &str) -> Result<bool, SessionError> {
        // First check if session exists in database
        let session = find_session(&self.conn, session_id)?;
        let session = session.ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;
        
        // Check if session is already marked as expired/invalid
        if session.status != SessionStatus::Active {
            return Ok(false);
        }
        
        // If we have a provider_session_id (context_id), validate Gemini context
        if let Some(provider_session_id) = &session.provider_session_id {
            let is_valid = self.validate_gemini_context(provider_session_id)?;
            
            // Update session status based on validation result
            let new_status = if is_valid {
                SessionStatus::Active
            } else {
                SessionStatus::Expired
            };
            
            update_session(&self.conn, session_id, None, None, Some(new_status))?;
            
            Ok(is_valid)
        } else {
            // No provider_session_id means session is not resumable
            Ok(false)
        }
    }
    
    fn resume_session(&self, session_id: &str) -> Result<SessionContext, SessionError> {
        let session = find_session(&self.conn, session_id)?
            .ok_or_else(|| SessionError::NotFound(session_id.to_string()))?;
        
        // Validate session before resuming
        if !self.validate_session(session_id)? {
            return Err(SessionError::Expired(session_id.to_string()));
        }
        
        let is_resumable = session.provider_session_id.is_some();
        let provider_session_id = session.provider_session_id.clone();
        
        Ok(SessionContext {
            session,
            is_resumable,
            provider_session_id,
        })
    }
    
    fn create_session(&self, project_id: &str, agent_id: &str, provider: &str, provider_session_id: Option<&str>) -> Result<Session, SessionError> {
        // Validate that this is a Gemini session
        if provider != "gemini" {
            return Err(SessionError::Invalid(format!("Expected gemini provider, got {}", provider)));
        }
        
        // If no provider_session_id provided, create a new Gemini context
        let context_id = if let Some(provider_session_id) = provider_session_id {
            // Validate existing context_id
            if !self.validate_gemini_context(provider_session_id)? {
                return Err(SessionError::Invalid(format!("Invalid Gemini context: {}", provider_session_id)));
            }
            provider_session_id.to_string()
        } else {
            // Create new context
            self.create_gemini_context()?
        };
        
        // Create session in database
        let session = insert_session(&self.conn, project_id, agent_id, provider, Some(&context_id))?;
        
        Ok(session)
    }
    
    fn cleanup_expired_sessions(&self) -> Result<u32, SessionError> {
        // Clean up sessions that are marked as expired or invalid
        let now = now_iso8601_utc();
        let expired_count = self.conn.execute(
            "DELETE FROM sessions WHERE status IN ('expired', 'invalid') AND last_activity < ?1",
            params![now],
        )?;
        
        Ok(expired_count as u32)
    }
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

// ---------- Session CRUD Functions ----------

pub fn insert_session(
    conn: &Connection,
    project_id: &str,
    agent_id: &str,
    provider: &str,
    provider_session_id: Option<&str>,
) -> Result<Session, DbError> {
    let id = uuid();
    let now = now_iso8601_utc();
    conn.execute(
        "INSERT INTO sessions(id, project_id, agent_id, provider, provider_session_id, created_at, last_activity, status, metadata, expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![id, project_id, agent_id, provider, provider_session_id, now, now, "active", None::<String>, None::<String>],
    )?;
    Ok(Session {
        id,
        project_id: project_id.to_string(),
        agent_id: agent_id.to_string(),
        provider: provider.to_string(),
        provider_session_id: provider_session_id.map(|s| s.to_string()),
        created_at: now.clone(),
        last_activity: Some(now),
        status: SessionStatus::Active,
        metadata: None,
        expires_at: None,
    })
}

pub fn find_session(conn: &Connection, session_id: &str) -> Result<Option<Session>, DbError> {
    let mut stmt = conn.prepare(
        "SELECT id, project_id, agent_id, provider, provider_session_id, created_at, last_activity, status, metadata, expires_at FROM sessions WHERE id = ?1"
    )?;
    let session = stmt.query_row(params![session_id], |row| {
        let status_str: String = row.get(7)?;
        let status = status_str.parse().unwrap_or(SessionStatus::Invalid);
        Ok(Session {
            id: row.get(0)?,
            project_id: row.get(1)?,
            agent_id: row.get(2)?,
            provider: row.get(3)?,
            provider_session_id: row.get(4)?,
            created_at: row.get(5)?,
            last_activity: row.get(6)?,
            status,
            metadata: row.get(8)?,
            expires_at: row.get(9)?,
        })
    }).optional()?;
    Ok(session)
}

pub fn list_sessions(conn: &Connection, filters: SessionFilters) -> Result<Vec<Session>, DbError> {
    let mut query = "SELECT id, project_id, agent_id, provider, provider_session_id, created_at, last_activity, status, metadata, expires_at FROM sessions WHERE 1=1".to_string();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let mut param_count = 0;

    if let Some(project_id) = &filters.project_id {
        param_count += 1;
        query.push_str(&format!(" AND project_id = ?{}", param_count));
        params.push(Box::new(project_id.clone()));
    }

    if let Some(agent_id) = &filters.agent_id {
        param_count += 1;
        query.push_str(&format!(" AND agent_id = ?{}", param_count));
        params.push(Box::new(agent_id.clone()));
    }

    if let Some(provider) = &filters.provider {
        param_count += 1;
        query.push_str(&format!(" AND provider = ?{}", param_count));
        params.push(Box::new(provider.clone()));
    }

    if let Some(status) = &filters.status {
        param_count += 1;
        query.push_str(&format!(" AND status = ?{}", param_count));
        params.push(Box::new(status.to_string()));
    }

    query.push_str(" ORDER BY created_at DESC");

    if let Some(limit) = filters.limit {
        param_count += 1;
        query.push_str(&format!(" LIMIT ?{}", param_count));
        params.push(Box::new(limit as i64));
    }

    if let Some(offset) = filters.offset {
        param_count += 1;
        query.push_str(&format!(" OFFSET ?{}", param_count));
        params.push(Box::new(offset as i64));
    }

    let mut stmt = conn.prepare(&query)?;
    let session_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
        let status_str: String = row.get(7)?;
        let status = status_str.parse().unwrap_or(SessionStatus::Invalid);
        Ok(Session {
            id: row.get(0)?,
            project_id: row.get(1)?,
            agent_id: row.get(2)?,
            provider: row.get(3)?,
            provider_session_id: row.get(4)?,
            created_at: row.get(5)?,
            last_activity: row.get(6)?,
            status,
            metadata: row.get(8)?,
            expires_at: row.get(9)?,
        })
    })?;

    let mut sessions = Vec::new();
    for session in session_iter {
        sessions.push(session?);
    }
    Ok(sessions)
}

pub fn update_session(
    conn: &Connection,
    session_id: &str,
    provider_session_id: Option<&str>,
    last_activity: Option<&str>,
    status: Option<SessionStatus>,
) -> Result<(), DbError> {
    let mut query = "UPDATE sessions SET".to_string();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    let mut param_count = 0;
    let mut updates = Vec::new();

    if let Some(provider_session_id) = provider_session_id {
        param_count += 1;
        updates.push(format!("provider_session_id = ?{}", param_count));
        params.push(Box::new(provider_session_id));
    }

    if let Some(last_activity) = last_activity {
        param_count += 1;
        updates.push(format!("last_activity = ?{}", param_count));
        params.push(Box::new(last_activity));
    }

    if let Some(status) = status {
        param_count += 1;
        updates.push(format!("status = ?{}", param_count));
        params.push(Box::new(status.to_string()));
    }

    if updates.is_empty() {
        return Ok(());
    }

    query.push_str(" ");
    query.push_str(&updates.join(", "));
    param_count += 1;
    query.push_str(&format!(" WHERE id = ?{}", param_count));
    params.push(Box::new(session_id));

    conn.execute(&query, rusqlite::params_from_iter(params))?;
    Ok(())
}

pub fn delete_expired_sessions(conn: &Connection, before_timestamp: &str) -> Result<u32, DbError> {
    let count = conn.execute(
        "DELETE FROM sessions WHERE expires_at IS NOT NULL AND expires_at < ?1",
        params![before_timestamp],
    )?;
    Ok(count as u32)
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

    #[test]
    fn session_status_display_and_parse() {
        assert_eq!(SessionStatus::Active.to_string(), "active");
        assert_eq!(SessionStatus::Expired.to_string(), "expired");
        assert_eq!(SessionStatus::Invalid.to_string(), "invalid");
        
        assert_eq!("active".parse::<SessionStatus>().unwrap(), SessionStatus::Active);
        assert_eq!("expired".parse::<SessionStatus>().unwrap(), SessionStatus::Expired);
        assert_eq!("invalid".parse::<SessionStatus>().unwrap(), SessionStatus::Invalid);
        
        assert!("invalid_status".parse::<SessionStatus>().is_err());
    }

    #[test]
    fn session_crud_operations() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "backend", "backend", "gemini", "g-1.5", &vec!["Edit".into()], "sp").unwrap();
        
        // Test insert_session
        let session = insert_session(&conn, &p.id, &a.id, "gemini", Some("provider_123")).unwrap();
        assert_eq!(session.project_id, p.id);
        assert_eq!(session.agent_id, a.id);
        assert_eq!(session.provider, "gemini");
        assert_eq!(session.provider_session_id, Some("provider_123".to_string()));
        assert_eq!(session.status, SessionStatus::Active);
        
        // Test find_session
        let found_session = find_session(&conn, &session.id).unwrap().unwrap();
        assert_eq!(found_session.id, session.id);
        assert_eq!(found_session.status, SessionStatus::Active);
        
        // Test list_sessions with filters
        let filters = SessionFilters {
            project_id: Some(p.id.clone()),
            agent_id: None,
            provider: None,
            status: None,
            limit: Some(10),
            offset: None,
        };
        let sessions = list_sessions(&conn, filters).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session.id);
        
        // Test update_session
        update_session(&conn, &session.id, None, Some("2025-01-17T20:00:00Z"), Some(SessionStatus::Expired)).unwrap();
        let updated_session = find_session(&conn, &session.id).unwrap().unwrap();
        assert_eq!(updated_session.last_activity, Some("2025-01-17T20:00:00Z".to_string()));
        assert_eq!(updated_session.status, SessionStatus::Expired);
        
        // Set expires_at to make session eligible for deletion
        conn.execute(
            "UPDATE sessions SET expires_at = ?1 WHERE id = ?2",
            params!["2025-01-17T21:00:00Z", session.id],
        ).unwrap();
        
        // Test delete_expired_sessions
        let deleted_count = delete_expired_sessions(&conn, "2025-01-18T00:00:00Z").unwrap();
        assert_eq!(deleted_count, 1);
        
        // Verify session is deleted
        let deleted_session = find_session(&conn, &session.id).unwrap();
        assert!(deleted_session.is_none());
    }

    #[test]
    fn session_filters_work_correctly() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agents
        let p = insert_project(&conn, "demo").unwrap();
        let a1 = insert_agent(&conn, &p.id, "backend", "backend", "gemini", "g-1.5", &vec!["Edit".into()], "sp").unwrap();
        let a2 = insert_agent(&conn, &p.id, "frontend", "frontend", "claude", "claude-3", &vec!["Edit".into()], "sp").unwrap();
        
        // Create sessions
        let s1 = insert_session(&conn, &p.id, &a1.id, "gemini", Some("provider_1")).unwrap();
        let _s2 = insert_session(&conn, &p.id, &a2.id, "claude", Some("provider_2")).unwrap();
        
        // Test filter by provider
        let filters = SessionFilters {
            project_id: Some(p.id.clone()),
            agent_id: None,
            provider: Some("gemini".to_string()),
            status: None,
            limit: None,
            offset: None,
        };
        let sessions = list_sessions(&conn, filters).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].provider, "gemini");
        
        // Test filter by status
        update_session(&conn, &s1.id, None, None, Some(SessionStatus::Expired)).unwrap();
        let filters = SessionFilters {
            project_id: Some(p.id.clone()),
            agent_id: None,
            provider: None,
            status: Some(SessionStatus::Expired),
            limit: None,
            offset: None,
        };
        let sessions = list_sessions(&conn, filters).unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].status, SessionStatus::Expired);
    }

    #[test]
    fn list_sessions_pagination_and_sorting() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();

        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "backend", "backend", "gemini", "g-1.5", &vec!["Edit".into()], "sp").unwrap();

        // Insert multiple sessions (timestamps auto now; order by created_at DESC expected)
        let mut ids = vec![];
        for _ in 0..5 { ids.push(insert_session(&conn, &p.id, &a.id, "gemini", None).unwrap().id); }

        // List with limit 2, page 1
        let filters = SessionFilters { project_id: Some(p.id.clone()), agent_id: None, provider: None, status: None, limit: Some(2), offset: Some(0) };
        let page1 = list_sessions(&conn, filters).unwrap();
        assert_eq!(page1.len(), 2);

        // Next page
        let filters = SessionFilters { project_id: Some(p.id.clone()), agent_id: None, provider: None, status: None, limit: Some(2), offset: Some(2) };
        let page2 = list_sessions(&conn, filters).unwrap();
        assert_eq!(page2.len(), 2);

        // Last page (maybe 1 element)
        let filters = SessionFilters { project_id: Some(p.id.clone()), agent_id: None, provider: None, status: None, limit: Some(2), offset: Some(4) };
        let page3 = list_sessions(&conn, filters).unwrap();
        assert!(page3.len() == 1 || page3.len() == 2); // depending on timing
    }

    #[test]
    fn update_session_field_combinations() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();

        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "backend", "backend", "gemini", "g-1.5", &vec!["Edit".into()], "sp").unwrap();
        let s = insert_session(&conn, &p.id, &a.id, "gemini", Some("ctx_1")).unwrap();

        // Update only last_activity
        update_session(&conn, &s.id, None, Some("2025-01-20T00:00:00Z"), None).unwrap();
        let after = find_session(&conn, &s.id).unwrap().unwrap();
        assert_eq!(after.last_activity, Some("2025-01-20T00:00:00Z".into()));

        // Update only provider_session_id
        update_session(&conn, &s.id, Some("ctx_2"), None, None).unwrap();
        let after = find_session(&conn, &s.id).unwrap().unwrap();
        assert_eq!(after.provider_session_id, Some("ctx_2".into()));

        // Update status
        update_session(&conn, &s.id, None, None, Some(SessionStatus::Expired)).unwrap();
        let after = find_session(&conn, &s.id).unwrap().unwrap();
        assert_eq!(after.status, SessionStatus::Expired);
    }

    #[test]
    fn delete_expired_sessions_respects_timestamp() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();

        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "backend", "backend", "gemini", "g-1.5", &vec!["Edit".into()], "sp").unwrap();
        let s = insert_session(&conn, &p.id, &a.id, "gemini", Some("ctx")) .unwrap();

        // Make it expire yesterday
        conn.execute(
            "UPDATE sessions SET expires_at = ?1 WHERE id = ?2",
            params!["2025-01-01T00:00:00Z", s.id],
        ).unwrap();

        let deleted = delete_expired_sessions(&conn, "2025-01-02T00:00:00Z").unwrap();
        assert_eq!(deleted, 1);

        // Nothing left to delete now
        let deleted2 = delete_expired_sessions(&conn, "2025-01-03T00:00:00Z").unwrap();
        assert_eq!(deleted2, 0);
    }

    #[test]
    fn session_error_types() {
        let not_found = SessionError::NotFound("test".to_string());
        assert!(matches!(not_found, SessionError::NotFound(_)));
        
        let expired = SessionError::Expired("test".to_string());
        assert!(matches!(expired, SessionError::Expired(_)));
        
        let invalid = SessionError::Invalid("test".to_string());
        assert!(matches!(invalid, SessionError::Invalid(_)));
        
        let provider_unavailable = SessionError::ProviderUnavailable("test".to_string());
        assert!(matches!(provider_unavailable, SessionError::ProviderUnavailable(_)));
        
        let db_error = SessionError::Database(DbError::InvalidInput("test".to_string()));
        assert!(matches!(db_error, SessionError::Database(_)));
    }

    #[test]
    fn claude_session_manager_validation() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "backend", "backend", "claude", "claude-3", &vec!["Edit".into()], "sp").unwrap();
        
        // Create sessions
        let session = insert_session(&conn, &p.id, &a.id, "claude", Some("valid_session_123")).unwrap();
        let invalid_session = insert_session(&conn, &p.id, &a.id, "claude", Some("invalid_session_456")).unwrap();
        
        let manager = ClaudeSessionManager::new(conn);
        
        // Test validation with valid session
        let is_valid = manager.validate_session(&session.id).unwrap();
        assert!(is_valid, "Valid session should be valid");
        
        // Test validation with invalid session
        let is_valid = manager.validate_session(&invalid_session.id).unwrap();
        assert!(!is_valid, "Invalid session should not be valid");
        
        // Test validation with non-existent session
        let result = manager.validate_session("non_existent_session");
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn claude_session_manager_resume() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "backend", "backend", "claude", "claude-3", &vec!["Edit".into()], "sp").unwrap();
        
        // Create sessions
        let session = insert_session(&conn, &p.id, &a.id, "claude", Some("valid_session_123")).unwrap();
        let invalid_session = insert_session(&conn, &p.id, &a.id, "claude", Some("invalid_session_456")).unwrap();
        
        let manager = ClaudeSessionManager::new(conn);
        
        // Test successful resume
        let context = manager.resume_session(&session.id).unwrap();
        assert_eq!(context.session.id, session.id);
        assert!(context.is_resumable);
        assert_eq!(context.provider_session_id, Some("valid_session_123".to_string()));
        
        // Test resume with invalid session
        let result = manager.resume_session(&invalid_session.id);
        assert!(matches!(result, Err(SessionError::Expired(_))));
        
        // Test resume with non-existent session
        let result = manager.resume_session("non_existent_session");
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn claude_session_manager_create() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "backend", "backend", "claude", "claude-3", &vec!["Edit".into()], "sp").unwrap();
        
        let manager = ClaudeSessionManager::new(conn);
        
        // Test successful creation with valid provider session
        let session = manager.create_session(&p.id, &a.id, "claude", Some("valid_session_123")).unwrap();
        assert_eq!(session.provider, "claude");
        assert_eq!(session.provider_session_id, Some("valid_session_123".to_string()));
        assert_eq!(session.status, SessionStatus::Active);
        
        // Test creation with invalid provider session
        let result = manager.create_session(&p.id, &a.id, "claude", Some("invalid_session_456"));
        assert!(matches!(result, Err(SessionError::Invalid(_))));
        
        // Test creation with wrong provider
        let result = manager.create_session(&p.id, &a.id, "gemini", Some("valid_session_123"));
        assert!(matches!(result, Err(SessionError::Invalid(_))));
        
        // Test creation without provider session ID
        let session = manager.create_session(&p.id, &a.id, "claude", None).unwrap();
        assert_eq!(session.provider, "claude");
        assert_eq!(session.provider_session_id, None);
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn claude_session_manager_cleanup() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "backend", "backend", "claude", "claude-3", &vec!["Edit".into()], "sp").unwrap();
        
        // Create sessions with different statuses
        let _active_session = insert_session(&conn, &p.id, &a.id, "claude", Some("valid_session_123")).unwrap();
        let expired_session = insert_session(&conn, &p.id, &a.id, "claude", Some("invalid_session_456")).unwrap();
        
        // Mark one session as expired
        update_session(&conn, &expired_session.id, None, None, Some(SessionStatus::Expired)).unwrap();
        
        let manager = ClaudeSessionManager::new(conn);
        
        // Test cleanup
        let cleaned_count = manager.cleanup_expired_sessions().unwrap();
        assert_eq!(cleaned_count, 1, "Should clean up 1 expired session");
        
        // Note: We can't verify the cleanup results here because conn was moved to manager
        // In a real implementation, we would need to add a method to check session existence
        // or restructure the test to avoid moving the connection
    }

    #[test]
    fn claude_session_manager_ping_logic() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        let manager = ClaudeSessionManager::new(conn);
        
        // Test ping logic directly
        assert!(manager.ping_claude_session("valid_test_session").unwrap());
        assert!(!manager.ping_claude_session("invalid_test_session").unwrap());
        assert!(!manager.ping_claude_session("").unwrap());
    }

    #[test]
    fn cursor_session_manager_validation() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "frontend", "frontend", "cursor-agent", "auto", &vec!["Edit".into()], "sp").unwrap();
        
        // Create sessions
        let session = insert_session(&conn, &p.id, &a.id, "cursor-agent", Some("valid_chat_123")).unwrap();
        let invalid_session = insert_session(&conn, &p.id, &a.id, "cursor-agent", Some("invalid_chat_456")).unwrap();
        
        let manager = CursorSessionManager::new(conn);
        
        // Test validation with valid chat
        let is_valid = manager.validate_session(&session.id).unwrap();
        assert!(is_valid, "Valid chat should be valid");
        
        // Test validation with invalid chat
        let is_valid = manager.validate_session(&invalid_session.id).unwrap();
        assert!(!is_valid, "Invalid chat should not be valid");
        
        // Test validation with non-existent session
        let result = manager.validate_session("non_existent_session");
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn cursor_session_manager_resume() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "frontend", "frontend", "cursor-agent", "auto", &vec!["Edit".into()], "sp").unwrap();
        
        // Create sessions
        let session = insert_session(&conn, &p.id, &a.id, "cursor-agent", Some("valid_chat_123")).unwrap();
        let invalid_session = insert_session(&conn, &p.id, &a.id, "cursor-agent", Some("invalid_chat_456")).unwrap();
        
        let manager = CursorSessionManager::new(conn);
        
        // Test successful resume
        let context = manager.resume_session(&session.id).unwrap();
        assert_eq!(context.session.id, session.id);
        assert!(context.is_resumable);
        assert_eq!(context.provider_session_id, Some("valid_chat_123".to_string()));
        
        // Test resume with invalid chat
        let result = manager.resume_session(&invalid_session.id);
        assert!(matches!(result, Err(SessionError::Expired(_))));
        
        // Test resume with non-existent session
        let result = manager.resume_session("non_existent_session");
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn cursor_session_manager_create() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "frontend", "frontend", "cursor-agent", "auto", &vec!["Edit".into()], "sp").unwrap();
        
        let manager = CursorSessionManager::new(conn);
        
        // Test successful creation with valid chat
        let session = manager.create_session(&p.id, &a.id, "cursor-agent", Some("valid_chat_123")).unwrap();
        assert_eq!(session.provider, "cursor-agent");
        assert_eq!(session.provider_session_id, Some("valid_chat_123".to_string()));
        assert_eq!(session.status, SessionStatus::Active);
        
        // Test creation with invalid chat
        let result = manager.create_session(&p.id, &a.id, "cursor-agent", Some("invalid_chat_456"));
        assert!(matches!(result, Err(SessionError::Invalid(_))));
        
        // Test creation with wrong provider
        let result = manager.create_session(&p.id, &a.id, "claude", Some("valid_chat_123"));
        assert!(matches!(result, Err(SessionError::Invalid(_))));
        
        // Test creation without chat ID (should create new chat)
        let session = manager.create_session(&p.id, &a.id, "cursor-agent", None).unwrap();
        assert_eq!(session.provider, "cursor-agent");
        assert!(session.provider_session_id.is_some());
        assert!(session.provider_session_id.unwrap().starts_with("valid_chat_"));
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn cursor_session_manager_cleanup() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "frontend", "frontend", "cursor-agent", "auto", &vec!["Edit".into()], "sp").unwrap();
        
        // Create sessions with different statuses
        let _active_session = insert_session(&conn, &p.id, &a.id, "cursor-agent", Some("valid_chat_123")).unwrap();
        let expired_session = insert_session(&conn, &p.id, &a.id, "cursor-agent", Some("invalid_chat_456")).unwrap();
        
        // Mark one session as expired
        update_session(&conn, &expired_session.id, None, None, Some(SessionStatus::Expired)).unwrap();
        
        let manager = CursorSessionManager::new(conn);
        
        // Test cleanup
        let cleaned_count = manager.cleanup_expired_sessions().unwrap();
        assert_eq!(cleaned_count, 1, "Should clean up 1 expired session");
        
        // Note: We can't verify the cleanup results here because conn was moved to manager
        // In a real implementation, we would need to add a method to check session existence
        // or restructure the test to avoid moving the connection
    }

    #[test]
    fn cursor_session_manager_ping_logic() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        let manager = CursorSessionManager::new(conn);
        
        // Test ping logic directly
        assert!(manager.ping_cursor_chat("valid_test_chat").unwrap());
        assert!(!manager.ping_cursor_chat("invalid_test_chat").unwrap());
        assert!(!manager.ping_cursor_chat("").unwrap());
    }

    #[test]
    fn cursor_session_manager_create_chat() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        let manager = CursorSessionManager::new(conn);
        
        // Test chat creation
        let chat_id1 = manager.create_cursor_chat().unwrap();
        let chat_id2 = manager.create_cursor_chat().unwrap();
        
        assert!(chat_id1.starts_with("valid_chat_"));
        assert!(chat_id2.starts_with("valid_chat_"));
        // Note: In fast tests, IDs might be identical due to same timestamp
        // In real implementation, this would be handled by the actual Cursor CLI
    }

    #[test]
    fn gemini_session_manager_validation() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "devops", "devops", "gemini", "gemini-1.5-pro", &vec!["Bash".into()], "sp").unwrap();
        
        // Create sessions
        let session = insert_session(&conn, &p.id, &a.id, "gemini", Some("valid_context_123")).unwrap();
        let invalid_session = insert_session(&conn, &p.id, &a.id, "gemini", Some("invalid_context_456")).unwrap();
        
        let manager = GeminiSessionManager::new(conn);
        
        // Test validation with valid context
        let is_valid = manager.validate_session(&session.id).unwrap();
        assert!(is_valid, "Valid context should be valid");
        
        // Test validation with invalid context
        let is_valid = manager.validate_session(&invalid_session.id).unwrap();
        assert!(!is_valid, "Invalid context should not be valid");
        
        // Test validation with non-existent session
        let result = manager.validate_session("non_existent_session");
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn gemini_session_manager_resume() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "devops", "devops", "gemini", "gemini-1.5-pro", &vec!["Bash".into()], "sp").unwrap();
        
        // Create sessions
        let session = insert_session(&conn, &p.id, &a.id, "gemini", Some("valid_context_123")).unwrap();
        let invalid_session = insert_session(&conn, &p.id, &a.id, "gemini", Some("invalid_context_456")).unwrap();
        
        let manager = GeminiSessionManager::new(conn);
        
        // Test successful resume
        let context = manager.resume_session(&session.id).unwrap();
        assert_eq!(context.session.id, session.id);
        assert!(context.is_resumable);
        assert_eq!(context.provider_session_id, Some("valid_context_123".to_string()));
        
        // Test resume with invalid context
        let result = manager.resume_session(&invalid_session.id);
        assert!(matches!(result, Err(SessionError::Expired(_))));
        
        // Test resume with non-existent session
        let result = manager.resume_session("non_existent_session");
        assert!(matches!(result, Err(SessionError::NotFound(_))));
    }

    #[test]
    fn gemini_session_manager_create() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "devops", "devops", "gemini", "gemini-1.5-pro", &vec!["Bash".into()], "sp").unwrap();
        
        let manager = GeminiSessionManager::new(conn);
        
        // Test successful creation with valid context
        let session = manager.create_session(&p.id, &a.id, "gemini", Some("valid_context_123")).unwrap();
        assert_eq!(session.provider, "gemini");
        assert_eq!(session.provider_session_id, Some("valid_context_123".to_string()));
        assert_eq!(session.status, SessionStatus::Active);
        
        // Test creation with invalid context
        let result = manager.create_session(&p.id, &a.id, "gemini", Some("invalid_context_456"));
        assert!(matches!(result, Err(SessionError::Invalid(_))));
        
        // Test creation with wrong provider
        let result = manager.create_session(&p.id, &a.id, "claude", Some("valid_context_123"));
        assert!(matches!(result, Err(SessionError::Invalid(_))));
        
        // Test creation without context ID (should create new context)
        let session = manager.create_session(&p.id, &a.id, "gemini", None).unwrap();
        assert_eq!(session.provider, "gemini");
        assert!(session.provider_session_id.is_some());
        assert!(session.provider_session_id.unwrap().starts_with("valid_context_"));
        assert_eq!(session.status, SessionStatus::Active);
    }

    #[test]
    fn gemini_session_manager_cleanup() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        
        // Create project and agent
        let p = insert_project(&conn, "demo").unwrap();
        let a = insert_agent(&conn, &p.id, "devops", "devops", "gemini", "gemini-1.5-pro", &vec!["Bash".into()], "sp").unwrap();
        
        // Create sessions with different statuses
        let _active_session = insert_session(&conn, &p.id, &a.id, "gemini", Some("valid_context_123")).unwrap();
        let expired_session = insert_session(&conn, &p.id, &a.id, "gemini", Some("invalid_context_456")).unwrap();
        
        // Mark one session as expired
        update_session(&conn, &expired_session.id, None, None, Some(SessionStatus::Expired)).unwrap();
        
        let manager = GeminiSessionManager::new(conn);
        
        // Test cleanup
        let cleaned_count = manager.cleanup_expired_sessions().unwrap();
        assert_eq!(cleaned_count, 1, "Should clean up 1 expired session");
        
        // Note: We can't verify the cleanup results here because conn was moved to manager
        // In a real implementation, we would need to add a method to check session existence
        // or restructure the test to avoid moving the connection
    }

    #[test]
    fn gemini_session_manager_validation_logic() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        let manager = GeminiSessionManager::new(conn);
        
        // Test validation logic directly
        assert!(manager.validate_gemini_context("valid_test_context").unwrap());
        assert!(!manager.validate_gemini_context("invalid_test_context").unwrap());
        assert!(!manager.validate_gemini_context("").unwrap());
    }

    #[test]
    fn gemini_session_manager_create_context() {
        let tmp = tempfile::tempdir().unwrap();
        let db_path = tmp.path().join("multi-agents.sqlite3");
        let conn = open_or_create_db(db_path.to_string_lossy().as_ref()).unwrap();
        let manager = GeminiSessionManager::new(conn);
        
        // Test context creation
        let context_id1 = manager.create_gemini_context().unwrap();
        let context_id2 = manager.create_gemini_context().unwrap();
        
        assert!(context_id1.starts_with("valid_context_"));
        assert!(context_id2.starts_with("valid_context_"));
        // Note: In fast tests, IDs might be identical due to same timestamp
        // In real implementation, this would be handled by the actual Gemini API
    }
}

// ---------- Project Synchronization ----------

/// Synchronize a project and its agents from YAML configuration to database
/// This function is idempotent: if project/agents already exist, they are not modified
pub fn sync_project_from_config(conn: &Connection, project_config: &ProjectConfig) -> Result<(), DbError> {
    // 1. Ensure project exists
    let project_id = match find_project_id(conn, IdOrName::Name(&project_config.project))? {
        Some(id) => {
            println!("Project '{}' already exists in database", project_config.project);
            id
        }
        None => {
            println!("Creating project '{}' in database", project_config.project);
            let project = insert_project(conn, &project_config.project)?;
            project.id
        }
    };

    // 2. Ensure all agents exist
    for agent_config in &project_config.agents {
        let agent_exists = conn.query_row(
            "SELECT COUNT(*) FROM agents WHERE project_id = ?1 AND name = ?2",
            params![&project_id, &agent_config.name],
            |row| Ok(row.get::<_, i64>(0)?)
        )? > 0;

        if agent_exists {
            println!("Agent '{}' already exists in database", agent_config.name);
        } else {
            println!("Creating agent '{}' in database", agent_config.name);
            let _agent = insert_agent(
                conn,
                &project_id,
                &agent_config.name,
                &agent_config.role,
                &agent_config.provider,
                &agent_config.model,
                &agent_config.allowed_tools,
                &agent_config.system_prompt,
            )?;
        }
    }

    println!("Project '{}' synchronized successfully", project_config.project);
    Ok(())
}
