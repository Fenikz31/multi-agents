//! M7 Optimization and Polish Tests
//! 
//! Tests for performance optimizations, error handling improvements,
//! debug logging, and code quality improvements for M7 modules.

use std::time::{Duration, Instant};
use tempfile::TempDir;
use crate::logging::events::NdjsonEvent;

/// Test performance optimizations for supervisor aggregation
#[test]
fn m7_optimization_supervisor_aggregation_performance() {
    let tmp_dir = TempDir::new().unwrap();
    let project = "m7_opt_agg";
    let roles = vec!["backend", "frontend", "devops"];
    let events_per_role = 100; // 300 total events
    
    // Setup isolated test environment
    let logs_dir = tmp_dir.path().join("logs").join(project);
    std::fs::create_dir_all(&logs_dir).unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();
    
    // Generate test events
    for role in &roles {
        let log_file = logs_dir.join(format!("{}.ndjson", role));
        for i in 0..events_per_role {
            let event = NdjsonEvent::new_routed(
                &chrono::Utc::now().to_rfc3339(),
                role,
                &format!("agent{}", i),
                "claude",
                Some(format!("b-{}", i / 10)), // Group events by broadcast
                Some(format!("m-{}-{}", role, i))
            );
            let _ = crate::logging::ndjson::write_ndjson_event(&log_file.to_string_lossy(), &event);
        }
    }
    
    // Ensure files are written before testing
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // Test aggregation performance
    let start = Instant::now();
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    let event_lines = sub.aggregate_tail(
        roles.iter().map(|r| r.to_string()).collect(),
        Some("routed".to_string()),
        300
    ).expect("aggregation should succeed");
    let agg_time = start.elapsed();
    
    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
    
    // Performance assertions - should be faster than current implementation
    assert!(agg_time < Duration::from_millis(100), "Aggregation should be < 100ms, got {:?}", agg_time);
    assert_eq!(event_lines.len(), 300, "Should aggregate all 300 events");
    
    println!("Optimized aggregation time: {:?}", agg_time);
}

/// Test performance optimizations for metrics computation
#[test]
fn m7_optimization_metrics_computation_performance() {
    let tmp_dir = TempDir::new().unwrap();
    let project = "m7_opt_metrics";
    let roles = vec!["backend", "frontend", "devops"];
    let events_per_role = 200; // 600 total events
    
    // Setup isolated test environment
    let logs_dir = tmp_dir.path().join("logs").join(project);
    std::fs::create_dir_all(&logs_dir).unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();
    
    // Generate test events with varying timestamps for latency calculation
    for role in &roles {
        let log_file = logs_dir.join(format!("{}.ndjson", role));
        for i in 0..events_per_role {
            let base_time = chrono::Utc::now() + chrono::Duration::milliseconds(i as i64);
            let event = NdjsonEvent::new_routed(
                &base_time.to_rfc3339(),
                role,
                &format!("agent{}", i),
                "claude",
                Some(format!("b-{}", i / 20)), // Group events by broadcast
                Some(format!("m-{}-{}", role, i))
            );
            let _ = crate::logging::ndjson::write_ndjson_event(&log_file.to_string_lossy(), &event);
        }
    }
    
    // Ensure files are written before testing
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // Test metrics computation performance
    let start = Instant::now();
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    let event_lines = sub.aggregate_tail(
        roles.iter().map(|r| r.to_string()).collect(),
        Some("routed".to_string()),
        600
    ).expect("aggregation should succeed");
    
    let events: Vec<NdjsonEvent> = event_lines.iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();
    
    let metrics = crate::supervisor::metrics::compute_routed_metrics_from_events(events).expect("metrics computation should succeed");
    let metrics_time = start.elapsed();
    
    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
    
    // Performance assertions - should be faster than current implementation
    assert!(metrics_time < Duration::from_millis(200), "Metrics computation should be < 200ms, got {:?}", metrics_time);
    assert_eq!(metrics.total, 600, "Should compute metrics for all 600 events");
    assert!(metrics.unique_broadcasts >= 30, "Should have at least 30 unique broadcasts");
    
    println!("Optimized metrics computation time: {:?}", metrics_time);
}

