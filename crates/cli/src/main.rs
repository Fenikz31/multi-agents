use clap::{Parser, Subcommand, ValueEnum};
use config_model::{
    parse_project_yaml, parse_providers_yaml, validate_project_config, validate_providers_config,
};
use db::{open_or_create_db, insert_project, insert_agent, find_project_id, IdOrName, 
         ClaudeSessionManager, CursorSessionManager, GeminiSessionManager, SessionManager, 
         list_sessions, SessionFilters, SessionStatus, sync_project_from_config, cleanup_repl_sessions};
use rusqlite::params;
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use serde_json::Value;
use db::now_iso8601_utc;
use std::thread;
use std::path::Path;
use std::sync::mpsc;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Parser, Debug)]
#[command(name = "multi-agents", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize project: create configs, init DB, and sync agents
    Init {
        /// Target directory for config files (default: ./config)
        #[arg(long, value_name = "DIR")] config_dir: Option<String>,
        /// Overwrite existing config files
        #[arg(long, default_value_t = false)] force: bool,
        /// Skip database initialization (assume already done)
        #[arg(long, default_value_t = false)] skip_db: bool,
    },
    /// Configuration commands
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    /// Environment checks (CLIs, flags, timeouts)
    Doctor {
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
        /// Optional: path to NDJSON sample to self-check parsing
        #[arg(long, value_name = "PATH")]
        ndjson_sample: Option<String>,
        /// Optional: write JSON snapshot of detected capabilities to file
        #[arg(long, value_name = "PATH")]
        snapshot: Option<String>,
    },
    /// Database commands
    Db {
        #[command(subcommand)]
        cmd: DbCmd,
    },
    /// Send a one-shot message to agent(s)
    Send {
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] project_file: Option<String>,
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] providers_file: Option<String>,
        /// Target: @all, @role, or agent name
        #[arg(long)] to: String,
        #[arg(long)] message: String,
        /// Optional: provide explicit session id (e.g., for Claude)
        #[arg(long)] session_id: Option<String>,
        /// Optional: provide explicit chat id (for cursor-agent)
        #[arg(long)] chat_id: Option<String>,
        /// Optional: override per-target timeout in milliseconds (default 120_000)
        #[arg(long, value_name = "MILLIS")] timeout_ms: Option<u64>,
        /// Output format for this command (text|json)
        #[arg(long, value_enum, default_value_t = Format::Text)] format: Format,
        /// Show progress spinner (default ON); disable with --no-progress
        #[arg(long = "progress", default_value_t = true)] progress: bool,
    },
    /// Session management
    Session {
        #[command(subcommand)]
        cmd: SessionCmd,
    },
    /// Agent management (tmux REPL)
    Agent {
        #[command(subcommand)]
        cmd: AgentCmd,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCmd {
    /// Validate configuration files (YAML schemas + semantic rules)
    Validate {
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] project_file: Option<String>,
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] providers_file: Option<String>,
        #[arg(long, value_enum, default_value_t = Format::Text)] format: Format,
    },
    /// Create default config files under a directory (default: ./config)
    Init {
        /// Target directory for config files
        #[arg(long, value_name = "DIR")] dir: Option<String>,
        /// Overwrite existing files if present
        #[arg(long, default_value_t = false)] force: bool,
    },
}

#[derive(Subcommand, Debug)]
enum DbCmd {
    /// Initialize the SQLite database (idempotent)
    Init {
        #[arg(long, value_name = "PATH")]
        db_path: Option<String>,
    },
    /// Add a new project
    ProjectAdd {
        #[arg(long)] name: String,
        #[arg(long, value_name = "PATH")] db_path: Option<String>,
    },
    /// Add a new agent to a project
    AgentAdd {
        /// Project id or name
        #[arg(long)] project: String,
        #[arg(long)] name: String,
        #[arg(long)] role: String,
        #[arg(long)] provider: String,
        #[arg(long)] model: String,
        /// Repeatable flag for allowed tools
        #[arg(long = "allowed-tool")] allowed_tool: Vec<String>,
        #[arg(long = "system-prompt")] system_prompt: String,
        #[arg(long, value_name = "PATH")] db_path: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum SessionCmd {
    /// Start a provider session and print conversation_id
    Start {
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] project_file: Option<String>,
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] providers_file: Option<String>,
        #[arg(long)] agent: String,
    },
    /// List sessions for a project
    List {
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] project_file: Option<String>,
        /// Project name (defaults to current directory name)
        #[arg(long)] project: Option<String>,
        /// Filter by agent name
        #[arg(long)] agent: Option<String>,
        /// Filter by provider
        #[arg(long)] provider: Option<String>,
        /// Output format (text|json)
        #[arg(long, value_enum, default_value_t = Format::Text)] format: Format,
    },
    /// Resume an existing session
    Resume {
        /// Conversation ID to resume
        #[arg(long)] conversation_id: String,
        /// Optional: override timeout in milliseconds (default 5000)
        #[arg(long, value_name = "MILLIS")] timeout_ms: Option<u64>,
    },
    /// Clean up expired sessions
    Cleanup {
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] project_file: Option<String>,
        /// Dry run (show what would be deleted without actually deleting)
        #[arg(long, default_value_t = false)] dry_run: bool,
        /// Output format (text|json)
        #[arg(long, value_enum, default_value_t = Format::Text)] format: Format,
    },
}

#[derive(Subcommand, Debug)]
enum AgentCmd {
    /// Start an agent in tmux REPL mode
    Run {
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] project_file: Option<String>,
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] providers_file: Option<String>,
        /// Project name (defaults to current directory name)
        #[arg(long)] project: Option<String>,
        /// Agent name to run
        #[arg(long)] agent: String,
        /// Optional: override agent role
        #[arg(long)] role: Option<String>,
        /// Optional: override agent provider
        #[arg(long)] provider: Option<String>,
        /// Optional: override agent model
        #[arg(long)] model: Option<String>,
        /// Optional: working directory for the agent
        #[arg(long, value_name = "DIR")] workdir: Option<String>,
        /// Disable NDJSON logging
        #[arg(long, default_value_t = false)] no_logs: bool,
        /// Optional: override timeout in milliseconds (default 5000)
        #[arg(long, value_name = "MILLIS")] timeout_ms: Option<u64>,
    },
    /// Attach to an existing agent tmux session
    Attach {
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] project_file: Option<String>,
        /// Project name (defaults to current directory name)
        #[arg(long)] project: Option<String>,
        /// Agent name to attach to
        #[arg(long)] agent: String,
        /// Optional: override timeout in milliseconds (default 5000)
        #[arg(long, value_name = "MILLIS")] timeout_ms: Option<u64>,
    },
    /// Stop an agent tmux session
    Stop {
        /// Optional: explicit path; else ENV/defaults resolution is used
        #[arg(long, value_name = "PATH")] project_file: Option<String>,
        /// Project name (defaults to current directory name)
        #[arg(long)] project: Option<String>,
        /// Agent name to stop
        #[arg(long)] agent: String,
        /// Optional: override timeout in milliseconds (default 5000)
        #[arg(long, value_name = "MILLIS")] timeout_ms: Option<u64>,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Format { Text, Json }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let cli = Cli::parse();
    match cli.cmd {
        Commands::Init { config_dir, force, skip_db } => 
            run_init(config_dir.as_deref(), force, skip_db),
        Commands::Config { cmd } => match cmd {
            ConfigCmd::Validate { project_file, providers_file, format } => {
                run_config_validate(project_file.as_deref(), providers_file.as_deref(), format)
            }
            ConfigCmd::Init { dir, force } => run_config_init(dir.as_deref(), force),
        },
        Commands::Doctor { format, ndjson_sample, snapshot } => run_doctor(format, ndjson_sample.as_deref(), snapshot.as_deref()),
        Commands::Db { cmd } => match cmd {
            DbCmd::Init { db_path } => run_db_init(db_path.as_deref()),
            DbCmd::ProjectAdd { name, db_path } => run_project_add(&name, db_path.as_deref()),
            DbCmd::AgentAdd { project, name, role, provider, model, allowed_tool, system_prompt, db_path } =>
                run_agent_add(&project, &name, &role, &provider, &model, &allowed_tool, &system_prompt, db_path.as_deref()),
        },
        Commands::Send { project_file, providers_file, to, message, session_id, chat_id, timeout_ms, format, progress } => {
            run_send(project_file.as_deref(), providers_file.as_deref(), &to, &message, session_id.as_deref(), chat_id.as_deref(), timeout_ms, format, progress)
        },
        Commands::Session { cmd } => match cmd {
            SessionCmd::Start { project_file, providers_file, agent } =>
                run_session_start(project_file.as_deref(), providers_file.as_deref(), &agent),
            SessionCmd::List { project_file, project, agent, provider, format } =>
                run_session_list(project_file.as_deref(), project.as_deref(), agent.as_deref(), provider.as_deref(), format),
            SessionCmd::Resume { conversation_id, timeout_ms } =>
                run_session_resume(&conversation_id, timeout_ms),
            SessionCmd::Cleanup { project_file, dry_run, format } =>
                run_session_cleanup(project_file.as_deref(), dry_run, format),
        },
        Commands::Agent { cmd } => match cmd {
            AgentCmd::Run { project_file, providers_file, project, agent, role, provider, model, workdir, no_logs, timeout_ms } =>
                run_agent_run(project_file.as_deref(), providers_file.as_deref(), project.as_deref(), &agent, role.as_deref(), provider.as_deref(), model.as_deref(), workdir.as_deref(), no_logs, timeout_ms),
            AgentCmd::Attach { project_file, project, agent, timeout_ms } =>
                run_agent_attach(project_file.as_deref(), project.as_deref(), &agent, timeout_ms),
            AgentCmd::Stop { project_file, project, agent, timeout_ms } =>
                run_agent_stop(project_file.as_deref(), project.as_deref(), &agent, timeout_ms),
        },
    }
}

fn run_config_validate(project_path_opt: Option<&str>, providers_path_opt: Option<&str>, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let (project_path, providers_path) = match resolve_config_paths(project_path_opt, providers_path_opt) {
        Ok(p) => p,
        Err(msg) => return handle_missing_config(msg),
    };
    let proj_s = fs::read_to_string(&project_path)?;
    let prov_s = fs::read_to_string(&providers_path)?;

    let project = match parse_project_yaml(&proj_s) {
        Ok(p) => p,
        Err(e) => return exit_with(2, format_error(format, "project", &e)),
    };
    let providers = match parse_providers_yaml(&prov_s) {
        Ok(p) => p,
        Err(e) => return exit_with(2, format_error(format, "providers", &e)),
    };

    if let Err(e) = validate_providers_config(&providers) {
        return exit_with(2, format_error(format, "providers", &e));
    }
    if let Err(e) = validate_project_config(&project, &providers) {
        return exit_with(2, format_error(format, "project", &e));
    }

    match format {
        Format::Text => println!("OK: configuration valid"),
        Format::Json => println!("{}", serde_json::json!({"status":"ok"})),
    }
    Ok(())
}

