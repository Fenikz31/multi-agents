//! Broadcast-specific metrics collection and tracking

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::logging::ndjson::{emit_metrics_event, emit_failure_metrics_event};

/// Broadcast metrics collector for tracking performance and success rates
#[derive(Debug, Clone)]
pub struct BroadcastMetrics {
    pub broadcast_id: String,
    pub project_id: String,
    pub start_time: Instant,
    pub target_count: usize,
    pub successful_count: usize,
    pub failed_count: usize,
    pub total_duration: Duration,
    pub agent_metrics: HashMap<String, AgentMetrics>,
}

/// Individual agent metrics within a broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub agent_id: String,
    pub role: String,
    pub provider: String,
    pub success: bool,
    pub duration_ms: u64,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub response_size: Option<usize>,
}

/// Broadcast performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastPerformanceSummary {
    pub broadcast_id: String,
    pub project_id: String,
    pub total_duration_ms: u64,
    pub success_rate: f64,
    pub average_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
    pub p95_response_time_ms: u64,
    pub throughput_per_second: f64,
    pub error_rate: f64,
    pub resource_usage: ResourceUsage,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_io_mb: f64,
    pub network_io_mb: f64,
}

impl BroadcastMetrics {
    /// Create a new broadcast metrics collector
    pub fn new(broadcast_id: String, project_id: String, target_count: usize) -> Self {
        Self {
            broadcast_id,
            project_id,
            start_time: Instant::now(),
            target_count,
            successful_count: 0,
            failed_count: 0,
            total_duration: Duration::ZERO,
            agent_metrics: HashMap::new(),
        }
    }

    /// Record a successful agent response
    pub fn record_success(&mut self, agent_id: String, role: String, provider: String, 
                         duration: Duration, response_size: Option<usize>) {
        self.successful_count += 1;
        self.agent_metrics.insert(agent_id.clone(), AgentMetrics {
            agent_id: agent_id.clone(),
            role: role.clone(),
            provider: provider.clone(),
            success: true,
            duration_ms: duration.as_millis() as u64,
            error_type: None,
            error_message: None,
            response_size,
        });

        // Emit success metrics
        let _ = emit_metrics_event(
            &self.project_id,
            &role,
            &agent_id,
            &provider,
            "broadcast_success",
            duration.as_millis() as u64,
            "success",
            Some(&format!("Response size: {} bytes", response_size.unwrap_or(0)))
        );
    }

    /// Record a failed agent response
    pub fn record_failure(&mut self, agent_id: String, role: String, provider: String,
                         duration: Duration, error_type: String, error_message: String) {
        self.failed_count += 1;
        self.agent_metrics.insert(agent_id.clone(), AgentMetrics {
            agent_id: agent_id.clone(),
            role: role.clone(),
            provider: provider.clone(),
            success: false,
            duration_ms: duration.as_millis() as u64,
            error_type: Some(error_type.clone()),
            error_message: Some(error_message.clone()),
            response_size: None,
        });

        // Emit failure metrics
        let _ = emit_failure_metrics_event(
            &self.project_id,
            &role,
            &agent_id,
            &provider,
            "broadcast_failure",
            &error_type,
            duration.as_millis() as u64,
            &error_message
        );
    }

