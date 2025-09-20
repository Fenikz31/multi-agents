//! Dashboard generation for broadcast monitoring

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::monitoring::{
    BroadcastPerformanceSummary, PerformanceStatus, ErrorAnalysisReport,
    MonitorPerformanceTrends, ErrorTrend, MonitorTrendDirection
};

/// Dashboard data aggregator for broadcast operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastDashboard {
    pub project_id: String,
    pub performance_data: PerformanceDashboardData,
    pub error_data: ErrorDashboardData,
    pub resource_data: ResourceDashboardData,
    pub trends_data: TrendsDashboardData,
}

/// Performance dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDashboardData {
    pub current_status: PerformanceStatus,
    pub recent_broadcasts: Vec<BroadcastPerformanceSummary>,
    pub performance_metrics: PerformanceMetrics,
    pub health_indicators: HealthIndicators,
}

/// Error dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDashboardData {
    pub error_analysis: ErrorAnalysisReport,
    pub error_trends: Vec<ErrorTrend>,
    pub error_distribution: HashMap<String, usize>,
    pub recent_errors: Vec<RecentError>,
}

/// Resource dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDashboardData {
    pub memory_usage: ResourceUsage,
    pub cpu_usage: ResourceUsage,
    pub network_usage: ResourceUsage,
    pub disk_usage: ResourceUsage,
    pub resource_trends: ResourceTrends,
}

/// Trends dashboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsDashboardData {
    pub performance_trends: MonitorPerformanceTrends,
    pub error_trends: Vec<ErrorTrend>,
    pub resource_trends: ResourceTrends,
    pub forecast: PerformanceForecast,
}

/// Performance metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub total_broadcasts: usize,
    pub success_rate: f64,
    pub average_response_time_ms: f64,
    pub p95_response_time_ms: u64,
    pub p99_response_time_ms: u64,
    pub throughput_per_second: f64,
    pub peak_throughput: f64,
    pub error_rate: f64,
    pub uptime_percentage: f64,
}

/// Health indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIndicators {
    pub overall_health: f64, // 0-100
    pub performance_health: f64,
    pub error_health: f64,
    pub resource_health: f64,
    pub status: HealthStatus,
    pub alerts_count: usize,
    pub critical_alerts: usize,
}

/// Health status levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,      // 80-100
    Warning,      // 60-79
    Critical,     // 40-59
    Down,         // 0-39
}

/// Resource usage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub current: f64,
    pub average: f64,
    pub peak: f64,
    pub trend: MonitorTrendDirection,
    pub unit: String,
}

/// Resource trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTrends {
    pub memory_trend: MonitorTrendDirection,
    pub cpu_trend: MonitorTrendDirection,
    pub network_trend: MonitorTrendDirection,
    pub disk_trend: MonitorTrendDirection,
    pub efficiency_score: f64, // 0-100
}

/// Recent error entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentError {
    pub timestamp: String,
    pub broadcast_id: String,
    pub agent_id: String,
    pub error_type: String,
    pub error_message: String,
    pub severity: ErrorSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Performance forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceForecast {
    pub next_hour_prediction: PerformancePrediction,
    pub next_day_prediction: PerformancePrediction,
    pub confidence_level: f64, // 0-100
    pub recommendations: Vec<String>,
}

/// Performance prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePrediction {
    pub expected_throughput: f64,
    pub expected_response_time_ms: f64,
    pub expected_error_rate: f64,
    pub expected_resource_usage: f64,
    pub risk_level: RiskLevel,
}

