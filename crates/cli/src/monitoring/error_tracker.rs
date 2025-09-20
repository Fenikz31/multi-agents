//! Error tracking and analysis for broadcast operations

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::logging::ndjson::emit_failure_metrics_event;

/// Error tracker for broadcast operations
#[derive(Debug, Clone)]
pub struct ErrorTracker {
    pub project_id: String,
    pub error_counts: HashMap<ErrorCategory, usize>,
    pub error_rates: HashMap<ErrorCategory, f64>,
    pub recent_errors: Vec<BroadcastError>,
    pub error_trends: HashMap<ErrorCategory, ErrorTrend>,
    pub alert_thresholds: ErrorAlertThresholds,
}

/// Categories of broadcast errors
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    Timeout,
    ProviderUnavailable,
    InvalidTarget,
    ConfigurationError,
    NetworkError,
    AuthenticationError,
    RateLimitExceeded,
    InternalError,
    Unknown,
}

/// Individual broadcast error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastError {
    pub broadcast_id: String,
    pub agent_id: String,
    pub role: String,
    pub provider: String,
    pub error_category: ErrorCategory,
    pub error_message: String,
    pub timestamp: String,
    pub duration_ms: u64,
    pub retry_count: usize,
}

/// Error trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorTrend {
    pub category: ErrorCategory,
    pub trend_direction: TrendDirection,
    pub recent_rate: f64,
    pub historical_rate: f64,
    pub change_percentage: f64,
}

/// Error alert thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAlertThresholds {
    pub max_error_rate: f64, // Maximum acceptable error rate (0.0-1.0)
    pub max_timeout_rate: f64,
    pub max_provider_unavailable_rate: f64,
    pub max_network_error_rate: f64,
    pub max_consecutive_errors: usize,
    pub alert_window_minutes: usize,
}

/// Trend direction for error analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Error analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysisReport {
    pub project_id: String,
    pub analysis_timestamp: String,
    pub total_errors: usize,
    pub error_rate: f64,
    pub top_error_categories: Vec<(ErrorCategory, usize, f64)>, // (category, count, rate)
    pub error_trends: Vec<ErrorTrend>,
    pub recommendations: Vec<String>,
    pub health_score: f64, // 0-100
}