    /// Finalize the broadcast metrics and return performance summary
    pub fn finalize(mut self) -> BroadcastPerformanceSummary {
        self.total_duration = self.start_time.elapsed();
        
        let total_responses = self.successful_count + self.failed_count;
        let success_rate = if total_responses > 0 {
            self.successful_count as f64 / total_responses as f64
        } else {
            0.0
        };

        let error_rate = 1.0 - success_rate;

        // Calculate response time statistics
        let response_times: Vec<u64> = self.agent_metrics.values()
            .map(|m| m.duration_ms)
            .collect();
        
        let average_response_time_ms = if !response_times.is_empty() {
            response_times.iter().sum::<u64>() as f64 / response_times.len() as f64
        } else {
            0.0
        };

        let min_response_time_ms = response_times.iter().min().copied().unwrap_or(0);
        let max_response_time_ms = response_times.iter().max().copied().unwrap_or(0);

        // Calculate P95 response time
        let mut sorted_times = response_times.clone();
        sorted_times.sort();
        let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
        let p95_response_time_ms = sorted_times.get(p95_index.min(sorted_times.len() - 1))
            .copied()
            .unwrap_or(0);

        // Calculate throughput
        let throughput_per_second = if self.total_duration.as_secs() > 0 {
            total_responses as f64 / self.total_duration.as_secs() as f64
        } else {
            0.0
        };

        // Get resource usage (simplified for now)
        let resource_usage = ResourceUsage {
            memory_usage_mb: self.estimate_memory_usage(),
            cpu_usage_percent: self.estimate_cpu_usage(),
            disk_io_mb: self.estimate_disk_io(),
            network_io_mb: self.estimate_network_io(),
        };

        let summary = BroadcastPerformanceSummary {
            broadcast_id: self.broadcast_id.clone(),
            project_id: self.project_id.clone(),
            total_duration_ms: self.total_duration.as_millis() as u64,
            success_rate,
            average_response_time_ms,
            min_response_time_ms,
            max_response_time_ms,
            p95_response_time_ms,
            throughput_per_second,
            error_rate,
            resource_usage,
        };

        // Emit final broadcast metrics
        let _ = emit_metrics_event(
            &self.project_id,
            "broadcast",
            &self.broadcast_id,
            "system",
            "broadcast_completed",
            self.total_duration.as_millis() as u64,
            "completed",
            Some(&format!(
                "Success rate: {:.2}%, Throughput: {:.2} ops/sec, P95: {}ms",
                success_rate * 100.0,
                throughput_per_second,
                p95_response_time_ms
            ))
        );

        summary
    }

    /// Estimate memory usage based on response sizes
    fn estimate_memory_usage(&self) -> f64 {
        let total_response_size: usize = self.agent_metrics.values()
            .filter_map(|m| m.response_size)
            .sum();
        
        // Convert bytes to MB and add overhead
        (total_response_size as f64 / 1_048_576.0) * 1.5
    }

    /// Estimate CPU usage based on duration and concurrency
    fn estimate_cpu_usage(&self) -> f64 {
        // Simplified estimation based on duration and number of agents
        let base_cpu = 10.0; // Base CPU usage
        let per_agent_cpu = 5.0; // Additional CPU per agent
        let duration_factor = self.total_duration.as_secs() as f64 / 10.0; // Longer operations use more CPU
        
        (base_cpu + (self.target_count as f64 * per_agent_cpu) * duration_factor).min(100.0)
    }

    /// Estimate disk I/O based on logging and data persistence
    fn estimate_disk_io(&self) -> f64 {
        // Estimate based on number of log entries and response sizes
        let log_entries = self.agent_metrics.len() * 3; // Start, response, end events
        let log_size_mb = (log_entries * 200) as f64 / 1_048_576.0; // ~200 bytes per log entry
        
        let response_size_mb: f64 = self.agent_metrics.values()
            .filter_map(|m| m.response_size)
            .sum::<usize>() as f64 / 1_048_576.0;
        
        log_size_mb + response_size_mb
    }

    /// Estimate network I/O based on message sizes and responses
    fn estimate_network_io(&self) -> f64 {
        // Estimate based on message sizes and response sizes
        let message_size = 1024; // Average message size in bytes
        let response_size: usize = self.agent_metrics.values()
            .filter_map(|m| m.response_size)
            .sum();
        
        let total_io = (message_size * self.target_count) + response_size;
        total_io as f64 / 1_048_576.0
    }

    /// Get current metrics snapshot
    pub fn get_snapshot(&self) -> BroadcastMetricsSnapshot {
        let current_duration = self.start_time.elapsed();
        let total_responses = self.successful_count + self.failed_count;
        
        BroadcastMetricsSnapshot {
            broadcast_id: self.broadcast_id.clone(),
            project_id: self.project_id.clone(),
            elapsed_ms: current_duration.as_millis() as u64,
            completed_agents: total_responses,
            successful_agents: self.successful_count,
            failed_agents: self.failed_count,
            success_rate: if total_responses > 0 {
                self.successful_count as f64 / total_responses as f64
            } else {
                0.0
            },
            average_response_time_ms: self.calculate_average_response_time(),
        }
    }

    /// Calculate average response time from completed agents
    fn calculate_average_response_time(&self) -> f64 {
        let response_times: Vec<u64> = self.agent_metrics.values()
            .map(|m| m.duration_ms)
            .collect();
        
        if response_times.is_empty() {
            0.0
        } else {
            response_times.iter().sum::<u64>() as f64 / response_times.len() as f64
        }
    }
}

