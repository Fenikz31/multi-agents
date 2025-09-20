//! Broadcast command implementation

use std::fs;
use std::time::Duration;
use config_model::{parse_project_yaml, parse_providers_yaml};
use db::{open_or_create_db, find_project_id, IdOrName, sync_project_from_config};
use crate::cli::commands::Format;
use crate::utils::{
    resolve_config_paths, handle_missing_config, DEFAULT_AGENT_TIMEOUT_MS, 
    exit_with
};
use crate::broadcast::{BroadcastManager, BroadcastMode, BroadcastTarget};
use crate::logging::log_ndjson;
use indicatif::{ProgressBar, ProgressStyle};

/// Run broadcast oneshot command
pub fn run_broadcast_oneshot(
    project_file: Option<&str>,
    providers_file: Option<&str>,
    project_name: Option<&str>,
    to: &str,
    message: &str,
    timeout_ms: Option<u64>,
    format: Format,
    progress: bool
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    
    // Resolve config paths
    let (project_path, providers_path) = match resolve_config_paths(project_file, providers_file) {
        Ok(p) => p,
        Err(msg) => return handle_missing_config(msg),
    };
    
    // Load configurations
    let proj_s = fs::read_to_string(&project_path)?;
    let prov_s = fs::read_to_string(&providers_path)?;
    let project = parse_project_yaml(&proj_s).map_err(|e| format!("project: {}", e))?;
    let _providers = parse_providers_yaml(&prov_s).map_err(|e| format!("providers: {}", e))?;
    
    // Determine project name
    let project_name = project_name.unwrap_or(&project.project);
    
    // Sync project to database
    let db_path = crate::utils::default_db_path();
    let conn = open_or_create_db(&db_path)?;
    sync_project_from_config(&conn, &project)
        .map_err(|e| format!("Failed to sync project: {}", e))?;
    
    // Get project ID
    let project_id = match find_project_id(&conn, IdOrName::Name(project_name))? {
        Some(pid) => pid,
        None => return exit_with(2, format!("Project not found: {}", project_name)),
    };
    
    // Get agents from database
    let mut stmt = conn.prepare("SELECT id, name, role, provider, model FROM agents WHERE project_id = ?1")?;
    let agents: Vec<db::Agent> = stmt.query_map([&project_id], |row| {
        Ok(db::Agent {
            id: row.get(0)?,
            project_id: project_id.clone(),
            name: row.get(1)?,
            role: row.get(2)?,
            provider: row.get(3)?,
            model: row.get(4)?,
            system_prompt: String::new(),
            allowed_tools: vec![],
        })
    })?.collect::<Result<Vec<_>, _>>()?;
    
    // Parse target
    let target = BroadcastTarget::from_str(to)
        .map_err(|e| format!("Invalid target '{}': {}", to, e))?;
    
    // Resolve agents
    let agent_names = target.resolve_agents(&agents)
        .map_err(|e| format!("Target resolution failed: {}", e))?;
    
    if agent_names.is_empty() {
        return exit_with(2, format!("No agents found for target '{}'", to));
    }
    
    // Create broadcast manager
    let effective_timeout = timeout_ms.unwrap_or(DEFAULT_AGENT_TIMEOUT_MS).min(DEFAULT_AGENT_TIMEOUT_MS);
    let timeout = Duration::from_millis(effective_timeout);
    let manager = BroadcastManager::new(project_name.to_string(), timeout);
    
    // Convert agent names to role:agent format for broadcast
    let targets: Vec<String> = agent_names.iter().map(|name| {
        let agent = agents.iter().find(|a| a.name == *name).unwrap();
        format!("{}:{}", agent.role, agent.name)
    }).collect();
    
    // Create progress bar if enabled
    let pb = if progress { Some(make_progress_bar()) } else { None };
    
    // Execute broadcast
    let summary = manager.broadcast_to_targets(&targets, message, BroadcastMode::Oneshot)?;
    
    // Finish progress bar
    if let Some(pb) = pb { pb.finish_and_clear(); }
    
    // Log broadcast completion
    let duration_ms = start_time.elapsed().as_millis() as u64;
    log_ndjson(
        project_name, 
        "broadcast", 
        "multi-agents", 
        Some(manager.broadcast_id()), 
        "system", 
        "broadcast_complete", 
        None, 
        Some(if summary.is_success() { 0 } else { 1 }), 
        None
    );
    
    // Output results
    match format {
        Format::Text => {
            println!("Broadcast completed in {}ms", duration_ms);
            println!("Targets: {}, Successful: {}, Failed: {}", 
                     summary.total_targets, summary.successful, summary.failed);
            println!("Broadcast ID: {}", summary.broadcast_id);
            
            if !summary.is_success() {
                for result in &summary.results {
                    if !result.success {
                        println!("  {}: FAILED - {}", result.target, 
                                result.error.as_deref().unwrap_or("Unknown error"));
                    }
                }
            }
        }
        Format::Json => {
            let json_result = serde_json::json!({
                "broadcast_id": summary.broadcast_id,
                "duration_ms": duration_ms,
                "total_targets": summary.total_targets,
                "successful": summary.successful,
                "failed": summary.failed,
                "status": summary.status(),
                "results": summary.results
            });
            println!("{}", json_result);
        }
    }
    
    // Exit with appropriate code
    if summary.is_success() {
        Ok(())
    } else if summary.successful > 0 {
        exit_with(1, format!("Broadcast completed with {} failures", summary.failed))
    } else {
        exit_with(8, "All broadcast targets failed".to_string())
    }
}