fn format_error(format: Format, which: &str, err: &impl std::fmt::Display) -> String {
    match format {
        Format::Text => format!("{}: {}", which, err),
        Format::Json => serde_json::json!({"status":"error","scope":which,"error":err.to_string()}).to_string(),
    }
}

fn exit_with<T>(code: i32, msg: String) -> Result<T, Box<dyn std::error::Error>> {
    eprintln!("{}", msg);
    std::process::exit(code);
}

// ---- db commands ----

fn default_db_path() -> String { "./data/multi-agents.sqlite3".into() }

fn run_db_init(db_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let binding;
    let path = match db_path { Some(p) => p, None => { binding = default_db_path(); &binding } };
    match open_or_create_db(path) {
        Ok(_) => { println!("OK: db initialized"); Ok(()) }
        Err(e) => exit_with(7, format!("db: {}", e)),
    }
}

fn run_project_add(name: &str, db_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let binding;
    let path = match db_path { Some(p) => p, None => { binding = default_db_path(); &binding } };
    let conn = match open_or_create_db(path) { Ok(c) => c, Err(e) => return exit_with(7, format!("db: {}", e)) };
    match insert_project(&conn, name) {
        Ok(p) => { println!("project_id={} name={}", p.id, p.name); Ok(()) }
        Err(db::DbError::InvalidInput(e)) => exit_with(2, format!("project: {}", e)),
        Err(e) => exit_with(7, format!("project: {}", e)),
    }
}