impl ErrorTracker {
    /// Create a new error tracker
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            error_counts: HashMap::new(),
            error_rates: HashMap::new(),
            recent_errors: Vec::new(),
            error_trends: HashMap::new(),
            alert_thresholds: ErrorAlertThresholds::default(),
        }
    }

    /// Record a broadcast error
    pub fn record_error(&mut self, error: BroadcastError) -> Vec<ErrorAlert> {
        let mut alerts = Vec::new();
        
        // Update error counts
        *self.error_counts.entry(error.error_category.clone()).or_insert(0) += 1;
        
        // Add to recent errors
        self.recent_errors.push(error.clone());
        
        // Keep only recent errors (within alert window)
        self.cleanup_old_errors();
        
        // Update error rates
        self.update_error_rates();
        
        // Update error trends
        self.update_error_trends();
        
        // Check for alerts
        alerts.extend(self.check_error_alerts());
        
        // Emit failure metrics
        let _ = emit_failure_metrics_event(
            &self.project_id,
            &error.role,
            &error.agent_id,
            &error.provider,
            "broadcast_error",
            &format!("{:?}", error.error_category),
            error.duration_ms,
            &error.error_message
        );
        
        alerts
    }

    /// Get current error rate for a category
    pub fn get_error_rate(&self, category: &ErrorCategory) -> f64 {
        self.error_rates.get(category).copied().unwrap_or(0.0)
    }

    /// Get total error rate across all categories
    pub fn get_total_error_rate(&self) -> f64 {
        let total_errors: usize = self.error_counts.values().sum();
        let total_operations = self.get_total_operations();
        
        if total_operations > 0 {
            total_errors as f64 / total_operations as f64
        } else {
            0.0
        }
    }

    /// Get total operations (errors + successes)
    fn get_total_operations(&self) -> usize {
        // This would typically come from a separate success counter
        // For now, we'll estimate based on recent errors
        self.recent_errors.len() * 10 // Assume 10:1 success to error ratio
    }

    /// Update error rates based on recent data
    fn update_error_rates(&mut self) {
        let total_operations = self.get_total_operations();
        
        if total_operations > 0 {
            for (category, count) in &self.error_counts {
                let rate = *count as f64 / total_operations as f64;
                self.error_rates.insert(category.clone(), rate);
            }
        }
    }

    /// Update error trends based on historical data
    fn update_error_trends(&mut self) {
        let window_start = chrono::Utc::now() - chrono::Duration::minutes(self.alert_thresholds.alert_window_minutes as i64);
        
        for category in &[
            ErrorCategory::Timeout,
            ErrorCategory::ProviderUnavailable,
            ErrorCategory::InvalidTarget,
            ErrorCategory::ConfigurationError,
            ErrorCategory::NetworkError,
            ErrorCategory::AuthenticationError,
            ErrorCategory::RateLimitExceeded,
            ErrorCategory::InternalError,
            ErrorCategory::Unknown,
        ] {
            let recent_errors: Vec<&BroadcastError> = self.recent_errors
                .iter()
                .filter(|e| e.error_category == *category)
                .filter(|e| {
                    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&e.timestamp) {
                        timestamp.timestamp() > window_start.timestamp()
                    } else {
                        false
                    }
                })
                .collect();
            
            let historical_errors: Vec<&BroadcastError> = self.recent_errors
                .iter()
                .filter(|e| e.error_category == *category)
                .filter(|e| {
                    if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&e.timestamp) {
                        timestamp.timestamp() <= window_start.timestamp()
                    } else {
                        false
                    }
                })
                .collect();
            
            let recent_rate = if recent_errors.is_empty() { 0.0 } else {
                recent_errors.len() as f64 / self.get_total_operations() as f64
            };
            
            let historical_rate = if historical_errors.is_empty() { 0.0 } else {
                historical_errors.len() as f64 / self.get_total_operations() as f64
            };
            
            let change_percentage = if historical_rate > 0.0 {
                ((recent_rate - historical_rate) / historical_rate) * 100.0
            } else {
                0.0
            };
            
            let trend_direction = if change_percentage > 10.0 {
                TrendDirection::Increasing
            } else if change_percentage < -10.0 {
                TrendDirection::Decreasing
            } else {
                TrendDirection::Stable
            };
            
            self.error_trends.insert(category.clone(), ErrorTrend {
                category: category.clone(),
                trend_direction,
                recent_rate,
                historical_rate,
                change_percentage,
            });
        }
    }

    /// Check for error alerts based on thresholds
    fn check_error_alerts(&self) -> Vec<ErrorAlert> {
        let mut alerts = Vec::new();
        
        // Check total error rate
        let total_error_rate = self.get_total_error_rate();
        if total_error_rate > self.alert_thresholds.max_error_rate {
            alerts.push(ErrorAlert::HighErrorRate {
                current_rate: total_error_rate,
                threshold: self.alert_thresholds.max_error_rate,
            });
        }
        
        // Check category-specific error rates
        for (category, rate) in &self.error_rates {
            let threshold = match category {
                ErrorCategory::Timeout => self.alert_thresholds.max_timeout_rate,
                ErrorCategory::ProviderUnavailable => self.alert_thresholds.max_provider_unavailable_rate,
                ErrorCategory::NetworkError => self.alert_thresholds.max_network_error_rate,
                _ => self.alert_thresholds.max_error_rate,
            };
            
            if *rate > threshold {
                alerts.push(ErrorAlert::HighCategoryErrorRate {
                    category: category.clone(),
                    current_rate: *rate,
                    threshold,
                });
            }
        }
        
        // Check for consecutive errors
        if let Some(consecutive_count) = self.get_consecutive_error_count() {
            if consecutive_count >= self.alert_thresholds.max_consecutive_errors {
                alerts.push(ErrorAlert::ConsecutiveErrors {
                    count: consecutive_count,
                    threshold: self.alert_thresholds.max_consecutive_errors,
                });
            }
        }
        
        alerts
    }

    /// Get count of consecutive errors
    fn get_consecutive_error_count(&self) -> Option<usize> {
        if self.recent_errors.is_empty() {
            return None;
        }
        
        let mut count = 0;
        for error in self.recent_errors.iter().rev() {
            if error.error_category != ErrorCategory::Unknown {
                count += 1;
            } else {
                break;
            }
        }
        
        Some(count)
    }

    /// Clean up old errors outside the alert window
    fn cleanup_old_errors(&mut self) {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::minutes(self.alert_thresholds.alert_window_minutes as i64);
        
        self.recent_errors.retain(|error| {
            if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&error.timestamp) {
                timestamp.timestamp() > cutoff_time.timestamp()
            } else {
                false
            }
        });
    }

    /// Generate comprehensive error analysis report
    pub fn generate_analysis_report(&self) -> ErrorAnalysisReport {
        let total_errors: usize = self.error_counts.values().sum();
        let total_error_rate = self.get_total_error_rate();
        
        // Get top error categories
        let mut category_stats: Vec<(ErrorCategory, usize, f64)> = self.error_counts
            .iter()
            .map(|(category, count)| {
                let rate = self.error_rates.get(category).copied().unwrap_or(0.0);
                (category.clone(), *count, rate)
            })
            .collect();
        category_stats.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Get error trends
        let error_trends: Vec<ErrorTrend> = self.error_trends.values().cloned().collect();
        
        // Generate recommendations
        let recommendations = self.generate_recommendations();
        
        // Calculate health score
        let health_score = self.calculate_health_score();
        
        ErrorAnalysisReport {
            project_id: self.project_id.clone(),
            analysis_timestamp: chrono::Utc::now().to_rfc3339(),
            total_errors,
            error_rate: total_error_rate,
            top_error_categories: category_stats,
            error_trends,
            recommendations,
            health_score,
        }
    }

    /// Generate recommendations based on error analysis
    fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Check for high timeout rates
        if let Some(timeout_rate) = self.error_rates.get(&ErrorCategory::Timeout) {
            if *timeout_rate > 0.1 {
                recommendations.push("High timeout rate detected. Consider increasing timeout values or optimizing agent performance.".to_string());
            }
        }
        
        // Check for provider availability issues
        if let Some(provider_rate) = self.error_rates.get(&ErrorCategory::ProviderUnavailable) {
            if *provider_rate > 0.05 {
                recommendations.push("Provider availability issues detected. Check provider health and consider implementing retry logic.".to_string());
            }
        }
        
        // Check for network errors
        if let Some(network_rate) = self.error_rates.get(&ErrorCategory::NetworkError) {
            if *network_rate > 0.05 {
                recommendations.push("Network error rate is high. Check network connectivity and consider implementing circuit breakers.".to_string());
            }
        }
        
        // Check for configuration errors
        if let Some(config_rate) = self.error_rates.get(&ErrorCategory::ConfigurationError) {
            if *config_rate > 0.02 {
                recommendations.push("Configuration errors detected. Review and validate configuration files.".to_string());
            }
        }
        
        // Check for authentication errors
        if let Some(auth_rate) = self.error_rates.get(&ErrorCategory::AuthenticationError) {
            if *auth_rate > 0.01 {
                recommendations.push("Authentication errors detected. Check API keys and authentication configuration.".to_string());
            }
        }
        
        // Check for rate limiting
        if let Some(rate_limit_rate) = self.error_rates.get(&ErrorCategory::RateLimitExceeded) {
            if *rate_limit_rate > 0.05 {
                recommendations.push("Rate limiting issues detected. Consider implementing backoff strategies or reducing request frequency.".to_string());
            }
        }
        
        if recommendations.is_empty() {
            recommendations.push("Error rates are within acceptable limits. Continue monitoring.".to_string());
        }
        
        recommendations
    }

    /// Calculate health score based on error rates
    fn calculate_health_score(&self) -> f64 {
        let mut health_score = 100.0;
        
        // Deduct points for high error rates
        let total_error_rate = self.get_total_error_rate();
        if total_error_rate > 0.1 {
            health_score -= 50.0;
        } else if total_error_rate > 0.05 {
            health_score -= 25.0;
        } else if total_error_rate > 0.01 {
            health_score -= 10.0;
        }
        
        // Deduct points for specific error categories
        for (category, rate) in &self.error_rates {
            let deduction = match category {
                ErrorCategory::Timeout => rate * 20.0,
                ErrorCategory::ProviderUnavailable => rate * 30.0,
                ErrorCategory::NetworkError => rate * 25.0,
                ErrorCategory::AuthenticationError => rate * 40.0,
                ErrorCategory::ConfigurationError => rate * 35.0,
                _ => rate * 10.0,
            };
            health_score -= deduction;
        }
        
        // Deduct points for consecutive errors
        if let Some(consecutive_count) = self.get_consecutive_error_count() {
            if consecutive_count > 5 {
                health_score -= 20.0;
            } else if consecutive_count > 3 {
                health_score -= 10.0;
            }
        }
        
        health_score.max(0.0)
    }

    /// Update alert thresholds
    pub fn update_alert_thresholds(&mut self, thresholds: ErrorAlertThresholds) {
        self.alert_thresholds = thresholds;
    }

    /// Clear all error data
    pub fn clear_all_data(&mut self) {
        self.error_counts.clear();
        self.error_rates.clear();
        self.recent_errors.clear();
        self.error_trends.clear();
    }
}