/// Snapshot of current broadcast metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMetricsSnapshot {
    pub broadcast_id: String,
    pub project_id: String,
    pub elapsed_ms: u64,
    pub completed_agents: usize,
    pub successful_agents: usize,
    pub failed_agents: usize,
    pub success_rate: f64,
    pub average_response_time_ms: f64,
}

/// Broadcast metrics aggregator for historical analysis
#[derive(Debug, Clone)]
pub struct BroadcastMetricsAggregator {
    pub project_id: String,
    pub historical_metrics: Vec<BroadcastPerformanceSummary>,
    pub daily_stats: HashMap<String, DailyStats>,
}

/// Daily statistics for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub total_broadcasts: usize,
    pub total_success_rate: f64,
    pub average_response_time_ms: f64,
    pub total_throughput: f64,
    pub error_breakdown: HashMap<String, usize>,
}

impl BroadcastMetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            historical_metrics: Vec::new(),
            daily_stats: HashMap::new(),
        }
    }

    /// Add a completed broadcast to historical data
    pub fn add_broadcast(&mut self, summary: BroadcastPerformanceSummary) {
        self.historical_metrics.push(summary.clone());
        
        // Update daily stats
        let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let daily_stats = self.daily_stats.entry(date.clone()).or_insert_with(|| DailyStats {
            date: date.clone(),
            total_broadcasts: 0,
            total_success_rate: 0.0,
            average_response_time_ms: 0.0,
            total_throughput: 0.0,
            error_breakdown: HashMap::new(),
        });
        
        daily_stats.total_broadcasts += 1;
        daily_stats.total_throughput += summary.throughput_per_second;
        
        // Calculate and update metrics after releasing the borrow
        let success_rate = self.calculate_daily_success_rate(&date);
        let avg_response_time = self.calculate_daily_avg_response_time(&date);
        
        // Update the stats after releasing the mutable borrow
        if let Some(stats) = self.daily_stats.get_mut(&date) {
            stats.total_success_rate = success_rate;
            stats.average_response_time_ms = avg_response_time;
        }
    }

    /// Calculate daily success rate
    fn calculate_daily_success_rate(&self, date: &str) -> f64 {
        let daily_broadcasts: Vec<&BroadcastPerformanceSummary> = self.historical_metrics
            .iter()
            .filter(|b| b.broadcast_id.starts_with(&format!("broadcast_{}", date.replace("-", ""))))
            .collect();
        
        if daily_broadcasts.is_empty() {
            0.0
        } else {
            daily_broadcasts.iter().map(|b| b.success_rate).sum::<f64>() / daily_broadcasts.len() as f64
        }
    }

    /// Calculate daily average response time
    fn calculate_daily_avg_response_time(&self, date: &str) -> f64 {
        let daily_broadcasts: Vec<&BroadcastPerformanceSummary> = self.historical_metrics
            .iter()
            .filter(|b| b.broadcast_id.starts_with(&format!("broadcast_{}", date.replace("-", ""))))
            .collect();
        
        if daily_broadcasts.is_empty() {
            0.0
        } else {
            daily_broadcasts.iter().map(|b| b.average_response_time_ms).sum::<f64>() / daily_broadcasts.len() as f64
        }
    }

    /// Get performance trends over time
    pub fn get_performance_trends(&self, days: usize) -> PerformanceTrends {
        let _cutoff_date = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let recent_metrics: Vec<&BroadcastPerformanceSummary> = self.historical_metrics
            .iter()
            .filter(|b| {
                // Parse broadcast_id to extract timestamp (simplified)
                b.broadcast_id.len() > 20 // Basic validation
            })
            .collect();
        
        PerformanceTrends {
            period_days: days,
            total_broadcasts: recent_metrics.len(),
            average_success_rate: recent_metrics.iter().map(|b| b.success_rate).sum::<f64>() / recent_metrics.len().max(1) as f64,
            average_response_time_ms: recent_metrics.iter().map(|b| b.average_response_time_ms).sum::<f64>() / recent_metrics.len().max(1) as f64,
            average_throughput: recent_metrics.iter().map(|b| b.throughput_per_second).sum::<f64>() / recent_metrics.len().max(1) as f64,
            trend_direction: self.calculate_trend_direction(&recent_metrics),
        }
    }

    /// Calculate trend direction (improving, declining, stable)
    fn calculate_trend_direction(&self, metrics: &[&BroadcastPerformanceSummary]) -> TrendDirection {
        if metrics.len() < 2 {
            return TrendDirection::Stable;
        }
        
        let mid_point = metrics.len() / 2;
        let early_avg = metrics[..mid_point].iter().map(|b| b.success_rate).sum::<f64>() / mid_point as f64;
        let late_avg = metrics[mid_point..].iter().map(|b| b.success_rate).sum::<f64>() / (metrics.len() - mid_point) as f64;
        
        let improvement = late_avg - early_avg;
        
        if improvement > 0.05 {
            TrendDirection::Improving
        } else if improvement < -0.05 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub period_days: usize,
    pub total_broadcasts: usize,
    pub average_success_rate: f64,
    pub average_response_time_ms: f64,
    pub average_throughput: f64,
    pub trend_direction: TrendDirection,
}