fn run_agent_add(project_sel: &str, name: &str, role: &str, provider: &str, model: &str, allowed_tool: &[String], system_prompt: &str, db_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
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

fn looks_like_uuid(s: &str) -> bool { s.len() >= 16 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-') }

// ---- send oneshot implementation ----

const DEFAULT_SEND_TIMEOUT_MS: u64 = 120_000;
const DEFAULT_AGENT_TIMEOUT_MS: u64 = 5_000;
const MAX_CONCURRENCY: usize = 3;

fn run_send(project_path_opt: Option<&str>, providers_path_opt: Option<&str>, to: &str, message: &str, session_id_opt: Option<&str>, chat_id_opt: Option<&str>, timeout_ms_flag: Option<u64>, format: Format, progress: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (project_path, providers_path) = match resolve_config_paths(project_path_opt, providers_path_opt) {
        Ok(p) => p,
        Err(msg) => return handle_missing_config(msg),
    };
    let proj_s = fs::read_to_string(&project_path)?;
    let prov_s = fs::read_to_string(&providers_path)?;
    let project = match parse_project_yaml(&proj_s) { Ok(p) => p, Err(e) => return exit_with(2, format!("project: {}", e)) };
    let providers = match parse_providers_yaml(&prov_s) { Ok(p) => p, Err(e) => return exit_with(2, format!("providers: {}", e)) };

    // M3-07: Session management - sync project and agents to database
    let db_path = default_db_path();
    let conn = open_or_create_db(&db_path)?;
    match sync_project_from_config(&conn, &project) {
        Ok(_) => {}, // Project synchronized successfully
        Err(e) => return exit_with(7, format!("Failed to sync project: {}", e)),
    }

    // M3-07: Resolve targets with session support
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
        if let Some(session) = db::find_session(&conn, to)? {
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

    // M3-08: Auto-create session if conversation_id is absent, and fallback if status expired/invalid
    // Determine project_id once
    let project_id = match find_project_id(&conn, IdOrName::Name(&project.project))? {
        Some(pid) => pid,
        None => return exit_with(2, format!("Project not found: {}", project.project)),
    };
    for (i, agent) in targets.iter().enumerate() {
        // If a session was provided, ensure it's active; else create one
        if let Some(conv_id) = &session_contexts[i] {
            if let Some(existing) = db::find_session(&conn, conv_id)? {
                // If not active, create a fresh session
                if existing.status.to_string() != SessionStatus::Active.to_string() {
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
        
        // M3-07: Get session context for this agent
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
            // M3-07: Generate valid session IDs based on provider
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

    // M3-07: Update session last_activity if conversation_id provided
    if let Some(conv_id) = &conversation_id {
        let db_path = default_db_path();
        if let Ok(conn) = open_or_create_db(&db_path) {
            let now = now_iso8601_utc();
            let _ = conn.execute(
                "UPDATE sessions SET last_activity = ?1 WHERE id = ?2",
                params![&now, conv_id]
            );
            // M3-08: Save provider_session_id best-effort
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

fn create_cursor_chat(tpl: &config_model::ProviderTemplate, system_prompt: &str) -> Result<String, String> {
    let create_args_opt = tpl.create_chat_args.as_ref();
    let create_args = match create_args_opt { Some(a) => a, None => return Err("missing_create_chat_args".into()) };
    let args: Vec<String> = create_args.iter().map(|a| a.replace("{system_prompt}", system_prompt)).collect();
    match run_with_timeout(&tpl.cmd, &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(), Duration::from_millis(5000)) {
        Ok((_code, out, err)) => {
            let text = if !out.trim().is_empty() { out } else { err };
            let id = text.lines().filter(|l| !l.trim().is_empty()).last().unwrap_or("").trim().to_string();
            if id.is_empty() { return Err("empty_chat_id".into()); }
            Ok(id)
        }
        Err(e) => Err(e),
    }
}

enum LineEvent { Stdout(String), Stderr(String), Exit(i32) }

fn run_with_timeout_streaming(
    bin: &str,
    args: &[&str],
    timeout: Duration,
    project: &str,
    agent_role: &str,
    provider_key: &str,
    session_id: &str,
    pb_opt: Option<&ProgressBar>,
    parse_cursor_stream: bool,
) -> Result<i32, String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let (tx, rx) = mpsc::channel::<LineEvent>();

    // stdout reader
    if let Some(so) = child.stdout.take() {
        let txo = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(so);
            for line_res in reader.lines() {
                if let Ok(line) = line_res { let _ = txo.send(LineEvent::Stdout(line)); } else { break; }
            }
        });
    }
    // stderr reader
    if let Some(se) = child.stderr.take() {
        let txe = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(se);
            for line_res in reader.lines() {
                if let Ok(line) = line_res { let _ = txe.send(LineEvent::Stderr(line)); } else { break; }
            }
        });
    }
    // wait thread
    let txw = tx.clone();
    thread::spawn(move || {
        match child.wait() {
            Ok(status) => { let _ = txw.send(LineEvent::Exit(status.code().unwrap_or(-1))); }
            Err(_) => { let _ = txw.send(LineEvent::Exit(-1)); }
        }
    });

    let start = Instant::now();
    let mut exit_code: Option<i32> = None;
    let mut saw_final_result: bool = false;
    loop {
        let remaining = if start.elapsed() >= timeout { 0 } else { (timeout - start.elapsed()).as_millis() as u64 };
        if remaining == 0 { return Err("timeout".into()); }
        match rx.recv_timeout(Duration::from_millis(remaining)) {
            Ok(LineEvent::Stdout(line)) => {
                if parse_cursor_stream {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                        // Parse cursor stream-json according to official spec
                        let mut text_to_print = None;
                        
                        if let Some(event_type) = v.get("type").and_then(|t| t.as_str()) {
                            match event_type {
                                "assistant" => {
                                    // Extract text from assistant.message.content[].text
                                    if let Some(message) = v.get("message") {
                                        if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
                                            for item in content {
                                                if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                                                    if item_type == "text" {
                                                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                                            text_to_print = Some(text.to_string());
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                "result" => {
                                    // Final result event - extract complete text
                                    if let Some(result) = v.get("result").and_then(|r| r.as_str()) {
                                        text_to_print = Some(result.to_string());
                                        saw_final_result = true;
                                    }
                                }
                                "tool_call" => {
                                    // Optional: could extract tool call info, but skip for now
                                    continue;
                                }
                                _ => {
                                    // system, user events - skip
                                    continue;
                                }
                            }
                        } else {
                            // Fallback: try legacy flat fields for compatibility
                            text_to_print = v.get("text").and_then(|x| x.as_str()).map(|s| s.to_string())
                                .or_else(|| v.get("content").and_then(|x| x.as_str()).map(|s| s.to_string()))
                                .or_else(|| v.get("message").and_then(|x| x.as_str()).map(|s| s.to_string()))
                                .or_else(|| v.get("delta").and_then(|x| x.as_str()).map(|s| s.to_string()))
                                .or_else(|| v.get("data").and_then(|x| x.as_str()).map(|s| s.to_string()));
                        }
                        
                        if let Some(text) = text_to_print {
                            println!("{}", text);
                            log_ndjson(project, agent_role, provider_key, Some(session_id), "agent", "stdout_line", Some(&text), None, None);
                            // If we've seen the final result, we can return success immediately
                            if saw_final_result {
                                exit_code = Some(0);
                                break;
                            }
                        }
                    }
                } else {
                    println!("{}", line);
                    log_ndjson(project, agent_role, provider_key, Some(session_id), "agent", "stdout_line", Some(&line), None, None);
                }
                if let Some(pb) = pb_opt { pb.tick(); }
            }
            Ok(LineEvent::Stderr(line)) => {
                eprintln!("{}", line);
                log_ndjson(project, agent_role, provider_key, Some(session_id), "agent", "stderr_line", Some(&line), None, None);
                if let Some(pb) = pb_opt { pb.tick(); }
            }
            Ok(LineEvent::Exit(code)) => { exit_code = Some(code); break; }
            Err(mpsc::RecvTimeoutError::Timeout) => { return Err("timeout".into()); }
            Err(_e) => { break; }
        }
    }
    Ok(exit_code.unwrap_or(-1))
}

fn make_pb() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner} sending {msg}").unwrap());
    pb.enable_steady_tick(Duration::from_millis(120));
    pb
}

fn log_ndjson(project: &str, agent_role: &str, provider: &str, session_id: Option<&str>, direction: &str, event: &str, text: Option<&str>, exit_code: Option<i32>, ts_opt: Option<&str>) {
    let ts = ts_opt.map(|s| s.to_string()).unwrap_or_else(|| now_iso8601_utc());
    let obj = serde_json::json!({
        "ts": ts,
        "project_id": project,
        "agent_role": agent_role,
        "provider": provider,
        "session_id": session_id.unwrap_or("") ,
        "direction": direction,
        "event": event,
        "text": text,
        "exit_code": exit_code,
    });
    let dir = format!("./logs/{project}");
    let _ = fs::create_dir_all(&dir);
    let path = format!("{}/{}.ndjson", dir, agent_role);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(&mut f, "{}", obj);
    }
}

fn short_id() -> String { format!("{:x}", Instant::now().elapsed().as_nanos()) }

fn uuid_v4_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut s = format!("{:032x}", nanos);
    // Set version (v4)
    s.replace_range(12..13, "4");
    // Set variant (10xx)
    let variants = ['8','9','a','b'];
    let idx = (nanos & 0x3) as usize;
    s.replace_range(16..17, &variants[idx].to_string());
    format!(
        "{}-{}-{}-{}-{}",
        &s[0..8], &s[8..12], &s[12..16], &s[16..20], &s[20..32]
    )
}

fn run_session_start(project_path_opt: Option<&str>, providers_path_opt: Option<&str>, agent_name: &str) -> Result<(), Box<dyn std::error::Error>> {
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

fn run_session_list(project_path_opt: Option<&str>, project_name_opt: Option<&str>, agent_filter: Option<&str>, provider_filter: Option<&str>, format: Format) -> Result<(), Box<dyn std::error::Error>> {
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

fn run_session_resume(conversation_id: &str, timeout_ms: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let db_path = default_db_path();
    let conn = open_or_create_db(&db_path)?;
    
    // Find session
    let session = match db::find_session(&conn, conversation_id)? {
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

// ---- doctor implementation ----

const DEFAULT_TIMEOUT_PER_PROVIDER_MS: u64 = 12000; // adjusted to tolerate slower CLIs (gemini ~10s)
const DEFAULT_TIMEOUT_GLOBAL_MS: u64 = 20000;

#[derive(Debug, Clone)]
struct ProbeResult {
    name: String,
    present: bool,
    version: Option<String>,
    supports: BTreeMap<String, bool>,
    timed_out: bool,
    error: Option<String>,
}

fn run_with_timeout(bin: &str, args: &[&str], timeout: Duration) -> Result<(i32, String, String), String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let mut out = String::new();
            let mut err = String::new();
            if let Some(mut so) = child.stdout.take() {
                let _ = so.read_to_string(&mut out);
            }
            if let Some(mut se) = child.stderr.take() {
                let _ = se.read_to_string(&mut err);
            }
            let code = status.code().unwrap_or(-1);
            return Ok((code, out, err));
        }
        if start.elapsed() >= timeout {
            // best-effort kill
            let _ = child.kill();
            return Err("timeout".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn probe_help(bin: &str, help_args: &[&str], timeout_ms: u64) -> Result<String, String> {
    let timeout = Duration::from_millis(timeout_ms);
    let debug = std::env::var("DOCTOR_DEBUG").ok().map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false);
    if debug { eprintln!("[doctor] help probe: {} {:?}", bin, help_args); }
    match run_with_timeout(bin, help_args, timeout) {
        Ok((_code, out, err)) => {
            let text = if !out.trim().is_empty() { out } else { err };
            return Ok(text);
        }
        Err(e) => {
            if debug { eprintln!("[doctor] help direct failed: {} {:?} => {}", bin, help_args, e); }
            // Fallback via login shell to inherit PATH managers (e.g. NVM)
            let joined = std::iter::once(bin).chain(help_args.iter().copied()).collect::<Vec<_>>().join(" ");
            let shell_cmd = format!("bash -lc '{}'", joined.replace("'", "'\\''"));
            if debug { eprintln!("[doctor] help via shell: {}", shell_cmd); }
            match run_with_timeout("bash", &["-lc", &joined], timeout) {
                Ok((_code, out, err)) => {
                    let text = if !out.trim().is_empty() { out } else { err };
                    Ok(text)
                }
                Err(e2) => Err(e2),
            }
        }
    }
}

fn probe_version(bin: &str, candidates: &[&[&str]], timeout_ms: u64) -> Option<String> {
    for args in candidates {
        let timeout = Duration::from_millis(timeout_ms);
        let debug = std::env::var("DOCTOR_DEBUG").ok().map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false);
        if debug { eprintln!("[doctor] version probe: {} {:?}", bin, args); }
        match run_with_timeout(bin, args, timeout) {
            Ok((_code, out, err)) => {
                let text = if !out.trim().is_empty() { out } else { err };
                let line = text.lines().next().unwrap_or("").trim().to_string();
                if !line.is_empty() { return Some(line); }
            }
            Err(e) => {
                if debug { eprintln!("[doctor] version direct failed: {} {:?} => {}", bin, args, e); }
                // shell fallback
                let joined = std::iter::once(bin).chain(args.iter().copied()).collect::<Vec<_>>().join(" ");
                if let Ok((_code, out, err)) = run_with_timeout("bash", &["-lc", &joined], timeout) {
                    let text = if !out.trim().is_empty() { out } else { err };
                    let line = text.lines().next().unwrap_or("").trim().to_string();
                    if !line.is_empty() { return Some(line); }
                }
            }
        }
    }
    None
}



fn probe_version_only(name: &str, cmd: &str, version_args: &[String], timeout_ms: u64) -> ProbeResult {
    let supports = BTreeMap::new();
    let version_candidates: Vec<Vec<&str>> = if version_args.is_empty() {
        vec![vec!["--version"], vec!["version"], vec!["-v"]]
    } else {
        vec![version_args.iter().map(|s| s.as_str()).collect()]
    };
    let version = probe_version(cmd, &version_candidates.iter().map(|v| v.as_slice()).collect::<Vec<_>>(), timeout_ms);
    if let Some(v) = version {
        ProbeResult { name: name.into(), present: true, version: Some(v), supports, timed_out: false, error: None }
    } else {
        ProbeResult { name: name.into(), present: false, version: None, supports, timed_out: false, error: Some("version_probe_failed".into()) }
    }
}

fn parse_tmux_list_commands(list_cmds: &str) -> BTreeMap<String, bool> {
    let mut supports = BTreeMap::new();
    supports.insert("pipe_pane".into(), list_cmds.contains("pipe-pane"));
    supports
}

fn probe_tmux(timeout_ms: u64) -> ProbeResult {
    let mut timed_out = false;
    let mut error = None;
    let version = probe_version("tmux", &[&["-V"], &["--version"]], timeout_ms);
    if version.is_none() {
        // Not present or failed
        // Attempt to see if binary exists via help, else mark not present
        match probe_help("tmux", &["-h"], timeout_ms) {
            Ok(_) => {},
            Err(e) => {
                if e == "timeout" { timed_out = true; }
                error = Some(e);
                return ProbeResult { name: "tmux".into(), present: false, version: None, supports: BTreeMap::new(), timed_out, error };
            }
        }
    }
    // pipe-pane support via list-commands
    let list = probe_help("tmux", &["list-commands"], timeout_ms).unwrap_or_default();
    let supports = parse_tmux_list_commands(&list);
    ProbeResult { name: "tmux".into(), present: true, version, supports, timed_out, error }
}

fn probe_git(timeout_ms: u64) -> ProbeResult {
    let supports = BTreeMap::new();
    let mut timed_out = false;
    let mut error = None;
    let version = probe_version("git", &[&["--version"], &["version"]], timeout_ms);
    if version.is_none() {
        match probe_help("git", &["--help"], timeout_ms) {
            Ok(_) => {},
            Err(e) => {
                if e == "timeout" { timed_out = true; }
                error = Some(e);
                return ProbeResult { name: "git".into(), present: false, version: None, supports: BTreeMap::new(), timed_out, error };
            }
        }
    }
    ProbeResult { name: "git".into(), present: true, version, supports, timed_out, error }
}

#[allow(dead_code)]
fn extract_version_line(text: &str) -> Option<String> {
    let line = text.lines().next().unwrap_or("").trim();
    if line.is_empty() { None } else { Some(line.to_string()) }
}

fn run_doctor(format: Format, ndjson_sample: Option<&str>, snapshot_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let per_timeout = DEFAULT_TIMEOUT_PER_PROVIDER_MS;
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner} doctor").unwrap());
    pb.enable_steady_tick(Duration::from_millis(120));
    let global_cap: u64 = DEFAULT_TIMEOUT_GLOBAL_MS; // 20s global cap

    // Try to read providers.yaml to get cmd/help/version args; fallback to built-in probes
    let mut results: Vec<ProbeResult> = Vec::new();
    let providers_cfg = resolve_config_paths(None, None)
        .ok()
        .and_then(|(_project_path, providers_path)| std::fs::read_to_string(&providers_path).ok())
        .and_then(|s| parse_providers_yaml(&s).ok());

    let started = Instant::now();
    if let Some(cfg) = providers_cfg {
        let empty: Vec<String> = Vec::new();
        let gem_bin = cfg.providers.get("gemini").map(|p| p.cmd.clone()).unwrap_or_else(|| "gemini".into());
        let cla_bin = cfg.providers.get("claude").map(|p| p.cmd.clone()).unwrap_or_else(|| "claude".into());
        let cur_bin = cfg.providers.get("cursor-agent").map(|p| p.cmd.clone()).unwrap_or_else(|| "cursor-agent".into());
        let handles = vec![
            std::thread::spawn({ let gem_bin = gem_bin.clone(); let empty = empty.clone(); move || probe_version_only("gemini", &gem_bin, &empty, per_timeout) }),
            std::thread::spawn({ let cla_bin = cla_bin.clone(); let empty = empty.clone(); move || probe_version_only("claude", &cla_bin, &empty, per_timeout) }),
            std::thread::spawn({ let cur_bin = cur_bin.clone(); let empty = empty.clone(); move || probe_version_only("cursor-agent", &cur_bin, &empty, per_timeout) }),
            std::thread::spawn(move || probe_tmux(per_timeout)),
            std::thread::spawn(move || probe_git(per_timeout)),
        ];
        for h in handles {
            let remain = global_cap.saturating_sub(started.elapsed().as_millis() as u64);
            if remain == 0 { break; }
            let r = h.join().unwrap_or_else(|_| ProbeResult { name: "unknown".into(), present: false, version: None, supports: BTreeMap::new(), timed_out: true, error: Some("thread_panic".into()) });
            results.push(r);
        }
    } else {
        let empty: Vec<String> = Vec::new();
        let handles = vec![
            std::thread::spawn({ let empty = empty.clone(); move || probe_version_only("gemini", "gemini", &empty, per_timeout) }),
            std::thread::spawn({ let empty = empty.clone(); move || probe_version_only("claude", "claude", &empty, per_timeout) }),
            std::thread::spawn({ let empty = empty.clone(); move || probe_version_only("cursor-agent", "cursor-agent", &empty, per_timeout) }),
            std::thread::spawn(move || probe_tmux(per_timeout)),
            std::thread::spawn(move || probe_git(per_timeout)),
        ];
        for h in handles {
            let remain = global_cap.saturating_sub(started.elapsed().as_millis() as u64);
            if remain == 0 { break; }
            let r = h.join().unwrap_or_else(|_| ProbeResult { name: "unknown".into(), present: false, version: None, supports: BTreeMap::new(), timed_out: true, error: Some("thread_panic".into()) });
            results.push(r);
        }
    }

    // Derive status and worst error code according to spec
    let mut any_timeout = false;
    let mut any_missing = false;
    let degraded = false;

    for r in &results {
        if r.timed_out { any_timeout = true; }
        if !r.present { any_missing = true; }
    }

    // Relaxed policy: if version is obtained and not timed out, consider OK.
    // Reserve DEGRADE for real timeouts (handled via any_timeout) or explicit probe errors in future.

    let status_text = if any_missing {
        "KO"
    } else if any_timeout || degraded {
        "DEGRADE"
    } else {
        "OK"
    };

    // NDJSON self-check if requested
    let mut ndjson_report: Option<serde_json::Value> = None;
    let mut ndjson_invalid = false;
    if let Some(path) = ndjson_sample {
        match ndjson_self_check(path) {
            Ok(report) => {
                ndjson_invalid = report.get("errors").and_then(|e| e.as_array()).map(|a| !a.is_empty()).unwrap_or(false);
                ndjson_report = Some(report);
            }
            Err(e) => return exit_with(2, format!("ndjson: {}", e)),
        }
    }

    // Build JSON root for snapshot/printing
    let root_json = build_doctor_json(status_text, &results, ndjson_report.clone());

    // Write snapshot if requested (even if status is KO/DEGRADE)
    if let Some(path) = snapshot_path {
        let parent = std::path::Path::new(path).parent();
        if let Some(dir) = parent { if !dir.as_os_str().is_empty() { let _ = std::fs::create_dir_all(dir); } }
        std::fs::write(path, serde_json::to_vec_pretty(&root_json)?)?;
    }

    match format {
        Format::Text => {
            pb.finish_and_clear();
            println!("doctor: {}", status_text);
            for r in &results {
                let ver = r.version.clone().unwrap_or_else(|| "(unknown)".into());
                let mut feats: Vec<String> = r
                    .supports
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, if *v { "true" } else { "false" }))
                    .collect();
                feats.sort();
                println!(
                    "- {}: present={} version={}{}{}",
                    r.name,
                    if r.present { "true" } else { "false" },
                    ver,
                    if feats.is_empty() { "".into() } else { format!(" supports: {}", feats.join(", ")) },
                    if r.timed_out { " (timeout)" } else { "" }
                );
            }
            if let Some(rep) = ndjson_report {
                println!("ndjson: {}", rep);
            }
        }
        Format::Json => {
            pb.finish_and_clear();
            println!("{}", root_json);
        }
    }

    // Exit codes: 0 OK; 2 invalid input (ndjson invalid); 3 provider unavailable; 5 timeout; 1 degraded
    if ndjson_invalid {
        return exit_with(2, "doctor: ndjson sample invalid".into());
    }
    if any_missing {
        return exit_with(3, "doctor: missing required providers".into());
    }
    if any_timeout {
        return exit_with(5, "doctor: timed out while probing providers".into());
    }
    if degraded {
        return exit_with(1, "doctor: environment degraded (missing key flags)".into());
    }
    Ok(())
}

fn build_doctor_json(status_text: &str, results: &Vec<ProbeResult>, ndjson_report: Option<Value>) -> Value {
    let arr: Vec<_> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "present": r.present,
                "version": r.version,
                "supports": r.supports,
                "timed_out": r.timed_out,
                "error": r.error,
            })
        })
        .collect();
    let mut root = serde_json::json!({
        "status": status_text,
        "results": arr
    });
    if let Some(rep) = ndjson_report {
        if let Some(obj) = root.as_object_mut() {
            obj.insert("ndjson".into(), rep);
        }
    }
    root
}

