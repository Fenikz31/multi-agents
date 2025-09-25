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

    /// Aggregate last N lines across roles, filter by event, sort by ts ascending
    pub fn aggregate_tail(
        &mut self,
        roles: Vec<String>,
        event_filter: Option<String>,
        max_lines_per_role: usize,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut all: Vec<String> = Vec::new();
        for role in roles {
            if let Ok(mut lines) = self.tail_and_filter(role, event_filter.clone(), max_lines_per_role) {
                all.append(&mut lines);
            }
        }
        // Sort by ts (RFC3339) ascending
        all.sort_by(|a, b| {
            let ta = extract_ts(a);
            let tb = extract_ts(b);
            ta.cmp(&tb)
        });
        Ok(all)
    }
}

fn extract_ts(line: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(line) {
        Ok(v) => v.get("ts").and_then(|t| t.as_str()).unwrap_or("").to_string(),
        Err(_) => String::new(),
    }
}


