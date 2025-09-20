//! Monitoring and metrics for Multi-Agents CLI
//! 
//! This module provides comprehensive monitoring and metrics collection
//! for broadcast operations, performance tracking, and observability.

pub mod broadcast_metrics;
pub mod performance_monitor;
pub mod error_tracker;
pub mod resource_monitor;
pub mod dashboard;
pub mod alerting;

// Re-export specific types to avoid conflicts
pub use broadcast_metrics::{
    BroadcastMetrics, AgentMetrics, BroadcastPerformanceSummary, 
    BroadcastMetricsSnapshot, BroadcastMetricsAggregator, DailyStats,
    PerformanceTrends as BroadcastPerformanceTrends, TrendDirection as BroadcastTrendDirection
};
pub use performance_monitor::{
    PerformanceMonitor, BroadcastPerformanceTracker, PerformanceThresholds,
    PerformanceSnapshot, PerformanceAlert, PerformanceStatus,
    PerformanceTrends as MonitorPerformanceTrends, TrendDirection as MonitorTrendDirection
};
pub use error_tracker::{
    ErrorTracker, ErrorCategory, BroadcastError, ErrorTrend, ErrorAlertThresholds,
    ErrorAnalysisReport, TrendDirection as ErrorTrendDirection
};
pub use resource_monitor::{
    ResourceMonitor, ResourceMetrics, MemoryMetrics, CpuMetrics, DiskMetrics, NetworkMetrics,
    ResourceTrend, ResourceLimits, ResourceSnapshot, ResourceUsageReport, ResourceTrends,
    ResourceAlert, AlertSeverity as ResourceAlertSeverity
};
pub use dashboard::{
    BroadcastDashboard, PerformanceDashboardData, ErrorDashboardData, ResourceDashboardData,
    TrendsDashboardData, PerformanceMetrics, HealthIndicators, HealthStatus, ResourceUsage,
    ResourceTrends as DashboardResourceTrends, RecentError, ErrorSeverity, PerformanceForecast,
    PerformancePrediction, RiskLevel, DashboardReport, DashboardSummary, DashboardAlert,
    AlertSeverity as DashboardAlertSeverity
};
pub use alerting::{
    AlertManager, AlertRule, AlertCondition, PerformanceMetric, ResourceType, ComparisonOperator,
    AlertSeverity, Alert, AlertStatus, NotificationChannel, NotificationChannelType,
    AlertEvaluationResult, NotificationResult
};
