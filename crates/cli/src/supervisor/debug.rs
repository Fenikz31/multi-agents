//! Debug logging utilities for supervisor operations

use std::time::Instant;

/// Debug logging configuration
#[derive(Debug)]
pub struct DebugConfig {
    pub enabled: bool,
    pub log_performance: bool,
    pub log_operations: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("MULTI_AGENTS_DEBUG").is_ok(),
            log_performance: std::env::var("MULTI_AGENTS_DEBUG_PERF").is_ok(),
            log_operations: std::env::var("MULTI_AGENTS_DEBUG_OPS").is_ok(),
        }
    }
}

/// Debug logger for supervisor operations
#[derive(Debug)]
pub struct DebugLogger {
    config: DebugConfig,
}

impl DebugLogger {
    pub fn new() -> Self {
        Self {
            config: DebugConfig::default(),
        }
    }

    pub fn with_config(config: DebugConfig) -> Self {
        Self { config }
    }

    /// Log operation start with timing
    pub fn log_operation_start(&self, operation: &str) -> Option<Instant> {
        if self.config.enabled && self.config.log_operations {
            println!("[DEBUG] Starting operation: {}", operation);
            Some(Instant::now())
        } else {
            None
        }
    }

    /// Log operation completion with timing
    pub fn log_operation_end(&self, operation: &str, start_time: Option<Instant>) {
        if self.config.enabled && self.config.log_operations {
            if let Some(start) = start_time {
                let duration = start.elapsed();
                println!("[DEBUG] Completed operation: {} in {:?}", operation, duration);
            } else {
                println!("[DEBUG] Completed operation: {}", operation);
            }
        }
    }

    /// Log performance metrics
    pub fn log_performance(&self, operation: &str, metrics: &str) {
        if self.config.enabled && self.config.log_performance {
            println!("[DEBUG] Performance - {}: {}", operation, metrics);
        }
    }

    /// Log error with context
    pub fn log_error(&self, operation: &str, error: &str) {
        if self.config.enabled {
            println!("[DEBUG] Error in {}: {}", operation, error);
        }
    }

    /// Log info message
    pub fn log_info(&self, message: &str) {
        if self.config.enabled {
            println!("[DEBUG] {}", message);
        }
    }

    /// Log aggregation statistics
    pub fn log_aggregation_stats(&self, roles_count: usize, total_lines: usize, filtered_lines: usize) {
        if self.config.enabled && self.config.log_operations {
            println!(
                "[DEBUG] Aggregation stats: {} roles, {} total lines, {} filtered lines",
                roles_count, total_lines, filtered_lines
            );
        }
    }

    /// Log metrics computation statistics
    pub fn log_metrics_stats(&self, total_events: usize, unique_broadcasts: usize, roles_count: usize) {
        if self.config.enabled && self.config.log_operations {
            println!(
                "[DEBUG] Metrics stats: {} events, {} unique broadcasts, {} roles",
                total_events, unique_broadcasts, roles_count
            );
        }
    }
}

impl Default for DebugLogger {
    fn default() -> Self {
        Self::new()
    }
}
