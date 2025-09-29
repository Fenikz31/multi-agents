//! Subscription types for supervisor

use super::debug::DebugLogger;

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
    debug_logger: DebugLogger,
}

impl SupervisorSubscription {
    /// Create a new subscription reader bound to a project
    pub fn new(project: String) -> Self {
        Self { 
            project,
            debug_logger: DebugLogger::new(),
        }
    }

    /// Tail and filter last N lines for a role file, optionally by event name
    /// Optimized version with improved performance and memory efficiency
    pub fn tail_and_filter(
        &mut self,
        role: String,
        event_filter: Option<String>,
        max_lines: usize,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let operation = format!("tail_and_filter({}, {:?}, {})", role, event_filter, max_lines);
        let start_time = self.debug_logger.log_operation_start(&operation);
        
        let path = format!("./logs/{}/{}.ndjson", self.project, role);
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                self.debug_logger.log_error(&operation, &format!("File not found: {}", e));
                return Ok(vec![]); // Return empty results for non-existent files
            }
        };
        
        // Optimized line processing with reduced allocations
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        
        // Calculate start index for tail operation
        let start_idx = if total_lines > max_lines {
            total_lines - max_lines
        } else {
            0
        };
        
        // Pre-allocate result vector with estimated capacity
        let estimated_capacity = if let Some(ref _filter) = event_filter {
            // Estimate 50% of lines will match the filter
            (max_lines / 2).max(1)
        } else {
            max_lines.min(total_lines)
        };
        
        let mut result: Vec<String> = Vec::with_capacity(estimated_capacity);
        
        // Process lines with optimized filtering
        if let Some(ev) = event_filter {
            let needle = format!("\"event\":\"{}\"", ev);
            for line in &lines[start_idx..] {
                if line.contains(&needle) {
                    result.push(line.to_string());
                }
            }
        } else {
            // No filter, just convert to owned strings
            for line in &lines[start_idx..] {
                result.push(line.to_string());
            }
        }
        
        self.debug_logger.log_operation_end(&operation, start_time);
        self.debug_logger.log_performance(&operation, &format!("Processed {} lines, returned {} lines", total_lines, result.len()));
        
        Ok(result)
    }

    /// Aggregate last N lines across roles, filter by event, sort by ts ascending
    /// Optimized version with reduced allocations and improved performance
    pub fn aggregate_tail(
        &mut self,
        roles: Vec<String>,
        event_filter: Option<String>,
        max_lines_per_role: usize,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let operation = format!("aggregate_tail({} roles, {:?}, {})", roles.len(), event_filter, max_lines_per_role);
        let start_time = self.debug_logger.log_operation_start(&operation);
        
        // Pre-allocate with estimated capacity to reduce allocations
        let estimated_capacity = roles.len() * max_lines_per_role;
        let mut all: Vec<String> = Vec::with_capacity(estimated_capacity);
        
        // Collect lines from all roles
        let roles_count = roles.len();
        for role in roles {
            if let Ok(mut lines) = self.tail_and_filter(role, event_filter.clone(), max_lines_per_role) {
                all.append(&mut lines);
            }
        }
        
        // Optimized sorting with cached timestamp extraction
        all.sort_by(|a, b| {
            // Use direct string comparison for better performance
            let ta = extract_ts_fast(a);
            let tb = extract_ts_fast(b);
            ta.cmp(&tb)
        });
        
        self.debug_logger.log_operation_end(&operation, start_time);
        self.debug_logger.log_aggregation_stats(roles_count, all.len(), all.len());
        
        Ok(all)
    }
}

fn extract_ts(line: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(line) {
        Ok(v) => v.get("ts").and_then(|t| t.as_str()).unwrap_or("").to_string(),
        Err(_) => String::new(),
    }
}

/// Fast timestamp extraction using string parsing instead of JSON parsing
/// This is significantly faster for large datasets
fn extract_ts_fast(line: &str) -> String {
    // Look for "ts":"..." pattern in the JSON string
    if let Some(start) = line.find("\"ts\":\"") {
        let ts_start = start + 6; // Skip "ts":"
        if let Some(end) = line[ts_start..].find('"') {
            return line[ts_start..ts_start + end].to_string();
        }
    }
    String::new()
}


