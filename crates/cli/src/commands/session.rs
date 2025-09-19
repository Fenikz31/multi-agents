//! Session management commands

use std::fs;
use config_model::{parse_project_yaml, parse_providers_yaml};
use db::{
    open_or_create_db, find_project_id, IdOrName, ClaudeSessionManager, CursorSessionManager, 
    GeminiSessionManager, SessionManager, list_sessions, SessionFilters, SessionStatus, 
    cleanup_repl_sessions, find_session
};
use rusqlite::params;
use std::time::{Duration, Instant};
use crate::cli::commands::Format;
use crate::utils::{resolve_config_paths, handle_missing_config, default_db_path, short_id, exit_with};
use crate::utils::timeouts::run_with_timeout;

/// Run session start command
pub fn run_session_start(project_path_opt: Option<&str>, providers_path_opt: Option<&str>, agent_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (project_path, providers_path) = match resolve_config_paths(project_path_opt, providers_path_opt) {
        Ok(p) => p,
        Err(msg) => return handle_missing_config(msg),
    };
    let proj_s = fs::read_to_string(&project_path)?;
    let prov_s = fs::read_to_string(&providers_path)?;
    let project = parse_project_yaml(&proj_s).map_err(|e| format!("project: {}", e))
        .or_else(|e| exit_with(2, e))?;
    let providers = parse_providers_yaml(&prov_s).map_err(|e| format!("providers: {}", e))
        .or_else(|e| exit_with(2, e))?;
    let agent = match project.agents.iter().find(|a| a.name == agent_name) {
        Some(a) => a,
        None => return exit_with(2, format!("unknown agent: {}", agent_name)),
    };
    let provider_key = &agent.provider;
    let tpl = match providers.providers.get(provider_key) {
        Some(t) => t,
        None => return exit_with(3, format!("provider not found: {}", provider_key)),
    };
    let conv_id = if provider_key.starts_with("cursor") {
        // create chat if args available
        if let Some(create_args) = &tpl.create_chat_args {
            let args: Vec<String> = create_args.iter()
                .map(|a| a.replace("{system_prompt}", &agent.system_prompt))
                .collect();
            match run_with_timeout(&tpl.cmd, &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(), Duration::from_millis(5000)) {
                Ok((_code, out, err)) => {
                    let text = if !out.trim().is_empty() { out } else { err };
                    // naive: take last non-empty line as chat_id
                    let id = text.lines().filter(|l| !l.trim().is_empty()).last().unwrap_or("").trim().to_string();
                    if id.is_empty() { return exit_with(4, "cursor create-chat returned empty id".into()); }
                    id
                }
                Err(e) => {
                    if e == "timeout" { return exit_with(5, "cursor create-chat timeout".into()); }
                    return exit_with(4, format!("cursor create-chat error: {}", e));
                }
            }
        } else {
            return exit_with(2, "cursor provider missing create_chat_args".into());
        }
    } else if provider_key == "claude" {
        format!("valid_session_{}", short_id())
    } else if provider_key == "gemini" {
        format!("valid_context_{}", short_id())
    } else {
        short_id()
    };
    // Save session to database
    let db_path = default_db_path();
    let conn = open_or_create_db(&db_path)?;
    
    // Find project and agent IDs
    let project_id = find_project_id(&conn, IdOrName::Name(&project.project))?
        .ok_or_else(|| format!("Project not found: {}", project.project))?;
    
    let agent_id = conn.query_row(
        "SELECT id FROM agents WHERE project_id = ?1 AND name = ?2",
        &[&project_id, &agent_name.to_string()],
        |row| Ok(row.get::<_, String>(0)?)
    )?;
    
    // Create appropriate SessionManager and session
    let manager: Box<dyn SessionManager> = match provider_key.as_str() {
        "claude" => Box::new(ClaudeSessionManager::new(conn)),
        "cursor-agent" => Box::new(CursorSessionManager::new(conn)),
        "gemini" => Box::new(GeminiSessionManager::new(conn)),
        _ => return exit_with(2, format!("Unsupported provider: {}", provider_key)),
    };
    
    // Create session with provider_session_id if available
    let provider_session_id = if provider_key.starts_with("cursor") {
        Some(conv_id.as_str())
    } else if provider_key == "claude" || provider_key == "gemini" {
        Some(conv_id.as_str())
    } else {
        None
    };
    
    match manager.create_session(&project_id, &agent_id, provider_key, provider_session_id) {
        Ok(session) => {
            println!("conversation_id={}", session.id);
        }
        Err(e) => {
            return exit_with(7, format!("Failed to create session: {}", e));
        }
    }
    
    Ok(())
}

