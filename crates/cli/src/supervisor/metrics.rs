//! Metrics computation for supervisor routed events

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedMetrics {
    pub total: usize,
    pub per_role: std::collections::HashMap<String, usize>,
    pub unique_broadcasts: usize,
}

/// Compute routed metrics from NDJSON lines
/// - total number of routed events
/// - count per role
/// - unique broadcast_id count
pub fn compute_routed_metrics(lines: Vec<String>) -> Result<RoutedMetrics, Box<dyn std::error::Error>> {
    use std::collections::{HashMap, HashSet};
    let mut total: usize = 0;
    let mut per_role: HashMap<String, usize> = HashMap::new();
    let mut broadcasts: HashSet<String> = HashSet::new();

    for line in lines {
        let v: serde_json::Value = serde_json::from_str(&line)?;
        let event = v.get("event").and_then(|e| e.as_str()).unwrap_or("");
        if event != "routed" { continue; }

        total += 1;
        if let Some(role) = v.get("agent_role").and_then(|r| r.as_str()) {
            *per_role.entry(role.to_string()).or_insert(0) += 1;
        }
        if let Some(bid) = v.get("broadcast_id").and_then(|b| b.as_str()) {
            if !bid.is_empty() { broadcasts.insert(bid.to_string()); }
        }
    }

    Ok(RoutedMetrics { total, per_role, unique_broadcasts: broadcasts.len() })
}


