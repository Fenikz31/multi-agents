//! Send command implementation

use std::fs;
use std::thread;
use std::time::Duration;
use config_model::{parse_project_yaml, parse_providers_yaml};
use db::{
    open_or_create_db, find_project_id, IdOrName, ClaudeSessionManager, CursorSessionManager, 
    GeminiSessionManager, SessionManager, find_session, now_iso8601_utc
};
use rusqlite::params;
use indicatif::{ProgressBar, ProgressStyle};
use crate::cli::commands::Format;
use crate::utils::{
    resolve_config_paths, handle_missing_config, default_db_path, DEFAULT_SEND_TIMEOUT_MS, 
    MAX_CONCURRENCY, short_id, uuid_v4_like, exit_with
};
use crate::utils::timeouts::run_with_timeout_streaming;
use crate::logging::log_ndjson;

/// Run send command
pub fn run_send(
    project_path_opt: Option<&str>, 
    providers_path_opt: Option<&str>, 
    to: &str, 
    message: &str, 
    session_id_opt: Option<&str>, 
    chat_id_opt: Option<&str>, 
    timeout_ms_flag: Option<u64>, 
    format: Format, 
    progress: bool
) -> Result<(), Box<dyn std::error::Error>> {
    let (project_path, providers_path) = match resolve_config_paths(project_path_opt, providers_path_opt) {
        Ok(p) => p,
        Err(msg) => return handle_missing_config(msg),
    };
    let proj_s = fs::read_to_string(&project_path)?;
    let prov_s = fs::read_to_string(&providers_path)?;
    let project = match parse_project_yaml(&proj_s) { Ok(p) => p, Err(e) => return exit_with(2, format!("project: {}", e)) };
    let providers = match parse_providers_yaml(&prov_s) { Ok(p) => p, Err(e) => return exit_with(2, format!("providers: {}", e)) };

    // Session management - sync project and agents to database
    let db_path = default_db_path();
    let conn = open_or_create_db(&db_path)?;
    match db::sync_project_from_config(&conn, &project) {
        Ok(_) => {}, // Project synchronized successfully
        Err(e) => return exit_with(7, format!("Failed to sync project: {}", e)),
    }

    // Resolve targets with session support
    let mut targets: Vec<&config_model::AgentConfig> = Vec::new();
    let mut session_contexts: Vec<Option<String>> = Vec::new();
    
    if to == "@all" {
        targets.extend(project.agents.iter());
        session_contexts.resize(targets.len(), None);
    } else if to.starts_with('@') {
        let role = &to[1..];
        targets.extend(project.agents.iter().filter(|a| a.role == role));
        session_contexts.resize(targets.len(), None);
    } else {
        // Check if 'to' is a conversation_id
        if let Some(session) = find_session(&conn, to)? {
            // Find the agent for this session
            if let Some(agent) = project.agents.iter().find(|a| {
                // Get agent_id from database
                match conn.query_row(
                    "SELECT id FROM agents WHERE project_id = ?1 AND name = ?2",
                    params![&session.project_id, &a.name],
                    |row| Ok(row.get::<_, String>(0)?)
                ) {
                    Ok(agent_id) => agent_id == session.agent_id,
                    Err(_) => false,
                }
            }) {
                targets.push(agent);
                session_contexts.push(Some(to.to_string()));
            } else {
                return exit_with(2, format!("send: session '{}' has no matching agent", to));
            }
        } else {
            // Try to find agent by name
            if let Some(agent) = project.agents.iter().find(|a| a.name == to) {
                targets.push(agent);
                session_contexts.push(None);
            } else {
                return exit_with(2, format!("send: no targets matched '{}'", to));
            }
        }
    }
    
    if targets.is_empty() { 
        return exit_with(2, format!("send: no targets matched '{}'", to)); 
    }

    // Auto-create session if conversation_id is absent, and fallback if status expired/invalid
    // Determine project_id once
    let project_id = match find_project_id(&conn, IdOrName::Name(&project.project))? {
        Some(pid) => pid,
        None => return exit_with(2, format!("Project not found: {}", project.project)),
    };
    for (i, agent) in targets.iter().enumerate() {
        // If a session was provided, ensure it's active; else create one
        if let Some(conv_id) = &session_contexts[i] {
            if let Some(existing) = find_session(&conn, conv_id)? {
                // If not active, create a fresh session
                if existing.status.to_string() != "active" {
                    // Lookup agent_id
                    let agent_id: String = conn.query_row(
                        "SELECT id FROM agents WHERE project_id = ?1 AND name = ?2",
                        params![&project_id, &agent.name],
                        |row| Ok(row.get::<_, String>(0)?)
                    )?;
                    // Create manager per provider
                    let conn_for_mgr = open_or_create_db(&db_path)?;
                    let manager: Box<dyn SessionManager> = match agent.provider.as_str() {
                        "claude" => Box::new(ClaudeSessionManager::new(conn_for_mgr)),
                        "cursor-agent" => Box::new(CursorSessionManager::new(open_or_create_db(&db_path)?)),
                        "gemini" => Box::new(GeminiSessionManager::new(open_or_create_db(&db_path)?)),
                        _ => return exit_with(2, format!("Unsupported provider: {}", agent.provider)),
                    };
                    let new_session = manager.create_session(&project_id, &agent_id, &agent.provider, None)
                        .map_err(|e| format!("Failed to create session: {}", e))?;
                    session_contexts[i] = Some(new_session.id);
                }
            } else {
                // Provided id not found -> create new
                let agent_id: String = conn.query_row(
                    "SELECT id FROM agents WHERE project_id = ?1 AND name = ?2",
                    params![&project_id, &agent.name],
                    |row| Ok(row.get::<_, String>(0)?)
                )?;
                let manager: Box<dyn SessionManager> = match agent.provider.as_str() {
                    "claude" => Box::new(ClaudeSessionManager::new(open_or_create_db(&db_path)?)),
                    "cursor-agent" => Box::new(CursorSessionManager::new(open_or_create_db(&db_path)?)),
                    "gemini" => Box::new(GeminiSessionManager::new(open_or_create_db(&db_path)?)),
                    _ => return exit_with(2, format!("Unsupported provider: {}", agent.provider)),
                };
                let new_session = manager.create_session(&project_id, &agent_id, &agent.provider, None)
                    .map_err(|e| format!("Failed to create session: {}", e))?;
                session_contexts[i] = Some(new_session.id);
            }
        } else {
            // No session provided -> create one now
            let agent_id: String = conn.query_row(
                "SELECT id FROM agents WHERE project_id = ?1 AND name = ?2",
                params![&project_id, &agent.name],
                |row| Ok(row.get::<_, String>(0)?)
            )?;
            let manager: Box<dyn SessionManager> = match agent.provider.as_str() {
                "claude" => Box::new(ClaudeSessionManager::new(open_or_create_db(&db_path)?)),
                "cursor-agent" => Box::new(CursorSessionManager::new(open_or_create_db(&db_path)?)),
                "gemini" => Box::new(GeminiSessionManager::new(open_or_create_db(&db_path)?)),
                _ => return exit_with(2, format!("Unsupported provider: {}", agent.provider)),
            };
            let new_session = manager.create_session(&project_id, &agent_id, &agent.provider, None)
                .map_err(|e| format!("Failed to create session: {}", e))?;
            session_contexts[i] = Some(new_session.id);
        }
    }

    // Execute with bounded concurrency
    let mut handles: Vec<std::thread::JoinHandle<i32>> = Vec::new();
    let mut results: Vec<i32> = Vec::new();
    let multi = targets.len() > 1;
    let per_timeout = timeout_ms_flag.unwrap_or(DEFAULT_SEND_TIMEOUT_MS);
    let pb = if progress { Some(make_pb()) } else { None };
    
    for (i, agent) in targets.iter().enumerate() {
        // batch if needed
        if handles.len() >= MAX_CONCURRENCY {
            let code = handles.remove(0).join().unwrap_or(1);
            results.push(code);
        }
        let provider_key = agent.provider.clone();
        let prov_cfg = providers.providers.get(&provider_key).cloned();
        let project_name = project.project.clone();
        let agent_role = agent.role.clone();
        let agent_allowed = agent.allowed_tools.clone();
        let agent_system = agent.system_prompt.clone();
        let message_owned = message.to_string();
        let session_id_owned = session_id_opt.map(|s| s.to_string());
        let chat_id_owned = chat_id_opt.map(|s| s.to_string());
        let print_header = multi;
        let pb_clone = pb.as_ref().map(|p| p.clone());
        
        // Get session context for this agent
        let conversation_id = session_contexts[i].clone();
        
        handles.push(thread::spawn(move || {
            match prov_cfg {
                Some(tpl) => run_oneshot_provider(
                    &project_name, &agent_role, &provider_key, &tpl,
                    &message_owned, &agent_system, &agent_allowed,
                    session_id_owned.as_deref(), chat_id_owned.as_deref(),
                    per_timeout,
                    print_header,
                    pb_clone,
                    conversation_id
                ),
                None => 3, // provider unavailable in config
            }
        }));
    }
    // join remaining
    for h in handles { results.push(h.join().unwrap_or(1)); }

    // derive overall exit code priority: 5 > 4 > 3 > 2 > 0
    let mut overall = 0;
    if results.iter().any(|&c| c == 5) { overall = 5; }
    else if results.iter().any(|&c| c == 4) { overall = 4; }
    else if results.iter().any(|&c| c == 3) { overall = 3; }
    else if results.iter().any(|&c| c == 2) { overall = 2; }
    if overall != 0 { return exit_with(overall, format!("send: {} targets processed with non-zero codes", results.len())); }

    if let Some(pb) = pb { pb.finish_and_clear(); }
    if let Format::Json = format {
        println!("{}", serde_json::json!({"status":"ok"}));
    }
    Ok(())
}

