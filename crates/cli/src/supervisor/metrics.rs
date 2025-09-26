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
/// Optimized version with improved performance and reduced allocations
pub fn compute_routed_metrics(lines: Vec<String>) -> Result<RoutedMetrics, Box<dyn std::error::Error>> {
    use std::collections::{HashMap, HashSet};
    
    // Pre-allocate collections with estimated capacity
    let estimated_events = lines.len();
    let mut per_role: HashMap<String, usize> = HashMap::with_capacity(estimated_events / 10);
    let mut broadcasts: HashSet<String> = HashSet::with_capacity(estimated_events / 20);
    let mut per_broadcast_timestamps: HashMap<String, Vec<String>> = HashMap::with_capacity(estimated_events / 20);
    let mut total: usize = 0;

    // Optimized parsing with early filtering
    for line in lines {
        // Fast check for routed events using string search instead of JSON parsing
        if !line.contains("\"event\":\"routed\"") {
            continue;
        }
        
        // Parse JSON only for routed events
        let v: serde_json::Value = serde_json::from_str(&line)?;
        
        total += 1;
        
        // Extract role with optimized string handling
        if let Some(role) = v.get("agent_role").and_then(|r| r.as_str()) {
            *per_role.entry(role.to_string()).or_insert(0) += 1;
        }
        
        // Extract broadcast_id and timestamp
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

/// Compute routed metrics from NdjsonEvent objects
/// Optimized version with improved performance and reduced allocations
pub fn compute_routed_metrics_from_events(events: Vec<crate::logging::events::NdjsonEvent>) -> Result<RoutedMetrics, Box<dyn std::error::Error>> {
    use std::collections::{HashMap, HashSet};
    
    // Pre-allocate collections with estimated capacity
    let estimated_events = events.len();
    let mut per_role: HashMap<String, usize> = HashMap::with_capacity(estimated_events / 10);
    let mut broadcasts: HashSet<String> = HashSet::with_capacity(estimated_events / 20);
    let mut per_broadcast_timestamps: HashMap<String, Vec<String>> = HashMap::with_capacity(estimated_events / 20);
    let mut total: usize = 0;

    // Optimized processing with early filtering
    for event in events {
        if event.event == "routed" {
            total += 1;
            *per_role.entry(event.agent_role.clone()).or_default() += 1;
            
            if let Some(broadcast_id) = &event.broadcast_id {
                broadcasts.insert(broadcast_id.clone());
                per_broadcast_timestamps.entry(broadcast_id.clone()).or_default().push(event.ts.clone());
            }
        }
    }

    // Compute p95 latency per broadcast
    let mut p95_latency_per_broadcast: HashMap<String, u64> = HashMap::new();
    for (broadcast_id, timestamps) in per_broadcast_timestamps {
        if timestamps.len() > 1 {
            let mut durations: Vec<u64> = Vec::new();
            for i in 1..timestamps.len() {
                if let (Ok(ts1), Ok(ts2)) = (
                    chrono::DateTime::parse_from_rfc3339(&timestamps[i-1]),
                    chrono::DateTime::parse_from_rfc3339(&timestamps[i])
                ) {
                    let duration = ts2.signed_duration_since(ts1).num_milliseconds() as u64;
                    durations.push(duration);
                }
            }
            durations.sort();
            let p95_index = (durations.len() as f64 * 0.95) as usize;
            let p95_latency = durations.get(p95_index.min(durations.len().saturating_sub(1))).cloned().unwrap_or(0);
            p95_latency_per_broadcast.insert(broadcast_id, p95_latency);
        } else {
            p95_latency_per_broadcast.insert(broadcast_id, 0);
        }
    }

    // Sort top roles by count
    let mut top_roles: Vec<(String, usize)> = per_role.into_iter().collect();
    top_roles.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    Ok(RoutedMetrics { total, per_role: top_roles.iter().map(|(k, v)| (k.clone(), *v)).collect(), unique_broadcasts: broadcasts.len(), p95_latency_per_broadcast, top_roles })
}