/// Trend direction indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_broadcast_metrics_creation() {
        let metrics = BroadcastMetrics::new(
            "test-broadcast-123".to_string(),
            "test-project".to_string(),
            3
        );
        
        assert_eq!(metrics.broadcast_id, "test-broadcast-123");
        assert_eq!(metrics.project_id, "test-project");
        assert_eq!(metrics.target_count, 3);
        assert_eq!(metrics.successful_count, 0);
        assert_eq!(metrics.failed_count, 0);
    }

    #[test]
    fn test_record_success() {
        let mut metrics = BroadcastMetrics::new(
            "test-broadcast-123".to_string(),
            "test-project".to_string(),
            3
        );
        
        metrics.record_success(
            "agent1".to_string(),
            "developer".to_string(),
            "claude".to_string(),
            Duration::from_millis(1500),
            Some(1024)
        );
        
        assert_eq!(metrics.successful_count, 1);
        assert_eq!(metrics.failed_count, 0);
        assert!(metrics.agent_metrics.contains_key("agent1"));
        
        let agent_metrics = &metrics.agent_metrics["agent1"];
        assert!(agent_metrics.success);
        assert_eq!(agent_metrics.duration_ms, 1500);
        assert_eq!(agent_metrics.response_size, Some(1024));
    }

    #[test]
    fn test_record_failure() {
        let mut metrics = BroadcastMetrics::new(
            "test-broadcast-123".to_string(),
            "test-project".to_string(),
            3
        );
        
        metrics.record_failure(
            "agent1".to_string(),
            "developer".to_string(),
            "claude".to_string(),
            Duration::from_millis(500),
            "timeout".to_string(),
            "Request timed out".to_string()
        );
        
        assert_eq!(metrics.successful_count, 0);
        assert_eq!(metrics.failed_count, 1);
        assert!(metrics.agent_metrics.contains_key("agent1"));
        
        let agent_metrics = &metrics.agent_metrics["agent1"];
        assert!(!agent_metrics.success);
        assert_eq!(agent_metrics.duration_ms, 500);
        assert_eq!(agent_metrics.error_type, Some("timeout".to_string()));
    }

    #[test]
    fn test_metrics_finalization() {
        let mut metrics = BroadcastMetrics::new(
            "test-broadcast-123".to_string(),
            "test-project".to_string(),
            2
        );
        
        metrics.record_success(
            "agent1".to_string(),
            "developer".to_string(),
            "claude".to_string(),
            Duration::from_millis(1000),
            Some(512)
        );
        
        metrics.record_success(
            "agent2".to_string(),
            "developer".to_string(),
            "gemini".to_string(),
            Duration::from_millis(2000),
            Some(1024)
        );
        
        let summary = metrics.finalize();
        
        assert_eq!(summary.broadcast_id, "test-broadcast-123");
        assert_eq!(summary.success_rate, 1.0);
        assert_eq!(summary.average_response_time_ms, 1500.0);
        assert_eq!(summary.min_response_time_ms, 1000);
        assert_eq!(summary.max_response_time_ms, 2000);
    }

    #[test]
    fn test_metrics_snapshot() {
        let mut metrics = BroadcastMetrics::new(
            "test-broadcast-123".to_string(),
            "test-project".to_string(),
            2
        );
        
        metrics.record_success(
            "agent1".to_string(),
            "developer".to_string(),
            "claude".to_string(),
            Duration::from_millis(1000),
            Some(512)
        );
        
        let snapshot = metrics.get_snapshot();
        
        assert_eq!(snapshot.broadcast_id, "test-broadcast-123");
        assert_eq!(snapshot.completed_agents, 1);
        assert_eq!(snapshot.successful_agents, 1);
        assert_eq!(snapshot.failed_agents, 0);
        assert_eq!(snapshot.success_rate, 1.0);
    }
}
