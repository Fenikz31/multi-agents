//! Performance monitoring for broadcast operations

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use crate::logging::ndjson::emit_metrics_event;

/// Real-time performance monitor for broadcast operations
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    pub project_id: String,
    pub active_broadcasts: HashMap<String, BroadcastPerformanceTracker>,
    pub performance_thresholds: PerformanceThresholds,
    pub historical_data: Vec<PerformanceSnapshot>,
}

/// Individual broadcast performance tracker
#[derive(Debug, Clone)]
pub struct BroadcastPerformanceTracker {
    pub broadcast_id: String,
    pub start_time: Instant,
    pub target_count: usize,
    pub completed_count: usize,
    pub response_times: Vec<Duration>,
    pub current_throughput: f64,
    pub peak_throughput: f64,
    pub memory_usage: f64,
    pub cpu_usage: f64,
}

/// Performance thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceThresholds {
    pub max_response_time_ms: u64,
    pub min_success_rate: f64,
    pub max_memory_usage_mb: f64,
    pub max_cpu_usage_percent: f64,
    pub min_throughput_per_second: f64,
    pub max_concurrent_broadcasts: usize,
}

/// Performance snapshot for historical analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: String,
    pub project_id: String,
    pub active_broadcasts: usize,
    pub total_throughput: f64,
    pub average_response_time_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub success_rate: f64,
}

