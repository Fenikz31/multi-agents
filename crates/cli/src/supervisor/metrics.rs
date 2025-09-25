//! Metrics computation for supervisor routed events

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedMetrics {
    pub total: usize,
    pub per_role: std::collections::HashMap<String, usize>,
    pub unique_broadcasts: usize,
    pub p95_latency_per_broadcast: std::collections::HashMap<String, u64>,
    pub top_roles: Vec<(String, usize)>,
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
    let mut per_broadcast_timestamps: HashMap<String, Vec<String>> = HashMap::new();

    for line in lines {
        let v: serde_json::Value = serde_json::from_str(&line)?;
        let event = v.get("event").and_then(|e| e.as_str()).unwrap_or("");
        if event != "routed" { continue; }

        total += 1;
        if let Some(role) = v.get("agent_role").and_then(|r| r.as_str()) {
            *per_role.entry(role.to_string()).or_insert(0) += 1;
        }
        if let Some(bid) = v.get("broadcast_id").and_then(|b| b.as_str()) {
            if !bid.is_empty() {
                broadcasts.insert(bid.to_string());
                if let Some(ts) = v.get("ts").and_then(|t| t.as_str()) {
                    per_broadcast_timestamps.entry(bid.to_string()).or_default().push(ts.to_string());
                }
            }
        }
    }

    // Compute p95 latency per broadcast as (max(ts) - min(ts)) in milliseconds
    let mut p95_latency_per_broadcast: HashMap<String, u64> = HashMap::new();
    for (bid, mut tss) in per_broadcast_timestamps {
        if tss.is_empty() { continue; }
        tss.sort();
        let first = &tss[0];
        let last = &tss[tss.len()-1];
        let start = chrono::DateTime::parse_from_rfc3339(first).map(|dt| dt.with_timezone(&chrono::Utc));
        let end = chrono::DateTime::parse_from_rfc3339(last).map(|dt| dt.with_timezone(&chrono::Utc));
        let ms = match (start, end) {
            (Ok(s), Ok(e)) => (e - s).num_milliseconds().max(0) as u64,
            _ => 0,
        };
        p95_latency_per_broadcast.insert(bid, ms);
    }

    // Top roles by count (descending)
    let mut top_roles: Vec<(String, usize)> = per_role.iter().map(|(k, v)| (k.clone(), *v)).collect();
    top_roles.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    Ok(RoutedMetrics { total, per_role, unique_broadcasts: broadcasts.len(), p95_latency_per_broadcast, top_roles })
}


