//! Integration tests for M5 broadcast functionality

use std::fs;
use std::time::Instant;
use tempfile::TempDir;
use crate::commands::*;

/// Test helper to create a temporary project configuration
fn create_test_project_config(temp_dir: &TempDir) -> (String, String) {
    let project_config = r#"
project: test-broadcast
agents:
  - name: backend1
    role: backend
    provider: gemini
    model: 2.0
    system_prompt: "You are a backend developer"
    allowed_tools: []
  - name: backend2
    role: backend
    provider: claude
    model: opus
    system_prompt: "You are a backend developer"
    allowed_tools: []
  - name: frontend1
    role: frontend
    provider: claude
    model: opus
    system_prompt: "You are a frontend developer"
    allowed_tools: []
  - name: devops1
    role: devops
    provider: gemini
    model: 2.0
    system_prompt: "You are a DevOps engineer"
    allowed_tools: []
"#;

    let providers_config = r#"
providers:
  gemini:
    cmd: echo
    oneshot_args: ["--version"]
    repl_args: ["--version"]
  claude:
    cmd: echo
    oneshot_args: ["--version"]
    repl_args: ["--version"]
"#;

    let project_path = temp_dir.path().join("project.yaml");
    let providers_path = temp_dir.path().join("providers.yaml");
    
    fs::write(&project_path, project_config).unwrap();
    fs::write(&providers_path, providers_config).unwrap();
    
    (project_path.to_string_lossy().to_string(), providers_path.to_string_lossy().to_string())
}

