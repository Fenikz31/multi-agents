//! Database commands implementation

use db::{open_or_create_db, insert_project, insert_agent, find_project_id, IdOrName};
use crate::utils::{default_db_path, looks_like_uuid, exit_with};

/// Run database initialization command
pub fn run_db_init(db_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let binding;
    let path = match db_path { Some(p) => p, None => { binding = default_db_path(); &binding } };
    match open_or_create_db(path) {
        Ok(_) => { println!("OK: db initialized"); Ok(()) }
        Err(e) => exit_with(7, format!("db: {}", e)),
    }
}

/// Run project add command
pub fn run_project_add(name: &str, db_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let binding;
    let path = match db_path { Some(p) => p, None => { binding = default_db_path(); &binding } };
    let conn = match open_or_create_db(path) { Ok(c) => c, Err(e) => return exit_with(7, format!("db: {}", e)) };
    match insert_project(&conn, name) {
        Ok(p) => { println!("project_id={} name={}", p.id, p.name); Ok(()) }
        Err(db::DbError::InvalidInput(e)) => exit_with(2, format!("project: {}", e)),
        Err(e) => exit_with(7, format!("project: {}", e)),
    }
}

/// Run agent add command
pub fn run_agent_add(project_sel: &str, name: &str, role: &str, provider: &str, model: &str, allowed_tool: &[String], system_prompt: &str, db_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let binding;
    let path = match db_path { Some(p) => p, None => { binding = default_db_path(); &binding } };
    let conn = match open_or_create_db(path) { Ok(c) => c, Err(e) => return exit_with(7, format!("db: {}", e)) };
    let project_id = match find_project_id(&conn, if looks_like_uuid(project_sel) { IdOrName::Id(project_sel) } else { IdOrName::Name(project_sel) })? {
        Some(id) => id,
        None => return exit_with(2, format!("project not found: {}", project_sel)),
    };
    match insert_agent(&conn, &project_id, name, role, provider, model, allowed_tool, system_prompt) {
        Ok(a) => { println!("agent_id={} project_id={} name={}", a.id, a.project_id, a.name); Ok(()) }
        Err(db::DbError::InvalidInput(e)) => exit_with(2, format!("agent: {}", e)),
        Err(e) => exit_with(7, format!("agent: {}", e)),
    }
}
