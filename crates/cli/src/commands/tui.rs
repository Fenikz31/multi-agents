//! TUI command: launches the ratatui-based application

use std::error::Error;
use std::time::Duration;

use crate::tui::app::TuiRuntime;
use crate::tui::state::StateManager;
use crate::utils::errors::exit_with;

/// Run the TUI for a given project with optional refresh rate (ms)
pub fn run_tui(project: &str, refresh_rate: Option<u64>) -> Result<(), Box<dyn Error>> {
    // Initialize state manager and pass selected project via context if needed later
    let state_manager = StateManager::new_with_project(Some(project.to_string()));
    let mut app = TuiRuntime::new(state_manager);
    if let Some(ms) = refresh_rate { app.set_tick_rate(Duration::from_millis(ms)); }
    match app.run() {
        Ok(()) => Ok(()),
        Err(err) => {
            let msg = format!("TUI error: {}", err);
            // Map to standardized exit codes: 5 timeout; 7 DB; 1 generic
            let lowered = msg.to_lowercase();
            let code = if lowered.contains("timeout") {
                5
            } else if lowered.contains("db") || lowered.contains("database") || lowered.contains("sqlite") {
                7
            } else {
                1
            };
            exit_with(code, msg)
        }
    }
}


