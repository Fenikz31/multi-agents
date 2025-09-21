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