/// Performance alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceAlert {
    HighResponseTime {
        broadcast_id: String,
        response_time_ms: u64,
        threshold_ms: u64,
    },
    LowSuccessRate {
        broadcast_id: String,
        success_rate: f64,
        threshold: f64,
    },
    HighMemoryUsage {
        broadcast_id: String,
        memory_usage_mb: f64,
        threshold_mb: f64,
    },
    HighCpuUsage {
        broadcast_id: String,
        cpu_usage_percent: f64,
        threshold_percent: f64,
    },
    LowThroughput {
        broadcast_id: String,
        throughput: f64,
        threshold: f64,
    },
    TooManyConcurrent {
        current_count: usize,
        max_allowed: usize,
    },
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            active_broadcasts: HashMap::new(),
            performance_thresholds: PerformanceThresholds::default(),
            historical_data: Vec::new(),
        }
    }

    /// Start tracking a new broadcast
    pub fn start_broadcast_tracking(&mut self, broadcast_id: String, target_count: usize) {
        let tracker = BroadcastPerformanceTracker {
            broadcast_id: broadcast_id.clone(),
            start_time: Instant::now(),
            target_count,
            completed_count: 0,
            response_times: Vec::new(),
            current_throughput: 0.0,
            peak_throughput: 0.0,
            memory_usage: 0.0,
            cpu_usage: 0.0,
        };
        
        self.active_broadcasts.insert(broadcast_id.clone(), tracker);
        
        // Emit start tracking event
        let _ = emit_metrics_event(
            &self.project_id,
            "broadcast",
            &broadcast_id,
            "system",
            "performance_tracking_started",
            0,
            "started",
            Some(&format!("Target count: {}", target_count))
        );
    }

    /// Record a response for a broadcast
    pub fn record_response(&mut self, broadcast_id: &str, response_time: Duration) -> Vec<PerformanceAlert> {
        let mut alerts = Vec::new();
        
        if let Some(tracker) = self.active_broadcasts.get_mut(broadcast_id) {
            tracker.completed_count += 1;
            tracker.response_times.push(response_time);
            
            // Update throughput
            let elapsed = tracker.start_time.elapsed();
            if elapsed.as_secs() > 0 {
                tracker.current_throughput = tracker.completed_count as f64 / elapsed.as_secs() as f64;
                if tracker.current_throughput > tracker.peak_throughput {
                    tracker.peak_throughput = tracker.current_throughput;
                }
            }
            
            // Check for performance alerts
            if response_time.as_millis() as u64 > self.performance_thresholds.max_response_time_ms {
                alerts.push(PerformanceAlert::HighResponseTime {
                    broadcast_id: broadcast_id.to_string(),
                    response_time_ms: response_time.as_millis() as u64,
                    threshold_ms: self.performance_thresholds.max_response_time_ms,
                });
            }
            
            // Calculate resource usage estimates (simplified)
            let memory_usage = tracker.response_times.len() as f64 * 0.1; // Simple estimation
            let cpu_usage = (tracker.completed_count as f64 / tracker.target_count as f64) * 50.0; // Simple estimation
            
            // Update resource usage
            tracker.memory_usage = memory_usage;
            tracker.cpu_usage = cpu_usage;
            
            // Check resource usage alerts
            if tracker.memory_usage > self.performance_thresholds.max_memory_usage_mb {
                alerts.push(PerformanceAlert::HighMemoryUsage {
                    broadcast_id: broadcast_id.to_string(),
                    memory_usage_mb: tracker.memory_usage,
                    threshold_mb: self.performance_thresholds.max_memory_usage_mb,
                });
            }
            
            if tracker.cpu_usage > self.performance_thresholds.max_cpu_usage_percent {
                alerts.push(PerformanceAlert::HighCpuUsage {
                    broadcast_id: broadcast_id.to_string(),
                    cpu_usage_percent: tracker.cpu_usage,
                    threshold_percent: self.performance_thresholds.max_cpu_usage_percent,
                });
            }
            
            if tracker.current_throughput < self.performance_thresholds.min_throughput_per_second {
                alerts.push(PerformanceAlert::LowThroughput {
                    broadcast_id: broadcast_id.to_string(),
                    throughput: tracker.current_throughput,
                    threshold: self.performance_thresholds.min_throughput_per_second,
                });
            }
        }
        
        alerts
    }

    /// Complete a broadcast and return final performance data
    pub fn complete_broadcast(&mut self, broadcast_id: &str, success_count: usize, failed_count: usize) -> Option<PerformanceSnapshot> {
        if let Some(tracker) = self.active_broadcasts.remove(broadcast_id) {
            let total_responses = success_count + failed_count;
            let success_rate = if total_responses > 0 {
                success_count as f64 / total_responses as f64
            } else {
                0.0
            };
            
            let average_response_time_ms = if !tracker.response_times.is_empty() {
                tracker.response_times.iter().sum::<Duration>().as_millis() as f64 / tracker.response_times.len() as f64
            } else {
                0.0
            };
            
            let snapshot = PerformanceSnapshot {
                timestamp: chrono::Utc::now().to_rfc3339(),
                project_id: self.project_id.clone(),
                active_broadcasts: self.active_broadcasts.len(),
                total_throughput: tracker.peak_throughput,
                average_response_time_ms,
                memory_usage_mb: tracker.memory_usage,
                cpu_usage_percent: tracker.cpu_usage,
                success_rate,
            };
            
            self.historical_data.push(snapshot.clone());
            
            // Emit completion metrics
            let _ = emit_metrics_event(
                &self.project_id,
                "broadcast",
                broadcast_id,
                "system",
                "performance_tracking_completed",
                tracker.start_time.elapsed().as_millis() as u64,
                "completed",
                Some(&format!(
                    "Success rate: {:.2}%, Avg response: {:.2}ms, Peak throughput: {:.2} ops/sec",
                    success_rate * 100.0,
                    average_response_time_ms,
                    tracker.peak_throughput
                ))
            );
            
            Some(snapshot)
        } else {
            None
        }
    }

    /// Get current performance status
    pub fn get_current_status(&self) -> PerformanceStatus {
        let total_throughput: f64 = self.active_broadcasts.values()
            .map(|t| t.current_throughput)
            .sum();
        
        let average_response_time_ms = self.calculate_average_response_time();
        let total_memory_usage: f64 = self.active_broadcasts.values()
            .map(|t| t.memory_usage)
            .sum();
        let total_cpu_usage: f64 = self.active_broadcasts.values()
            .map(|t| t.cpu_usage)
            .sum();
        
        PerformanceStatus {
            active_broadcasts: self.active_broadcasts.len(),
            total_throughput,
            average_response_time_ms,
            total_memory_usage_mb: total_memory_usage,
            total_cpu_usage_percent: total_cpu_usage,
            performance_health: self.calculate_performance_health(),
        }
    }

    /// Calculate performance health score (0-100)
    fn calculate_performance_health(&self) -> f64 {
        let mut health_score: f64 = 100.0;
        
        // Check concurrent broadcasts
        if self.active_broadcasts.len() > self.performance_thresholds.max_concurrent_broadcasts {
            health_score -= 20.0;
        }
        
        // Check average response time
        let avg_response_time = self.calculate_average_response_time();
        if avg_response_time > self.performance_thresholds.max_response_time_ms as f64 {
            health_score -= 30.0;
        }
        
        // Check memory usage
        let total_memory: f64 = self.active_broadcasts.values()
            .map(|t| t.memory_usage)
            .sum();
        if total_memory > self.performance_thresholds.max_memory_usage_mb {
            health_score -= 25.0;
        }
        
        // Check CPU usage
        let total_cpu: f64 = self.active_broadcasts.values()
            .map(|t| t.cpu_usage)
            .sum();
        if total_cpu > self.performance_thresholds.max_cpu_usage_percent {
            health_score -= 25.0;
        }
        
        health_score.max(0.0)
    }

    /// Calculate average response time across all active broadcasts
    fn calculate_average_response_time(&self) -> f64 {
        let all_response_times: Vec<Duration> = self.active_broadcasts.values()
            .flat_map(|t| &t.response_times)
            .cloned()
            .collect();
        
        if all_response_times.is_empty() {
            0.0
        } else {
            all_response_times.iter().sum::<Duration>().as_millis() as f64 / all_response_times.len() as f64
        }
    }


    /// Get performance trends over time
    pub fn get_performance_trends(&self, hours: usize) -> PerformanceTrends {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::hours(hours as i64);
        let recent_snapshots: Vec<&PerformanceSnapshot> = self.historical_data
            .iter()
            .filter(|s| {
                if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&s.timestamp) {
                    timestamp.timestamp() > cutoff_time.timestamp()
                } else {
                    false
                }
            })
            .collect();
        
        if recent_snapshots.is_empty() {
            return PerformanceTrends {
                period_hours: hours,
                total_snapshots: 0,
                average_throughput: 0.0,
                average_response_time_ms: 0.0,
                average_memory_usage_mb: 0.0,
                average_cpu_usage_percent: 0.0,
                average_success_rate: 0.0,
                trend_direction: TrendDirection::Stable,
            };
        }
        
        let average_throughput = recent_snapshots.iter().map(|s| s.total_throughput).sum::<f64>() / recent_snapshots.len() as f64;
        let average_response_time_ms = recent_snapshots.iter().map(|s| s.average_response_time_ms).sum::<f64>() / recent_snapshots.len() as f64;
        let average_memory_usage_mb = recent_snapshots.iter().map(|s| s.memory_usage_mb).sum::<f64>() / recent_snapshots.len() as f64;
        let average_cpu_usage_percent = recent_snapshots.iter().map(|s| s.cpu_usage_percent).sum::<f64>() / recent_snapshots.len() as f64;
        let average_success_rate = recent_snapshots.iter().map(|s| s.success_rate).sum::<f64>() / recent_snapshots.len() as f64;
        
        let trend_direction = self.calculate_trend_direction(&recent_snapshots);
        
        PerformanceTrends {
            period_hours: hours,
            total_snapshots: recent_snapshots.len(),
            average_throughput,
            average_response_time_ms,
            average_memory_usage_mb,
            average_cpu_usage_percent,
            average_success_rate,
            trend_direction,
        }
    }

    /// Calculate trend direction based on recent performance
    fn calculate_trend_direction(&self, snapshots: &[&PerformanceSnapshot]) -> TrendDirection {
        if snapshots.len() < 2 {
            return TrendDirection::Stable;
        }
        
        let mid_point = snapshots.len() / 2;
        let early_avg_response = snapshots[..mid_point].iter().map(|s| s.average_response_time_ms).sum::<f64>() / mid_point as f64;
        let late_avg_response = snapshots[mid_point..].iter().map(|s| s.average_response_time_ms).sum::<f64>() / (snapshots.len() - mid_point) as f64;
        
        let early_avg_success = snapshots[..mid_point].iter().map(|s| s.success_rate).sum::<f64>() / mid_point as f64;
        let late_avg_success = snapshots[mid_point..].iter().map(|s| s.success_rate).sum::<f64>() / (snapshots.len() - mid_point) as f64;
        
        let response_improvement = early_avg_response - late_avg_response; // Lower is better
        let success_improvement = late_avg_success - early_avg_success; // Higher is better
        
        let overall_improvement = response_improvement + success_improvement;
        
        if overall_improvement > 0.1 {
            TrendDirection::Improving
        } else if overall_improvement < -0.1 {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        }
    }

    /// Update performance thresholds
    pub fn update_thresholds(&mut self, thresholds: PerformanceThresholds) {
        self.performance_thresholds = thresholds;
    }

    /// Clear historical data older than specified days
    pub fn cleanup_historical_data(&mut self, keep_days: usize) {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::days(keep_days as i64);
        
        self.historical_data.retain(|snapshot| {
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&snapshot.timestamp) {
                timestamp.timestamp() > cutoff_time.timestamp()
            } else {
                false
            }
        });
    }
}

