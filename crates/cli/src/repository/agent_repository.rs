//! Agent repository implementation
//! 
//! Handles all database operations related to agents.

use std::error::Error;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use db::Agent;
use super::Repository;

/// Repository for agent data operations
pub struct AgentRepository {
    conn: Arc<Mutex<Connection>>,
}

impl AgentRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
    
    /// Find agents by project ID
    pub fn find_by_project_id(&self, project_id: &str) -> Result<Vec<Agent>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, name, role, provider, model, system_prompt, allowed_tools_json FROM agents WHERE project_id = ?1")?;
        let rows = stmt.query_map([project_id], |row| {
            Ok(Agent {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                role: row.get(3)?,
                provider: row.get(4)?,
                model: row.get(5)?,
                system_prompt: row.get(6)?,
                allowed_tools: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_else(|_| vec![]),
            })
        })?;
        
        let mut agents = Vec::new();
        for row in rows {
            agents.push(row?);
        }
        Ok(agents)
    }
    
    /// Find agents by role
    pub fn find_by_role(&self, project_id: &str, role: &str) -> Result<Vec<Agent>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, name, role, provider, model, system_prompt, allowed_tools_json FROM agents WHERE project_id = ?1 AND role = ?2")?;
        let rows = stmt.query_map([project_id, role], |row| {
            Ok(Agent {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                role: row.get(3)?,
                provider: row.get(4)?,
                model: row.get(5)?,
                system_prompt: row.get(6)?,
                allowed_tools: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_else(|_| vec![]),
            })
        })?;
        
        let mut agents = Vec::new();
        for row in rows {
            agents.push(row?);
        }
        Ok(agents)
    }
    
    /// Find agent by name within project
    pub fn find_by_name(&self, project_id: &str, name: &str) -> Result<Option<Agent>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, name, role, provider, model, system_prompt, allowed_tools_json FROM agents WHERE project_id = ?1 AND name = ?2")?;
        let mut rows = stmt.query_map([project_id, name], |row| {
            Ok(Agent {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                role: row.get(3)?,
                provider: row.get(4)?,
                model: row.get(5)?,
                system_prompt: row.get(6)?,
                allowed_tools: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_else(|_| vec![]),
            })
        })?;
        
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
}

impl Repository<Agent, String> for AgentRepository {
    fn find_by_id(&self, id: String) -> Result<Option<Agent>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, name, role, provider, model, system_prompt, allowed_tools_json FROM agents WHERE id = ?1")?;
        let mut rows = stmt.query_map([&id], |row| {
            Ok(Agent {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                role: row.get(3)?,
                provider: row.get(4)?,
                model: row.get(5)?,
                system_prompt: row.get(6)?,
                allowed_tools: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_else(|_| vec![]),
            })
        })?;
        
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
    
    fn find_all(&self) -> Result<Vec<Agent>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, project_id, name, role, provider, model, system_prompt, allowed_tools_json FROM agents ORDER BY id DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Agent {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
                role: row.get(3)?,
                provider: row.get(4)?,
                model: row.get(5)?,
                system_prompt: row.get(6)?,
                allowed_tools: serde_json::from_str(&row.get::<_, String>(7)?).unwrap_or_else(|_| vec![]),
            })
        })?;
        
        let mut agents = Vec::new();
        for row in rows {
            agents.push(row?);
        }
        Ok(agents)
    }
    
    fn create(&self, agent: &Agent) -> Result<String, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("INSERT INTO agents (id, project_id, name, role, provider, model, system_prompt, allowed_tools_json) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)")?;
        let allowed_tools_json = serde_json::to_string(&agent.allowed_tools)?;
        stmt.execute([&agent.id, &agent.project_id, &agent.name, &agent.role, &agent.provider, &agent.model, &agent.system_prompt, &allowed_tools_json])?;
        Ok(agent.id.clone())
    }
    
    fn update(&self, agent: &Agent) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("UPDATE agents SET project_id = ?1, name = ?2, role = ?3, provider = ?4, model = ?5, system_prompt = ?6, allowed_tools_json = ?7 WHERE id = ?8")?;
        let allowed_tools_json = serde_json::to_string(&agent.allowed_tools)?;
        stmt.execute([&agent.project_id, &agent.name, &agent.role, &agent.provider, &agent.model, &agent.system_prompt, &allowed_tools_json, &agent.id])?;
        Ok(())
    }
    
    fn delete(&self, id: String) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("DELETE FROM agents WHERE id = ?1")?;
        stmt.execute([&id])?;
        Ok(())
    }
}