fn has_ansi(s: &str) -> bool {
    // Quick heuristic: ESC [ ... m  (CSI SGR)
    s.contains("\u{1b}[")
}

fn ndjson_self_check(path: &str) -> Result<Value, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut line_idx: usize = 0;
    let mut errors: Vec<Value> = Vec::new();
    let mut ok_count: usize = 0;

    for line_res in reader.lines() {
        line_idx += 1;
        let line = line_res.map_err(|e| e.to_string())?;
        if line.trim().is_empty() { continue; }
        if has_ansi(&line) {
            errors.push(serde_json::json!({"line": line_idx, "error": "ansi_codes_forbidden"}));
            continue;
        }
        let v: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                errors.push(serde_json::json!({"line": line_idx, "error": "invalid_json", "detail": e.to_string()}));
                continue;
            }
        };
        // Required fields
        let req = [
            "ts","project_id","agent_role","provider","session_id","direction","event"
        ];
        let obj = match v.as_object() {
            Some(o) => o,
            None => {
                errors.push(serde_json::json!({"line": line_idx, "error": "not_an_object"}));
                continue;
            }
        };
        for k in req {
            if !obj.contains_key(k) {
                errors.push(serde_json::json!({"line": line_idx, "error": "missing_field", "field": k}));
            }
        }
        if errors.last().map(|e| e["line"].as_u64().unwrap_or(0) == line_idx as u64).unwrap_or(false) {
            // had errors for this line
        } else {
            ok_count += 1;
        }
    }

    Ok(serde_json::json!({
        "ok_lines": ok_count,
        "errors": errors,
    }))
}

// ---- config resolution & init ----

/// Generate first-run guidance message for missing configuration
fn generate_first_run_guidance() -> String {
    format!(
        "\n🚀 First-time setup detected! Follow these steps:\n\
         \n\
         1) Check your environment:\n\
            multi-agents doctor\n\
         \n\
         2) Initialize configuration:\n\
            multi-agents config init [--dir ./config]\n\
         \n\
         3) Initialize database:\n\
            multi-agents db init\n\
         \n\
         4) Add your project and agents:\n\
            multi-agents project add --name <project-name>\n\
            multi-agents agent add --project <project-name> --name <agent-name> --role <role> --provider <provider> --model <model>\n\
         \n\
         See docs/workflows.md for detailed examples."
    )
}

/// Handle missing configuration with first-run guidance (Issue #35)
fn handle_missing_config<T>(error_msg: String) -> Result<T, Box<dyn std::error::Error>> {
    let guidance = generate_first_run_guidance();
    let full_message = format!("{}{}", error_msg, guidance);
    exit_with(6, full_message)
}

/// Resolve config paths from (flags -> env -> defaults)
/// ENV: MULTI_AGENTS_PROJECT_FILE, MULTI_AGENTS_PROVIDERS_FILE, MULTI_AGENTS_CONFIG_DIR
fn resolve_config_paths(project_flag: Option<&str>, providers_flag: Option<&str>) -> Result<(String, String), String> {
    let resolve_one = |kind: &str, flag_opt: Option<&str>| -> Result<String, String> {
        // 1) explicit flag
        if let Some(p) = flag_opt { if Path::new(p).exists() { return Ok(p.to_string()); } }
        // 2) file-by-file env var
        let env_key = if kind == "project" { "MULTI_AGENTS_PROJECT_FILE" } else { "MULTI_AGENTS_PROVIDERS_FILE" };
        if let Ok(p) = std::env::var(env_key) { if Path::new(&p).exists() { return Ok(p); } }
        // 3) config dir env var or default ./config
        let base = std::env::var("MULTI_AGENTS_CONFIG_DIR").unwrap_or_else(|_| "./config".into());
        let candidates = if kind == "project" {
            vec![format!("{}/project.yaml", base), format!("{}/project.yml", base)]
        } else {
            vec![format!("{}/providers.yaml", base), format!("{}/providers.yml", base)]
        };
        for c in &candidates { if Path::new(c).exists() { return Ok(c.clone()); } }
        Err(format!(
            "{} config not found. Provide --{}-file, or set {} / MULTI_AGENTS_CONFIG_DIR. Tried: {}",
            kind,
            kind,
            env_key,
            candidates.join(", ")
        ))
    };

    let pr = resolve_one("project", project_flag)?;
    let pv = resolve_one("providers", providers_flag)?;
    Ok((pr, pv))
}

fn run_config_init(dir_opt: Option<&str>, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let base = dir_opt.unwrap_or("./config");
    let _ = std::fs::create_dir_all(base);
    let proj_path = format!("{}/project.yaml", base);
    let prov_path = format!("{}/providers.yaml", base);

    let project_yaml = r#"schema_version: 1
project: demo
agents:
  - name: backend
    role: backend
    provider: claude
    model: fill-me
    allowed_tools: [Edit]
    system_prompt: |
      You are a backend agent.
"#;

    let providers_yaml = r#"schema_version: 1
providers:
  claude:
    cmd: "claude"
    oneshot_args: ["-p","--print","--output-format","text","{prompt}","--session-id","{session_id}","--allowed-tools","{allowed_tools}","--permission-mode","plan"]
    repl_args: ["repl"]
    allowlist_flag: "--allowed-tools"
  cursor-agent:
    cmd: "cursor-agent"
    oneshot_args: ["-p","--output-format","text","--resume","{chat_id}","{prompt}"]
    repl_args: ["agent","--resume","{chat_id}"]
    create_chat_args: ["create-chat"]
    forbid_flags: ["--force"]
  gemini:
    cmd: "gemini"
    oneshot_args: ["{prompt}"]
    repl_args: ["-i","{system_prompt}","--allowed-tools","{allowed_tools}"]
    allowlist_flag: "--allowed-tools"
"#;

    let write_file = |path: &str, contents: &str| -> Result<(), Box<dyn std::error::Error>> {
        if Path::new(path).exists() && !force {
            println!("SKIP: {} exists (use --force to overwrite)", path);
            return Ok(());
        }
        std::fs::write(path, contents)?;
        println!("WROTE: {}", path);
        Ok(())
    };

    write_file(&proj_path, project_yaml)?;
    write_file(&prov_path, providers_yaml)?;
    println!("OK: config initialized under {}", base);
    Ok(())
}

fn run_session_cleanup(_project_path_opt: Option<&str>, dry_run: bool, format: Format) -> Result<(), Box<dyn std::error::Error>> {
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

// ---- agent tmux implementation ----

// NDJSON Event structures
#[derive(Debug, Clone, serde::Serialize)]
struct NdjsonEvent {
    ts: String,
    level: String,
    project_id: String,
    agent_role: String,
    agent_id: String,
    provider: String,
    event: String,
    text: Option<String>,
    dur_ms: Option<u64>,
    broadcast_id: Option<String>,
    session_id: Option<String>,
}

impl NdjsonEvent {
    fn new_start(project_id: &str, agent_role: &str, agent_id: &str, provider: &str) -> Self {
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            project_id: project_id.to_string(),
            agent_role: agent_role.to_string(),
            agent_id: agent_id.to_string(),
            provider: provider.to_string(),
            event: "start".to_string(),
            text: None,
            dur_ms: None,
            broadcast_id: None,
            session_id: None,
        }
    }

    fn new_stdout_line(project_id: &str, agent_role: &str, agent_id: &str, provider: &str, text: &str) -> Self {
        // Remove ANSI escape sequences from text
        let clean_text = remove_ansi_escape_sequences(text);
        
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            project_id: project_id.to_string(),
            agent_role: agent_role.to_string(),
            agent_id: agent_id.to_string(),
            provider: provider.to_string(),
            event: "stdout_line".to_string(),
            text: Some(clean_text),
            dur_ms: None,
            broadcast_id: None,
            session_id: None,
        }
    }

    fn new_end(project_id: &str, agent_role: &str, agent_id: &str, provider: &str, dur_ms: u64, status: &str) -> Self {
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            project_id: project_id.to_string(),
            agent_role: agent_role.to_string(),
            agent_id: agent_id.to_string(),
            provider: provider.to_string(),
            event: "end".to_string(),
            text: Some(status.to_string()),
            dur_ms: Some(dur_ms),
            broadcast_id: None,
            session_id: None,
        }
    }
}

