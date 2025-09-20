//! Resource monitoring for broadcast operations

use serde::{Deserialize, Serialize};
use crate::logging::ndjson::emit_metrics_event;

/// Resource monitor for tracking system resources during broadcast operations
#[derive(Debug, Clone)]
pub struct ResourceMonitor {
    pub project_id: String,
    pub resource_metrics: ResourceMetrics,
    pub resource_history: Vec<ResourceSnapshot>,
    pub resource_limits: ResourceLimits,
    pub monitoring_enabled: bool,
}

/// Current resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub memory: MemoryMetrics,
    pub cpu: CpuMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub timestamp: String,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub used_mb: f64,
    pub available_mb: f64,
    pub total_mb: f64,
    pub usage_percentage: f64,
    pub peak_usage_mb: f64,
    pub trend: ResourceTrend,
}

/// CPU usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percentage: f64,
    pub load_average: [f64; 3], // 1min, 5min, 15min
    pub cores: usize,
    pub peak_usage_percentage: f64,
    pub trend: ResourceTrend,
}

/// Disk usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub used_mb: f64,
    pub available_mb: f64,
    pub total_mb: f64,
    pub usage_percentage: f64,
    pub read_mb_per_sec: f64,
    pub write_mb_per_sec: f64,
    pub trend: ResourceTrend,
}

/// Network usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bandwidth_mbps: f64,
    pub trend: ResourceTrend,
}

/// Resource trend indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceTrend {
    Increasing,
    Decreasing,
    Stable,
    Unknown,
}

/// Resource limits for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: f64,
    pub max_cpu_percentage: f64,
    pub max_disk_usage_percentage: f64,
    pub max_network_bandwidth_mbps: f64,
    pub warning_threshold: f64, // 0.0-1.0
    pub critical_threshold: f64, // 0.0-1.0
}

/// Historical resource snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub timestamp: String,
    pub memory_usage_mb: f64,
    pub cpu_usage_percentage: f64,
    pub disk_usage_percentage: f64,
    pub network_bandwidth_mbps: f64,
    pub active_broadcasts: usize,
}

/// Resource usage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageReport {
    pub project_id: String,
    pub generated_at: String,
    pub current_metrics: ResourceMetrics,
    pub resource_efficiency: f64, // 0-100
    pub recommendations: Vec<String>,
    pub alerts: Vec<ResourceAlert>,
    pub trends: ResourceTrends,
}

/// Resource trends analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTrends {
    pub memory_trend: TrendAnalysis,
    pub cpu_trend: TrendAnalysis,
    pub disk_trend: TrendAnalysis,
    pub network_trend: TrendAnalysis,
    pub overall_efficiency_trend: TrendAnalysis,
}

/// Trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub direction: ResourceTrend,
    pub change_percentage: f64,
    pub confidence: f64, // 0-100
    pub prediction: Option<f64>,
}