/// Risk levels for predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl BroadcastDashboard {
    /// Create a new dashboard
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            performance_data: PerformanceDashboardData::default(),
            error_data: ErrorDashboardData::default(),
            resource_data: ResourceDashboardData::default(),
            trends_data: TrendsDashboardData::default(),
        }
    }

    /// Update dashboard with new performance data
    pub fn update_performance_data(&mut self, 
        current_status: PerformanceStatus,
        recent_broadcasts: Vec<BroadcastPerformanceSummary>,
        performance_metrics: PerformanceMetrics
    ) {
        self.performance_data.current_status = current_status;
        self.performance_data.recent_broadcasts = recent_broadcasts;
        self.performance_data.performance_metrics = performance_metrics;
        self.performance_data.health_indicators = self.calculate_health_indicators();
    }

    /// Update dashboard with new error data
    pub fn update_error_data(&mut self, 
        error_analysis: ErrorAnalysisReport,
        error_trends: Vec<ErrorTrend>,
        recent_errors: Vec<RecentError>
    ) {
        self.error_data.error_analysis = error_analysis;
        self.error_data.error_trends = error_trends;
        self.error_data.recent_errors = recent_errors;
        self.error_data.error_distribution = self.calculate_error_distribution();
    }

    /// Update dashboard with new resource data
    pub fn update_resource_data(&mut self, resource_data: ResourceDashboardData) {
        self.resource_data = resource_data;
    }

    /// Update dashboard with new trends data
    pub fn update_trends_data(&mut self, trends_data: TrendsDashboardData) {
        self.trends_data = trends_data;
    }

    /// Generate comprehensive dashboard report
    pub fn generate_dashboard_report(&self) -> DashboardReport {
        DashboardReport {
            project_id: self.project_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            dashboard_data: self.clone(),
            summary: self.generate_summary(),
            alerts: self.generate_alerts(),
            recommendations: self.generate_recommendations(),
        }
    }

    /// Generate dashboard summary
    fn generate_summary(&self) -> DashboardSummary {
        DashboardSummary {
            overall_health: self.performance_data.health_indicators.overall_health,
            status: self.performance_data.health_indicators.status.clone(),
            active_broadcasts: self.performance_data.current_status.active_broadcasts,
            total_broadcasts_today: self.performance_data.performance_metrics.total_broadcasts,
            success_rate: self.performance_data.performance_metrics.success_rate,
            average_response_time_ms: self.performance_data.performance_metrics.average_response_time_ms,
            error_rate: self.performance_data.performance_metrics.error_rate,
            alerts_count: self.performance_data.health_indicators.alerts_count,
        }
    }

    /// Generate alerts from dashboard data
    fn generate_alerts(&self) -> Vec<DashboardAlert> {
        let mut alerts = Vec::new();
        
        // Performance alerts
        if self.performance_data.health_indicators.performance_health < 70.0 {
            alerts.push(DashboardAlert {
                severity: AlertSeverity::Warning,
                category: "Performance".to_string(),
                message: "Performance health is below optimal levels".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }
        
        // Error alerts
        if self.performance_data.health_indicators.error_health < 80.0 {
            alerts.push(DashboardAlert {
                severity: AlertSeverity::Warning,
                category: "Errors".to_string(),
                message: "Error rate is higher than expected".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }
        
        // Resource alerts
        if self.performance_data.health_indicators.resource_health < 75.0 {
            alerts.push(DashboardAlert {
                severity: AlertSeverity::Warning,
                category: "Resources".to_string(),
                message: "Resource usage is approaching limits".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }
        
        // Critical alerts
        if self.performance_data.health_indicators.critical_alerts > 0 {
            alerts.push(DashboardAlert {
                severity: AlertSeverity::Critical,
                category: "System".to_string(),
                message: format!("{} critical issues detected", self.performance_data.health_indicators.critical_alerts),
                timestamp: chrono::Utc::now().to_rfc3339(),
            });
        }
        
        alerts
    }

    /// Generate recommendations based on dashboard data
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Performance recommendations
        if self.performance_data.performance_metrics.average_response_time_ms > 3000.0 {
            recommendations.push("Consider optimizing agent response times or increasing timeout values".to_string());
        }
        
        if self.performance_data.performance_metrics.throughput_per_second < 1.0 {
            recommendations.push("Throughput is low. Consider scaling up or optimizing broadcast operations".to_string());
        }
        
        // Error recommendations
        if self.performance_data.performance_metrics.error_rate > 0.05 {
            recommendations.push("Error rate is high. Review error logs and implement better error handling".to_string());
        }
        
        // Resource recommendations
        if self.resource_data.memory_usage.current > 80.0 {
            recommendations.push("Memory usage is high. Consider optimizing memory usage or scaling resources".to_string());
        }
        
        if self.resource_data.cpu_usage.current > 80.0 {
            recommendations.push("CPU usage is high. Consider load balancing or scaling resources".to_string());
        }
        
        // Trend-based recommendations
        match self.trends_data.performance_trends.trend_direction {
            MonitorTrendDirection::Declining => {
                recommendations.push("Performance is declining. Investigate recent changes and optimize operations".to_string());
            }
            MonitorTrendDirection::Improving => {
                recommendations.push("Performance is improving. Continue current practices".to_string());
            }
            MonitorTrendDirection::Stable => {
                recommendations.push("Performance is stable. Monitor for any changes".to_string());
            }
        }
        
        if recommendations.is_empty() {
            recommendations.push("System is operating normally. Continue monitoring".to_string());
        }
        
        recommendations
    }

    /// Calculate health indicators
    fn calculate_health_indicators(&self) -> HealthIndicators {
        let performance_health = self.calculate_performance_health();
        let error_health = self.calculate_error_health();
        let resource_health = self.calculate_resource_health();
        
        let overall_health = (performance_health + error_health + resource_health) / 3.0;
        
        let status = match overall_health {
            h if h >= 80.0 => HealthStatus::Healthy,
            h if h >= 60.0 => HealthStatus::Warning,
            h if h >= 40.0 => HealthStatus::Critical,
            _ => HealthStatus::Down,
        };
        
        let alerts_count = self.count_alerts();
        let critical_alerts = self.count_critical_alerts();
        
        HealthIndicators {
            overall_health,
            performance_health,
            error_health,
            resource_health,
            status,
            alerts_count,
            critical_alerts,
        }
    }

    /// Calculate performance health score
    fn calculate_performance_health(&self) -> f64 {
        let mut score: f64 = 100.0;
        
        // Deduct for high response times
        if self.performance_data.performance_metrics.average_response_time_ms > 5000.0 {
            score -= 30.0;
        } else if self.performance_data.performance_metrics.average_response_time_ms > 3000.0 {
            score -= 15.0;
        }
        
        // Deduct for low throughput
        if self.performance_data.performance_metrics.throughput_per_second < 0.5 {
            score -= 25.0;
        } else if self.performance_data.performance_metrics.throughput_per_second < 1.0 {
            score -= 10.0;
        }
        
        // Deduct for low success rate
        if self.performance_data.performance_metrics.success_rate < 0.9 {
            score -= 20.0;
        } else if self.performance_data.performance_metrics.success_rate < 0.95 {
            score -= 10.0;
        }
        
        score.max(0.0)
    }

    /// Calculate error health score
    fn calculate_error_health(&self) -> f64 {
        let mut score: f64 = 100.0;
        
        // Deduct for high error rate
        if self.performance_data.performance_metrics.error_rate > 0.1 {
            score -= 50.0;
        } else if self.performance_data.performance_metrics.error_rate > 0.05 {
            score -= 25.0;
        } else if self.performance_data.performance_metrics.error_rate > 0.01 {
            score -= 10.0;
        }
        
        // Deduct for recent errors
        if self.error_data.recent_errors.len() > 10 {
            score -= 20.0;
        } else if self.error_data.recent_errors.len() > 5 {
            score -= 10.0;
        }
        
        score.max(0.0)
    }

    /// Calculate resource health score
    fn calculate_resource_health(&self) -> f64 {
        let mut score: f64 = 100.0;
        
        // Deduct for high memory usage
        if self.resource_data.memory_usage.current > 90.0 {
            score -= 30.0;
        } else if self.resource_data.memory_usage.current > 80.0 {
            score -= 15.0;
        }
        
        // Deduct for high CPU usage
        if self.resource_data.cpu_usage.current > 90.0 {
            score -= 30.0;
        } else if self.resource_data.cpu_usage.current > 80.0 {
            score -= 15.0;
        }
        
        // Deduct for high disk usage
        if self.resource_data.disk_usage.current > 90.0 {
            score -= 20.0;
        } else if self.resource_data.disk_usage.current > 80.0 {
            score -= 10.0;
        }
        
        score.max(0.0)
    }

    /// Count total alerts
    fn count_alerts(&self) -> usize {
        // This would typically count from actual alert sources
        // For now, return a calculated value based on health scores
        let mut count = 0;
        
        if self.performance_data.health_indicators.performance_health < 70.0 {
            count += 1;
        }
        if self.performance_data.health_indicators.error_health < 80.0 {
            count += 1;
        }
        if self.performance_data.health_indicators.resource_health < 75.0 {
            count += 1;
        }
        
        count
    }

    /// Count critical alerts
    fn count_critical_alerts(&self) -> usize {
        let mut count = 0;
        
        if self.performance_data.health_indicators.performance_health < 50.0 {
            count += 1;
        }
        if self.performance_data.health_indicators.error_health < 60.0 {
            count += 1;
        }
        if self.performance_data.health_indicators.resource_health < 50.0 {
            count += 1;
        }
        
        count
    }

    /// Calculate error distribution
    fn calculate_error_distribution(&self) -> HashMap<String, usize> {
        let mut distribution = HashMap::new();
        
        for error in &self.error_data.recent_errors {
            let count = distribution.entry(error.error_type.clone()).or_insert(0);
            *count += 1;
        }
        
        distribution
    }
}

/// Comprehensive dashboard report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardReport {
    pub project_id: String,
    pub generated_at: String,
    pub dashboard_data: BroadcastDashboard,
    pub summary: DashboardSummary,
    pub alerts: Vec<DashboardAlert>,
    pub recommendations: Vec<String>,
}

/// Dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub overall_health: f64,
    pub status: HealthStatus,
    pub active_broadcasts: usize,
    pub total_broadcasts_today: usize,
    pub success_rate: f64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub alerts_count: usize,
}

/// Dashboard alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAlert {
    pub severity: AlertSeverity,
    pub category: String,
    pub message: String,
    pub timestamp: String,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl Default for PerformanceDashboardData {
    fn default() -> Self {
        Self {
            current_status: PerformanceStatus {
                active_broadcasts: 0,
                total_throughput: 0.0,
                average_response_time_ms: 0.0,
                total_memory_usage_mb: 0.0,
                total_cpu_usage_percent: 0.0,
                performance_health: 100.0,
            },
            recent_broadcasts: Vec::new(),
            performance_metrics: PerformanceMetrics::default(),
            health_indicators: HealthIndicators {
                overall_health: 100.0,
                performance_health: 100.0,
                error_health: 100.0,
                resource_health: 100.0,
                status: HealthStatus::Healthy,
                alerts_count: 0,
                critical_alerts: 0,
            },
        }
    }
}

impl Default for ErrorDashboardData {
    fn default() -> Self {
        Self {
            error_analysis: ErrorAnalysisReport {
                project_id: String::new(),
                analysis_timestamp: chrono::Utc::now().to_rfc3339(),
                total_errors: 0,
                error_rate: 0.0,
                top_error_categories: Vec::new(),
                error_trends: Vec::new(),
                recommendations: Vec::new(),
                health_score: 100.0,
            },
            error_trends: Vec::new(),
            error_distribution: HashMap::new(),
            recent_errors: Vec::new(),
        }
    }
}

impl Default for ResourceDashboardData {
    fn default() -> Self {
        Self {
            memory_usage: ResourceUsage {
                current: 0.0,
                average: 0.0,
                peak: 0.0,
                trend: MonitorTrendDirection::Stable,
                unit: "MB".to_string(),
            },
            cpu_usage: ResourceUsage {
                current: 0.0,
                average: 0.0,
                peak: 0.0,
                trend: MonitorTrendDirection::Stable,
                unit: "%".to_string(),
            },
            network_usage: ResourceUsage {
                current: 0.0,
                average: 0.0,
                peak: 0.0,
                trend: MonitorTrendDirection::Stable,
                unit: "MB/s".to_string(),
            },
            disk_usage: ResourceUsage {
                current: 0.0,
                average: 0.0,
                peak: 0.0,
                trend: MonitorTrendDirection::Stable,
                unit: "MB".to_string(),
            },
            resource_trends: ResourceTrends {
                memory_trend: MonitorTrendDirection::Stable,
                cpu_trend: MonitorTrendDirection::Stable,
                network_trend: MonitorTrendDirection::Stable,
                disk_trend: MonitorTrendDirection::Stable,
                efficiency_score: 100.0,
            },
        }
    }
}

impl Default for TrendsDashboardData {
    fn default() -> Self {
        Self {
            performance_trends: MonitorPerformanceTrends {
                period_hours: 24,
                total_snapshots: 0,
                average_throughput: 0.0,
                average_response_time_ms: 0.0,
                average_memory_usage_mb: 0.0,
                average_cpu_usage_percent: 0.0,
                average_success_rate: 0.0,
                trend_direction: MonitorTrendDirection::Stable,
            },
            error_trends: Vec::new(),
            resource_trends: ResourceTrends {
                memory_trend: MonitorTrendDirection::Stable,
                cpu_trend: MonitorTrendDirection::Stable,
                network_trend: MonitorTrendDirection::Stable,
                disk_trend: MonitorTrendDirection::Stable,
                efficiency_score: 100.0,
            },
            forecast: PerformanceForecast {
                next_hour_prediction: PerformancePrediction {
                    expected_throughput: 0.0,
                    expected_response_time_ms: 0.0,
                    expected_error_rate: 0.0,
                    expected_resource_usage: 0.0,
                    risk_level: RiskLevel::Low,
                },
                next_day_prediction: PerformancePrediction {
                    expected_throughput: 0.0,
                    expected_response_time_ms: 0.0,
                    expected_error_rate: 0.0,
                    expected_resource_usage: 0.0,
                    risk_level: RiskLevel::Low,
                },
                confidence_level: 0.0,
                recommendations: Vec::new(),
            },
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            total_broadcasts: 0,
            success_rate: 1.0,
            average_response_time_ms: 0.0,
            p95_response_time_ms: 0,
            p99_response_time_ms: 0,
            throughput_per_second: 0.0,
            peak_throughput: 0.0,
            error_rate: 0.0,
            uptime_percentage: 100.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dashboard_creation() {
        let dashboard = BroadcastDashboard::new("test-project".to_string());
        assert_eq!(dashboard.project_id, "test-project");
    }

    #[test]
    fn test_performance_data_update() {
        let mut dashboard = BroadcastDashboard::new("test-project".to_string());
        
        let status = PerformanceStatus {
            active_broadcasts: 2,
            total_throughput: 5.0,
            average_response_time_ms: 1000.0,
            total_memory_usage_mb: 50.0,
            total_cpu_usage_percent: 30.0,
            performance_health: 90.0,
        };
        
        let metrics = PerformanceMetrics::default();
        
        dashboard.update_performance_data(status, Vec::new(), metrics);
        
        assert_eq!(dashboard.performance_data.current_status.active_broadcasts, 2);
        assert_eq!(dashboard.performance_data.current_status.total_throughput, 5.0);
    }

    #[test]
    fn test_health_calculation() {
        let mut dashboard = BroadcastDashboard::new("test-project".to_string());
        
        let status = PerformanceStatus {
            active_broadcasts: 0,
            total_throughput: 0.0,
            average_response_time_ms: 1000.0,
            total_memory_usage_mb: 0.0,
            total_cpu_usage_percent: 0.0,
            performance_health: 100.0,
        };
        
        let metrics = PerformanceMetrics {
            success_rate: 0.95,
            average_response_time_ms: 1000.0,
            throughput_per_second: 2.0,
            error_rate: 0.05,
            ..Default::default()
        };
        
        dashboard.update_performance_data(status, Vec::new(), metrics);
        
        let health = dashboard.performance_data.health_indicators;
        assert!(health.overall_health > 0.0);
        assert!(health.performance_health > 0.0);
    }

    #[test]
    fn test_dashboard_report_generation() {
        let dashboard = BroadcastDashboard::new("test-project".to_string());
        let report = dashboard.generate_dashboard_report();
        
        assert_eq!(report.project_id, "test-project");
        assert!(!report.generated_at.is_empty());
        assert!(report.summary.overall_health >= 0.0);
    }
}
