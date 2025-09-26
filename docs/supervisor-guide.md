# Supervisor Guide

## Overview

The Supervisor module in Multi-Agents CLI provides comprehensive monitoring and management capabilities for multi-agent systems. It enables real-time monitoring of agent activities, log aggregation, and metrics computation for routed events.

## SupervisorManager

The `SupervisorManager` is the central component for managing supervisor subscriptions and computing metrics.

### Key Features

- **Subscription Management**: Create and manage subscriptions to system logs and events
- **Metrics Computation**: Calculate comprehensive metrics for routed events
- **Integration Points**: Seamless integration with broadcast and logging systems

### Basic Usage

```rust
use crate::supervisor::manager::SupervisorManager;

// Create a new supervisor manager
let mut manager = SupervisorManager::new();

// Subscribe to system events
let subscription = Subscription {
    project: "my-project".to_string(),
    roles: vec!["backend".to_string(), "frontend".to_string()],
};

let subscription_id = manager.subscribe(subscription)?;
println!("Created subscription: {}", subscription_id);

// Unsubscribe when done
manager.unsubscribe(&subscription_id)?;
```

### Metrics Computation

The SupervisorManager provides two methods for computing routed metrics:

#### From NDJSON Lines

```rust
// Compute metrics from raw NDJSON log lines
let log_lines = vec![
    r#"{"ts":"2025-01-15T10:00:00.000Z","level":"info","project_id":"demo","agent_role":"backend","agent_id":"backend1","provider":"claude","event":"routed","session_id":"test-session-1","broadcast_id":"broadcast-123","message_id":"msg-456","text":"Message routed successfully","dur_ms":50}"#.to_string(),
    // ... more log lines
];

let metrics = SupervisorManager::routed_summary(log_lines)?;
println!("Total routed events: {}", metrics.total);
println!("Unique broadcasts: {}", metrics.unique_broadcasts);
```

#### From NdjsonEvent Objects

```rust
use crate::logging::events::NdjsonEvent;

// Compute metrics from parsed event objects
let events = vec![
    NdjsonEvent {
        ts: "2025-01-15T10:00:00.000Z".to_string(),
        level: "info".to_string(),
        project_id: "demo".to_string(),
        agent_role: "backend".to_string(),
        agent_id: "backend1".to_string(),
        provider: "claude".to_string(),
        event: "routed".to_string(),
        text: Some("Message routed successfully".to_string()),
        dur_ms: Some(50),
        broadcast_id: Some("broadcast-123".to_string()),
        session_id: Some("test-session-1".to_string()),
        message_id: Some("msg-456".to_string()),
    },
    // ... more events
];

let metrics = SupervisorManager::routed_summary_from_events(events)?;
```

## SupervisorSubscription

The `SupervisorSubscription` provides real-time access to agent logs with filtering and aggregation capabilities.

### Key Features

- **Log Tail and Filter**: Read and filter recent log entries by role and event type
- **Multi-Role Aggregation**: Aggregate logs from multiple roles with chronological sorting
- **Event Filtering**: Filter logs by specific event types (e.g., "routed", "start", "end")

### Basic Usage

```rust
use crate::supervisor::subscription::SupervisorSubscription;

// Create a subscription for a specific project
let mut subscription = SupervisorSubscription::new("my-project".to_string());

// Tail and filter logs for a specific role
let backend_events = subscription.tail_and_filter(
    "backend".to_string(),
    Some("routed".to_string()),
    100  // max lines
)?;

println!("Found {} routed events in backend logs", backend_events.len());

// Aggregate logs from multiple roles
let all_events = subscription.aggregate_tail(
    vec!["backend".to_string(), "frontend".to_string(), "devops".to_string()],
    Some("routed".to_string()),
    50  // max lines per role
)?;

println!("Aggregated {} events from all roles", all_events.len());
```

### Advanced Filtering

```rust
// Filter by specific event types
let start_events = subscription.tail_and_filter(
    "backend".to_string(),
    Some("start".to_string()),
    100
)?;

let end_events = subscription.tail_and_filter(
    "backend".to_string(),
    Some("end".to_string()),
    100
)?;

// Get all events (no filter)
let all_backend_events = subscription.tail_and_filter(
    "backend".to_string(),
    None,  // no event filter
    100
)?;
```

