//! Subscription types for supervisor

/// Subscription request for supervisor logs/events
#[derive(Debug, Clone)]
pub struct Subscription {
    pub project: String,
    pub roles: Vec<String>,
}

/// Simple subscription reader for NDJSON logs
#[derive(Debug, Default)]
pub struct SupervisorSubscription {
    project: String,
}

impl SupervisorSubscription {
    /// Create a new subscription reader bound to a project
    pub fn new(project: String) -> Self {
        Self { project }
    }

    /// Tail and filter last N lines for a role file, optionally by event name
    pub fn tail_and_filter(
        &mut self,
        role: String,
        event_filter: Option<String>,
        max_lines: usize,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let path = format!("./logs/{}/{}.ndjson", self.project, role);
        let content = std::fs::read_to_string(&path)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        if lines.len() > max_lines {
            let start = lines.len() - max_lines;
            lines = lines.split_off(start);
        }
        if let Some(ev) = event_filter {
            let needle = format!("\"event\":\"{}\"", ev);
            lines.retain(|l| l.contains(&needle));
        }
        Ok(lines)
    }
}