/// Current performance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStatus {
    pub active_broadcasts: usize,
    pub total_throughput: f64,
    pub average_response_time_ms: f64,
    pub total_memory_usage_mb: f64,
    pub total_cpu_usage_percent: f64,
    pub performance_health: f64,
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    pub period_hours: usize,
    pub total_snapshots: usize,
    pub average_throughput: f64,
    pub average_response_time_ms: f64,
    pub average_memory_usage_mb: f64,
    pub average_cpu_usage_percent: f64,
    pub average_success_rate: f64,
    pub trend_direction: TrendDirection,
}

/// Trend direction indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_response_time_ms: 5000, // 5 seconds
            min_success_rate: 0.95, // 95%
            max_memory_usage_mb: 100.0, // 100 MB
            max_cpu_usage_percent: 80.0, // 80%
            min_throughput_per_second: 1.0, // 1 operation per second
            max_concurrent_broadcasts: 10, // 10 concurrent broadcasts
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new("test-project".to_string());
        assert_eq!(monitor.project_id, "test-project");
        assert!(monitor.active_broadcasts.is_empty());
        assert!(monitor.historical_data.is_empty());
    }

    #[test]
    fn test_start_broadcast_tracking() {
        let mut monitor = PerformanceMonitor::new("test-project".to_string());
        monitor.start_broadcast_tracking("broadcast-123".to_string(), 3);
        
        assert!(monitor.active_broadcasts.contains_key("broadcast-123"));
        let tracker = &monitor.active_broadcasts["broadcast-123"];
        assert_eq!(tracker.target_count, 3);
        assert_eq!(tracker.completed_count, 0);
    }

    #[test]
    fn test_record_response() {
        let mut monitor = PerformanceMonitor::new("test-project".to_string());
        monitor.start_broadcast_tracking("broadcast-123".to_string(), 3);
        
        let alerts = monitor.record_response("broadcast-123", Duration::from_millis(1000));
        
        let tracker = &monitor.active_broadcasts["broadcast-123"];
        assert_eq!(tracker.completed_count, 1);
        assert_eq!(tracker.response_times.len(), 1);
        assert_eq!(tracker.response_times[0], Duration::from_millis(1000));
        assert!(alerts.is_empty()); // Should not trigger alerts for normal response time
    }

    #[test]
    fn test_high_response_time_alert() {
        let mut monitor = PerformanceMonitor::new("test-project".to_string());
        monitor.start_broadcast_tracking("broadcast-123".to_string(), 3);
        
        // Set low threshold to trigger alert
        monitor.performance_thresholds.max_response_time_ms = 500;
        
        let alerts = monitor.record_response("broadcast-123", Duration::from_millis(1000));
        
        assert!(!alerts.is_empty());
        match &alerts[0] {
            PerformanceAlert::HighResponseTime { broadcast_id, response_time_ms, threshold_ms } => {
                assert_eq!(broadcast_id, "broadcast-123");
                assert_eq!(*response_time_ms, 1000);
                assert_eq!(*threshold_ms, 500);
            }
            _ => panic!("Expected HighResponseTime alert"),
        }
    }

    #[test]
    fn test_complete_broadcast() {
        let mut monitor = PerformanceMonitor::new("test-project".to_string());
        monitor.start_broadcast_tracking("broadcast-123".to_string(), 3);
        
        monitor.record_response("broadcast-123", Duration::from_millis(1000));
        monitor.record_response("broadcast-123", Duration::from_millis(2000));
        
        let snapshot = monitor.complete_broadcast("broadcast-123", 2, 0);
        
        assert!(snapshot.is_some());
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.project_id, "test-project");
        assert_eq!(snapshot.success_rate, 1.0);
        assert_eq!(snapshot.average_response_time_ms, 1500.0);
        assert!(monitor.active_broadcasts.is_empty());
        assert_eq!(monitor.historical_data.len(), 1);
    }

    #[test]
    fn test_performance_status() {
        let mut monitor = PerformanceMonitor::new("test-project".to_string());
        monitor.start_broadcast_tracking("broadcast-123".to_string(), 3);
        monitor.record_response("broadcast-123", Duration::from_millis(1000));
        
        let status = monitor.get_current_status();
        
        assert_eq!(status.active_broadcasts, 1);
        assert!(status.total_throughput > 0.0);
        assert_eq!(status.average_response_time_ms, 1000.0);
        assert!(status.performance_health > 0.0);
    }
}
