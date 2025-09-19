//! CLI command definitions

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "multi-agents", version)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
pub enum ConfigCmd {
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
pub enum DbCmd {
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
pub enum SessionCmd {
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
pub enum AgentCmd {
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
        /// Custom logs directory (default: ./logs)
        #[arg(long, value_name = "DIR")] logs_dir: Option<String>,
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
pub enum Format { 
    Text, 
    Json 
}
