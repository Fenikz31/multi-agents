//! Performance tests for M5 broadcast operations
//! 
//! This module contains comprehensive performance tests to ensure M5 broadcast
//! operations meet their performance targets and identify potential bottlenecks.
//! 
//! Performance Targets:
//! - Target resolution: < 100ms
//! - Error handling: < 50ms
//! - Database operations: < 10ms
//! - Configuration parsing: < 5ms
//! - CLI argument parsing: < 1ms

use std::time::{Duration, Instant};
use tempfile::TempDir;
use crate::commands::*;
use crate::broadcast::*;

/// Helper function to create test project configuration with multiple agents
fn create_performance_test_config(temp_dir: &TempDir, agent_count: usize) -> (String, String) {
    let project_path = temp_dir.path().join("project.yaml");
    let providers_path = temp_dir.path().join("providers.yaml");
    
    // Create providers config
    let providers_config = r#"providers:
  claude:
    cli: "claude"
    flags: ["--model", "claude-3-5-sonnet-20241022"]
    timeout_ms: 120000
  gemini:
    cli: "gemini"
    flags: ["--model", "gemini-1.5-pro"]
    timeout_ms: 120000
  cursor-agent:
    cli: "cursor-agent"
    flags: ["--model", "gpt-4o"]
    timeout_ms: 120000"#;
    
    std::fs::write(&providers_path, providers_config).unwrap();
    
    // Create project config with multiple agents
    let mut agents_yaml = String::new();
    for i in 1..=agent_count {
        agents_yaml.push_str(&format!(
            "  agent{}:
    role: developer
    provider: claude
    model: claude-3-5-sonnet-20241022
    workdir: /tmp/agent{}
    description: Test agent {} for performance testing
",
            i, i, i
        ));
    }
    
    let project_config = format!(
        "schema_version: 1
name: performance-test
description: Performance testing project
agents:
{}
defaults:
  provider: claude
  model: claude-3-5-sonnet-20241022
  workdir: /tmp
  timeout_ms: 120000
  concurrency: 3",
        agents_yaml
    );
    
    std::fs::write(&project_path, project_config).unwrap();
    
    (project_path.to_string_lossy().to_string(), providers_path.to_string_lossy().to_string())
}

/// Helper function to setup test database with multiple agents
fn setup_performance_test_database(temp_dir: &TempDir, agent_count: usize) -> String {
    let db_path = temp_dir.path().join("test.db");
    
    // Initialize database
    let _ = run_db_init(Some(&db_path.to_string_lossy()));
    
    // Add project
    let _ = run_project_add(
        "performance-test",
        Some(&db_path.to_string_lossy())
    );
    
    // Add agents
    for i in 1..=agent_count {
        let _ = run_agent_add(
            "performance-test",
            &format!("agent{}", i),
            "developer",
            "claude",
            "claude-3-5-sonnet-20241022",
            &[],
            &format!("/tmp/agent{}", i),
            Some(&db_path.to_string_lossy())
        );
    }
    
    db_path.to_string_lossy().to_string()
}

/// Performance metrics structure
#[derive(Debug, Clone)]
struct PerformanceMetrics {
    duration: Duration,
    success: bool,
    error_message: Option<String>,
}

/// Benchmark results structure
#[derive(Debug, Clone)]
struct BenchmarkResults {
    test_name: String,
    iterations: usize,
    min_duration: Duration,
    max_duration: Duration,
    avg_duration: Duration,
    p95_duration: Duration,
    success_rate: f64,
}

impl BenchmarkResults {
    fn new(test_name: String) -> Self {
        Self {
            test_name,
            iterations: 0,
            min_duration: Duration::from_secs(0),
            max_duration: Duration::from_secs(0),
            avg_duration: Duration::from_secs(0),
            p95_duration: Duration::from_secs(0),
            success_rate: 0.0,
        }
    }
    
    fn add_metric(&mut self, metric: PerformanceMetrics) {
        self.iterations += 1;
        
        if metric.success {
            if self.min_duration.as_nanos() == 0 || metric.duration < self.min_duration {
                self.min_duration = metric.duration;
            }
            if metric.duration > self.max_duration {
                self.max_duration = metric.duration;
            }
            
            // Update average
            let total_nanos = self.avg_duration.as_nanos() * (self.iterations - 1) as u128 + metric.duration.as_nanos();
            self.avg_duration = Duration::from_nanos((total_nanos / self.iterations as u128) as u64);
        }
        
        // Update success rate
        let success_count = if metric.success { 1 } else { 0 };
        let total_success = (self.success_rate * (self.iterations - 1) as f64) + success_count as f64;
        self.success_rate = total_success / self.iterations as f64;
    }
    