## RoutedMetrics

The `RoutedMetrics` struct provides comprehensive metrics for routed events analysis.

### Metrics Structure

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutedMetrics {
    pub total: usize,                                    // Total number of routed events
    pub per_role: HashMap<String, usize>,               // Count per role
    pub unique_broadcasts: usize,                       // Number of unique broadcast IDs
    pub p95_latency_per_broadcast: HashMap<String, u64>, // P95 latency per broadcast (ms)
    pub top_roles: Vec<(String, usize)>,                // Top roles by event count
}
```

### Metrics Interpretation

- **`total`**: Total number of routed events across all roles
- **`per_role`**: Breakdown of routed events by agent role
- **`unique_broadcasts`**: Number of unique broadcast operations
- **`p95_latency_per_broadcast`**: 95th percentile latency for each broadcast operation
- **`top_roles`**: Roles sorted by routed event count (descending)

### Usage Examples

```rust
// Compute and analyze metrics
let metrics = SupervisorManager::routed_summary(log_lines)?;

// Display summary
println!("=== Routed Events Summary ===");
println!("Total routed events: {}", metrics.total);
println!("Unique broadcasts: {}", metrics.unique_broadcasts);

// Display per-role breakdown
println!("\n=== Events per Role ===");
for (role, count) in &metrics.per_role {
    println!("{}: {} events", role, count);
}

// Display top roles
println!("\n=== Top Roles ===");
for (i, (role, count)) in metrics.top_roles.iter().enumerate() {
    println!("{}. {}: {} events", i + 1, role, count);
}

// Display latency metrics
println!("\n=== Broadcast Latency (P95) ===");
for (broadcast_id, latency_ms) in &metrics.p95_latency_per_broadcast {
    println!("{}: {}ms", broadcast_id, latency_ms);
}
```

## Routed Events

Routed events are special NDJSON log entries that track message routing operations in the multi-agent system.

### Event Structure

```json
{
  "ts": "2025-01-15T10:00:00.000Z",
  "level": "info",
  "project_id": "demo",
  "agent_role": "backend",
  "agent_id": "backend1",
  "provider": "claude",
  "event": "routed",
  "text": "Message routed successfully",
  "dur_ms": 50,
  "broadcast_id": "broadcast-123",
  "session_id": "test-session-1",
  "message_id": "msg-456"
}
```

### Key Fields

- **`event`**: Always "routed" for routed events
- **`broadcast_id`**: Unique identifier for the broadcast operation
- **`message_id`**: Unique identifier for the specific message
- **`agent_role`**: Role of the agent that received the routed message
- **`agent_id`**: Specific agent that received the message
- **`dur_ms`**: Duration of the routing operation in milliseconds
- **`ts`**: Timestamp of the routing event (RFC3339 format)

### Event Generation

Routed events are automatically generated when using the `send --to @role` or `send --to @all` commands:

```bash
# These commands generate routed events
multi-agents send --to @backend --message "Database schema updated"
multi-agents send --to @all --message "Starting deployment"
```

## Integration with CLI Commands

### Send Command Integration

The supervisor integrates seamlessly with the `send` command routing functionality:

```bash
# Send to specific role (generates routed events)
multi-agents send --to @backend --message "Review API changes"

# Send to all agents (generates routed events for all roles)
multi-agents send --to @all --message "Project status update"

# Send to specific agent (generates routed event for that agent)
multi-agents send --to backend --message "Implement new endpoint"
```

### Log File Structure

Routed events are stored in NDJSON format in the logs directory:

```
./logs/
├── {project}/
│   ├── backend.ndjson    # Backend agent routed events
│   ├── frontend.ndjson   # Frontend agent routed events
│   └── devops.ndjson     # DevOps agent routed events
```

## Examples

### Complete Workflow Example

```rust
use crate::supervisor::{manager::SupervisorManager, subscription::SupervisorSubscription};

// 1. Create supervisor manager
let mut manager = SupervisorManager::new();

// 2. Create subscription for monitoring
let mut subscription = SupervisorSubscription::new("demo-project".to_string());

