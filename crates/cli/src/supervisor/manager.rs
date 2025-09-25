//! Supervisor manager: subscriptions to system logs/events

use std::collections::HashMap;
use crate::supervisor::subscription::Subscription;
use crate::supervisor::metrics::{self, RoutedMetrics};

/// Simple in-memory supervisor manager for subscriptions
pub struct SupervisorManager {
    subscriptions: HashMap<String, Subscription>,
}

impl SupervisorManager {
    /// Create a new supervisor manager
    pub fn new() -> Self {
        Self { subscriptions: HashMap::new() }
    }

    /// Subscribe to system logs/events and return subscription id
    pub fn subscribe(&mut self, sub: Subscription) -> Result<String, String> {
        let id = crate::utils::uuid_v4_like();
        self.subscriptions.insert(id.clone(), sub);
        Ok(id)
    }

    /// Unsubscribe by id (idempotent)
    pub fn unsubscribe(&mut self, id: &str) -> Result<(), String> {
        let _ = self.subscriptions.remove(id);
        Ok(())
    }

    /// Compute a routed summary from NDJSON lines (integration point for broadcast→supervisor)
    pub fn routed_summary(lines: Vec<String>) -> Result<RoutedMetrics, Box<dyn std::error::Error>> {
        metrics::compute_routed_metrics(lines)
    }

    /// Compute a routed summary from NdjsonEvent objects (integration point for broadcast→supervisor)
    pub fn routed_summary_from_events(events: Vec<crate::logging::events::NdjsonEvent>) -> Result<RoutedMetrics, Box<dyn std::error::Error>> {
        metrics::compute_routed_metrics_from_events(events)
    }
}


