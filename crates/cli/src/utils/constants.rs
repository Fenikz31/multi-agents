//! Constants used throughout the application

/// Default timeout for send operations (120 seconds)
pub const DEFAULT_SEND_TIMEOUT_MS: u64 = 120_000;

/// Default timeout for agent operations (5 seconds)
pub const DEFAULT_AGENT_TIMEOUT_MS: u64 = 5_000;

/// Maximum concurrency for one-shot operations
pub const MAX_CONCURRENCY: usize = 3;

/// Default timeout per provider for doctor command (12 seconds)
pub const DEFAULT_TIMEOUT_PER_PROVIDER_MS: u64 = 12000;

/// Default global timeout for doctor command (20 seconds)
pub const DEFAULT_TIMEOUT_GLOBAL_MS: u64 = 20000;

/// Retry configuration for tmux operations
pub const TMUX_RETRY_ATTEMPTS: u32 = 2;
pub const TMUX_RETRY_DELAY_MS: u64 = 100;

/// Default database path (deprecated - use resolve_db_path() instead)
#[deprecated(note = "Use resolve_db_path() from db_path module instead")]
pub fn default_db_path() -> String { 
    "./data/multi-agents.sqlite3".into() 
}
