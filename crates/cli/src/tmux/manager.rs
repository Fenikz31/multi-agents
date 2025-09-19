//! Tmux session and window management

use std::time::Duration;
use crate::utils::errors::exit_with;
use super::retry::tmux_command_with_retry;

/// Tmux manager for handling session and window operations
pub struct TmuxManager {
    timeout: Duration,
}

impl TmuxManager {
    /// Create a new TmuxManager with the specified timeout
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Check if a tmux session exists
    pub fn has_session(&self, session_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        match tmux_command_with_retry(&["has-session", "-t", session_name], self.timeout, "check session exists") {
            Ok((code, _, _)) => Ok(code == 0),
            Err(_) => Ok(false),
        }
    }

    /// Create a new tmux session
    pub fn create_session(&self, session_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        match tmux_command_with_retry(&["new-session", "-d", "-s", session_name], self.timeout, "create session") {
            Ok((code, _, err)) if code != 0 => {
                return exit_with(8, format!("tmux create session: {}", err));
            }
            Err(e) => {
                return exit_with(8, format!("tmux create session: {}", e));
            }
            _ => {} // Success
        }
        Ok(())
    }

    /// Check if a window exists in a session
    pub fn window_exists(&self, session_name: &str, window_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        match tmux_command_with_retry(&["list-windows", "-t", session_name, "-F", "#{window_name}"], self.timeout, "list windows") {
            Ok((code, out, _)) if code == 0 => Ok(out.lines().any(|line| line.trim() == window_name)),
            Ok((_, _, _)) => Ok(false), // Non-zero exit code
            Err(_) => Ok(false),
        }
    }

    /// Create a new window in a session
    pub fn create_window(&self, session_name: &str, window_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        match tmux_command_with_retry(&["new-window", "-t", session_name, "-n", window_name], self.timeout, "create window") {
            Ok((code, _, err)) if code != 0 => {
                return exit_with(8, format!("tmux create window: {}", err));
            }
            Err(e) => {
                return exit_with(8, format!("tmux create window: {}", e));
            }
            _ => {} // Success
        }
        Ok(())
    }

    /// Set up pipe-pane for logging
    pub fn setup_pipe_pane(&self, session_name: &str, window_name: &str, log_file: &str) -> Result<(), Box<dyn std::error::Error>> {
        let target = format!("{}:{}", session_name, window_name);
        match tmux_command_with_retry(&["pipe-pane", "-t", &target, "-o", &format!("cat >> {}", log_file)], self.timeout, "setup pipe-pane") {
            Ok((code, _, err)) if code != 0 => {
                eprintln!("Warning: Failed to set up logging: {}", err);
            }
            Err(e) => {
                eprintln!("Warning: Failed to set up logging after retries: {}", e);
            }
            _ => {} // Success
        }
        Ok(())
    }

    /// Send keys to a window
    pub fn send_keys(&self, session_name: &str, window_name: &str, keys: &str) -> Result<(), Box<dyn std::error::Error>> {
        let target = format!("{}:{}", session_name, window_name);
        match tmux_command_with_retry(&["send-keys", "-t", &target, keys, "Enter"], self.timeout, "send keys") {
            Ok((code, _, err)) if code != 0 => {
                return exit_with(8, format!("tmux send keys: {}", err));
            }
            Err(e) => {
                return exit_with(8, format!("tmux send keys: {}", e));
            }
            _ => {} // Success
        }
        Ok(())
    }

    /// Kill a window
    pub fn kill_window(&self, session_name: &str, window_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let target = format!("{}:{}", session_name, window_name);
        match tmux_command_with_retry(&["kill-window", "-t", &target], self.timeout, "kill window") {
            Ok((code, _, err)) if code != 0 => {
                // Even if kill-window fails, we consider it idempotent if the window doesn't exist
                if err.contains("not found") || err.contains("doesn't exist") {
                    println!("Agent window already stopped in tmux session '{}'", session_name);
                    return Ok(());
                }
                return exit_with(8, format!("tmux kill window: {}", err));
            }
            Err(e) => {
                return exit_with(8, format!("tmux kill window: {}", e));
            }
            _ => {} // Success
        }
        Ok(())
    }

    /// Attach to a session
    pub fn attach_session(&self, session_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        match tmux_command_with_retry(&["attach-session", "-t", session_name], self.timeout, "attach to session") {
            Ok((code, _, err)) if code != 0 => {
                return exit_with(8, format!("tmux attach session: {}", err));
            }
            Err(e) => {
                return exit_with(8, format!("tmux attach session: {}", e));
            }
            _ => {} // Success - this will block until user detaches
        }
        Ok(())
    }
}
