//! Tmux retry logic for race conditions

use std::time::Duration;
use crate::utils::{TMUX_RETRY_ATTEMPTS, TMUX_RETRY_DELAY_MS};
use crate::utils::timeouts::run_with_timeout;

/// Execute a tmux command with retry logic for race conditions
pub fn tmux_command_with_retry(
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
pub fn is_race_condition(error: &str) -> bool {
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
