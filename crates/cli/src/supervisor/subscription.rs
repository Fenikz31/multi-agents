//! Subscription types for supervisor

/// Subscription request for supervisor logs/events
#[derive(Debug, Clone)]
pub struct Subscription {
    pub project: String,
    pub roles: Vec<String>,
}