/// Test improved error handling with detailed error messages
#[test]
fn m7_optimization_improved_error_handling() {
    let tmp_dir = TempDir::new().unwrap();
    let project = "m7_opt_errors";
    
    // Setup isolated test environment
    let logs_dir = tmp_dir.path().join("logs").join(project);
    std::fs::create_dir_all(&logs_dir).unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();
    
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    
    // Test 1: Non-existent file should return empty results with debug info
    let result = sub.tail_and_filter("nonexistent".to_string(), Some("routed".to_string()), 100);
    assert!(result.is_ok(), "Non-existent file should return Ok with empty results");
    assert_eq!(result.unwrap().len(), 0, "Non-existent file should return empty results");
    
    // Test 2: Invalid JSON should be handled gracefully
    let invalid_log = logs_dir.join("invalid.ndjson");
    std::fs::write(&invalid_log, "invalid json content\n{\"valid\": \"json\"}\n").unwrap();
    
    let result = sub.tail_and_filter("invalid".to_string(), Some("routed".to_string()), 100);
    assert!(result.is_ok(), "Invalid JSON should be handled gracefully");
    let lines = result.unwrap();
    assert_eq!(lines.len(), 1, "Should return only valid JSON lines");
    
    // Test 3: Large file should be handled efficiently
    let large_log = logs_dir.join("large.ndjson");
    let mut content = String::new();
    for i in 0..1000 {
        let event = NdjsonEvent::new_routed(
            &chrono::Utc::now().to_rfc3339(),
            "test",
            &format!("agent{}", i),
            "claude",
            Some(format!("b-{}", i)),
            Some(format!("m-{}", i))
        );
        content.push_str(&serde_json::to_string(&event).unwrap());
        content.push('\n');
    }
    std::fs::write(&large_log, content).unwrap();
    
    let start = Instant::now();
    let result = sub.tail_and_filter("large".to_string(), Some("routed".to_string()), 100);
    let duration = start.elapsed();
    
    assert!(result.is_ok(), "Large file should be handled efficiently");
    assert!(duration < Duration::from_millis(50), "Large file processing should be < 50ms, got {:?}", duration);
    assert_eq!(result.unwrap().len(), 100, "Should respect max_lines limit");
    
    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
    
    println!("Error handling optimizations: large file processed in {:?}", duration);
}

/// Test debug logging improvements
#[test]
fn m7_optimization_debug_logging_improvements() {
    let tmp_dir = TempDir::new().unwrap();
    let project = "m7_opt_debug";
    
    // Setup isolated test environment
    let logs_dir = tmp_dir.path().join("logs").join(project);
    std::fs::create_dir_all(&logs_dir).unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();
    
    // Test debug logging for supervisor operations
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    
    // Create test events
    let log_file = logs_dir.join("backend.ndjson");
    for i in 0..10 {
        let event = NdjsonEvent::new_routed(
            &chrono::Utc::now().to_rfc3339(),
            "backend",
            &format!("agent{}", i),
            "claude",
            Some(format!("b-{}", i)),
            Some(format!("m-{}", i))
        );
        let _ = crate::logging::ndjson::write_ndjson_event(&log_file.to_string_lossy(), &event);
    }
    
    // Ensure files are written before testing
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // Test that operations complete successfully with debug info
    let result = sub.tail_and_filter("backend".to_string(), Some("routed".to_string()), 100);
    assert!(result.is_ok(), "Debug logging should not interfere with operations");
    assert_eq!(result.unwrap().len(), 10, "Should return all 10 events");
    
    // Test aggregation with debug info
    let result = sub.aggregate_tail(vec!["backend".to_string()], Some("routed".to_string()), 100);
    assert!(result.is_ok(), "Aggregation with debug logging should succeed");
    assert_eq!(result.unwrap().len(), 10, "Should aggregate all 10 events");
    
    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
    
    println!("Debug logging improvements: operations completed successfully");
}