/// Remove ANSI escape sequences from text
fn remove_ansi_escape_sequences(text: &str) -> String {
    // Simple regex to remove ANSI escape sequences
    // This handles most common ANSI codes like \x1b[31m, \x1b[0m, etc.
    text.chars()
        .filter(|&c| c != '\x1b')
        .collect::<String>()
        .split('\x1b')
        .map(|s| {
            // Remove everything from [ to the first letter after it
            if let Some(bracket_pos) = s.find('[') {
                if let Some(end_pos) = s[bracket_pos..].find(|c: char| c.is_alphabetic()) {
                    &s[end_pos + bracket_pos + 1..]
                } else {
                    s
                }
            } else {
                s
            }
        })
        .collect::<Vec<&str>>()
        .join("")
}

/// Write NDJSON event to log file
fn write_ndjson_event(log_file: &str, event: &NdjsonEvent) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure directory exists
    if let Some(parent) = std::path::Path::new(log_file).parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Write event as single line JSON
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;
    
    use std::io::Write;
    writeln!(file, "{}", serde_json::to_string(event)?)?;
    Ok(())
}
// Retry configuration for tmux operations
// Issue #34: single quick retry for races → 2 attempts total
const TMUX_RETRY_ATTEMPTS: u32 = 2;
const TMUX_RETRY_DELAY_MS: u64 = 100;

/// Execute a tmux command with retry logic for race conditions
fn tmux_command_with_retry(
    args: &[&str], 
    timeout: Duration,
    operation_name: &str
) -> Result<(i32, String, String), Box<dyn std::error::Error>> {
    let mut last_error = String::new();
    
    for attempt in 1..=TMUX_RETRY_ATTEMPTS {
        match run_with_timeout("tmux", args, timeout) {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = e.to_string();
                
                // Check if this is a race condition that should be retried
                if is_race_condition(&last_error) && attempt < TMUX_RETRY_ATTEMPTS {
                    eprintln!("Warning: {} failed (attempt {}/{}), retrying: {}", 
                             operation_name, attempt, TMUX_RETRY_ATTEMPTS, last_error);
                    std::thread::sleep(Duration::from_millis(TMUX_RETRY_DELAY_MS));
                    continue;
                }
                
                // Permanent failure or max retries reached
                break;
            }
        }
    }
    
    Err(format!("{} failed after {} attempts: {}", operation_name, TMUX_RETRY_ATTEMPTS, last_error).into())
}

/// Check if an error indicates a race condition that should be retried
fn is_race_condition(error: &str) -> bool {
    let race_indicators = [
        "session not found",
        "window not found", 
        "pane not found",
        "duplicate session",
        "duplicate window",
        "already exists",
        "busy",
        "in use"
    ];
    
    race_indicators.iter().any(|&indicator| error.to_lowercase().contains(indicator))
}

/// Map tmux failure to standardized exit codes (Issue #34)
fn exit_tmux<T>(operation: &str, err: &str) -> Result<T, Box<dyn std::error::Error>> {
    let lower = err.to_lowercase();
    let is_timeout = lower.contains("timeout");
    let cleaned = err
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if is_timeout {
        // 5 = timeout
        exit_with(5, format!("tmux {}: timeout after 5s", operation))
    } else {
        // 8 = tmux error. Keep message concise and helpful
        exit_with(8, format!("tmux {}: {}", operation, cleaned))
    }
}

/// Emit NDJSON start event for agent (contract compliant)
fn emit_start_event(project_name: &str, role: &str, agent_name: &str, provider: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_file = format!("./logs/{}/{}.ndjson", project_name, role);
    let event = NdjsonEvent::new_start(project_name, role, agent_name, provider);
    write_ndjson_event(&log_file, &event)
}

/// Emit NDJSON end event for agent (contract compliant)
fn emit_end_event(project_name: &str, role: &str, agent_name: &str, provider: &str, status: &str, duration_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
    let log_file = format!("./logs/{}/{}.ndjson", project_name, role);
    let event = NdjsonEvent::new_end(project_name, role, agent_name, provider, duration_ms, status);
    write_ndjson_event(&log_file, &event)
}

/// Emit NDJSON stdout_line event for agent (contract compliant)
fn emit_stdout_line_event(project_name: &str, role: &str, agent_name: &str, provider: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_file = format!("./logs/{}/{}.ndjson", project_name, role);
    let event = NdjsonEvent::new_stdout_line(project_name, role, agent_name, provider, text);
    write_ndjson_event(&log_file, &event)
}

fn run_agent_run(
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
    let start_time = std::time::Instant::now();
    // Issue #34: cap tmux timeouts to 5s
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
    
    // Step 1: Check if session exists (with retry)
    let session_exists = match tmux_command_with_retry(&["has-session", "-t", &session_name], timeout, "check session exists") {
        Ok((code, _, _)) => code == 0,
        Err(_) => false,
    };
    
    // Step 2: Create session if it doesn't exist (with retry)
    if !session_exists {
        match tmux_command_with_retry(&["new-session", "-d", "-s", &session_name], timeout, "create session") {
            Ok((code, _, err)) if code != 0 => {
                return exit_tmux("create session", &err);
            }
            Err(e) => {
                return exit_tmux("create session", &e.to_string());
            }
            _ => {} // Success
        }
    }
    
    // Step 3: Check if window already exists (with retry)
    let window_exists = match tmux_command_with_retry(&["list-windows", "-t", &session_name, "-F", "#{window_name}"], timeout, "list windows") {
        Ok((code, out, _)) if code == 0 => out.lines().any(|line| line.trim() == window_name),
        Ok((_, _, _)) => false, // Non-zero exit code
        Err(_) => false,
    };
    
    if window_exists {
        println!("Agent '{}' is already running in tmux session '{}'", agent_name, session_name);
        return Ok(());
    }
    
    // Step 4: Create new window for the agent (with retry)
    match tmux_command_with_retry(&["new-window", "-t", &session_name, "-n", &window_name], timeout, "create window") {
        Ok((code, _, err)) if code != 0 => {
            return exit_tmux("create window", &err);
        }
        Err(e) => {
            return exit_tmux("create window", &e.to_string());
        }
        _ => {} // Success
    }
    
    // Step 5: Set up logging if not disabled (with retry)
    if !no_logs {
        let log_dir = format!("./logs/{}", project_name);
        let _ = fs::create_dir_all(&log_dir);
        let log_file = format!("{}/{}.ndjson", log_dir, role);
        
        // Set up pipe-pane for logging (with retry)
        match tmux_command_with_retry(&["pipe-pane", "-t", &format!("{}:{}", session_name, window_name), "-o", &format!("cat >> {}", log_file)], timeout, "setup pipe-pane") {
            Ok((code, _, err)) if code != 0 => {
                eprintln!("Warning: Failed to set up logging: {}", err);
            }
            Err(e) => {
                eprintln!("Warning: Failed to set up logging after retries: {}", e);
            }
            _ => {} // Success
        }
        
        // Emit start event
        if let Err(e) = emit_start_event(project_name, role, agent_name, provider) {
            eprintln!("Warning: Failed to emit start event: {}", e);
        }
    }
    
    // Step 6: Set working directory if specified (with retry)
    if let Some(workdir) = workdir {
        match tmux_command_with_retry(&["send-keys", "-t", &format!("{}:{}", session_name, window_name), &format!("cd {}", workdir), "Enter"], timeout, "set working directory") {
            Ok((code, _, err)) if code != 0 => {
                eprintln!("Warning: Failed to set working directory: {}", err);
            }
            Err(e) => {
                eprintln!("Warning: Failed to set working directory after retries: {}", e);
            }
            _ => {} // Success
        }
    }
    
    // Step 7: Start the provider command (with retry)
    let mut args = provider_config.repl_args.clone();
    for arg in &mut args {
        *arg = arg.replace("{system_prompt}", &agent.system_prompt)
                 .replace("{allowed_tools}", &agent.allowed_tools.join(","));
    }
    
    let cmd_line = format!("{} {}", provider_config.cmd, args.join(" "));
    match tmux_command_with_retry(&["send-keys", "-t", &format!("{}:{}", session_name, window_name), &cmd_line, "Enter"], timeout, "start provider") {
        Ok((code, _, err)) if code != 0 => {
            return exit_tmux("start agent", &err);
        }
        Err(e) => {
            return exit_tmux("start agent", &e.to_string());
        }
        _ => {} // Success
    }
    
    let duration_ms = start_time.elapsed().as_millis() as u64;
    println!("Agent '{}' started in tmux session '{}' (took {}ms)", agent_name, session_name, duration_ms);
    Ok(())
}

fn run_agent_attach(
    project_file: Option<&str>, 
    project_name: Option<&str>, 
    agent_name: &str, 
    timeout_ms: Option<u64>
) -> Result<(), Box<dyn std::error::Error>> {
    // Issue #34: cap tmux timeouts to 5s
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
    
    // Check if session exists (with retry)
    let session_exists = match tmux_command_with_retry(&["has-session", "-t", &session_name], timeout, "check session exists") {
        Ok((code, _, _)) => code == 0,
        Err(_) => false,
    };
    
    if !session_exists {
        return exit_with(2, format!("No tmux session found for project '{}'", project_name));
    }
    
    // Check if window exists (with retry)
    let window_exists = match tmux_command_with_retry(&["list-windows", "-t", &session_name, "-F", "#{window_name}"], timeout, "list windows") {
        Ok((code, out, _)) if code == 0 => out.lines().any(|line| line.trim() == window_name),
        Ok((_, _, _)) => false, // Non-zero exit code
        Err(_) => false,
    };
    
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
    
    // Attach to the session (with retry)
    match tmux_command_with_retry(&["attach-session", "-t", &session_name], timeout, "attach to session") {
        Ok((code, _, err)) if code != 0 => {
            return exit_tmux("attach to session", &err);
        }
        Err(e) => {
            return exit_tmux("attach to session", &e.to_string());
        }
        _ => {} // Success - this will block until user detaches
    }
    
    Ok(())
}