/// Error alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorAlert {
    HighErrorRate {
        current_rate: f64,
        threshold: f64,
    },
    HighCategoryErrorRate {
        category: ErrorCategory,
        current_rate: f64,
        threshold: f64,
    },
    ConsecutiveErrors {
        count: usize,
        threshold: usize,
    },
}

impl Default for ErrorAlertThresholds {
    fn default() -> Self {
        Self {
            max_error_rate: 0.05, // 5%
            max_timeout_rate: 0.1, // 10%
            max_provider_unavailable_rate: 0.05, // 5%
            max_network_error_rate: 0.05, // 5%
            max_consecutive_errors: 5,
            alert_window_minutes: 60, // 1 hour
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_tracker_creation() {
        let tracker = ErrorTracker::new("test-project".to_string());
        assert_eq!(tracker.project_id, "test-project");
        assert!(tracker.error_counts.is_empty());
        assert!(tracker.recent_errors.is_empty());
    }

    #[test]
    fn test_record_error() {
        let mut tracker = ErrorTracker::new("test-project".to_string());
        
        let error = BroadcastError {
            broadcast_id: "broadcast-123".to_string(),
            agent_id: "agent1".to_string(),
            role: "developer".to_string(),
            provider: "claude".to_string(),
            error_category: ErrorCategory::Timeout,
            error_message: "Request timed out".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: 5000,
            retry_count: 0,
        };
        
        let alerts = tracker.record_error(error);
        
        assert_eq!(tracker.error_counts.get(&ErrorCategory::Timeout), Some(&1));
        assert_eq!(tracker.recent_errors.len(), 1);
        assert!(alerts.is_empty()); // Should not trigger alerts for single error
    }

    #[test]
    fn test_error_rate_calculation() {
        let mut tracker = ErrorTracker::new("test-project".to_string());
        
        // Record multiple errors
        for i in 0..5 {
            let error = BroadcastError {
                broadcast_id: format!("broadcast-{}", i),
                agent_id: format!("agent{}", i),
                role: "developer".to_string(),
                provider: "claude".to_string(),
                error_category: ErrorCategory::Timeout,
                error_message: "Request timed out".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                duration_ms: 5000,
                retry_count: 0,
            };
            tracker.record_error(error);
        }
        
        let error_rate = tracker.get_error_rate(&ErrorCategory::Timeout);
        assert!(error_rate > 0.0);
    }

    #[test]
    fn test_high_error_rate_alert() {
        let mut tracker = ErrorTracker::new("test-project".to_string());
        
        // Set low threshold to trigger alert
        tracker.alert_thresholds.max_error_rate = 0.01; // 1%
        
        // Record many errors to exceed threshold
        for i in 0..20 {
            let error = BroadcastError {
                broadcast_id: format!("broadcast-{}", i),
                agent_id: format!("agent{}", i),
                role: "developer".to_string(),
                provider: "claude".to_string(),
                error_category: ErrorCategory::Timeout,
                error_message: "Request timed out".to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                duration_ms: 5000,
                retry_count: 0,
            };
            tracker.record_error(error);
        }
        
        let alerts = tracker.check_error_alerts();
        assert!(!alerts.is_empty());
    }

    #[test]
    fn test_analysis_report_generation() {
        let mut tracker = ErrorTracker::new("test-project".to_string());
        
        // Record some errors
        let error = BroadcastError {
            broadcast_id: "broadcast-123".to_string(),
            agent_id: "agent1".to_string(),
            role: "developer".to_string(),
            provider: "claude".to_string(),
            error_category: ErrorCategory::Timeout,
            error_message: "Request timed out".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: 5000,
            retry_count: 0,
        };
        tracker.record_error(error);
        
        let report = tracker.generate_analysis_report();
        
        assert_eq!(report.project_id, "test-project");
        assert_eq!(report.total_errors, 1);
        assert!(report.health_score > 0.0);
        assert!(!report.recommendations.is_empty());
    }
}
