//! Project repository implementation
//! 
//! Handles all database operations related to projects.

use std::error::Error;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use db::Project;
use super::Repository;

/// Repository for project data operations
pub struct ProjectRepository {
    conn: Arc<Mutex<Connection>>,
}

impl ProjectRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
    
    /// Find project by name
    pub fn find_by_name(&self, name: &str) -> Result<Option<Project>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM projects WHERE name = ?1")?;
        let mut rows = stmt.query_map([name], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
    
    /// Find projects by pattern
    pub fn find_by_pattern(&self, pattern: &str) -> Result<Vec<Project>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM projects WHERE name LIKE ?1")?;
        let pattern = format!("%{}%", pattern);
        let rows = stmt.query_map([&pattern], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        
        let mut projects = Vec::new();
        for row in rows {
            projects.push(row?);
        }
        Ok(projects)
    }
}

impl Repository<Project, String> for ProjectRepository {
    fn find_by_id(&self, id: String) -> Result<Option<Project>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM projects WHERE id = ?1")?;
        let mut rows = stmt.query_map([&id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
    
    fn find_all(&self) -> Result<Vec<Project>, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM projects ORDER BY id DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        
        let mut projects = Vec::new();
        for row in rows {
            projects.push(row?);
        }
        Ok(projects)
    }
    
    fn create(&self, project: &Project) -> Result<String, Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("INSERT INTO projects (id, name) VALUES (?1, ?2)")?;
        stmt.execute([&project.id, &project.name])?;
        Ok(project.id.clone())
    }
    
    fn update(&self, project: &Project) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("UPDATE projects SET name = ?1 WHERE id = ?2")?;
        stmt.execute([&project.name, &project.id])?;
        Ok(())
    }
    
    fn delete(&self, id: String) -> Result<(), Box<dyn Error>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("DELETE FROM projects WHERE id = ?1")?;
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
        
        // Create projects table
        conn.execute(
            "CREATE TABLE projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        ).expect("Failed to create projects table");
        
        Arc::new(Mutex::new(conn))
    }
    
    #[test]
    fn test_create_project() {
        let conn = setup_test_db();
        let repo = ProjectRepository::new(conn);
        
        let project = Project {
            id: "test-1".to_string(),
            name: "test-project".to_string(),
        };
        
        let result = repo.create(&project);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-1");
    }
    
    #[test]
    fn test_find_by_id() {
        let conn = setup_test_db();
        let repo = ProjectRepository::new(conn);
        
        let project = Project {
            id: "test-1".to_string(),
            name: "test-project".to_string(),
        };
        
        repo.create(&project).expect("Failed to create project");
        
        let found = repo.find_by_id("test-1".to_string()).expect("Failed to find project");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test-project");
    }
    
    #[test]
    fn test_find_by_name() {
        let conn = setup_test_db();
        let repo = ProjectRepository::new(conn);
        
        let project = Project {
            id: "test-1".to_string(),
            name: "test-project".to_string(),
        };
        
        repo.create(&project).expect("Failed to create project");
        
        let found = repo.find_by_name("test-project").expect("Failed to find project");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, "test-1");
    }
    
    #[test]
    fn test_find_all() {
        let conn = setup_test_db();
        let repo = ProjectRepository::new(conn);
        
        let project1 = Project {
            id: "test-1".to_string(),
            name: "test-project-1".to_string(),
        };
        
        let project2 = Project {
            id: "test-2".to_string(),
            name: "test-project-2".to_string(),
        };
        
        repo.create(&project1).expect("Failed to create project 1");
        repo.create(&project2).expect("Failed to create project 2");
        
        let all = repo.find_all().expect("Failed to find all projects");
        assert_eq!(all.len(), 2);
    }
}
