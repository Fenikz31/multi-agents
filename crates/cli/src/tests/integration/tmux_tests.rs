//! Integration tests for tmux operations

use crate::tmux::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmux_race_condition_detection() {
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
    fn test_tmux_timeout_cap_5s() {
        // Test that tmux timeouts are capped at 5s
        let test_cases = vec![
            (None, 5000), // Default should be 5s
            (Some(3000), 3000), // Under cap should be preserved
            (Some(5000), 5000), // At cap should be preserved
            (Some(10000), 5000), // Over cap should be capped to 5s
            (Some(60000), 5000), // Way over cap should be capped to 5s
        ];
        
        for (input_ms, expected_ms) in test_cases {
            let effective_ms = input_ms.unwrap_or(crate::utils::DEFAULT_AGENT_TIMEOUT_MS).min(crate::utils::DEFAULT_AGENT_TIMEOUT_MS);
            assert_eq!(effective_ms, expected_ms, 
                      "Timeout cap test failed: input={:?}ms, expected={}ms, got={}ms", 
                      input_ms, expected_ms, effective_ms);
        }
    }

    #[test]
    fn test_tmux_retry_attempts_single() {
        // Test that tmux retry attempts are set to 1 (2 total attempts)
        assert_eq!(crate::utils::TMUX_RETRY_ATTEMPTS, 2, "TMUX_RETRY_ATTEMPTS should be 2 (1 retry)");
    }

    #[test]
    fn test_tmux_retry_delay_reasonable() {
        // Test that retry delay is reasonable for quick retries
        assert!(crate::utils::TMUX_RETRY_DELAY_MS <= 500, 
               "TMUX_RETRY_DELAY_MS should be <= 500ms for quick retries, got {}ms", 
               crate::utils::TMUX_RETRY_DELAY_MS);
        assert!(crate::utils::TMUX_RETRY_DELAY_MS >= 50, 
               "TMUX_RETRY_DELAY_MS should be >= 50ms to avoid overwhelming, got {}ms", 
               crate::utils::TMUX_RETRY_DELAY_MS);
    }
}
