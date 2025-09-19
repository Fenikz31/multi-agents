//! Agent management commands (tmux operations)

use std::fs;
use std::time::{Duration, Instant};
use config_model::{parse_project_yaml, parse_providers_yaml};
use crate::utils::{resolve_config_paths, handle_missing_config, DEFAULT_AGENT_TIMEOUT_MS, exit_with};
use crate::tmux::manager::TmuxManager;
use crate::logging::{emit_start_event, emit_end_event};

/// Run agent run command
pub fn run_agent_run(
    project_file: Option<&str>, 
    providers_file: Option<&str>, 
    project_name: Option<&str>, 
    agent_name: &str, 
    role_override: Option<&str>, 
    provider_override: Option<&str>, 
    model_override: Option<&str>, 
    workdir: Option<&str>, 
    no_logs: bool, 
    timeout_ms: Option<u64>
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    // Cap tmux timeouts to 5s
    let effective_ms = timeout_ms.unwrap_or(DEFAULT_AGENT_TIMEOUT_MS).min(DEFAULT_AGENT_TIMEOUT_MS);
    let timeout = Duration::from_millis(effective_ms);
    
    // Resolve config paths
    let (project_path, providers_path) = match resolve_config_paths(project_file, providers_file) {
        Ok(p) => p,
        Err(msg) => return handle_missing_config(msg),
    };
    
    // Load configurations
    let proj_s = fs::read_to_string(&project_path)?;
    let prov_s = fs::read_to_string(&providers_path)?;
    let project = parse_project_yaml(&proj_s).map_err(|e| format!("project: {}", e))?;
    let providers = parse_providers_yaml(&prov_s).map_err(|e| format!("providers: {}", e))?;
    
    // Determine project name
    let project_name = project_name.unwrap_or(&project.project);
    
    // Find agent configuration
    let agent = project.agents.iter()
        .find(|a| a.name == agent_name)
        .ok_or_else(|| format!("Agent '{}' not found in project '{}'", agent_name, project_name))?;
    
    // Apply overrides
    let role = role_override.unwrap_or(&agent.role);
    let provider = provider_override.unwrap_or(&agent.provider);
    let _model = model_override.unwrap_or(&agent.model);
    
    // Get provider configuration
    let provider_config = providers.providers.get(provider)
        .ok_or_else(|| format!("Provider '{}' not found in configuration", provider))?;
    
    // Build tmux session and window names
    let session_name = format!("proj:{}", project_name);
    let window_name = format!("{}:{}", role, agent_name);
    
    // Create tmux manager and run agent
    let tmux_manager = TmuxManager::new(timeout);
    
    // Step 1: Check if session exists
    let session_exists = tmux_manager.has_session(&session_name)?;
    
    // Step 2: Create session if it doesn't exist
    if !session_exists {
        tmux_manager.create_session(&session_name)?;
    }
    
    // Step 3: Check if window already exists
    let window_exists = tmux_manager.window_exists(&session_name, &window_name)?;
    
    if window_exists {
        println!("Agent '{}' is already running in tmux session '{}'", agent_name, session_name);
        return Ok(());
    }
    
    // Step 4: Create new window for the agent
    tmux_manager.create_window(&session_name, &window_name)?;
    
    // Step 5: Set up logging if not disabled
    if !no_logs {
        let log_dir = format!("./logs/{}", project_name);
        let _ = fs::create_dir_all(&log_dir);
        let log_file = format!("{}/{}.ndjson", log_dir, role);
        
        // Set up pipe-pane for logging
        tmux_manager.setup_pipe_pane(&session_name, &window_name, &log_file)?;
        
        // Emit start event
        if let Err(e) = emit_start_event(project_name, role, agent_name, provider) {
            eprintln!("Warning: Failed to emit start event: {}", e);
        }
    }
    
    // Step 6: Set working directory if specified
    if let Some(workdir) = workdir {
        tmux_manager.send_keys(&session_name, &window_name, &format!("cd {}", workdir))?;
    }
    
    // Step 7: Start the provider command
    let mut args = provider_config.repl_args.clone();
    for arg in &mut args {
        *arg = arg.replace("{system_prompt}", &agent.system_prompt)
                 .replace("{allowed_tools}", &agent.allowed_tools.join(","));
    }
    
    let cmd_line = format!("{} {}", provider_config.cmd, args.join(" "));
    tmux_manager.send_keys(&session_name, &window_name, &cmd_line)?;
    
    let duration_ms = start_time.elapsed().as_millis() as u64;
    println!("Agent '{}' started in tmux session '{}' (took {}ms)", agent_name, session_name, duration_ms);
    Ok(())
}