// 3. Monitor routed events in real-time
let routed_events = subscription.tail_and_filter(
    "backend".to_string(),
    Some("routed".to_string()),
    100
)?;

// 4. Compute metrics
let metrics = SupervisorManager::routed_summary(routed_events)?;

// 5. Display results
println!("Backend routed {} messages in {} unique broadcasts", 
         metrics.total, metrics.unique_broadcasts);
```

### Multi-Role Monitoring Example

```rust
// Monitor multiple roles simultaneously
let all_events = subscription.aggregate_tail(
    vec!["backend".to_string(), "frontend".to_string(), "devops".to_string()],
    Some("routed".to_string()),
    50
)?;

let metrics = SupervisorManager::routed_summary(all_events)?;

// Analyze role distribution
for (role, count) in &metrics.per_role {
    let percentage = (count * 100) / metrics.total;
    println!("{}: {} events ({}%)", role, count, percentage);
}
```

### Latency Analysis Example

```rust
// Analyze broadcast latency
let metrics = SupervisorManager::routed_summary(log_lines)?;

println!("=== Broadcast Performance Analysis ===");
for (broadcast_id, latency_ms) in &metrics.p95_latency_per_broadcast {
    if *latency_ms > 1000 {
        println!("⚠️  {}: {}ms (slow)", broadcast_id, latency_ms);
    } else if *latency_ms > 500 {
        println!("⚡ {}: {}ms (moderate)", broadcast_id, latency_ms);
    } else {
        println!("✅ {}: {}ms (fast)", broadcast_id, latency_ms);
    }
}
```

## Best Practices

### 1. Efficient Log Monitoring

- Use appropriate `max_lines` limits to avoid memory issues
- Filter by specific event types when possible
- Monitor logs in real-time for immediate feedback

### 2. Metrics Analysis

- Regularly compute metrics to track system performance
- Monitor latency trends to identify performance issues
- Use role-based analysis to understand agent workload distribution

### 3. Error Handling

- Always handle potential errors from supervisor operations
- Implement retry logic for transient failures
- Log supervisor operations for debugging

### 4. Resource Management

- Unsubscribe from subscriptions when no longer needed
- Limit the number of concurrent subscriptions
- Use appropriate timeouts for long-running operations

## Troubleshooting

### Common Issues

1. **Log files not found**: Ensure the project directory exists in `./logs/`
2. **Empty metrics**: Check that routed events are being generated
3. **High latency**: Monitor system performance and network conditions
4. **Memory issues**: Reduce `max_lines` limits for large log files

### Debug Information

Enable debug logging to troubleshoot supervisor operations:

```rust
// Enable debug logging
env_logger::init();

// Monitor supervisor operations
let events = subscription.tail_and_filter(role, event_filter, max_lines)?;
println!("Retrieved {} events", events.len());
```

## API Reference

### SupervisorManager

- `new() -> SupervisorManager`: Create a new supervisor manager
- `subscribe(sub: Subscription) -> Result<String, String>`: Subscribe to system events
- `unsubscribe(id: &str) -> Result<(), String>`: Unsubscribe by ID
- `routed_summary(lines: Vec<String>) -> Result<RoutedMetrics, Box<dyn Error>>`: Compute metrics from NDJSON lines
- `routed_summary_from_events(events: Vec<NdjsonEvent>) -> Result<RoutedMetrics, Box<dyn Error>>`: Compute metrics from event objects

### SupervisorSubscription

- `new(project: String) -> SupervisorSubscription`: Create a new subscription
- `tail_and_filter(role: String, event_filter: Option<String>, max_lines: usize) -> Result<Vec<String>, Box<dyn Error>>`: Tail and filter logs
- `aggregate_tail(roles: Vec<String>, event_filter: Option<String>, max_lines_per_role: usize) -> Result<Vec<String>, Box<dyn Error>>`: Aggregate logs from multiple roles

### RoutedMetrics

- `total: usize`: Total routed events
- `per_role: HashMap<String, usize>`: Events per role
- `unique_broadcasts: usize`: Unique broadcast count
- `p95_latency_per_broadcast: HashMap<String, u64>`: P95 latency per broadcast
- `top_roles: Vec<(String, usize)>`: Top roles by event count