/// Run broadcast repl command
pub fn run_broadcast_repl(
    project_file: Option<&str>,
    project_name: Option<&str>,
    to: &str,
    message: &str,
    timeout_ms: Option<u64>,
    format: Format,
    progress: bool
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    
    // Resolve config paths
    let (project_path, _) = match resolve_config_paths(project_file, None) {
        Ok(p) => p,
        Err(msg) => return handle_missing_config(msg),
    };
    
    // Load project configuration
    let proj_s = fs::read_to_string(&project_path)?;
    let project = parse_project_yaml(&proj_s).map_err(|e| format!("project: {}", e))?;
    
    // Determine project name
    let project_name = project_name.unwrap_or(&project.project);
    
    // Get agents from project config
    let agents: Vec<db::Agent> = project.agents.iter().map(|a| {
        db::Agent {
            id: format!("{}-{}", project_name, a.name),
            project_id: project_name.to_string(),
            name: a.name.clone(),
            role: a.role.clone(),
            provider: a.provider.clone(),
            model: a.model.clone(),
            system_prompt: a.system_prompt.clone(),
            allowed_tools: a.allowed_tools.clone(),
        }
    }).collect();
    
    // Parse target
    let target = BroadcastTarget::from_str(to)
        .map_err(|e| format!("Invalid target '{}': {}", to, e))?;
    
    // Resolve agents
    let agent_names = target.resolve_agents(&agents)
        .map_err(|e| format!("Target resolution failed: {}", e))?;
    
    if agent_names.is_empty() {
        return exit_with(2, format!("No agents found for target '{}'", to));
    }
    
    // Create broadcast manager
    let effective_timeout = timeout_ms.unwrap_or(DEFAULT_AGENT_TIMEOUT_MS).min(DEFAULT_AGENT_TIMEOUT_MS);
    let timeout = Duration::from_millis(effective_timeout);
    let manager = BroadcastManager::new(project_name.to_string(), timeout);
    
    // Convert agent names to role:agent format for broadcast
    let targets: Vec<String> = agent_names.iter().map(|name| {
        let agent = agents.iter().find(|a| a.name == *name).unwrap();
        format!("{}:{}", agent.role, agent.name)
    }).collect();
    
    // Create progress bar if enabled
    let pb = if progress { Some(make_progress_bar()) } else { None };
    
    // Execute broadcast
    let summary = manager.broadcast_to_targets(&targets, message, BroadcastMode::Repl)?;
    
    // Finish progress bar
    if let Some(pb) = pb { pb.finish_and_clear(); }
    
    // Log broadcast completion
    let duration_ms = start_time.elapsed().as_millis() as u64;
    log_ndjson(
        project_name, 
        "broadcast", 
        "multi-agents", 
        Some(manager.broadcast_id()), 
        "system", 
        "broadcast_complete", 
        None, 
        Some(if summary.is_success() { 0 } else { 1 }), 
        None
    );
    
    // Output results
    match format {
        Format::Text => {
            println!("Broadcast completed in {}ms", duration_ms);
            println!("Targets: {}, Successful: {}, Failed: {}", 
                     summary.total_targets, summary.successful, summary.failed);
            println!("Broadcast ID: {}", summary.broadcast_id);
            
            if !summary.is_success() {
                for result in &summary.results {
                    if !result.success {
                        println!("  {}: FAILED - {}", result.target, 
                                result.error.as_deref().unwrap_or("Unknown error"));
                    }
                }
            }
        }
        Format::Json => {
            let json_result = serde_json::json!({
                "broadcast_id": summary.broadcast_id,
                "duration_ms": duration_ms,
                "total_targets": summary.total_targets,
                "successful": summary.successful,
                "failed": summary.failed,
                "status": summary.status(),
                "results": summary.results
            });
            println!("{}", json_result);
        }
    }
    
    // Exit with appropriate code
    if summary.is_success() {
        Ok(())
    } else if summary.successful > 0 {
        exit_with(1, format!("Broadcast completed with {} failures", summary.failed))
    } else {
        exit_with(8, "All broadcast targets failed".to_string())
    }
}

/// Create progress bar for broadcast operations
fn make_progress_bar() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner} broadcasting {msg}").unwrap());
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_progress_bar_creation() {
        let pb = make_progress_bar();
        assert!(!pb.is_finished());
    }
}
