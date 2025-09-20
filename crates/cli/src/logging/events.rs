//! NDJSON Event structures

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct NdjsonEvent {
    pub ts: String,
    pub level: String,
    pub project_id: String,
    pub agent_role: String,
    pub agent_id: String,
    pub provider: String,
    pub event: String,
    pub text: Option<String>,
    pub dur_ms: Option<u64>,
    pub broadcast_id: Option<String>,
    pub session_id: Option<String>,
}

impl NdjsonEvent {
    pub fn new_start(project_id: &str, agent_role: &str, agent_id: &str, provider: &str) -> Self {
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            project_id: project_id.to_string(),
            agent_role: agent_role.to_string(),
            agent_id: agent_id.to_string(),
            provider: provider.to_string(),
            event: "start".to_string(),
            text: None,
            dur_ms: None,
            broadcast_id: None,
            session_id: None,
        }
    }

    pub fn new_stdout_line(project_id: &str, agent_role: &str, agent_id: &str, provider: &str, text: &str) -> Self {
        // Remove ANSI escape sequences from text
        let clean_text = super::ndjson::remove_ansi_escape_sequences(text);
        
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            project_id: project_id.to_string(),
            agent_role: agent_role.to_string(),
            agent_id: agent_id.to_string(),
            provider: provider.to_string(),
            event: "stdout_line".to_string(),
            text: Some(clean_text),
            dur_ms: None,
            broadcast_id: None,
            session_id: None,
        }
    }

    pub fn new_end(project_id: &str, agent_role: &str, agent_id: &str, provider: &str, dur_ms: u64, status: &str) -> Self {
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            project_id: project_id.to_string(),
            agent_role: agent_role.to_string(),
            agent_id: agent_id.to_string(),
            provider: provider.to_string(),
            event: "end".to_string(),
            text: Some(status.to_string()),
            dur_ms: Some(dur_ms),
            broadcast_id: None,
            session_id: None,
        }
    }

    pub fn new_metrics(
        project_id: &str, 
        agent_role: &str, 
        agent_id: &str, 
        provider: &str,
        event_type: &str,
        dur_ms: u64,
        _status: &str,
        details: Option<&str>
    ) -> Self {
        let text = match details {
            Some(d) => Some(format!("{}: {}", event_type, d)),
            None => Some(event_type.to_string()),
        };
        
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            project_id: project_id.to_string(),
            agent_role: agent_role.to_string(),
            agent_id: agent_id.to_string(),
            provider: provider.to_string(),
            event: "metrics".to_string(),
            text,
            dur_ms: Some(dur_ms),
            broadcast_id: None,
            session_id: None,
        }
    }

    /// Create a categorized failure metrics event
    pub fn new_failure_metrics(
        project_id: &str,
        agent_role: &str,
        agent_id: &str,
        provider: &str,
        failure_category: &str,
        failure_type: &str,
        dur_ms: u64,
        error_details: &str
    ) -> Self {
        let text = Some(format!("{}: {} - {}", failure_category, failure_type, error_details));
        
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            level: "error".to_string(),
            project_id: project_id.to_string(),
            agent_role: agent_role.to_string(),
            agent_id: agent_id.to_string(),
            provider: provider.to_string(),
            event: "metrics".to_string(),
            text,
            dur_ms: Some(dur_ms),
            broadcast_id: None,
            session_id: None,
        }
    }

    /// Create a start event with broadcast_id for M5 preparation
    pub fn new_start_with_broadcast(
        project_id: &str, 
        agent_role: &str, 
        agent_id: &str, 
        provider: &str,
        broadcast_id: Option<&str>
    ) -> Self {
        Self {
            ts: chrono::Utc::now().to_rfc3339(),
            level: "info".to_string(),
            project_id: project_id.to_string(),
            agent_role: agent_role.to_string(),
            agent_id: agent_id.to_string(),
            provider: provider.to_string(),
            event: "start".to_string(),
            text: None,
            dur_ms: None,
            broadcast_id: broadcast_id.map(|s| s.to_string()),
            session_id: None,
        }
    }
}

// Note: remove_ansi_escape_sequences is defined in ndjson.rs to avoid duplication