/// Test memory optimization for large datasets
#[test]
fn m7_optimization_memory_efficiency() {
    let tmp_dir = TempDir::new().unwrap();
    let project = "m7_opt_memory";
    let roles = vec!["backend", "frontend", "devops"];
    let events_per_role = 500; // 1500 total events
    
    // Setup isolated test environment
    let logs_dir = tmp_dir.path().join("logs").join(project);
    std::fs::create_dir_all(&logs_dir).unwrap();
    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp_dir.path()).unwrap();
    
    // Generate large dataset
    for role in &roles {
        let log_file = logs_dir.join(format!("{}.ndjson", role));
        for i in 0..events_per_role {
            let event = NdjsonEvent::new_routed(
                &chrono::Utc::now().to_rfc3339(),
                role,
                &format!("agent{}", i),
                "claude",
                Some(format!("b-{}", i / 10)),
                Some(format!("m-{}-{}", role, i))
            );
            let _ = crate::logging::ndjson::write_ndjson_event(&log_file.to_string_lossy(), &event);
        }
    }
    
    // Ensure files are written before testing
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // Test memory-efficient processing
    let start = Instant::now();
    let mut sub = crate::supervisor::subscription::SupervisorSubscription::new(project.to_string());
    
    // Process in chunks to test memory efficiency
    let chunk_size = 100;
    let mut total_processed = 0;
    
    for role in &roles {
        let result = sub.tail_and_filter(role.to_string(), Some("routed".to_string()), chunk_size);
        assert!(result.is_ok(), "Chunked processing should succeed");
        total_processed += result.unwrap().len();
    }
    
    let processing_time = start.elapsed();
    
    // Restore original directory
    std::env::set_current_dir(original_dir).unwrap();
    
    // Memory efficiency assertions
    assert!(processing_time < Duration::from_millis(300), "Memory-efficient processing should be < 300ms, got {:?}", processing_time);
    assert_eq!(total_processed, 1500, "Should process all 1500 events");
    
    println!("Memory optimization: processed {} events in {:?}", total_processed, processing_time);
}

/// Test code quality improvements and refactoring
#[test]
fn m7_optimization_code_quality_improvements() {
    // Test that optimized code maintains the same functionality
    let events = vec![
        NdjsonEvent::new_routed(
            &chrono::Utc::now().to_rfc3339(),
            "backend",
            "agent1",
            "claude",
            Some("b1".to_string()),
            Some("m1".to_string())
        ),
        NdjsonEvent::new_routed(
            &chrono::Utc::now().to_rfc3339(),
            "frontend",
            "agent2",
            "claude",
            Some("b1".to_string()),
            Some("m2".to_string())
        ),
        NdjsonEvent::new_routed(
            &chrono::Utc::now().to_rfc3339(),
            "backend",
            "agent3",
            "claude",
            Some("b2".to_string()),
            Some("m3".to_string())
        ),
    ];
    
    // Test both metrics computation methods produce consistent results
    let metrics_from_events = crate::supervisor::metrics::compute_routed_metrics_from_events(events.clone()).expect("metrics from events should succeed");
    
    // Convert to lines and test the other method
    let lines: Vec<String> = events.iter()
        .map(|event| serde_json::to_string(event).unwrap())
        .collect();
    let metrics_from_lines = crate::supervisor::metrics::compute_routed_metrics(lines).expect("metrics from lines should succeed");
    
    // Results should be identical
    assert_eq!(metrics_from_events.total, metrics_from_lines.total, "Total should be consistent");
    assert_eq!(metrics_from_events.unique_broadcasts, metrics_from_lines.unique_broadcasts, "Unique broadcasts should be consistent");
    assert_eq!(metrics_from_events.per_role, metrics_from_lines.per_role, "Per role counts should be consistent");
    
    println!("Code quality improvements: consistent results between methods");
}