/// Resource alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAlert {
    pub resource_type: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub current_value: f64,
    pub threshold: f64,
    pub timestamp: String,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            resource_metrics: ResourceMetrics::default(),
            resource_history: Vec::new(),
            resource_limits: ResourceLimits::default(),
            monitoring_enabled: true,
        }
    }

    /// Update resource metrics
    pub fn update_metrics(&mut self) -> Result<(), String> {
        if !self.monitoring_enabled {
            return Ok(());
        }

        let memory = self.collect_memory_metrics()?;
        let cpu = self.collect_cpu_metrics()?;
        let disk = self.collect_disk_metrics()?;
        let network = self.collect_network_metrics()?;

        self.resource_metrics = ResourceMetrics {
            memory,
            cpu,
            disk,
            network,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Add to history
        self.add_to_history();

        // Emit metrics
        self.emit_resource_metrics();

        Ok(())
    }

    /// Collect memory metrics
    fn collect_memory_metrics(&self) -> Result<MemoryMetrics, String> {
        // Simplified memory collection - in a real implementation,
        // this would use system APIs to get actual memory usage
        let total_mb = 8192.0; // 8GB total
        let used_mb = self.estimate_memory_usage();
        let available_mb = total_mb - used_mb;
        let usage_percentage = (used_mb / total_mb) * 100.0;

        Ok(MemoryMetrics {
            used_mb,
            available_mb,
            total_mb,
            usage_percentage,
            peak_usage_mb: used_mb, // Would track peak over time
            trend: self.calculate_memory_trend(),
        })
    }

    /// Collect CPU metrics
    fn collect_cpu_metrics(&self) -> Result<CpuMetrics, String> {
        // Simplified CPU collection
        let usage_percentage = self.estimate_cpu_usage();
        let load_average = [0.5, 0.6, 0.7]; // Would get from system
        let cores = num_cpus::get();

        Ok(CpuMetrics {
            usage_percentage,
            load_average,
            cores,
            peak_usage_percentage: usage_percentage, // Would track peak
            trend: self.calculate_cpu_trend(),
        })
    }

    /// Collect disk metrics
    fn collect_disk_metrics(&self) -> Result<DiskMetrics, String> {
        // Simplified disk collection
        let total_mb = 100000.0; // 100GB total
        let used_mb = self.estimate_disk_usage();
        let available_mb = total_mb - used_mb;
        let usage_percentage = (used_mb / total_mb) * 100.0;

        Ok(DiskMetrics {
            used_mb,
            available_mb,
            total_mb,
            usage_percentage,
            read_mb_per_sec: 0.0, // Would track I/O
            write_mb_per_sec: 0.0,
            trend: self.calculate_disk_trend(),
        })
    }

    /// Collect network metrics
    fn collect_network_metrics(&self) -> Result<NetworkMetrics, String> {
        // Simplified network collection
        Ok(NetworkMetrics {
            bytes_sent: 0, // Would track actual network usage
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            bandwidth_mbps: 0.0,
            trend: self.calculate_network_trend(),
        })
    }

    /// Estimate memory usage based on active operations
    fn estimate_memory_usage(&self) -> f64 {
        let base_memory = 50.0; // Base memory in MB
        let per_broadcast_memory = 10.0; // Additional memory per broadcast
        let active_broadcasts = self.get_active_broadcast_count();
        
        base_memory + (active_broadcasts as f64 * per_broadcast_memory)
    }

    /// Estimate CPU usage based on active operations
    fn estimate_cpu_usage(&self) -> f64 {
        let base_cpu = 5.0; // Base CPU usage in percent
        let per_broadcast_cpu = 2.0; // Additional CPU per broadcast
        let active_broadcasts = self.get_active_broadcast_count();
        
        (base_cpu + (active_broadcasts as f64 * per_broadcast_cpu)).min(100.0)
    }

    /// Estimate disk usage
    fn estimate_disk_usage(&self) -> f64 {
        let base_disk = 1000.0; // Base disk usage in MB
        let log_size = self.estimate_log_size();
        let data_size = self.estimate_data_size();
        
        base_disk + log_size + data_size
    }

    /// Estimate log file size
    fn estimate_log_size(&self) -> f64 {
        // Estimate based on number of log entries
        let entries_per_broadcast = 10; // Approximate log entries per broadcast
        let bytes_per_entry = 200; // Average bytes per log entry
        let active_broadcasts = self.get_active_broadcast_count();
        
        (active_broadcasts * entries_per_broadcast * bytes_per_entry) as f64 / 1_048_576.0 // Convert to MB
    }

    /// Estimate data file size
    fn estimate_data_size(&self) -> f64 {
        // Estimate SQLite database size
        let base_db_size = 1.0; // Base database size in MB
        let per_broadcast_data = 0.1; // Additional data per broadcast
        let active_broadcasts = self.get_active_broadcast_count();
        
        base_db_size + (active_broadcasts as f64 * per_broadcast_data)
    }

    /// Get active broadcast count (simplified)
    fn get_active_broadcast_count(&self) -> usize {
        // This would typically come from the broadcast manager
        // For now, return a simulated value
        self.resource_history.len().min(10)
    }

    /// Calculate memory trend
    fn calculate_memory_trend(&self) -> ResourceTrend {
        if self.resource_history.len() < 2 {
            return ResourceTrend::Unknown;
        }
        
        let recent = &self.resource_history[self.resource_history.len() - 1];
        let previous = &self.resource_history[self.resource_history.len() - 2];
        
        let change = recent.memory_usage_mb - previous.memory_usage_mb;
        let change_percentage = (change / previous.memory_usage_mb) * 100.0;
        
        if change_percentage > 5.0 {
            ResourceTrend::Increasing
        } else if change_percentage < -5.0 {
            ResourceTrend::Decreasing
        } else {
            ResourceTrend::Stable
        }
    }

    /// Calculate CPU trend
    fn calculate_cpu_trend(&self) -> ResourceTrend {
        if self.resource_history.len() < 2 {
            return ResourceTrend::Unknown;
        }
        
        let recent = &self.resource_history[self.resource_history.len() - 1];
        let previous = &self.resource_history[self.resource_history.len() - 2];
        
        let change = recent.cpu_usage_percentage - previous.cpu_usage_percentage;
        
        if change > 5.0 {
            ResourceTrend::Increasing
        } else if change < -5.0 {
            ResourceTrend::Decreasing
        } else {
            ResourceTrend::Stable
        }
    }

    /// Calculate disk trend
    fn calculate_disk_trend(&self) -> ResourceTrend {
        if self.resource_history.len() < 2 {
            return ResourceTrend::Unknown;
        }
        
        let recent = &self.resource_history[self.resource_history.len() - 1];
        let previous = &self.resource_history[self.resource_history.len() - 2];
        
        let change = recent.disk_usage_percentage - previous.disk_usage_percentage;
        
        if change > 1.0 {
            ResourceTrend::Increasing
        } else if change < -1.0 {
            ResourceTrend::Decreasing
        } else {
            ResourceTrend::Stable
        }
    }

    /// Calculate network trend
    fn calculate_network_trend(&self) -> ResourceTrend {
        // Simplified - would analyze network usage over time
        ResourceTrend::Stable
    }

    /// Add current metrics to history
    fn add_to_history(&mut self) {
        let snapshot = ResourceSnapshot {
            timestamp: self.resource_metrics.timestamp.clone(),
            memory_usage_mb: self.resource_metrics.memory.used_mb,
            cpu_usage_percentage: self.resource_metrics.cpu.usage_percentage,
            disk_usage_percentage: self.resource_metrics.disk.usage_percentage,
            network_bandwidth_mbps: self.resource_metrics.network.bandwidth_mbps,
            active_broadcasts: self.get_active_broadcast_count(),
        };
        
        self.resource_history.push(snapshot);
        
        // Keep only recent history (last 1000 entries)
        if self.resource_history.len() > 1000 {
            self.resource_history.remove(0);
        }
    }

    /// Emit resource metrics to logging system
    fn emit_resource_metrics(&self) {
        let _ = emit_metrics_event(
            &self.project_id,
            "resource",
            "system",
            "system",
            "resource_usage",
            0,
            "collected",
            Some(&format!(
                "Memory: {:.1}MB ({:.1}%), CPU: {:.1}%, Disk: {:.1}%",
                self.resource_metrics.memory.used_mb,
                self.resource_metrics.memory.usage_percentage,
                self.resource_metrics.cpu.usage_percentage,
                self.resource_metrics.disk.usage_percentage
            ))
        );
    }

    /// Generate resource usage report
    pub fn generate_report(&self) -> ResourceUsageReport {
        let recommendations = self.generate_recommendations();
        let alerts = self.check_resource_alerts();
        let trends = self.analyze_trends();

        ResourceUsageReport {
            project_id: self.project_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            current_metrics: self.resource_metrics.clone(),
            resource_efficiency: self.calculate_efficiency(),
            recommendations,
            alerts,
            trends,
        }
    }

    /// Generate resource recommendations
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Memory recommendations
        if self.resource_metrics.memory.usage_percentage > 80.0 {
            recommendations.push("High memory usage detected. Consider optimizing memory usage or scaling resources.".to_string());
        }
        
        // CPU recommendations
        if self.resource_metrics.cpu.usage_percentage > 80.0 {
            recommendations.push("High CPU usage detected. Consider load balancing or scaling resources.".to_string());
        }
        
        // Disk recommendations
        if self.resource_metrics.disk.usage_percentage > 90.0 {
            recommendations.push("Disk usage is very high. Consider cleaning up logs or expanding storage.".to_string());
        }
        
        // Trend-based recommendations
        match self.resource_metrics.memory.trend {
            ResourceTrend::Increasing => {
                recommendations.push("Memory usage is increasing. Monitor for potential memory leaks.".to_string());
            }
            ResourceTrend::Decreasing => {
                recommendations.push("Memory usage is decreasing. System is optimizing well.".to_string());
            }
            _ => {}
        }
        
        if recommendations.is_empty() {
            recommendations.push("Resource usage is within normal limits. Continue monitoring.".to_string());
        }
        
        recommendations
    }

    /// Check for resource alerts
    fn check_resource_alerts(&self) -> Vec<ResourceAlert> {
        let mut alerts = Vec::new();
        let now = chrono::Utc::now().to_rfc3339();
        
        // Memory alerts
        if self.resource_metrics.memory.usage_percentage > self.resource_limits.max_memory_mb {
            alerts.push(ResourceAlert {
                resource_type: "Memory".to_string(),
                severity: AlertSeverity::Critical,
                message: "Memory usage exceeds critical threshold".to_string(),
                current_value: self.resource_metrics.memory.usage_percentage,
                threshold: self.resource_limits.max_memory_mb,
                timestamp: now.clone(),
            });
        } else if self.resource_metrics.memory.usage_percentage > self.resource_limits.max_memory_mb * self.resource_limits.warning_threshold {
            alerts.push(ResourceAlert {
                resource_type: "Memory".to_string(),
                severity: AlertSeverity::Warning,
                message: "Memory usage approaching threshold".to_string(),
                current_value: self.resource_metrics.memory.usage_percentage,
                threshold: self.resource_limits.max_memory_mb * self.resource_limits.warning_threshold,
                timestamp: now.clone(),
            });
        }
        
        // CPU alerts
        if self.resource_metrics.cpu.usage_percentage > self.resource_limits.max_cpu_percentage {
            alerts.push(ResourceAlert {
                resource_type: "CPU".to_string(),
                severity: AlertSeverity::Critical,
                message: "CPU usage exceeds critical threshold".to_string(),
                current_value: self.resource_metrics.cpu.usage_percentage,
                threshold: self.resource_limits.max_cpu_percentage,
                timestamp: now.clone(),
            });
        }
        
        // Disk alerts
        if self.resource_metrics.disk.usage_percentage > self.resource_limits.max_disk_usage_percentage {
            alerts.push(ResourceAlert {
                resource_type: "Disk".to_string(),
                severity: AlertSeverity::Critical,
                message: "Disk usage exceeds critical threshold".to_string(),
                current_value: self.resource_metrics.disk.usage_percentage,
                threshold: self.resource_limits.max_disk_usage_percentage,
                timestamp: now.clone(),
            });
        }
        
        alerts
    }

    /// Analyze resource trends
    fn analyze_trends(&self) -> ResourceTrends {
        ResourceTrends {
            memory_trend: self.analyze_trend("memory", |s| s.memory_usage_mb),
            cpu_trend: self.analyze_trend("cpu", |s| s.cpu_usage_percentage),
            disk_trend: self.analyze_trend("disk", |s| s.disk_usage_percentage),
            network_trend: self.analyze_trend("network", |s| s.network_bandwidth_mbps),
            overall_efficiency_trend: self.analyze_efficiency_trend(),
        }
    }

    /// Analyze trend for a specific resource
    fn analyze_trend<F>(&self, _resource_name: &str, get_value: F) -> TrendAnalysis
    where
        F: Fn(&ResourceSnapshot) -> f64,
    {
        if self.resource_history.len() < 3 {
            return TrendAnalysis {
                direction: ResourceTrend::Unknown,
                change_percentage: 0.0,
                confidence: 0.0,
                prediction: None,
            };
        }
        
        let recent = &self.resource_history[self.resource_history.len() - 1];
        let previous = &self.resource_history[self.resource_history.len() - 2];
        
        let current_value = get_value(recent);
        let previous_value = get_value(previous);
        
        let change_percentage = if previous_value > 0.0 {
            ((current_value - previous_value) / previous_value) * 100.0
        } else {
            0.0
        };
        
        let direction = if change_percentage > 5.0 {
            ResourceTrend::Increasing
        } else if change_percentage < -5.0 {
            ResourceTrend::Decreasing
        } else {
            ResourceTrend::Stable
        };
        
        let confidence = (self.resource_history.len() as f64 / 10.0).min(100.0);
        
        TrendAnalysis {
            direction,
            change_percentage,
            confidence,
            prediction: Some(current_value + (change_percentage / 100.0) * current_value),
        }
    }

    /// Analyze overall efficiency trend
    fn analyze_efficiency_trend(&self) -> TrendAnalysis {
        // Calculate efficiency based on resource usage vs. active broadcasts
        let mut efficiency_scores = Vec::new();
        
        for snapshot in &self.resource_history {
            let memory_efficiency = if snapshot.active_broadcasts > 0 {
                100.0 - (snapshot.memory_usage_mb / snapshot.active_broadcasts as f64)
            } else {
                100.0
            };
            
            let cpu_efficiency = if snapshot.active_broadcasts > 0 {
                100.0 - (snapshot.cpu_usage_percentage / snapshot.active_broadcasts as f64)
            } else {
                100.0
            };
            
            let efficiency = (memory_efficiency + cpu_efficiency) / 2.0;
            efficiency_scores.push(efficiency);
        }
        
        if efficiency_scores.len() < 2 {
            return TrendAnalysis {
                direction: ResourceTrend::Unknown,
                change_percentage: 0.0,
                confidence: 0.0,
                prediction: None,
            };
        }
        
        let recent = efficiency_scores[efficiency_scores.len() - 1];
        let previous = efficiency_scores[efficiency_scores.len() - 2];
        
        let change_percentage = ((recent - previous) / previous) * 100.0;
        
        let direction = if change_percentage > 5.0 {
            ResourceTrend::Increasing
        } else if change_percentage < -5.0 {
            ResourceTrend::Decreasing
        } else {
            ResourceTrend::Stable
        };
        
        TrendAnalysis {
            direction,
            change_percentage,
            confidence: 80.0,
            prediction: Some(recent + (change_percentage / 100.0) * recent),
        }
    }

    /// Calculate overall resource efficiency
    fn calculate_efficiency(&self) -> f64 {
        let memory_efficiency = 100.0 - self.resource_metrics.memory.usage_percentage;
        let cpu_efficiency = 100.0 - self.resource_metrics.cpu.usage_percentage;
        let disk_efficiency = 100.0 - self.resource_metrics.disk.usage_percentage;
        
        (memory_efficiency + cpu_efficiency + disk_efficiency) / 3.0
    }

    /// Update resource limits
    pub fn update_limits(&mut self, limits: ResourceLimits) {
        self.resource_limits = limits;
    }

    /// Enable or disable monitoring
    pub fn set_monitoring_enabled(&mut self, enabled: bool) {
        self.monitoring_enabled = enabled;
    }

    /// Get resource history
    pub fn get_history(&self, limit: Option<usize>) -> Vec<&ResourceSnapshot> {
        let mut history: Vec<&ResourceSnapshot> = self.resource_history.iter().collect();
        
        if let Some(limit) = limit {
            history.truncate(limit);
        }
        
        history
    }
}