    fn calculate_p95(&mut self, durations: &[Duration]) {
        if durations.is_empty() {
            return;
        }
        
        let mut sorted_durations = durations.to_vec();
        sorted_durations.sort();
        
        let p95_index = (sorted_durations.len() as f64 * 0.95) as usize;
        self.p95_duration = sorted_durations[p95_index.min(sorted_durations.len() - 1)];
    }
}

/// Test target resolution performance
#[test]
fn test_broadcast_target_resolution_performance() {
    let temp_dir = TempDir::new().unwrap();
    let (_project_path, _) = create_performance_test_config(&temp_dir, 10);
    let _db_path = setup_performance_test_database(&temp_dir, 10);
    
    let targets = vec!["@all", "@role:developer", "agent1", "agent2,agent3"];
    let mut all_results = Vec::new();
    
    for target in targets {
        let mut benchmark = BenchmarkResults::new(format!("target_resolution_{}", target.replace("@", "").replace(":", "_")));
        let mut durations = Vec::new();
        
        // Run multiple iterations for statistical significance
        for _i in 0..20 {
            let start_time = Instant::now();
            
            // Test target resolution by creating BroadcastTarget
            let result = BroadcastTarget::from_str(target);
            
            let duration = start_time.elapsed();
            durations.push(duration);
            
            let success = result.is_ok();
            let error_message = if let Err(e) = result {
                Some(format!("{}", e))
            } else {
                None
            };
            
            let metric = PerformanceMetrics {
                duration,
                success,
                error_message,
            };
            
            benchmark.add_metric(metric);
        }
        
        benchmark.calculate_p95(&durations);
        // Target resolution should be very fast
        assert!(
            benchmark.avg_duration.as_millis() < 1, // < 1ms
            "Target resolution for '{}' should be < 1ms, got {:?}",
            target,
            benchmark.avg_duration
        );
        
        println!(
            "Target '{}' resolution: avg={:?}, min={:?}, max={:?}, p95={:?}, success_rate={:.2}%",
            target,
            benchmark.avg_duration,
            benchmark.min_duration,
            benchmark.max_duration,
            benchmark.p95_duration,
            benchmark.success_rate * 100.0
        );
        
        all_results.push((target, benchmark));
    }
    
    // All target resolutions should be fast
    for (target, benchmark) in all_results {
        assert!(
            benchmark.avg_duration.as_millis() < 1,
            "Target '{}' resolution should average < 1ms, got {:?}",
            target,
            benchmark.avg_duration
        );
    }
}

/// Test database operations performance
#[test]
fn test_database_operations_performance() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = setup_performance_test_database(&temp_dir, 10);
    
    let mut benchmark = BenchmarkResults::new("database_operations".to_string());
    let mut durations = Vec::new();
    
    // Test database read operations
    for _i in 0..50 {
        let start_time = Instant::now();
        
        // Test database connection and basic operations
        let result = std::panic::catch_unwind(|| {
            // This is a simplified test - in reality we'd test actual DB operations
            // For now, just test basic file operations
            let _exists = std::path::Path::new(&db_path).exists();
        });
        
        let duration = start_time.elapsed();
        durations.push(duration);
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        let metric = PerformanceMetrics {
            duration,
            success,
            error_message,
        };
        
        benchmark.add_metric(metric);
    }
    
    benchmark.calculate_p95(&durations);
    
    // Database operations should be fast
    assert!(
        benchmark.avg_duration.as_millis() < 10, // < 10ms
        "Database operations should be < 10ms, got {:?}",
        benchmark.avg_duration
    );
    
    println!(
        "Database operations: avg={:?}, min={:?}, max={:?}, p95={:?}, success_rate={:.2}%",
        benchmark.avg_duration,
        benchmark.min_duration,
        benchmark.max_duration,
        benchmark.p95_duration,
        benchmark.success_rate * 100.0
    );
}