/// Run one-shot provider command
fn run_oneshot_provider(
    project: &str,
    agent_role: &str,
    provider_key: &str,
    tpl: &config_model::ProviderTemplate,
    prompt: &str,
    system_prompt: &str,
    allowed_tools: &[String],
    session_id_opt: Option<&str>,
    chat_id_opt: Option<&str>,
    timeout_ms: u64,
    print_header: bool,
    pb_opt: Option<ProgressBar>,
    conversation_id: Option<String>,
) -> i32 {
    let bin = tpl.cmd.clone();
    if bin.trim().is_empty() { return 3; }
    let allowed_join = allowed_tools.join(",");
    // Build args with placeholder replacement and conditional removal of session_id flag pair
    let mut unresolved = false;
    let session_id_val_opt: Option<String> = match session_id_opt {
        Some(s) if !s.trim().is_empty() => Some(s.to_string()),
        _ => {
            // Generate valid session IDs based on provider
            if provider_key == "claude" {
                Some(format!("valid_session_{}", short_id()))
            } else if provider_key == "gemini" {
                Some(format!("valid_context_{}", short_id()))
            } else {
                Some(uuid_v4_like())
            }
        },
    };
    let mut args: Vec<String> = Vec::new();
    let mut i = 0;
    while i < tpl.oneshot_args.len() {
        let tok = &tpl.oneshot_args[i];
        if tok == "--session-id" {
            let next = tpl.oneshot_args.get(i + 1);
            if next.map(|n| n.contains("{session_id}")).unwrap_or(false) {
                if let Some(val) = &session_id_val_opt {
                    args.push("--session-id".into());
                    args.push(val.clone());
                } // else skip both tokens entirely
                i += 2;
                continue;
            }
        }
        let mut replaced = tok.clone();
        if replaced.contains("{chat_id}") {
            if let Some(cid) = chat_id_opt { replaced = replaced.replace("{chat_id}", cid); } else { unresolved = true; }
        }
        replaced = replaced.replace("{prompt}", prompt)
            .replace("{system_prompt}", system_prompt)
            .replace("{allowed_tools}", &allowed_join);
        if replaced.contains("{session_id}") {
            if let Some(val) = &session_id_val_opt {
                replaced = replaced.replace("{session_id}", val);
            } else {
                // No session id provided: drop this token
                i += 1;
                continue;
            }
        }
        args.push(replaced);
        i += 1;
    }
    // If cursor requires chat_id and none provided, try auto-create chat
    if unresolved {
        if provider_key.starts_with("cursor") {
            match create_cursor_chat(tpl, system_prompt) {
                Ok(chat_id) => {
                    // Rebuild args with chat_id now available
                    args.clear();
                    i = 0;
                    while i < tpl.oneshot_args.len() {
                        let tok = &tpl.oneshot_args[i];
                        if tok == "--session-id" {
                            let next = tpl.oneshot_args.get(i + 1);
                            if next.map(|n| n.contains("{session_id}")).unwrap_or(false) {
                                if let Some(val) = &session_id_val_opt {
                                    args.push("--session-id".into());
                                    args.push(val.clone());
                                }
                                i += 2;
                                continue;
                            }
                        }
                        let mut replaced = tok.clone();
                        if replaced.contains("{chat_id}") { replaced = replaced.replace("{chat_id}", &chat_id); }
                        replaced = replaced
                            .replace("{prompt}", prompt)
                            .replace("{system_prompt}", system_prompt)
                            .replace("{allowed_tools}", &allowed_join);
                        if replaced.contains("{session_id}") {
                            if let Some(val) = &session_id_val_opt {
                                replaced = replaced.replace("{session_id}", val);
                            } else { i += 1; continue; }
                        }
                        args.push(replaced);
                        i += 1;
                    }
                }
                Err(e) => {
                    if e == "timeout" { return 5; }
                    return 4;
                }
            }
        } else {
            return 2;
        }
    }

    // Compose final session id for logging (best-effort)
    let final_session_id = if provider_key.starts_with("cursor") {
        chat_id_opt.unwrap_or("")
    } else {
        session_id_val_opt.as_deref().unwrap_or("")
    };

    // Update session last_activity if conversation_id provided
    if let Some(conv_id) = &conversation_id {
        let db_path = default_db_path();
        if let Ok(conn) = open_or_create_db(&db_path) {
            let now = now_iso8601_utc();
            let _ = conn.execute(
                "UPDATE sessions SET last_activity = ?1 WHERE id = ?2",
                params![&now, conv_id]
            );
            // Save provider_session_id best-effort
            if !final_session_id.is_empty() {
                let _ = conn.execute(
                    "UPDATE sessions SET provider_session_id = ?1 WHERE id = ?2",
                    params![&final_session_id, conv_id]
                );
            }
        }
    }

    // Execute
    let start_ts = now_iso8601_utc();
    log_ndjson(project, agent_role, provider_key, Some(final_session_id), "system", "start", None, None, Some(&start_ts));
    if print_header {
        println!("=== role:{} provider:{} ===", agent_role, provider_key);
    }
    // For cursor-agent, enforce stream-json output to avoid blocking and parse JSON to text
    let mut args_final = args;
    let mut parse_cursor_stream = false;
    if provider_key.starts_with("cursor") {
        parse_cursor_stream = true;
        let mut idx = None;
        for (i, t) in args_final.iter().enumerate() {
            if t == "--output-format" { idx = Some(i); break; }
        }
        if let Some(i) = idx {
            if i + 1 < args_final.len() { args_final[i + 1] = "stream-json".into(); }
            else { args_final.push("stream-json".into()); }
        } else {
            args_final.push("--output-format".into());
            args_final.push("stream-json".into());
        }
    }
    if let Some(pb) = &pb_opt { pb.set_message(format!("{}:{}", agent_role, provider_key)); }
    match run_with_timeout_streaming(&bin, &args_final.iter().map(|s| s.as_str()).collect::<Vec<_>>(), Duration::from_millis(timeout_ms), project, agent_role, provider_key, final_session_id, pb_opt.as_ref(), parse_cursor_stream) {
        Ok(code) => {
            log_ndjson(project, agent_role, provider_key, Some(final_session_id), "system", "end", None, Some(code), None);
            if code == 0 { 0 } else { 4 }
        }
        Err(e) => {
            if e == "timeout" { log_ndjson(project, agent_role, provider_key, Some(final_session_id), "system", "end", None, Some(5), None); 5 }
            else if e.contains("No such file") || e.contains("not found") { 3 }
            else { 4 }
        }
    }
}

/// Create cursor chat
fn create_cursor_chat(tpl: &config_model::ProviderTemplate, system_prompt: &str) -> Result<String, String> {
    let create_args_opt = tpl.create_chat_args.as_ref();
    let create_args = match create_args_opt { Some(a) => a, None => return Err("missing_create_chat_args".into()) };
    let args: Vec<String> = create_args.iter().map(|a| a.replace("{system_prompt}", system_prompt)).collect();
    match crate::utils::timeouts::run_with_timeout(&tpl.cmd, &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(), Duration::from_millis(5000)) {
        Ok((_code, out, err)) => {
            let text = if !out.trim().is_empty() { out } else { err };
            let id = text.lines().filter(|l| !l.trim().is_empty()).last().unwrap_or("").trim().to_string();
            if id.is_empty() { return Err("empty_chat_id".into()); }
            Ok(id)
        }
        Err(e) => Err(e),
    }
}

/// Make progress bar
fn make_pb() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner} sending {msg}").unwrap());
    pb.enable_steady_tick(Duration::from_millis(120));
    pb
}