impl Default for ResourceMetrics {
    fn default() -> Self {
        Self {
            memory: MemoryMetrics {
                used_mb: 0.0,
                available_mb: 0.0,
                total_mb: 0.0,
                usage_percentage: 0.0,
                peak_usage_mb: 0.0,
                trend: ResourceTrend::Unknown,
            },
            cpu: CpuMetrics {
                usage_percentage: 0.0,
                load_average: [0.0, 0.0, 0.0],
                cores: 1,
                peak_usage_percentage: 0.0,
                trend: ResourceTrend::Unknown,
            },
            disk: DiskMetrics {
                used_mb: 0.0,
                available_mb: 0.0,
                total_mb: 0.0,
                usage_percentage: 0.0,
                read_mb_per_sec: 0.0,
                write_mb_per_sec: 0.0,
                trend: ResourceTrend::Unknown,
            },
            network: NetworkMetrics {
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
                bandwidth_mbps: 0.0,
                trend: ResourceTrend::Unknown,
            },
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 100.0, // 100MB
            max_cpu_percentage: 80.0, // 80%
            max_disk_usage_percentage: 90.0, // 90%
            max_network_bandwidth_mbps: 100.0, // 100 Mbps
            warning_threshold: 0.8, // 80%
            critical_threshold: 0.9, // 90%
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_monitor_creation() {
        let monitor = ResourceMonitor::new("test-project".to_string());
        assert_eq!(monitor.project_id, "test-project");
        assert!(monitor.monitoring_enabled);
    }

    #[test]
    fn test_update_metrics() {
        let mut monitor = ResourceMonitor::new("test-project".to_string());
        let result = monitor.update_metrics();
        
        assert!(result.is_ok());
        assert!(!monitor.resource_history.is_empty());
    }

    #[test]
    fn test_resource_report_generation() {
        let mut monitor = ResourceMonitor::new("test-project".to_string());
        monitor.update_metrics().unwrap();
        
        let report = monitor.generate_report();
        
        assert_eq!(report.project_id, "test-project");
        assert!(report.resource_efficiency >= 0.0);
        assert!(report.resource_efficiency <= 100.0);
    }

    #[test]
    fn test_resource_alerts() {
        let mut monitor = ResourceMonitor::new("test-project".to_string());
        
        // Set low limits to trigger alerts
        monitor.resource_limits.max_memory_mb = 10.0;
        monitor.resource_limits.max_cpu_percentage = 10.0;
        
        monitor.update_metrics().unwrap();
        
        let report = monitor.generate_report();
        
        // Should have alerts due to low limits
        assert!(!report.alerts.is_empty());
    }

    #[test]
    fn test_trend_analysis() {
        let mut monitor = ResourceMonitor::new("test-project".to_string());
        
        // Add some history for trend analysis
        for i in 0..5 {
            monitor.resource_history.push(ResourceSnapshot {
                timestamp: chrono::Utc::now().to_rfc3339(),
                memory_usage_mb: 50.0 + (i as f64 * 10.0),
                cpu_usage_percentage: 20.0 + (i as f64 * 5.0),
                disk_usage_percentage: 30.0 + (i as f64 * 2.0),
                network_bandwidth_mbps: 10.0 + (i as f64 * 1.0),
                active_broadcasts: i,
            });
        }
        
        let trends = monitor.analyze_trends();
        
        assert!(trends.memory_trend.confidence > 0.0);
        assert!(trends.cpu_trend.confidence > 0.0);
    }
}
