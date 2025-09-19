//! Tmux operations and utilities

/// Map tmux failure to standardized exit codes
pub fn exit_tmux<T>(operation: &str, err: &str) -> Result<T, Box<dyn std::error::Error>> {
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
        crate::utils::errors::exit_with(5, format!("tmux {}: timeout after 5s", operation))
    } else {
        // 8 = tmux error. Keep message concise and helpful
        crate::utils::errors::exit_with(8, format!("tmux {}: {}", operation, cleaned))
    }
}