/// Run agent attach command
pub fn run_agent_attach(
    project_file: Option<&str>, 
    project_name: Option<&str>, 
    agent_name: &str, 
    timeout_ms: Option<u64>
) -> Result<(), Box<dyn std::error::Error>> {
    // Cap tmux timeouts to 5s
    let effective_ms = timeout_ms.unwrap_or(DEFAULT_AGENT_TIMEOUT_MS).min(DEFAULT_AGENT_TIMEOUT_MS);
    let timeout = Duration::from_millis(effective_ms);
    
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
    
    // Find agent configuration
    let agent = project.agents.iter()
        .find(|a| a.name == agent_name)
        .ok_or_else(|| format!("Agent '{}' not found in project '{}'", agent_name, project_name))?;
    
    // Build tmux session and window names
    let session_name = format!("proj:{}", project_name);
    let window_name = format!("{}:{}", agent.role, agent_name);
    
    // Create tmux manager
    let tmux_manager = TmuxManager::new(timeout);
    
    // Check if session exists
    let session_exists = tmux_manager.has_session(&session_name)?;
    
    if !session_exists {
        return exit_with(2, format!("No tmux session found for project '{}'", project_name));
    }
    
    // Check if window exists
    let window_exists = tmux_manager.window_exists(&session_name, &window_name)?;
    
    if !window_exists {
        return exit_with(2, format!("Agent '{}' is not running in tmux session '{}'", agent_name, session_name));
    }
    
    // Check if we're in a headless environment
    let is_headless = std::env::var("DISPLAY").is_err() && std::env::var("SSH_TTY").is_ok();
    
    if is_headless {
        // Provide fallback message for headless mode
        println!("Cannot attach to tmux session in headless mode.");
        println!("Session '{}' is running with window '{}'.", session_name, window_name);
        println!("To attach manually, run: tmux attach-session -t {}", session_name);
        println!("To view logs, run: tail -f ./logs/{}/{}.ndjson", project_name, agent.role);
        return Ok(());
    }
    
    // Attach to the session
    tmux_manager.attach_session(&session_name)?;
    
    Ok(())
}

/// Run agent stop command
pub fn run_agent_stop(
    project_file: Option<&str>, 
    project_name: Option<&str>, 
    agent_name: &str, 
    timeout_ms: Option<u64>
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    // Cap tmux timeouts to 5s
    let effective_ms = timeout_ms.unwrap_or(DEFAULT_AGENT_TIMEOUT_MS).min(DEFAULT_AGENT_TIMEOUT_MS);
    let timeout = Duration::from_millis(effective_ms);
    
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
    
    // Find agent configuration
    let agent = project.agents.iter()
        .find(|a| a.name == agent_name)
        .ok_or_else(|| format!("Agent '{}' not found in project '{}'", agent_name, project_name))?;
    
    // Build tmux session and window names
    let session_name = format!("proj:{}", project_name);
    let window_name = format!("{}:{}", agent.role, agent_name);
    
    // Create tmux manager
    let tmux_manager = TmuxManager::new(timeout);
    
    // Check if session exists - idempotent
    let session_exists = tmux_manager.has_session(&session_name)?;
    
    if !session_exists {
        println!("No tmux session found for project '{}' - nothing to stop", project_name);
        return Ok(());
    }
    
    // Check if window exists - idempotent
    let window_exists = tmux_manager.window_exists(&session_name, &window_name)?;
    
    if !window_exists {
        println!("Agent '{}' is not running in tmux session '{}' - nothing to stop", agent_name, session_name);
        return Ok(());
    }
    
    // Emit end event before stopping
    let duration_ms = start_time.elapsed().as_millis() as u64;
    if let Err(e) = emit_end_event(project_name, &agent.role, agent_name, &agent.provider, "stopped", duration_ms) {
        eprintln!("Warning: Failed to emit end event: {}", e);
    }
    
    // Kill the window - idempotent operation
    tmux_manager.kill_window(&session_name, &window_name)?;
    
    let total_duration_ms = start_time.elapsed().as_millis() as u64;
    println!("Agent '{}' stopped in tmux session '{}' (took {}ms)", agent_name, session_name, total_duration_ms);
    Ok(())
}
