//! Alerting system for broadcast monitoring

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::monitoring::{
    BroadcastPerformanceSummary, PerformanceStatus, ErrorAnalysisReport
};

/// Alert manager for broadcast operations
#[derive(Debug, Clone)]
pub struct AlertManager {
    pub project_id: String,
    pub alert_rules: Vec<AlertRule>,
    pub active_alerts: HashMap<String, Alert>,
    pub alert_history: Vec<Alert>,
    pub notification_channels: Vec<NotificationChannel>,
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub cooldown_minutes: u64,
    pub notification_channels: Vec<String>,
}

/// Alert condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    PerformanceThreshold {
        metric: PerformanceMetric,
        operator: ComparisonOperator,
        threshold: f64,
    },
    ErrorRateThreshold {
        max_error_rate: f64,
        time_window_minutes: u64,
    },
    ConsecutiveErrors {
        max_consecutive: usize,
    },
    ResourceUsageThreshold {
        resource: ResourceType,
        operator: ComparisonOperator,
        threshold: f64,
    },
    HealthScoreThreshold {
        min_health_score: f64,
    },
    Custom {
        expression: String,
    },
}

/// Performance metrics for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMetric {
    ResponseTime,
    Throughput,
    SuccessRate,
    ErrorRate,
    MemoryUsage,
    CpuUsage,
    ActiveBroadcasts,
}

/// Resource types for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Memory,
    Cpu,
    Disk,
    Network,
}

/// Comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Active alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub title: String,
    pub message: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub created_at: String,
    pub last_updated: String,
    pub acknowledged_at: Option<String>,
    pub resolved_at: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Suppressed,
}

/// Notification channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: NotificationChannelType,
    pub config: HashMap<String, String>,
    pub enabled: bool,
}

/// Notification channel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannelType {
    Email,
    Slack,
    Webhook,
    Console,
    File,
}

/// Alert evaluation result
#[derive(Debug, Clone)]
pub struct AlertEvaluationResult {
    pub triggered_alerts: Vec<Alert>,
    pub resolved_alerts: Vec<String>,
    pub notifications_sent: Vec<NotificationResult>,
}

