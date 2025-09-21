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