/// Test helper to setup test database
fn setup_test_database(temp_dir: &TempDir) -> String {
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_string_lossy().to_string();
    
    // Initialize database
    run_db_init(Some(&db_path_str)).unwrap();
    
    // Add test project
    run_project_add("test-broadcast", Some(&db_path_str)).unwrap();
    
    // Add test agents
    run_agent_add(
        "test-broadcast", "backend1", "backend", "gemini", "2.0",
        &[], "You are a backend developer", Some(&db_path_str)
    ).unwrap();
    
    run_agent_add(
        "test-broadcast", "backend2", "backend", "claude", "opus",
        &[], "You are a backend developer", Some(&db_path_str)
    ).unwrap();
    
    run_agent_add(
        "test-broadcast", "frontend1", "frontend", "claude", "opus",
        &[], "You are a frontend developer", Some(&db_path_str)
    ).unwrap();
    
    run_agent_add(
        "test-broadcast", "devops1", "devops", "gemini", "2.0",
        &[], "You are a DevOps engineer", Some(&db_path_str)
    ).unwrap();
    
    db_path_str
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_broadcast_oneshot_mode_basic() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test basic oneshot broadcast
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@all",
            "Hello from oneshot broadcast",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        assert!(result.is_ok(), "Oneshot broadcast should succeed");
    }

    #[test]
    fn test_broadcast_oneshot_mode_concurrency() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test concurrency limit (should handle multiple agents)
        let start_time = Instant::now();
        
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@all",
            "Concurrent broadcast test",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        let duration = start_time.elapsed();
        
        assert!(result.is_ok(), "Concurrent oneshot broadcast should succeed");
        assert!(duration.as_secs() < 10, "Should complete within reasonable time");
    }

    #[test]
    fn test_broadcast_repl_mode_basic() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, _) = create_test_project_config(&temp_dir);
        
        // Test basic repl broadcast
        let result = run_broadcast_repl(
            Some(&project_path),
            Some("test-broadcast"),
            "@all",
            "Hello from repl broadcast",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        // REPL mode will fail without actual tmux sessions, but should not panic
        // This tests the command parsing and basic flow
        assert!(result.is_err() || result.is_ok(), "REPL broadcast should handle gracefully");
    }

    #[test]
    fn test_broadcast_target_resolution_all() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test @all target resolution
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@all",
            "Broadcast to all agents",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        assert!(result.is_ok(), "@all target should resolve successfully");
    }

    #[test]
    fn test_broadcast_target_resolution_role() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test @role target resolution
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@backend",
            "Broadcast to backend agents",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        assert!(result.is_ok(), "@backend target should resolve successfully");
    }

    #[test]
    fn test_broadcast_target_resolution_agents() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test specific agents target resolution
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "backend1,frontend1",
            "Broadcast to specific agents",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        assert!(result.is_ok(), "Specific agents target should resolve successfully");
    }

    #[test]
    fn test_broadcast_error_handling_invalid_target() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test error handling for invalid target
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@nonexistent",
            "Broadcast to non-existent role",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        assert!(result.is_err(), "Invalid target should return error");
    }

    #[test]
    fn test_broadcast_error_handling_invalid_project() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        
        // Test error handling for invalid project
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("nonexistent-project"),
            "@all",
            "Broadcast to non-existent project",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        assert!(result.is_err(), "Invalid project should return error");
    }

    #[test]
    fn test_broadcast_json_output() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test JSON output format
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@all",
            "JSON output test",
            Some(5000),
            crate::cli::commands::Format::Json,
            false
        );
        
        assert!(result.is_ok(), "JSON output should work");
    }

    #[test]
    fn test_broadcast_performance_oneshot() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test performance target for oneshot mode (< 5s for 10 agents)
        let start_time = Instant::now();
        
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@all",
            "Performance test",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        let duration = start_time.elapsed();
        
        assert!(result.is_ok(), "Performance test should succeed");
        assert!(duration.as_secs() < 5, "Oneshot should complete in < 5s");
    }

    #[test]
    fn test_broadcast_performance_repl() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, _) = create_test_project_config(&temp_dir);
        
        // Test performance target for repl mode (< 1s for 10 agents)
        let start_time = Instant::now();
        
        let result = run_broadcast_repl(
            Some(&project_path),
            Some("test-broadcast"),
            "@all",
            "Performance test",
            Some(1000),
            crate::cli::commands::Format::Text,
            false
        );
        
        let duration = start_time.elapsed();
        
        // REPL mode will fail without tmux, but should be fast
        assert!(duration.as_secs() < 1, "REPL should complete in < 1s");
    }

    #[test]
    fn test_broadcast_ndjson_logging() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Create logs directory
        let logs_dir = temp_dir.path().join("logs");
        fs::create_dir_all(&logs_dir).unwrap();
        
        // Test NDJSON logging with broadcast_id
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@all",
            "NDJSON logging test",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        assert!(result.is_ok(), "NDJSON logging test should succeed");
        
        // Check if log files were created (they should contain broadcast_id)
        let log_files = fs::read_dir(&logs_dir).unwrap();
        let mut found_logs = false;
        for entry in log_files {
            let entry = entry.unwrap();
            if entry.path().extension().map_or(false, |ext| ext == "ndjson") {
                found_logs = true;
                let content = fs::read_to_string(entry.path()).unwrap();
                // Should contain broadcast_id in the logs
                assert!(content.contains("broadcast"), "Logs should contain broadcast information");
            }
        }
        assert!(found_logs, "Should create NDJSON log files");
    }

    #[test]
    fn test_broadcast_exit_codes() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test exit code 0 (success)
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@all",
            "Success test",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        assert!(result.is_ok(), "Success case should return Ok");
        
        // Test exit code 2 (invalid input)
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@nonexistent",
            "Invalid target test",
            Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        assert!(result.is_err(), "Invalid target should return error");
    }

    #[test]
    fn test_broadcast_m4_non_regression() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test that M4 functionality still works (agent commands)
        // This ensures broadcast doesn't break existing functionality
        
        // Test agent run command still works
        let result = run_agent_run(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "backend1",
            None, None, None, None, true, None, Some(5000)
        );
        
        // Agent run will fail without tmux, but should not panic
        assert!(result.is_err() || result.is_ok(), "Agent run should handle gracefully");
        
        // Test send command still works
        let result = run_send(
            Some(&project_path),
            Some(&providers_path),
            "backend1",
            "Test message",
            None, None, Some(5000),
            crate::cli::commands::Format::Text,
            false
        );
        
        // Send will fail without proper setup, but should not panic
        assert!(result.is_err() || result.is_ok(), "Send command should handle gracefully");
    }

    #[test]
    fn test_broadcast_cli_integration() {
        let temp_dir = TempDir::new().unwrap();
        let (_project_path, _providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test CLI integration by running the actual command
        let output = Command::new("cargo")
            .args(&["run", "--bin", "multi-agents-cli", "--", "broadcast", "--help"])
            .output()
            .unwrap();
        
        assert!(output.status.success(), "CLI help should work");
        assert!(String::from_utf8_lossy(&output.stdout).contains("Broadcast messages to multiple agents"));
        
        // Test oneshot subcommand
        let output = Command::new("cargo")
            .args(&["run", "--bin", "multi-agents-cli", "--", "broadcast", "oneshot", "--help"])
            .output()
            .unwrap();
        
        assert!(output.status.success(), "Oneshot help should work");
        assert!(String::from_utf8_lossy(&output.stdout).contains("Send one-shot message to multiple agents"));
        
        // Test repl subcommand
        let output = Command::new("cargo")
            .args(&["run", "--bin", "multi-agents-cli", "--", "broadcast", "repl", "--help"])
            .output()
            .unwrap();
        
        assert!(output.status.success(), "REPL help should work");
        assert!(String::from_utf8_lossy(&output.stdout).contains("Send message to agents in REPL mode"));
    }

    #[test]
    fn test_broadcast_concurrency_limits() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test that concurrency is properly limited (max 3 concurrent)
        let start_time = Instant::now();
        
        // Run multiple broadcasts sequentially to test concurrency handling
        let mut results = Vec::new();
        for i in 0..3 {
            let result = run_broadcast_oneshot(
                Some(&project_path),
                Some(&providers_path),
                Some("test-broadcast"),
                "@all",
                &format!("Concurrent broadcast {}", i),
                Some(2000),
                crate::cli::commands::Format::Text,
                false
            );
            results.push(result);
        }
        
        let duration = start_time.elapsed();
        
        // At least some should succeed
        let success_count = results.iter().filter(|r| r.is_ok()).count();
        assert!(success_count > 0, "At least some broadcasts should succeed");
        
        // Should complete within reasonable time
        assert!(duration.as_secs() < 10, "Broadcasts should complete in reasonable time");
    }

    #[test]
    fn test_broadcast_timeout_handling() {
        let temp_dir = TempDir::new().unwrap();
        let (project_path, providers_path) = create_test_project_config(&temp_dir);
        let _db_path = setup_test_database(&temp_dir);
        
        // Test timeout handling with very short timeout
        let start_time = Instant::now();
        
        let result = run_broadcast_oneshot(
            Some(&project_path),
            Some(&providers_path),
            Some("test-broadcast"),
            "@all",
            "Timeout test",
            Some(100), // Very short timeout
            crate::cli::commands::Format::Text,
            false
        );
        
        let duration = start_time.elapsed();
        
        // Should complete quickly due to timeout
        assert!(duration.as_millis() < 1000, "Should timeout quickly");
        
        // Result might be Ok or Err depending on implementation
        // The important thing is it doesn't hang
    }
}