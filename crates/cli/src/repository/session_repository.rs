//! Session repository implementation
//! 
//! Handles all database operations related to sessions.

use std::error::Error;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use db::{Session, SessionStatus, SessionType};
use super::Repository;

/// Repository for session data operations
pub struct SessionRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SessionRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
    
    /// Find sessions by project ID
    pub fn find_by_project_id(&self, project_id: &str) -> Result<Vec<Session>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, agent_id, provider, provider_session_id, created_at, last_activity, status, metadata, expires_at, session_type FROM sessions WHERE project_id = ?1")?;
        let rows = stmt.query_map([project_id], |row| {
            Ok(Session {
                id: row.get(0)?,
                project_id: row.get(1)?,
                agent_id: row.get(2)?,
                provider: row.get(3)?,
                provider_session_id: row.get(4)?,
                created_at: row.get(5)?,
                last_activity: row.get(6)?,
                status: SessionStatus::Active, // Default status
                metadata: row.get(8)?,
                expires_at: row.get(9)?,
                session_type: SessionType::Chat, // Default type
            })
        })?;
        
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }
    
    /// Find active sessions (recently created)
    pub fn find_active_sessions(&self, project_id: &str, hours: i64) -> Result<Vec<Session>, Box<dyn Error>> {
        let query = format!("SELECT id, project_id, agent_id, provider, provider_session_id, created_at, last_activity, status, metadata, expires_at, session_type FROM sessions WHERE project_id = ?1 AND datetime(created_at) > datetime('now', '-{} hours')", hours);
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map([project_id], |row| {
            Ok(Session {
                id: row.get(0)?,
                project_id: row.get(1)?,
                agent_id: row.get(2)?,
                provider: row.get(3)?,
                provider_session_id: row.get(4)?,
                created_at: row.get(5)?,
                last_activity: row.get(6)?,
                status: SessionStatus::Active, // Default status
                metadata: row.get(8)?,
                expires_at: row.get(9)?,
                session_type: SessionType::Chat, // Default type
            })
        })?;
        
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }
}

impl Repository<Session, String> for SessionRepository {
    fn find_by_id(&self, id: String) -> Result<Option<Session>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, agent_id, provider, provider_session_id, created_at, last_activity, status, metadata, expires_at, session_type FROM sessions WHERE id = ?1")?;
        let mut rows = stmt.query_map([&id], |row| {
            Ok(Session {
                id: row.get(0)?,
                project_id: row.get(1)?,
                agent_id: row.get(2)?,
                provider: row.get(3)?,
                provider_session_id: row.get(4)?,
                created_at: row.get(5)?,
                last_activity: row.get(6)?,
                status: SessionStatus::Active, // Default status
                metadata: row.get(8)?,
                expires_at: row.get(9)?,
                session_type: SessionType::Chat, // Default type
            })
        })?;
        
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
    
    fn find_all(&self) -> Result<Vec<Session>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, agent_id, provider, provider_session_id, created_at, last_activity, status, metadata, expires_at, session_type FROM sessions ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                project_id: row.get(1)?,
                agent_id: row.get(2)?,
                provider: row.get(3)?,
                provider_session_id: row.get(4)?,
                created_at: row.get(5)?,
                last_activity: row.get(6)?,
                status: SessionStatus::Active, // Default status
                metadata: row.get(8)?,
                expires_at: row.get(9)?,
                session_type: SessionType::Chat, // Default type
            })
        })?;
        
        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(row?);
        }
        Ok(sessions)
    }
    
    fn create(&self, session: &Session) -> Result<String, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("INSERT INTO sessions (id, project_id, agent_id, provider, provider_session_id, created_at, last_activity, status, metadata, expires_at, session_type) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)")?;
        
        let provider_session_id = session.provider_session_id.as_ref().map(|s| s.as_str()).unwrap_or("");
        let last_activity = session.last_activity.as_ref().map(|s| s.as_str()).unwrap_or("");
        let metadata = session.metadata.as_ref().map(|s| s.as_str()).unwrap_or("");
        let expires_at = session.expires_at.as_ref().map(|s| s.as_str()).unwrap_or("");
        let status = "Active";
        let session_type = "Chat";
        
        stmt.execute([&session.id, &session.project_id, &session.agent_id, &session.provider, provider_session_id, &session.created_at, last_activity, status, metadata, expires_at, session_type])?;
        Ok(session.id.clone())
    }
    
    fn update(&self, session: &Session) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("UPDATE sessions SET project_id = ?1, agent_id = ?2, provider = ?3, provider_session_id = ?4, created_at = ?5, last_activity = ?6, status = ?7, metadata = ?8, expires_at = ?9, session_type = ?10 WHERE id = ?11")?;
        
        let provider_session_id = session.provider_session_id.as_ref().map(|s| s.as_str()).unwrap_or("");
        let last_activity = session.last_activity.as_ref().map(|s| s.as_str()).unwrap_or("");
        let metadata = session.metadata.as_ref().map(|s| s.as_str()).unwrap_or("");
        let expires_at = session.expires_at.as_ref().map(|s| s.as_str()).unwrap_or("");
        let status = "Active";
        let session_type = "Chat";
        
        stmt.execute([&session.project_id, &session.agent_id, &session.provider, provider_session_id, &session.created_at, last_activity, status, metadata, expires_at, session_type, &session.id])?;
        Ok(())
    }
    
    fn delete(&self, id: String) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("DELETE FROM sessions WHERE id = ?1")?;
        stmt.execute([&id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use db::now_iso8601_utc;
    
    fn setup_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        
        // Create sessions table
        conn.execute(
            "CREATE TABLE sessions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                agent_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                provider_session_id TEXT,
                created_at TEXT NOT NULL,
                last_activity TEXT,
                status TEXT NOT NULL,
                metadata TEXT,
                expires_at TEXT,
                session_type TEXT NOT NULL
            )",
            [],
        ).expect("Failed to create sessions table");
        
        Arc::new(Mutex::new(conn))
    }
    
    #[test]
    fn test_create_session() {
        let conn = setup_test_db();
        let repo = SessionRepository::new(conn);
        
        let session = Session {
            id: "test-1".to_string(),
            project_id: "project-1".to_string(),
            agent_id: "agent-1".to_string(),
            provider: "gemini".to_string(),
            provider_session_id: Some("provider-1".to_string()),
            created_at: now_iso8601_utc(),
            last_activity: None,
            status: SessionStatus::Active,
            metadata: None,
            expires_at: None,
            session_type: SessionType::Chat,
        };
        
        let result = repo.create(&session);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-1");
    }
    
    #[test]
    fn test_find_by_project_id() {
        let conn = setup_test_db();
        let repo = SessionRepository::new(conn);
        
        let session = Session {
            id: "test-1".to_string(),
            project_id: "project-1".to_string(),
            agent_id: "agent-1".to_string(),
            provider: "gemini".to_string(),
            provider_session_id: Some("provider-1".to_string()),
            created_at: now_iso8601_utc(),
            last_activity: None,
            status: SessionStatus::Active,
            metadata: None,
            expires_at: None,
            session_type: SessionType::Chat,
        };
        
        repo.create(&session).expect("Failed to create session");
        
        let found = repo.find_by_project_id("project-1").expect("Failed to find sessions");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].agent_id, "agent-1");
    }
}