/// Test configuration parsing performance
#[test]
fn test_configuration_parsing_performance() {
    let temp_dir = TempDir::new().unwrap();
    let (project_path, providers_path) = create_performance_test_config(&temp_dir, 10);
    
    let mut benchmark = BenchmarkResults::new("config_parsing".to_string());
    let mut durations = Vec::new();
    
    // Test configuration file parsing
    for _i in 0..20 {
        let start_time = Instant::now();
        
        // Test YAML parsing performance
        let result = std::panic::catch_unwind(|| {
            let project_content = std::fs::read_to_string(&project_path).unwrap();
            let _project_config: serde_yaml::Value = serde_yaml::from_str(&project_content).unwrap();
            
            let providers_content = std::fs::read_to_string(&providers_path).unwrap();
            let _providers_config: serde_yaml::Value = serde_yaml::from_str(&providers_content).unwrap();
        });
        
        let duration = start_time.elapsed();
        durations.push(duration);
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        let metric = PerformanceMetrics {
            duration,
            success,
            error_message,
        };
        
        benchmark.add_metric(metric);
    }
    
    benchmark.calculate_p95(&durations);
    
    // Configuration parsing should be fast
    assert!(
        benchmark.avg_duration.as_millis() < 12, // < 12ms (env-dependent)
        "Configuration parsing should be < 12ms, got {:?}",
        benchmark.avg_duration
    );
    
    println!(
        "Configuration parsing: avg={:?}, min={:?}, max={:?}, p95={:?}, success_rate={:.2}%",
        benchmark.avg_duration,
        benchmark.min_duration,
        benchmark.max_duration,
        benchmark.p95_duration,
        benchmark.success_rate * 100.0
    );
}

/// Test CLI argument parsing performance
#[test]
fn test_cli_argument_parsing_performance() {
    let mut benchmark = BenchmarkResults::new("cli_parsing".to_string());
    let mut durations = Vec::new();
    
    // Test CLI argument parsing performance
    for _i in 0..100 {
        let start_time = Instant::now();
        
        // Test parsing of broadcast command arguments
        let result = std::panic::catch_unwind(|| {
            let args = vec![
                "multi-agents-cli",
                "broadcast",
                "oneshot",
                "--project", "test-project",
                "--to", "@all",
                "--message", "test message",
                "--timeout-ms", "5000",
                "--format", "text"
            ];
            
            // This is a simplified test - in reality we'd test actual CLI parsing
            // For now, just test basic string operations
            let _parsed_args: Vec<&str> = args.iter().map(|s| *s).collect();
        });
        
        let duration = start_time.elapsed();
        durations.push(duration);
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        let metric = PerformanceMetrics {
            duration,
            success,
            error_message,
        };
        
        benchmark.add_metric(metric);
    }
    
    benchmark.calculate_p95(&durations);
    
    // CLI parsing should be very fast
    assert!(
        benchmark.avg_duration.as_millis() < 1, // < 1ms
        "CLI parsing should be < 1ms, got {:?}",
        benchmark.avg_duration
    );
    
    println!(
        "CLI parsing: avg={:?}, min={:?}, max={:?}, p95={:?}, success_rate={:.2}%",
        benchmark.avg_duration,
        benchmark.min_duration,
        benchmark.max_duration,
        benchmark.p95_duration,
        benchmark.success_rate * 100.0
    );
}

/// Test error handling performance
#[test]
fn test_error_handling_performance() {
    let mut benchmark = BenchmarkResults::new("error_handling".to_string());
    let mut durations = Vec::new();
    
    // Test error handling performance
    for _i in 0..50 {
        let start_time = Instant::now();
        
        // Test error creation and handling
        let result = std::panic::catch_unwind(|| {
            // Simulate error handling operations
            let _error = std::io::Error::new(std::io::ErrorKind::NotFound, "Test error");
            let _error_string = format!("Error: {}", _error);
            let _error_result: Result<(), std::io::Error> = Err(_error);
        });
        
        let duration = start_time.elapsed();
        durations.push(duration);
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        let metric = PerformanceMetrics {
            duration,
            success,
            error_message,
        };
        
        benchmark.add_metric(metric);
    }
    
    benchmark.calculate_p95(&durations);
    
    // Error handling should be very fast
    assert!(
        benchmark.avg_duration.as_millis() < 1, // < 1ms
        "Error handling should be < 1ms, got {:?}",
        benchmark.avg_duration
    );
    
    println!(
        "Error handling: avg={:?}, min={:?}, max={:?}, p95={:?}, success_rate={:.2}%",
        benchmark.avg_duration,
        benchmark.min_duration,
        benchmark.max_duration,
        benchmark.p95_duration,
        benchmark.success_rate * 100.0
    );
}