/// Notification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResult {
    pub channel_id: String,
    pub success: bool,
    pub message: String,
    pub timestamp: String,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            alert_rules: Vec::new(),
            active_alerts: HashMap::new(),
            alert_history: Vec::new(),
            notification_channels: Vec::new(),
        }
    }

    /// Add an alert rule
    pub fn add_alert_rule(&mut self, rule: AlertRule) {
        self.alert_rules.push(rule);
    }

    /// Add a notification channel
    pub fn add_notification_channel(&mut self, channel: NotificationChannel) {
        self.notification_channels.push(channel);
    }

    /// Evaluate alerts based on current data
    pub fn evaluate_alerts(&mut self, 
        performance_data: &PerformanceStatus,
        error_data: &ErrorAnalysisReport,
        recent_broadcasts: &[BroadcastPerformanceSummary]
    ) -> AlertEvaluationResult {
        let mut triggered_alerts = Vec::new();
        let mut resolved_alerts = Vec::new();
        let mut notifications_sent = Vec::new();

        let rules = self.alert_rules.clone();
        for rule in &rules {
            if !rule.enabled {
                continue;
            }

            // Check if rule should trigger
            if self.should_trigger_alert(rule, performance_data, error_data, recent_broadcasts) {
                // Check cooldown
                if self.is_alert_in_cooldown(rule) {
                    continue;
                }

                // Create alert
                let alert = self.create_alert(rule, performance_data, error_data);
                let alert_id = alert.id.clone();
                
                triggered_alerts.push(alert.clone());
                self.active_alerts.insert(alert_id.clone(), alert.clone());

                // Send notifications
                let notification_results = self.send_notifications(&alert, &rule.notification_channels);
                notifications_sent.extend(notification_results);
            } else {
                // Check if any active alerts for this rule should be resolved
                if let Some(alert_id) = self.get_active_alert_id_for_rule(rule) {
                    if self.should_resolve_alert(rule, performance_data, error_data, recent_broadcasts) {
                        self.resolve_alert(&alert_id);
                        resolved_alerts.push(alert_id);
                    }
                }
            }
        }

        AlertEvaluationResult {
            triggered_alerts,
            resolved_alerts,
            notifications_sent,
        }
    }

    /// Check if an alert rule should trigger
    fn should_trigger_alert(&self, 
        rule: &AlertRule, 
        performance_data: &PerformanceStatus,
        error_data: &ErrorAnalysisReport,
        recent_broadcasts: &[BroadcastPerformanceSummary]
    ) -> bool {
        match &rule.condition {
            AlertCondition::PerformanceThreshold { metric, operator, threshold } => {
                let value = self.get_performance_metric_value(metric, performance_data, recent_broadcasts);
                self.compare_values(value, operator.clone(), *threshold)
            }
            AlertCondition::ErrorRateThreshold { max_error_rate, time_window_minutes: _ } => {
                error_data.error_rate > *max_error_rate
            }
            AlertCondition::ConsecutiveErrors { max_consecutive: _ } => {
                // This would need to be tracked separately
                false // Placeholder
            }
            AlertCondition::ResourceUsageThreshold { resource, operator, threshold } => {
                let value = self.get_resource_usage_value(resource, performance_data);
                self.compare_values(value, operator.clone(), *threshold)
            }
            AlertCondition::HealthScoreThreshold { min_health_score } => {
                performance_data.performance_health < *min_health_score
            }
            AlertCondition::Custom { expression: _ } => {
                // Custom expression evaluation would be implemented here
                false // Placeholder
            }
        }
    }

    /// Get performance metric value
    fn get_performance_metric_value(&self, 
        metric: &PerformanceMetric, 
        performance_data: &PerformanceStatus,
        recent_broadcasts: &[BroadcastPerformanceSummary]
    ) -> f64 {
        match metric {
            PerformanceMetric::ResponseTime => performance_data.average_response_time_ms,
            PerformanceMetric::Throughput => performance_data.total_throughput,
            PerformanceMetric::SuccessRate => {
                if recent_broadcasts.is_empty() {
                    1.0
                } else {
                    recent_broadcasts.iter().map(|b| b.success_rate).sum::<f64>() / recent_broadcasts.len() as f64
                }
            }
            PerformanceMetric::ErrorRate => {
                if recent_broadcasts.is_empty() {
                    0.0
                } else {
                    recent_broadcasts.iter().map(|b| b.error_rate).sum::<f64>() / recent_broadcasts.len() as f64
                }
            }
            PerformanceMetric::MemoryUsage => performance_data.total_memory_usage_mb,
            PerformanceMetric::CpuUsage => performance_data.total_cpu_usage_percent,
            PerformanceMetric::ActiveBroadcasts => performance_data.active_broadcasts as f64,
        }
    }

    /// Get resource usage value
    fn get_resource_usage_value(&self, resource: &ResourceType, performance_data: &PerformanceStatus) -> f64 {
        match resource {
            ResourceType::Memory => performance_data.total_memory_usage_mb,
            ResourceType::Cpu => performance_data.total_cpu_usage_percent,
            ResourceType::Disk => 0.0, // Would need to be tracked separately
            ResourceType::Network => 0.0, // Would need to be tracked separately
        }
    }

    /// Compare values using operator
    fn compare_values(&self, value: f64, operator: ComparisonOperator, threshold: f64) -> bool {
        match operator {
            ComparisonOperator::GreaterThan => value > threshold,
            ComparisonOperator::LessThan => value < threshold,
            ComparisonOperator::GreaterThanOrEqual => value >= threshold,
            ComparisonOperator::LessThanOrEqual => value <= threshold,
            ComparisonOperator::Equal => (value - threshold).abs() < f64::EPSILON,
            ComparisonOperator::NotEqual => (value - threshold).abs() >= f64::EPSILON,
        }
    }

    /// Check if alert is in cooldown period
    fn is_alert_in_cooldown(&self, rule: &AlertRule) -> bool {
        if let Some(alert) = self.active_alerts.values().find(|a| a.rule_id == rule.id) {
            if let Ok(last_updated) = chrono::DateTime::parse_from_rfc3339(&alert.last_updated) {
                let cooldown_duration = chrono::Duration::minutes(rule.cooldown_minutes as i64);
                let now = chrono::Utc::now();
                return now.timestamp() - last_updated.timestamp() < cooldown_duration.num_seconds();
            }
        }
        false
    }

    /// Create a new alert
    fn create_alert(&self, 
        rule: &AlertRule, 
        performance_data: &PerformanceStatus,
        error_data: &ErrorAnalysisReport
    ) -> Alert {
        let alert_id = format!("alert_{}_{}", rule.id, chrono::Utc::now().timestamp());
        let now = chrono::Utc::now().to_rfc3339();
        
        let (title, message) = self.generate_alert_content(rule, performance_data, error_data);
        
        Alert {
            id: alert_id,
            rule_id: rule.id.clone(),
            title,
            message,
            severity: rule.severity.clone(),
            status: AlertStatus::Active,
            created_at: now.clone(),
            last_updated: now,
            acknowledged_at: None,
            resolved_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Generate alert content
    fn generate_alert_content(&self, 
        rule: &AlertRule, 
        performance_data: &PerformanceStatus,
        error_data: &ErrorAnalysisReport
    ) -> (String, String) {
        let title = format!("{} - {}", rule.name, self.project_id);
        
        let message = match &rule.condition {
            AlertCondition::PerformanceThreshold { metric, operator, threshold } => {
                let value = self.get_performance_metric_value(metric, performance_data, &[]);
                format!(
                    "Performance alert: {} {} {} (current: {:.2})",
                    format!("{:?}", metric),
                    format!("{:?}", operator),
                    threshold,
                    value
                )
            }
            AlertCondition::ErrorRateThreshold { max_error_rate, .. } => {
                format!(
                    "Error rate alert: Current error rate {:.2}% exceeds threshold {:.2}%",
                    error_data.error_rate * 100.0,
                    max_error_rate * 100.0
                )
            }
            AlertCondition::ResourceUsageThreshold { resource, operator, threshold } => {
                let value = self.get_resource_usage_value(resource, performance_data);
                format!(
                    "Resource usage alert: {} {} {} (current: {:.2})",
                    format!("{:?}", resource),
                    format!("{:?}", operator),
                    threshold,
                    value
                )
            }
            AlertCondition::HealthScoreThreshold { min_health_score } => {
                format!(
                    "Health score alert: Current health {:.1} below threshold {:.1}",
                    performance_data.performance_health,
                    min_health_score
                )
            }
            _ => rule.description.clone(),
        };
        
        (title, message)
    }

    /// Send notifications for an alert
    fn send_notifications(&self, alert: &Alert, channel_ids: &[String]) -> Vec<NotificationResult> {
        let mut results = Vec::new();
        
        for channel_id in channel_ids {
            if let Some(channel) = self.notification_channels.iter().find(|c| c.id == *channel_id && c.enabled) {
                let result = self.send_notification(channel, alert);
                results.push(result);
            }
        }
        
        results
    }

    /// Send notification via specific channel
    fn send_notification(&self, channel: &NotificationChannel, alert: &Alert) -> NotificationResult {
        let timestamp = chrono::Utc::now().to_rfc3339();
        
        match channel.channel_type {
            NotificationChannelType::Console => {
                println!("ALERT: {} - {}", alert.title, alert.message);
                NotificationResult {
                    channel_id: channel.id.clone(),
                    success: true,
                    message: "Alert printed to console".to_string(),
                    timestamp,
                }
            }
            NotificationChannelType::File => {
                // Write to file
                let default_path = "alerts.log".to_string();
                let file_path = channel.config.get("file_path").unwrap_or(&default_path);
                let log_entry = format!("[{}] {} - {}\n", timestamp, alert.title, alert.message);
                
                if let Err(e) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(file_path)
                    .and_then(|mut f| std::io::Write::write_all(&mut f, log_entry.as_bytes()))
                {
                    return NotificationResult {
                        channel_id: channel.id.clone(),
                        success: false,
                        message: format!("Failed to write to file: {}", e),
                        timestamp,
                    };
                }
                
                NotificationResult {
                    channel_id: channel.id.clone(),
                    success: true,
                    message: format!("Alert written to {}", file_path),
                    timestamp,
                }
            }
            NotificationChannelType::Email => {
                // Email notification would be implemented here
                NotificationResult {
                    channel_id: channel.id.clone(),
                    success: false,
                    message: "Email notifications not implemented".to_string(),
                    timestamp,
                }
            }
            NotificationChannelType::Slack => {
                // Slack notification would be implemented here
                NotificationResult {
                    channel_id: channel.id.clone(),
                    success: false,
                    message: "Slack notifications not implemented".to_string(),
                    timestamp,
                }
            }
            NotificationChannelType::Webhook => {
                // Webhook notification would be implemented here
                NotificationResult {
                    channel_id: channel.id.clone(),
                    success: false,
                    message: "Webhook notifications not implemented".to_string(),
                    timestamp,
                }
            }
        }
    }

    /// Check if alert should be resolved
    fn should_resolve_alert(&self, 
        rule: &AlertRule, 
        performance_data: &PerformanceStatus,
        error_data: &ErrorAnalysisReport,
        recent_broadcasts: &[BroadcastPerformanceSummary]
    ) -> bool {
        !self.should_trigger_alert(rule, performance_data, error_data, recent_broadcasts)
    }

    /// Get active alert ID for a rule
    fn get_active_alert_id_for_rule(&self, rule: &AlertRule) -> Option<String> {
        self.active_alerts.values()
            .find(|alert| alert.rule_id == rule.id && matches!(alert.status, AlertStatus::Active))
            .map(|alert| alert.id.clone())
    }

    /// Resolve an alert
    fn resolve_alert(&mut self, alert_id: &str) {
        if let Some(alert) = self.active_alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Resolved;
            alert.resolved_at = Some(chrono::Utc::now().to_rfc3339());
            alert.last_updated = chrono::Utc::now().to_rfc3339();
            
            // Move to history
            self.alert_history.push(alert.clone());
            self.active_alerts.remove(alert_id);
        }
    }

    /// Acknowledge an alert
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> bool {
        if let Some(alert) = self.active_alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Acknowledged;
            alert.acknowledged_at = Some(chrono::Utc::now().to_rfc3339());
            alert.last_updated = chrono::Utc::now().to_rfc3339();
            true
        } else {
            false
        }
    }

    /// Get active alerts
    pub fn get_active_alerts(&self) -> Vec<&Alert> {
        self.active_alerts.values().collect()
    }

    /// Get alert history
    pub fn get_alert_history(&self, limit: Option<usize>) -> Vec<&Alert> {
        let mut history: Vec<&Alert> = self.alert_history.iter().collect();
        history.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        if let Some(limit) = limit {
            history.truncate(limit);
        }
        
        history
    }

    /// Create default alert rules
    pub fn create_default_rules(&mut self) {
        // High response time alert
        self.add_alert_rule(AlertRule {
            id: "high_response_time".to_string(),
            name: "High Response Time".to_string(),
            description: "Alert when average response time exceeds threshold".to_string(),
            condition: AlertCondition::PerformanceThreshold {
                metric: PerformanceMetric::ResponseTime,
                operator: ComparisonOperator::GreaterThan,
                threshold: 5000.0, // 5 seconds
            },
            severity: AlertSeverity::Warning,
            enabled: true,
            cooldown_minutes: 5,
            notification_channels: vec!["console".to_string()],
        });

        // High error rate alert
        self.add_alert_rule(AlertRule {
            id: "high_error_rate".to_string(),
            name: "High Error Rate".to_string(),
            description: "Alert when error rate exceeds threshold".to_string(),
            condition: AlertCondition::ErrorRateThreshold {
                max_error_rate: 0.1, // 10%
                time_window_minutes: 10,
            },
            severity: AlertSeverity::Critical,
            enabled: true,
            cooldown_minutes: 2,
            notification_channels: vec!["console".to_string()],
        });

        // Low success rate alert
        self.add_alert_rule(AlertRule {
            id: "low_success_rate".to_string(),
            name: "Low Success Rate".to_string(),
            description: "Alert when success rate falls below threshold".to_string(),
            condition: AlertCondition::PerformanceThreshold {
                metric: PerformanceMetric::SuccessRate,
                operator: ComparisonOperator::LessThan,
                threshold: 0.9, // 90%
            },
            severity: AlertSeverity::Warning,
            enabled: true,
            cooldown_minutes: 5,
            notification_channels: vec!["console".to_string()],
        });

        // High memory usage alert
        self.add_alert_rule(AlertRule {
            id: "high_memory_usage".to_string(),
            name: "High Memory Usage".to_string(),
            description: "Alert when memory usage exceeds threshold".to_string(),
            condition: AlertCondition::ResourceUsageThreshold {
                resource: ResourceType::Memory,
                operator: ComparisonOperator::GreaterThan,
                threshold: 100.0, // 100 MB
            },
            severity: AlertSeverity::Warning,
            enabled: true,
            cooldown_minutes: 10,
            notification_channels: vec!["console".to_string()],
        });

        // Low health score alert
        self.add_alert_rule(AlertRule {
            id: "low_health_score".to_string(),
            name: "Low Health Score".to_string(),
            description: "Alert when system health score falls below threshold".to_string(),
            condition: AlertCondition::HealthScoreThreshold {
                min_health_score: 70.0, // 70%
            },
            severity: AlertSeverity::Critical,
            enabled: true,
            cooldown_minutes: 3,
            notification_channels: vec!["console".to_string()],
        });
    }

    /// Create default notification channels
    pub fn create_default_channels(&mut self) {
        // Console channel
        self.add_notification_channel(NotificationChannel {
            id: "console".to_string(),
            name: "Console Output".to_string(),
            channel_type: NotificationChannelType::Console,
            config: HashMap::new(),
            enabled: true,
        });

        // File channel
        let mut file_config = HashMap::new();
        file_config.insert("file_path".to_string(), "./logs/alerts.log".to_string());
        
        self.add_notification_channel(NotificationChannel {
            id: "file".to_string(),
            name: "File Logging".to_string(),
            channel_type: NotificationChannelType::File,
            config: file_config,
            enabled: true,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_manager_creation() {
        let manager = AlertManager::new("test-project".to_string());
        assert_eq!(manager.project_id, "test-project");
        assert!(manager.alert_rules.is_empty());
        assert!(manager.active_alerts.is_empty());
    }

    #[test]
    fn test_add_alert_rule() {
        let mut manager = AlertManager::new("test-project".to_string());
        
        let rule = AlertRule {
            id: "test_rule".to_string(),
            name: "Test Rule".to_string(),
            description: "Test alert rule".to_string(),
            condition: AlertCondition::PerformanceThreshold {
                metric: PerformanceMetric::ResponseTime,
                operator: ComparisonOperator::GreaterThan,
                threshold: 1000.0,
            },
            severity: AlertSeverity::Warning,
            enabled: true,
            cooldown_minutes: 5,
            notification_channels: vec!["console".to_string()],
        };
        
        manager.add_alert_rule(rule);
        assert_eq!(manager.alert_rules.len(), 1);
    }

    #[test]
    fn test_alert_evaluation() {
        let mut manager = AlertManager::new("test-project".to_string());
        manager.create_default_rules();
        manager.create_default_channels();
        
        let performance_data = PerformanceStatus {
            active_broadcasts: 0,
            total_throughput: 0.0,
            average_response_time_ms: 6000.0, // Should trigger high response time alert
            total_memory_usage_mb: 0.0,
            total_cpu_usage_percent: 0.0,
            performance_health: 100.0,
        };
        
        let error_data = ErrorAnalysisReport {
            project_id: "test-project".to_string(),
            analysis_timestamp: chrono::Utc::now().to_rfc3339(),
            total_errors: 0,
            error_rate: 0.05,
            top_error_categories: Vec::new(),
            error_trends: Vec::new(),
            recommendations: Vec::new(),
            health_score: 100.0,
        };
        
        let result = manager.evaluate_alerts(&performance_data, &error_data, &[]);
        
        // Should trigger high response time alert
        assert!(!result.triggered_alerts.is_empty());
    }

    #[test]
    fn test_alert_acknowledgment() {
        let mut manager = AlertManager::new("test-project".to_string());
        manager.create_default_rules();
        manager.create_default_channels();
        
        let performance_data = PerformanceStatus {
            active_broadcasts: 0,
            total_throughput: 0.0,
            average_response_time_ms: 6000.0,
            total_memory_usage_mb: 0.0,
            total_cpu_usage_percent: 0.0,
            performance_health: 100.0,
        };
        
        let error_data = ErrorAnalysisReport {
            project_id: "test-project".to_string(),
            analysis_timestamp: chrono::Utc::now().to_rfc3339(),
            total_errors: 0,
            error_rate: 0.05,
            top_error_categories: Vec::new(),
            error_trends: Vec::new(),
            recommendations: Vec::new(),
            health_score: 100.0,
        };
        
        let result = manager.evaluate_alerts(&performance_data, &error_data, &[]);
        
        if let Some(alert) = result.triggered_alerts.first() {
            let acknowledged = manager.acknowledge_alert(&alert.id);
            assert!(acknowledged);
        }
    }
}
