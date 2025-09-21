//! Repository pattern implementation for data access layer
//! 
//! This module provides a clean abstraction over the SQLite database operations,
//! implementing the Repository pattern to separate data access logic from business logic.

pub mod project_repository;
pub mod agent_repository;
pub mod session_repository;

use std::error::Error;
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

/// Generic repository trait for common database operations
pub trait Repository<T, ID> {
    /// Find entity by ID
    fn find_by_id(&self, id: ID) -> Result<Option<T>, Box<dyn Error>>;
    
    /// Find all entities
    fn find_all(&self) -> Result<Vec<T>, Box<dyn Error>>;
    
    /// Create new entity
    fn create(&self, entity: &T) -> Result<ID, Box<dyn Error>>;
    
    /// Update existing entity
    fn update(&self, entity: &T) -> Result<(), Box<dyn Error>>;
    
    /// Delete entity by ID
    fn delete(&self, id: ID) -> Result<(), Box<dyn Error>>;
}

/// Repository manager for coordinating all repositories
pub struct RepositoryManager {
    pub projects: project_repository::ProjectRepository,
    pub agents: agent_repository::AgentRepository,
    pub sessions: session_repository::SessionRepository,
}

impl RepositoryManager {
    /// Create new repository manager with database connection
    pub fn new(conn: Connection) -> Self {
        let shared_conn = Arc::new(Mutex::new(conn));
        Self {
            projects: project_repository::ProjectRepository::new(shared_conn.clone()),
            agents: agent_repository::AgentRepository::new(shared_conn.clone()),
            sessions: session_repository::SessionRepository::new(shared_conn),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    
    #[test]
    fn test_repository_manager_creation() {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
        let _manager = RepositoryManager::new(conn);
        // Test passes if no panic occurs
    }
}
