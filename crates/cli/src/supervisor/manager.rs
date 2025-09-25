//! Supervisor manager: subscriptions to system logs/events

use std::collections::HashMap;
use crate::supervisor::subscription::Subscription;

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
}