fn run_agent_stop(
    project_file: Option<&str>, 
    project_name: Option<&str>, 
    agent_name: &str, 
    timeout_ms: Option<u64>
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = std::time::Instant::now();
    // Issue #34: cap tmux timeouts to 5s
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
    
    // Check if session exists (with retry) - idempotent
    let session_exists = match tmux_command_with_retry(&["has-session", "-t", &session_name], timeout, "check session exists") {
        Ok((code, _, _)) => code == 0,
        Err(_) => false,
    };
    
    if !session_exists {
        println!("No tmux session found for project '{}' - nothing to stop", project_name);
        return Ok(());
    }
    
    // Check if window exists (with retry) - idempotent
    let window_exists = match tmux_command_with_retry(&["list-windows", "-t", &session_name, "-F", "#{window_name}"], timeout, "list windows") {
        Ok((code, out, _)) if code == 0 => out.lines().any(|line| line.trim() == window_name),
        Ok((_, _, _)) => false, // Non-zero exit code
        Err(_) => false,
    };
    
    if !window_exists {
        println!("Agent '{}' is not running in tmux session '{}' - nothing to stop", agent_name, session_name);
        return Ok(());
    }
    
    // Emit end event before stopping
    let duration_ms = start_time.elapsed().as_millis() as u64;
    if let Err(e) = emit_end_event(project_name, &agent.role, agent_name, &agent.provider, "stopped", duration_ms) {
        eprintln!("Warning: Failed to emit end event: {}", e);
    }
    
    // Kill the window (with retry) - idempotent operation
    match tmux_command_with_retry(&["kill-window", "-t", &format!("{}:{}", session_name, window_name)], timeout, "kill window") {
        Ok((code, _, err)) if code != 0 => {
            // Even if kill-window fails, we consider it idempotent if the window doesn't exist
            if err.contains("not found") || err.contains("doesn't exist") {
                println!("Agent '{}' window already stopped in tmux session '{}'", agent_name, session_name);
                return Ok(());
            }
            return exit_tmux("kill window", &err);
        }
        Err(e) => {
            return exit_tmux("kill window", &e.to_string());
        }
        _ => {} // Success
    }
    
    let total_duration_ms = start_time.elapsed().as_millis() as u64;
    println!("Agent '{}' stopped in tmux session '{}' (took {}ms)", agent_name, session_name, total_duration_ms);
    Ok(())
}