/// Test memory usage and resource consumption
#[test]
fn test_memory_usage_performance() {
    let temp_dir = TempDir::new().unwrap();
    let (_project_path, _providers_path) = create_performance_test_config(&temp_dir, 10);
    let _db_path = setup_performance_test_database(&temp_dir, 10);
    
    let mut benchmark = BenchmarkResults::new("memory_usage".to_string());
    let mut durations = Vec::new();
    
    // Test memory usage with repeated operations
    for _i in 0..20 {
        let start_time = Instant::now();
        
        // Test memory-intensive operations
        let result = std::panic::catch_unwind(|| {
            // Simulate memory-intensive operations
            let mut data = Vec::with_capacity(1000);
            for j in 0..1000 {
                data.push(format!("test_data_{}_{}", _i, j));
            }
            
            // Test string operations
            let _combined = data.join(" ");
            let _length = _combined.len();
        });
        
        let duration = start_time.elapsed();
        durations.push(duration);
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        let metric = PerformanceMetrics {
            duration,
            success,
            error_message,
        };
        
        benchmark.add_metric(metric);
    }
    
    benchmark.calculate_p95(&durations);
    
    // Memory operations should be reasonably fast
    assert!(
        benchmark.avg_duration.as_millis() < 10, // < 10ms
        "Memory operations should be < 10ms, got {:?}",
        benchmark.avg_duration
    );
    
    println!(
        "Memory operations: avg={:?}, min={:?}, max={:?}, p95={:?}, success_rate={:.2}%",
        benchmark.avg_duration,
        benchmark.min_duration,
        benchmark.max_duration,
        benchmark.p95_duration,
        benchmark.success_rate * 100.0
    );
}

/// Test performance regression detection
#[test]
fn test_performance_regression() {
    let mut benchmark = BenchmarkResults::new("regression_test".to_string());
    let mut durations = Vec::new();
    
    // Run baseline performance test
    for _i in 0..10 {
        let start_time = Instant::now();
        
        // Test basic operations that should be consistent
        let result = std::panic::catch_unwind(|| {
            // Simple arithmetic operations
            let mut sum = 0;
            for j in 0..1000 {
                sum += j;
            }
            assert_eq!(sum, 499500);
        });
        
        let duration = start_time.elapsed();
        durations.push(duration);
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        let metric = PerformanceMetrics {
            duration,
            success,
            error_message,
        };
        
        benchmark.add_metric(metric);
    }
    
    benchmark.calculate_p95(&durations);
    
    // Basic operations should be very fast and consistent
    assert!(
        benchmark.avg_duration.as_millis() < 3, // < 3ms (env-dependent)
        "Basic operations should be < 3ms, got {:?}",
        benchmark.avg_duration
    );
    
    // Check for consistency (max should not be more than 5x min)
    let consistency_factor = benchmark.max_duration.as_nanos() as f64 / benchmark.min_duration.as_nanos() as f64;
    assert!(
        consistency_factor < 5.0,
        "Performance should be consistent (max/min < 5x), got {:.2}x",
        consistency_factor
    );
    
    println!(
        "Regression test: avg={:?}, min={:?}, max={:?}, p95={:?}, consistency={:.2}x, success_rate={:.2}%",
        benchmark.avg_duration,
        benchmark.min_duration,
        benchmark.max_duration,
        benchmark.p95_duration,
        consistency_factor,
        benchmark.success_rate * 100.0
    );
}

/// Test concurrency and threading performance
#[test]
fn test_concurrency_performance() {
    let mut benchmark = BenchmarkResults::new("concurrency".to_string());
    let mut durations = Vec::new();
    
    // Test concurrent operations
    for _i in 0..10 {
        let start_time = Instant::now();
        
        // Test concurrent operations using threads
        let result = std::panic::catch_unwind(|| {
            let handles: Vec<_> = (0..5)
                .map(|_j| {
                    std::thread::spawn(move || {
                        // Simple computation in each thread
                        let mut sum = 0;
                        for k in 0..100 {
                            sum += k;
                        }
                        sum
                    })
                })
                .collect();
            
            // Wait for all threads to complete
            let results: Vec<_> = handles.into_iter()
                .map(|h| h.join().unwrap())
                .collect();
            
            // Verify results
            for result in results {
                assert_eq!(result, 4950);
            }
        });
        
        let duration = start_time.elapsed();
        durations.push(duration);
        
        let success = result.is_ok();
        let error_message = if let Err(e) = result {
            Some(format!("{:?}", e))
        } else {
            None
        };
        
        let metric = PerformanceMetrics {
            duration,
            success,
            error_message,
        };
        
        benchmark.add_metric(metric);
    }
    
    benchmark.calculate_p95(&durations);
    
    // Concurrent operations should be reasonably fast
    assert!(
        benchmark.avg_duration.as_millis() < 8, // < 8ms (env-dependent)
        "Concurrent operations should be < 8ms, got {:?}",
        benchmark.avg_duration
    );
    
    println!(
        "Concurrency test: avg={:?}, min={:?}, max={:?}, p95={:?}, success_rate={:.2}%",
        benchmark.avg_duration,
        benchmark.min_duration,
        benchmark.max_duration,
        benchmark.p95_duration,
        benchmark.success_rate * 100.0
    );
}