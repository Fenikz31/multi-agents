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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use db::now_iso8601_utc;
    
    fn setup_test_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        
        // Create agents table
        conn.execute(
            "CREATE TABLE agents (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                system_prompt TEXT NOT NULL,
                allowed_tools TEXT NOT NULL,
                created_at TEXT NOT NULL,
                UNIQUE(project_id, name)
            )",
            [],
        ).expect("Failed to create agents table");
        
        Arc::new(Mutex::new(conn))
    }
    
    #[test]
    fn test_create_agent() {
        let conn = setup_test_db();
        let repo = AgentRepository::new(conn);
        
        let agent = Agent {
            id: "test-1".to_string(),
            project_id: "project-1".to_string(),
            name: "backend1".to_string(),
            role: "backend".to_string(),
            provider: "gemini".to_string(),
            model: "2.0".to_string(),
            system_prompt: "You are a backend developer".to_string(),
            allowed_tools: vec!["tool1".to_string(), "tool2".to_string()],
        };
        
        let result = repo.create(&agent);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-1");
    }
    
    #[test]
    fn test_find_by_project_id() {
        let conn = setup_test_db();
        let repo = AgentRepository::new(conn);
        
        let agent = Agent {
            id: "test-1".to_string(),
            project_id: "project-1".to_string(),
            name: "backend1".to_string(),
            role: "backend".to_string(),
            provider: "gemini".to_string(),
            model: "2.0".to_string(),
            system_prompt: "You are a backend developer".to_string(),
            allowed_tools: vec![],
        };
        
        repo.create(&agent).expect("Failed to create agent");
        
        let found = repo.find_by_project_id("project-1").expect("Failed to find agents");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "backend1");
    }
    
    #[test]
    fn test_find_by_role() {
        let conn = setup_test_db();
        let repo = AgentRepository::new(conn);
        
        let agent = Agent {
            id: "test-1".to_string(),
            project_id: "project-1".to_string(),
            name: "backend1".to_string(),
            role: "backend".to_string(),
            provider: "gemini".to_string(),
            model: "2.0".to_string(),
            system_prompt: "You are a backend developer".to_string(),
            allowed_tools: vec![],
        };
        
        repo.create(&agent).expect("Failed to create agent");
        
        let found = repo.find_by_role("project-1", "backend").expect("Failed to find agents by role");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].role, "backend");
    }
}