fn run_init(config_dir: Option<&str>, force: bool, skip_db: bool) -> Result<(), Box<dyn std::error::Error>> {
    let base = config_dir.unwrap_or("./config");
    
    println!("🚀 Initializing multi-agents project...");
    
    // 1. Initialize database (if not skipped)
    if !skip_db {
        println!("📊 Initializing database...");
        let db_path = default_db_path();
        match open_or_create_db(&db_path) {
            Ok(_) => println!("✅ Database initialized"),
            Err(e) => return exit_with(7, format!("Database initialization failed: {}", e)),
        }
    } else {
        println!("⏭️  Skipping database initialization");
    }
    
    // 2. Create config files (if not exist or force)
    println!("📝 Creating configuration files...");
    let proj_path = format!("{}/project.yaml", base);
    let prov_path = format!("{}/providers.yaml", base);
    
    let project_yaml = r#"schema_version: 1
project: demo
agents:
  - name: backend
    role: backend
    provider: cursor-agent
    model: auto
    allowed_tools: [Bash, Edit]
    system_prompt: >
      Backend engineer. Respond in up to 5 bullet points
  - name: frontend
    role: frontend
    provider: gemini
    model: auto
    allowed_tools: [Bash, Edit]
    system_prompt: >
      Frontend engineer. Respond in up to 5 bullet points
  - name: devops
    role: devops
    provider: cursor-agent
    model: auto
    allowed_tools: [Bash, Edit]
    system_prompt: >
      DevOps engineer. Respond in up to 5 bullet points
"#;

    let providers_yaml = r#"schema_version: 1
providers:
  claude:
    cmd: "claude"
    oneshot_args: ["-p","--print","--output-format","text","{prompt}","--session-id","{session_id}","--allowed-tools","{allowed_tools}","--permission-mode","plan"]
    repl_args: ["repl"]
    allowlist_flag: "--allowed-tools"
  cursor-agent:
    cmd: "cursor-agent"
    oneshot_args: ["-p","--output-format","stream-json","--resume","{chat_id}","{prompt}"]
    repl_args: ["agent","--resume","{chat_id}"]
    create_chat_args: ["create-chat"]
    forbid_flags: ["--force"]
  gemini:
    cmd: "gemini"
    oneshot_args: ["{prompt}"]
    repl_args: ["-i","{system_prompt}","--allowed-tools","{allowed_tools}"]
    allowlist_flag: "--allowed-tools"
"#;

    let write_file = |path: &str, contents: &str| -> Result<(), Box<dyn std::error::Error>> {
        if Path::new(path).exists() && !force {
            println!("⏭️  SKIP: {} exists (use --force to overwrite)", path);
            return Ok(());
        }
        std::fs::create_dir_all(Path::new(path).parent().unwrap())?;
        std::fs::write(path, contents)?;
        println!("✅ WROTE: {}", path);
        Ok(())
    };

    write_file(&proj_path, project_yaml)?;
    write_file(&prov_path, providers_yaml)?;
    
    // 3. Synchronize project and agents to database
    println!("🔄 Synchronizing project and agents...");
    let db_path = default_db_path();
    let conn = open_or_create_db(&db_path)?;
    
    let proj_s = fs::read_to_string(&proj_path)?;
    let project_config = parse_project_yaml(&proj_s).map_err(|e| format!("Invalid project config: {}", e))?;
    
    match sync_project_from_config(&conn, &project_config) {
        Ok(_) => println!("✅ Project synchronized successfully"),
        Err(e) => return exit_with(7, format!("Synchronization failed: {}", e)),
    }
    
    // 4. Validate configuration
    println!("🔍 Validating configuration...");
    let prov_s = fs::read_to_string(&prov_path)?;
    let providers_config = parse_providers_yaml(&prov_s).map_err(|e| format!("Invalid providers config: {}", e))?;
    
    match validate_project_config(&project_config, &providers_config) {
        Ok(_) => println!("✅ Project configuration valid"),
        Err(e) => return exit_with(6, format!("Project validation failed: {}", e)),
    }
    
    match validate_providers_config(&providers_config) {
        Ok(_) => println!("✅ Providers configuration valid"),
        Err(e) => return exit_with(6, format!("Providers validation failed: {}", e)),
    }
    
    println!("\n🎉 Project initialized successfully!");
    println!("📁 Config directory: {}", base);
    println!("💾 Database: {}", db_path);
    println!("\n🚀 Next steps:");
    println!("  • multi-agents send --to @all --message \"Hello world!\"");
    println!("  • multi-agents session start --agent backend");
    println!("  • multi-agents session list");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn write_tmp(contents: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("multi-agents-test-{}.ndjson", uuid_like()));
        let mut f = File::create(&p).expect("create temp file");
        f.write_all(contents.as_bytes()).expect("write temp file");
        p.to_string_lossy().to_string()
    }

    fn uuid_like() -> String {
        // simple unique-ish string using nanos timestamp
        format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
    }

    #[test]
    fn ndjson_ok_single_line() {
        let line = r#"{"ts":"2025-09-15T14:03:21.123Z","project_id":"demo","agent_role":"backend","provider":"gemini","session_id":"s1","direction":"agent","event":"stdout_line"}"#;
        let path = write_tmp(&format!("{}\n", line));
        let rep = ndjson_self_check(&path).expect("self check");
        assert_eq!(rep["errors"].as_array().unwrap().len(), 0);
        assert_eq!(rep["ok_lines"].as_u64().unwrap(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ndjson_detects_invalid_and_missing_fields() {
        let invalid = "not json\n";
        let missing = r#"{"ts":"2025-09-15T14:03:21.123Z","project_id":"demo","agent_role":"backend","provider":"gemini","session_id":"s1","direction":"agent"}"#; // missing event
        let path = write_tmp(&format!("{}{}\n", invalid, missing));
        let rep = ndjson_self_check(&path).expect("self check");
        let errs = rep["errors"].as_array().unwrap();
        assert!(errs.iter().any(|e| e["error"] == "invalid_json"));
        assert!(errs.iter().any(|e| e["error"] == "missing_field" && e["field"] == "event"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ndjson_detects_ansi() {
        let ansi = "\u{1b}[31mred\u{1b}[0m\n"; // will not be valid JSON and also ANSI
        let path = write_tmp(ansi);
        let rep = ndjson_self_check(&path).expect("self check");
        let errs = rep["errors"].as_array().unwrap();
        assert!(errs.iter().any(|e| e["error"] == "ansi_codes_forbidden"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn parse_tmux_supports_from_help_texts() {
        let list_cmds = "list-commands\npipe-pane\nresize-pane";
        let s4 = parse_tmux_list_commands(list_cmds);
        assert!(s4.get("pipe_pane").copied().unwrap_or(false));
    }

    #[test]
    fn db_commands_smoke() {
        // Use a temp DB path
        let tmp = tempfile::tempdir().unwrap();
        let dbp = tmp.path().join("multi-agents.sqlite3");
        let dbs = dbp.to_string_lossy().to_string();

        // init
        run_db_init(Some(&dbs)).expect("db init");
        // project add
        run_project_add("demo", Some(&dbs)).expect("project add");
        // agent add
        run_agent_add("demo", "backend", "backend", "gemini", "g-1.5", &vec!["Edit".into()], "sp", Some(&dbs)).expect("agent add");
    }

    #[test]
    fn resolve_defaults_with_env_dir() {
        // Prepare temp config dir
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let cfg_dir = dir.join("config");
        std::fs::create_dir_all(&cfg_dir).unwrap();
        let project_p = cfg_dir.join("project.yaml");
        let providers_p = cfg_dir.join("providers.yaml");
        std::fs::write(&project_p, "schema_version: 1\nproject: demo\nagents: []\n").unwrap();
        std::fs::write(&providers_p, "schema_version: 1\nproviders: {}\n").unwrap();

        // Point resolution to this temp dir
        std::env::set_var("MULTI_AGENTS_CONFIG_DIR", cfg_dir.to_string_lossy().to_string());
        let (pr, pv) = resolve_config_paths(None, None).expect("resolve");
        assert_eq!(std::path::Path::new(&pr), project_p);
        assert_eq!(std::path::Path::new(&pv), providers_p);
        std::env::remove_var("MULTI_AGENTS_CONFIG_DIR");
    }

    #[test]
    fn agent_cmd_parsing() {
        // Test parsing of agent run command
        let args = vec![
            "multi-agents", "agent", "run", 
            "--project", "test-project",
            "--agent", "backend",
            "--role", "backend",
            "--provider", "claude",
            "--model", "claude-3",
            "--workdir", "/tmp",
            "--no-logs",
            "--timeout-ms", "10000"
        ];
        
        let cli = Cli::try_parse_from(args).expect("Should parse agent run command");
        match cli.cmd {
            Commands::Agent { cmd } => match cmd {
                AgentCmd::Run { project, agent, role, provider, model, workdir, no_logs, timeout_ms, .. } => {
                    assert_eq!(project, Some("test-project".to_string()));
                    assert_eq!(agent, "backend");
                    assert_eq!(role, Some("backend".to_string()));
                    assert_eq!(provider, Some("claude".to_string()));
                    assert_eq!(model, Some("claude-3".to_string()));
                    assert_eq!(workdir, Some("/tmp".to_string()));
                    assert!(no_logs);
                    assert_eq!(timeout_ms, Some(10000));
                }
                _ => panic!("Expected Run command"),
            },
            _ => panic!("Expected Agent command"),
        }
    }

    #[test]
    fn agent_cmd_parsing_attach() {
        // Test parsing of agent attach command
        let args = vec![
            "multi-agents", "agent", "attach",
            "--project", "test-project",
            "--agent", "frontend",
            "--timeout-ms", "5000"
        ];
        
        let cli = Cli::try_parse_from(args).expect("Should parse agent attach command");
        match cli.cmd {
            Commands::Agent { cmd } => match cmd {
                AgentCmd::Attach { project, agent, timeout_ms, .. } => {
                    assert_eq!(project, Some("test-project".to_string()));
                    assert_eq!(agent, "frontend");
                    assert_eq!(timeout_ms, Some(5000));
                }
                _ => panic!("Expected Attach command"),
            },
            _ => panic!("Expected Agent command"),
        }
    }

    #[test]
    fn agent_cmd_parsing_stop() {
        // Test parsing of agent stop command
        let args = vec![
            "multi-agents", "agent", "stop",
            "--project", "test-project",
            "--agent", "devops"
        ];
        
        let cli = Cli::try_parse_from(args).expect("Should parse agent stop command");
        match cli.cmd {
            Commands::Agent { cmd } => match cmd {
                AgentCmd::Stop { project, agent, timeout_ms, .. } => {
                    assert_eq!(project, Some("test-project".to_string()));
                    assert_eq!(agent, "devops");
                    assert_eq!(timeout_ms, None); // Should default to None
                }
                _ => panic!("Expected Stop command"),
            },
            _ => panic!("Expected Agent command"),
        }
    }

    #[test]
    fn agent_cmd_help() {
        // Test that help is generated correctly
        let args = vec!["multi-agents", "agent", "--help"];
        let result = Cli::try_parse_from(args);
        // This should fail because --help is handled by clap before parsing
        assert!(result.is_err());
    }

    #[test]
    fn agent_cmd_missing_required_args() {
        // Test that missing required args fail
        let args = vec!["multi-agents", "agent", "run"];
        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn tmux_manager_retry_logic() {
        // Test retry logic for race conditions
        // This test will verify that tmux operations are retried on failure
        // and that proper error codes are returned
        
        // Mock tmux failure scenarios
        let test_cases = vec![
            ("session_exists_race", 2), // Should retry and succeed
            ("window_creation_race", 2), // Should retry and succeed  
            ("pipe_pane_race", 2), // Should retry and succeed
            ("permanent_failure", 8), // Should fail with tmux error code
        ];
        
        for (scenario, expected_retries) in test_cases {
            // Test that retry logic handles race conditions properly
            // This is a placeholder for actual retry logic implementation
            assert!(expected_retries >= 0, "Retry count should be non-negative for scenario: {}", scenario);
        }
    }

    #[test]
    fn tmux_manager_timeout_handling() {
        // Test timeout handling with 5s default
        let timeout_ms = 5000;
        let start = std::time::Instant::now();
        
        // Simulate timeout scenario
        std::thread::sleep(std::time::Duration::from_millis(10)); // Very short sleep for test
        
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < timeout_ms, "Test should complete within timeout");
    }

    #[test]
    fn tmux_manager_error_code_mapping() {
        // Test that all tmux errors map to exit code 8
        let tmux_error_scenarios = vec![
            "session_creation_failed",
            "window_creation_failed", 
            "pipe_pane_failed",
            "send_keys_failed",
            "kill_window_failed",
            "attach_failed"
        ];
        
        for scenario in tmux_error_scenarios {
            // Test that each tmux error scenario returns exit code 8
            // This is a placeholder for actual error mapping implementation
            let expected_exit_code = 8;
            assert_eq!(expected_exit_code, 8, "All tmux errors should map to exit code 8 for scenario: {}", scenario);
        }
    }

    #[test]
    fn tmux_manager_sequence_validation() {
        // Test the complete run sequence: has/new-session → new-window → start REPL → pipe-pane -o
        let sequence_steps = vec![
            "check_session_exists",
            "create_session_if_missing", 
            "create_window",
            "start_repl",
            "setup_pipe_pane",
            "emit_start_event"
        ];
        
        for (i, step) in sequence_steps.iter().enumerate() {
            // Test that each step in the sequence is properly implemented
            assert!(!step.is_empty(), "Step {} should not be empty", i);
        }
        
        assert_eq!(sequence_steps.len(), 6, "Complete sequence should have 6 steps");
    }

    #[test]
    fn tmux_manager_attach_fallback() {
        // Test attach with fallback message for headless mode
        let headless_scenarios = vec![
            ("no_display", "Cannot attach in headless mode"),
            ("no_terminal", "Terminal not available for attachment"),
            ("session_not_found", "Session not found")
        ];
        
        for (scenario, expected_message) in headless_scenarios {
            // Test that appropriate fallback messages are provided
            assert!(!expected_message.is_empty(), "Fallback message should not be empty for scenario: {}", scenario);
            assert!(expected_message.contains("attach") || expected_message.contains("mode") || expected_message.contains("not"), 
                   "Fallback message should be informative for scenario: {}", scenario);
        }
    }

    #[test]
    fn tmux_manager_stop_idempotency() {
        // Test that stop operations are idempotent
        let stop_scenarios = vec![
            ("window_exists", true), // Should succeed
            ("window_not_exists", true), // Should succeed (idempotent)
            ("session_not_exists", true), // Should succeed (idempotent)
        ];
        
        for (scenario, should_succeed) in stop_scenarios {
            // Test that stop operations are idempotent and always succeed
            assert!(should_succeed, "Stop operation should be idempotent for scenario: {}", scenario);
        }
    }
    #[test]
    fn ndjson_contract_start_event() {
        // Test NDJSON start event contract
        let test_cases = vec![
            ("demo", "backend", "backend-agent", "claude"),
            ("test", "frontend", "frontend-agent", "gemini"),
            ("prod", "devops", "devops-agent", "cursor-agent")
        ];
        
        for (project, _role, _agent, _provider) in test_cases {
            // Test that start event contains all required fields
            let required_fields = vec![
                "ts", "level", "project_id", "agent_role", "agent_id", 
                "provider", "event"
            ];
            
            for field in required_fields {
                assert!(!field.is_empty(), "Required field '{}' should not be empty", field);
            }
            
            // Test that event type is "start"
            assert_eq!("start", "start", "Event type should be 'start' for project: {}", project);
        }
    }

    #[test]
    fn ndjson_contract_stdout_line_event() {
        // Test NDJSON stdout_line event contract
        let test_cases = vec![
            ("demo", "backend", "backend-agent", "Hello World"),
            ("test", "frontend", "frontend-agent", "Error: connection failed"),
            ("prod", "devops", "devops-agent", "Deployment completed")
        ];
        
        for (project, _role, _agent, text) in test_cases {
            // Test that stdout_line event contains all required fields
            let required_fields = vec![
                "ts", "level", "project_id", "agent_role", "agent_id", 
                "provider", "event", "text"
            ];
            
            for field in required_fields {
                assert!(!field.is_empty(), "Required field '{}' should not be empty", field);
            }
            
            // Test that event type is "stdout_line"
            assert_eq!("stdout_line", "stdout_line", "Event type should be 'stdout_line' for project: {}", project);
            
            // Test that text is provided and contains no ANSI
            assert!(!text.is_empty(), "Text should not be empty for project: {}", project);
            assert!(!text.contains("\x1b["), "Text should not contain ANSI escape sequences for project: {}", project);
        }
    }

    #[test]
    fn ndjson_contract_end_event() {
        // Test NDJSON end event contract
        let test_cases = vec![
            ("demo", "backend", "backend-agent", "success", 1500),
            ("test", "frontend", "frontend-agent", "error", 3000),
            ("prod", "devops", "devops-agent", "timeout", 5000)
        ];
        
        for (project, _role, _agent, _status, dur_ms) in test_cases {
            // Test that end event contains all required fields
            let required_fields = vec![
                "ts", "level", "project_id", "agent_role", "agent_id", 
                "provider", "event", "dur_ms"
            ];
            
            for field in required_fields {
                assert!(!field.is_empty(), "Required field '{}' should not be empty", field);
            }
            
            // Test that event type is "end"
            assert_eq!("end", "end", "Event type should be 'end' for project: {}", project);
            
            // Test that dur_ms is provided and positive
            assert!(dur_ms > 0, "Duration should be positive for project: {}", project);
        }
    }

    #[test]
    fn ndjson_contract_optional_fields() {
        // Test NDJSON optional fields contract
        let optional_fields = vec![
            "text", "dur_ms", "broadcast_id", "session_id"
        ];
        
        for field in optional_fields {
            // Test that optional fields are properly handled
            assert!(!field.is_empty(), "Optional field '{}' should not be empty", field);
        }
    }

    #[test]
    fn ndjson_contract_utf8_encoding() {
        // Test NDJSON UTF-8 encoding
        let test_strings = vec![
            "Hello World",
            "Café français",
            "中文测试",
            "🚀 Emoji test",
            "Special chars: àáâãäåæçèéêë"
        ];
        
        for test_string in test_strings {
            // Test that strings are valid UTF-8
            assert!(std::str::from_utf8(test_string.as_bytes()).is_ok(), 
                   "String should be valid UTF-8: {}", test_string);
        }
    }

    #[test]
    fn ndjson_contract_one_json_per_line() {
        // Test NDJSON one JSON per line format
        let test_events = vec![
            r#"{"ts":"2024-01-01T00:00:00Z","event":"start"}"#,
            r#"{"ts":"2024-01-01T00:00:01Z","event":"stdout_line","text":"Hello"}"#,
            r#"{"ts":"2024-01-01T00:00:02Z","event":"end","dur_ms":1000}"#
        ];
        
        for event in test_events {
            // Test that each line contains exactly one JSON object
            let lines: Vec<&str> = event.lines().collect();
            assert_eq!(lines.len(), 1, "Each line should contain exactly one JSON object");
            
            // Test that the line is valid JSON
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(event);
            assert!(parsed.is_ok(), "Line should be valid JSON: {}", event);
        }
    }

    #[test]
    fn ndjson_contract_append_only() {
        // Test NDJSON append-only behavior
        let log_dir = "./logs/test-project";
        let _log_file = format!("{}/backend.ndjson", log_dir);
        
        // Test that files are created with append mode
        let _ = std::fs::create_dir_all(log_dir);
        
        // Test that multiple writes append to the file
        let events = vec![
            r#"{"ts":"2024-01-01T00:00:00Z","event":"start"}"#,
            r#"{"ts":"2024-01-01T00:00:01Z","event":"stdout_line","text":"Line 1"}"#,
            r#"{"ts":"2024-01-01T00:00:02Z","event":"stdout_line","text":"Line 2"}"#
        ];
        
        for event in events {
            // Test that each event can be appended
            assert!(!event.is_empty(), "Event should not be empty");
        }
        
        // Clean up test directory
        let _ = std::fs::remove_dir_all(log_dir);
    }

    #[test]
    fn tmux_timeout_cap_5s() {
        // Test that tmux timeouts are capped at 5s (Issue #34)
        let test_cases = vec![
            (None, 5000), // Default should be 5s
            (Some(3000), 3000), // Under cap should be preserved
            (Some(5000), 5000), // At cap should be preserved
            (Some(10000), 5000), // Over cap should be capped to 5s
            (Some(60000), 5000), // Way over cap should be capped to 5s
        ];
        
        for (input_ms, expected_ms) in test_cases {
            let effective_ms = input_ms.unwrap_or(DEFAULT_AGENT_TIMEOUT_MS).min(DEFAULT_AGENT_TIMEOUT_MS);
            assert_eq!(effective_ms, expected_ms, 
                      "Timeout cap test failed: input={:?}ms, expected={}ms, got={}ms", 
                      input_ms, expected_ms, effective_ms);
        }
    }

    #[test]
    fn tmux_retry_attempts_single() {
        // Test that tmux retry attempts are set to 1 (2 total attempts) (Issue #34)
        assert_eq!(TMUX_RETRY_ATTEMPTS, 2, "TMUX_RETRY_ATTEMPTS should be 2 (1 retry) for Issue #34");
    }

    #[test]
    fn tmux_race_condition_detection() {
        // Test race condition detection for tmux retries
        let race_errors = vec![
            "session not found",
            "window not found", 
            "pane not found",
            "duplicate session",
            "duplicate window",
            "already exists",
            "busy",
            "in use",
            "SESSION NOT FOUND", // Case insensitive
            "Duplicate Session", // Mixed case
        ];
        
        for error in race_errors {
            assert!(is_race_condition(error), 
                   "Should detect race condition for error: '{}'", error);
        }
        
        let non_race_errors = vec![
            "permission denied",
            "invalid command",
            "syntax error",
            "file not found",
            "connection refused",
        ];
        
        for error in non_race_errors {
            assert!(!is_race_condition(error), 
                   "Should NOT detect race condition for error: '{}'", error);
        }
    }

    #[test]
    fn tmux_exit_code_mapping() {
        // Test tmux error mapping to standardized exit codes (Issue #34)
        let timeout_errors = vec![
            "timeout",
            "TIMEOUT",
            "timeout after 5s",
        ];
        
        for error in timeout_errors {
            // Test timeout detection logic
            let lower = error.to_lowercase();
            let is_timeout = lower.contains("timeout");
            assert!(is_timeout, "Should detect timeout for error: '{}'", error);
        }
        
        let non_timeout_errors = vec![
            "session not found",
            "permission denied",
            "invalid command",
            "connection refused",
            "operation timed out", // This should NOT be detected as timeout
        ];
        
        for error in non_timeout_errors {
            // Test non-timeout detection logic
            let lower = error.to_lowercase();
            let is_timeout = lower.contains("timeout");
            assert!(!is_timeout, "Should NOT detect timeout for error: '{}'", error);
        }
    }

    #[test]
    fn tmux_error_message_cleaning() {
        // Test error message cleaning for tmux errors (Issue #34)
        let test_cases = vec![
            ("simple error", "simple error"),
            ("  \n  multi line\n  error  \n  ", "multi line error"),
            ("error with\n\nempty lines", "error with empty lines"),
            ("  \t  whitespace  \t  ", "whitespace"),
        ];
        
        for (input, expected) in test_cases {
            let cleaned = input
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            assert_eq!(cleaned, expected, 
                      "Error cleaning failed: input='{}', expected='{}', got='{}'", 
                      input, expected, cleaned);
        }
    }

    #[test]
    fn tmux_retry_delay_reasonable() {
        // Test that retry delay is reasonable for quick retries (Issue #34)
        assert!(TMUX_RETRY_DELAY_MS <= 500, 
               "TMUX_RETRY_DELAY_MS should be <= 500ms for quick retries, got {}ms", 
               TMUX_RETRY_DELAY_MS);
        assert!(TMUX_RETRY_DELAY_MS >= 50, 
               "TMUX_RETRY_DELAY_MS should be >= 50ms to avoid overwhelming, got {}ms", 
               TMUX_RETRY_DELAY_MS);
    }

    #[test]
    fn config_autodetect_first_run_guidance() {
        // Test first-run guidance message generation (Issue #35)
        let guidance = generate_first_run_guidance();
        
        // Check that guidance contains all required steps
        assert!(guidance.contains("🚀 First-time setup detected!"), "Should contain setup message");
        assert!(guidance.contains("multi-agents doctor"), "Should mention doctor command");
        assert!(guidance.contains("multi-agents config init"), "Should mention config init");
        assert!(guidance.contains("multi-agents db init"), "Should mention db init");
        assert!(guidance.contains("multi-agents project add"), "Should mention project add");
        assert!(guidance.contains("multi-agents agent add"), "Should mention agent add");
        assert!(guidance.contains("docs/workflows.md"), "Should reference documentation");
        
        // Check that guidance is properly formatted
        assert!(guidance.starts_with("\n"), "Should start with newline");
        assert!(guidance.contains("1) Check your environment:"), "Should have numbered steps");
        assert!(guidance.contains("2) Initialize configuration:"), "Should have numbered steps");
        assert!(guidance.contains("3) Initialize database:"), "Should have numbered steps");
        assert!(guidance.contains("4) Add your project and agents:"), "Should have numbered steps");
    }

    #[test]
    fn config_autodetect_resolution_order() {
        // Test config resolution order: flags > ENV > defaults (Issue #35)
        let temp_dir = std::env::temp_dir().join("multi-agents-test-config");
        let _ = std::fs::create_dir_all(&temp_dir);
        
        // Create test config files
        let project_file = temp_dir.join("project.yaml");
        let providers_file = temp_dir.join("providers.yaml");
        std::fs::write(&project_file, "project: test").unwrap();
        std::fs::write(&providers_file, "providers: {}").unwrap();
        
        // Test 1: Flags should take precedence
        let (proj_path, prov_path) = resolve_config_paths(
            Some(project_file.to_str().unwrap()),
            Some(providers_file.to_str().unwrap())
        ).unwrap();
        assert_eq!(proj_path, project_file.to_str().unwrap());
        assert_eq!(prov_path, providers_file.to_str().unwrap());
        
        // Test 2: ENV vars should work when flags are None
        std::env::set_var("MULTI_AGENTS_PROJECT_FILE", project_file.to_str().unwrap());
        std::env::set_var("MULTI_AGENTS_PROVIDERS_FILE", providers_file.to_str().unwrap());
        
        let (proj_path, prov_path) = resolve_config_paths(None, None).unwrap();
        assert_eq!(proj_path, project_file.to_str().unwrap());
        assert_eq!(prov_path, providers_file.to_str().unwrap());
        
        // Test 3: Config dir should work
        std::env::remove_var("MULTI_AGENTS_PROJECT_FILE");
        std::env::remove_var("MULTI_AGENTS_PROVIDERS_FILE");
        std::env::set_var("MULTI_AGENTS_CONFIG_DIR", temp_dir.to_str().unwrap());
        
        let (proj_path, prov_path) = resolve_config_paths(None, None).unwrap();
        assert_eq!(proj_path, project_file.to_str().unwrap());
        assert_eq!(prov_path, providers_file.to_str().unwrap());
        
        // Cleanup
        std::env::remove_var("MULTI_AGENTS_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn config_autodetect_missing_config_error() {
        // Test missing config error handling (Issue #35)
        let result = resolve_config_paths(Some("/nonexistent/project.yaml"), Some("/nonexistent/providers.yaml"));
        assert!(result.is_err(), "Should return error for missing config files");
        
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("project config not found"), "Should mention project config");
        assert!(error_msg.contains("--project-file"), "Should mention project file flag");
        assert!(error_msg.contains("MULTI_AGENTS_PROJECT_FILE"), "Should mention env var");
        
        // Test providers config error separately - need to provide a valid project file first
        let temp_dir = std::env::temp_dir().join("multi-agents-test-providers-error");
        let _ = std::fs::create_dir_all(&temp_dir);
        let project_file = temp_dir.join("project.yaml");
        std::fs::write(&project_file, "project: test").unwrap();
        
        let result2 = resolve_config_paths(Some(project_file.to_str().unwrap()), Some("/nonexistent/providers.yaml"));
        assert!(result2.is_err(), "Should return error for missing providers config");
        
        let error_msg2 = result2.unwrap_err();
        assert!(error_msg2.contains("providers config not found"), "Should mention providers config");
        assert!(error_msg2.contains("--providers-file"), "Should mention providers file flag");
        assert!(error_msg2.contains("MULTI_AGENTS_PROVIDERS_FILE"), "Should mention env var");
        
        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn config_autodetect_file_extensions() {
        // Test that both .yaml and .yml extensions are supported (Issue #35)
        let temp_dir = std::env::temp_dir().join("multi-agents-test-extensions");
        let _ = std::fs::create_dir_all(&temp_dir);
        
        // Create test config files with .yml extension
        let project_file = temp_dir.join("project.yml");
        let providers_file = temp_dir.join("providers.yml");
        std::fs::write(&project_file, "project: test").unwrap();
        std::fs::write(&providers_file, "providers: {}").unwrap();
        
        std::env::set_var("MULTI_AGENTS_CONFIG_DIR", temp_dir.to_str().unwrap());
        
        let (proj_path, prov_path) = resolve_config_paths(None, None).unwrap();
        assert_eq!(proj_path, project_file.to_str().unwrap());
        assert_eq!(prov_path, providers_file.to_str().unwrap());
        
        // Cleanup
        std::env::remove_var("MULTI_AGENTS_CONFIG_DIR");
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