/// Run session list command
pub fn run_session_list(project_path_opt: Option<&str>, project_name_opt: Option<&str>, agent_filter: Option<&str>, provider_filter: Option<&str>, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let (project_path, _providers_path) = match resolve_config_paths(project_path_opt, None) {
        Ok(p) => p,
        Err(msg) => return handle_missing_config(msg),
    };
    
    // Determine project name (default to current directory name)
    let project_name = if let Some(name) = project_name_opt {
        name.to_string()
    } else {
        // Get current directory name as default project
        std::env::current_dir()?
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Cannot determine current directory name")?
            .to_string()
    };
    
    let db_path = default_db_path();
    let conn = open_or_create_db(&db_path)?;
    
    // Find project ID
    let project_id = find_project_id(&conn, IdOrName::Name(&project_name))?
        .ok_or_else(|| format!("Project not found: {}", project_name))?;
    
    // Build filters
    let mut filters = SessionFilters {
        project_id: Some(project_id.clone()),
        agent_id: None,
        provider: provider_filter.map(|s| s.to_string()),
        status: Some(SessionStatus::Active),
        session_type: None, // Include both chat and repl sessions
        limit: Some(50), // Default limit
        offset: Some(0),
    };
    
    // If agent filter provided, find agent ID
    if let Some(agent_name) = agent_filter {
        let proj_s = fs::read_to_string(&project_path)?;
        let project = parse_project_yaml(&proj_s).map_err(|e| format!("project: {}", e))?;
        let _agent = project.agents.iter().find(|a| a.name == agent_name)
            .ok_or_else(|| format!("unknown agent: {}", agent_name))?;
        
        // Find agent ID in database
        let agent_id = conn.query_row(
            "SELECT id FROM agents WHERE project_id = ?1 AND name = ?2",
            &[&project_id, &agent_name.to_string()],
            |row| Ok(row.get::<_, String>(0)?)
        )?;
        filters.agent_id = Some(agent_id);
    }
    
    // List sessions
    let sessions = list_sessions(&conn, filters)?;
    
    match format {
        Format::Text => {
            if sessions.is_empty() {
                println!("No sessions found for project '{}'", project_name);
                return Ok(());
            }
            
            println!("Sessions for project '{}':", project_name);
            println!("{:<36} {:<12} {:<12} {:<8} {:<20}", "ID", "Agent", "Provider", "Status", "Created");
            println!("{}", "-".repeat(88));
            
            for session in sessions {
                let created = session.created_at.split('T').next().unwrap_or(&session.created_at);
                println!("{:<36} {:<12} {:<12} {:<8} {:<20}", 
                    session.id, 
                    session.agent_id, 
                    session.provider, 
                    session.status, 
                    created
                );
            }
        }
        Format::Json => {
            let json = serde_json::json!({
                "project": project_name,
                "sessions": sessions.iter().map(|s| serde_json::json!({
                    "id": s.id,
                    "agent_id": s.agent_id,
                    "provider": s.provider,
                    "status": s.status.to_string(),
                    "created_at": s.created_at,
                    "last_activity": s.last_activity,
                    "provider_session_id": s.provider_session_id
                })).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }
    
    Ok(())
}

/// Run session resume command
pub fn run_session_resume(conversation_id: &str, timeout_ms: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = default_db_path();
    let conn = open_or_create_db(&db_path)?;
    
    // Find session
    let session = match find_session(&conn, conversation_id)? {
        Some(s) => s,
        None => return exit_with(2, format!("Session not found: {}", conversation_id)),
    };
    
    // Create appropriate SessionManager
    let manager: Box<dyn SessionManager> = match session.provider.as_str() {
        "claude" => Box::new(ClaudeSessionManager::new(conn)),
        "cursor-agent" => Box::new(CursorSessionManager::new(conn)),
        "gemini" => Box::new(GeminiSessionManager::new(conn)),
        _ => return exit_with(2, format!("Unsupported provider: {}", session.provider)),
    };
    
    // Resume session with timeout
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(5000));
    let start = Instant::now();
    
    match manager.resume_session(conversation_id) {
        Ok(context) => {
            let elapsed = start.elapsed();
            if elapsed > timeout {
                return exit_with(5, "Session resume timeout".into());
            }
            
            println!("Session resumed successfully");
            println!("conversation_id={}", context.session.id);
            if let Some(provider_id) = context.provider_session_id {
                println!("provider_session_id={}", provider_id);
            }
            println!("is_resumable={}", context.is_resumable);
        }
        Err(e) => {
            let elapsed = start.elapsed();
            if elapsed > timeout {
                return exit_with(5, "Session resume timeout".into());
            }
            return exit_with(2, format!("Failed to resume session: {}", e));
        }
    }
    
    Ok(())
}

/// Run session cleanup command
pub fn run_session_cleanup(_project_path_opt: Option<&str>, dry_run: bool, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = default_db_path();
    let conn = open_or_create_db(&db_path)?;
    
    // Find expired sessions (older than 24 hours with no activity)
    let cutoff_time = {
        let now = std::time::SystemTime::now();
        let cutoff = now - std::time::Duration::from_secs(24 * 60 * 60); // 24 hours
        // Convert to ISO 8601 format like the database uses
        let cutoff_duration = cutoff.duration_since(std::time::UNIX_EPOCH).unwrap();
        format!("{}.{:09}Z", 
            chrono::DateTime::<chrono::Utc>::from_timestamp(cutoff_duration.as_secs() as i64, 0)
                .unwrap()
                .format("%Y-%m-%dT%H:%M:%S"),
            cutoff_duration.subsec_nanos()
        )
    };
    
    // Clean up REPL sessions (Issue #36)
    let repl_cleaned = if dry_run {
        // Query REPL sessions that would be cleaned up
        let mut stmt = conn.prepare(
            "SELECT id, project_id, agent_id, provider, created_at, last_activity, type
             FROM sessions 
             WHERE type = 'repl' 
             AND (last_activity < ?1 OR created_at < ?1) 
             AND status = 'active'"
        )?;
        
        let session_iter = stmt.query_map(params![&cutoff_time], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "project_id": row.get::<_, String>(1)?,
                "agent_id": row.get::<_, String>(2)?,
                "provider": row.get::<_, String>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "last_activity": row.get::<_, Option<String>>(5)?,
                "type": row.get::<_, String>(6)?
            }))
        })?;
        
        session_iter.collect::<Result<Vec<_>, _>>()?
    } else {
        // Actually clean up REPL sessions
        let cleaned_count = cleanup_repl_sessions(&conn)?;
        
        vec![serde_json::json!({
            "repl_cleaned_count": cleaned_count,
            "cutoff_time": cutoff_time
        })]
    };
    
    let expired_sessions = if dry_run {
        // Query expired sessions without deleting
        let mut stmt = conn.prepare(
            "SELECT id, project_id, agent_id, provider, created_at, last_activity, type
             FROM sessions 
             WHERE (last_activity IS NULL OR last_activity < ?1) 
             AND created_at < ?1
             AND type = 'chat'"
        )?;
        
        let session_iter = stmt.query_map(params![&cutoff_time], |row| {
            Ok(serde_json::json!({
                "id": row.get::<_, String>(0)?,
                "project_id": row.get::<_, String>(1)?,
                "agent_id": row.get::<_, String>(2)?,
                "provider": row.get::<_, String>(3)?,
                "created_at": row.get::<_, String>(4)?,
                "last_activity": row.get::<_, Option<String>>(5)?,
                "type": row.get::<_, String>(6)?
            }))
        })?;
        
        session_iter.collect::<Result<Vec<_>, _>>()?
    } else {
        // Actually delete expired chat sessions
        let deleted_count = conn.execute(
            "DELETE FROM sessions 
             WHERE (last_activity IS NULL OR last_activity < ?1) 
             AND created_at < ?1
             AND type = 'chat'",
            params![&cutoff_time]
        )?;
        
        vec![serde_json::json!({
            "chat_deleted_count": deleted_count,
            "cutoff_time": cutoff_time
        })]
    };
    
    match format {
        Format::Text => {
            if dry_run {
                println!("Dry run: Found {} expired chat sessions", expired_sessions.len());
                for session in &expired_sessions {
                    println!("  - {} ({}) [chat]", session["id"], session["provider"]);
                }
                println!("Dry run: Found {} expired REPL sessions", repl_cleaned.len());
                for session in &repl_cleaned {
                    println!("  - {} ({}) [repl]", session["id"], session["provider"]);
                }
            } else {
                let chat_result = expired_sessions.first();
                let repl_result = repl_cleaned.first();
                let chat_count = chat_result.and_then(|r| r["chat_deleted_count"].as_u64()).unwrap_or(0);
                let repl_count = repl_result.and_then(|r| r["repl_cleaned_count"].as_u64()).unwrap_or(0);
                println!("Cleaned up {} expired chat sessions", chat_count);
                println!("Marked {} REPL sessions as expired", repl_count);
            }
        }
        Format::Json => {
            let output = if dry_run {
                serde_json::json!({
                    "dry_run": true,
                    "expired_chat_sessions": expired_sessions,
                    "expired_repl_sessions": repl_cleaned,
                    "cutoff_time": cutoff_time
                })
            } else {
                serde_json::json!({
                    "dry_run": false,
                    "chat_result": expired_sessions.first().unwrap_or(&serde_json::Value::Null),
                    "repl_result": repl_cleaned.first().unwrap_or(&serde_json::Value::Null)
                })
            };
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }
    
    Ok(())
}
