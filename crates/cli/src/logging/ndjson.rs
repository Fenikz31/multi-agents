//! NDJSON logging utilities

use std::fs;
use std::io::Write;
use db::now_iso8601_utc;
use super::events::NdjsonEvent;

/// Write NDJSON event to log file with enhanced error handling
pub fn write_ndjson_event(log_file: &str, event: &NdjsonEvent) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure directory exists with permission check
    if let Some(parent) = std::path::Path::new(log_file).parent() {
        match std::fs::create_dir_all(parent) {
            Ok(_) => {},
            Err(e) => {
                return Err(format!("Failed to create log directory '{}': {}. Try using --logs-dir option or --no-logs to disable logging", parent.display(), e).into());
            }
        }
    }
    
    // Test write permissions before attempting to write
    let test_file = format!("{}.test", log_file);
    if let Err(e) = std::fs::write(&test_file, "test") {
        let _ = std::fs::remove_file(&test_file);
        return Err(format!("No write permission to log directory '{}': {}. Try using --logs-dir option or --no-logs to disable logging", 
                          std::path::Path::new(log_file).parent().unwrap_or(std::path::Path::new(".")).display(), e).into());
    }
    let _ = std::fs::remove_file(&test_file);
    
    // Write event as single line JSON
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;
    
    writeln!(file, "{}", serde_json::to_string(event)?)?;
    Ok(())
}

/// Log NDJSON event with standard format
pub fn log_ndjson(
    project: &str, 
    agent_role: &str, 
    provider: &str, 
    session_id: Option<&str>, 
    direction: &str, 
    event: &str, 
    text: Option<&str>, 
    exit_code: Option<i32>, 
    ts_opt: Option<&str>
) {
    let ts = ts_opt.map(|s| s.to_string()).unwrap_or_else(|| now_iso8601_utc());
    let obj = serde_json::json!({
        "ts": ts,
        "project_id": project,
        "agent_role": agent_role,
        "provider": provider,
        "session_id": session_id.unwrap_or("") ,
        "direction": direction,
        "event": event,
        "text": text,
        "exit_code": exit_code,
    });
    let dir = format!("./logs/{project}");
    let _ = fs::create_dir_all(&dir);
    let path = format!("{}/{}.ndjson", dir, agent_role);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(&mut f, "{}", obj);
    }
}

/// Emit NDJSON start event for agent (contract compliant)
pub fn emit_start_event(project_name: &str, role: &str, agent_name: &str, provider: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_file = format!("./logs/{}/{}.ndjson", project_name, role);
    let event = NdjsonEvent::new_start(project_name, role, agent_name, provider);
    write_ndjson_event(&log_file, &event)
}

/// Emit NDJSON end event for agent (contract compliant)
pub fn emit_end_event(project_name: &str, role: &str, agent_name: &str, provider: &str, status: &str, duration_ms: u64) -> Result<(), Box<dyn std::error::Error>> {
    let log_file = format!("./logs/{}/{}.ndjson", project_name, role);
    let event = NdjsonEvent::new_end(project_name, role, agent_name, provider, duration_ms, status);
    write_ndjson_event(&log_file, &event)
}

/// Emit NDJSON stdout_line event for agent (contract compliant)
pub fn emit_stdout_line_event(project_name: &str, role: &str, agent_name: &str, provider: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let log_file = format!("./logs/{}/{}.ndjson", project_name, role);
    let event = NdjsonEvent::new_stdout_line(project_name, role, agent_name, provider, text);
    write_ndjson_event(&log_file, &event)
}

/// Check if a string has ANSI escape sequences
pub fn has_ansi(s: &str) -> bool {
    // Quick heuristic: ESC [ ... m  (CSI SGR)
    s.contains("\u{1b}[")
}

/// Remove ANSI escape sequences from text
pub fn remove_ansi_escape_sequences(text: &str) -> String {
    // Remove common ANSI escape sequences
    let mut result = text.to_string();
    
    // Remove CSI sequences (ESC [ ... m)
    result = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap().replace_all(&result, "").to_string();
    
    // Remove other common escape sequences
    result = regex::Regex::new(r"\x1b\[[0-9;]*[A-Za-z]").unwrap().replace_all(&result, "").to_string();
    result = regex::Regex::new(r"\x1b\]0;[^\x07]*\x07").unwrap().replace_all(&result, "").to_string(); // OSC sequences
    
    result
}

/// Limit line length to prevent log bloat
pub fn limit_line_length(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}... [truncated {} chars]", &text[..max_length-20], text.len() - max_length + 20)
    }
}

/// Clean text for NDJSON logging (remove ANSI, limit length)
pub fn clean_text_for_logging(text: &str, max_length: usize) -> String {
    let cleaned = remove_ansi_escape_sequences(text);
    limit_line_length(&cleaned, max_length)
}

/// Emit metrics event for NDJSON logging
pub fn emit_metrics_event(
    project_name: &str,
    role: &str,
    agent_name: &str,
    provider: &str,
    event_type: &str,
    duration_ms: u64,
    status: &str,
    details: Option<&str>
) -> Result<(), Box<dyn std::error::Error>> {
    let log_file = format!("./logs/{}/{}.ndjson", project_name, role);
    let event = NdjsonEvent::new_metrics(project_name, role, agent_name, provider, event_type, duration_ms, status, details);
    write_ndjson_event(&log_file, &event)
}

/// Emit failure metrics event for NDJSON logging
pub fn emit_failure_metrics_event(
    project_name: &str,
    role: &str,
    agent_name: &str,
    provider: &str,
    failure_category: &str,
    failure_type: &str,
    duration_ms: u64,
    error_details: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let log_file = format!("./logs/{}/{}.ndjson", project_name, role);
    let event = NdjsonEvent::new_failure_metrics(project_name, role, agent_name, provider, failure_category, failure_type, duration_ms, error_details);
    write_ndjson_event(&log_file, &event)
}

/// Self-check NDJSON file for validity
pub fn ndjson_self_check(path: &str) -> Result<serde_json::Value, String> {
    use std::io::BufRead;
    use std::fs::File;
    use std::io::BufReader;
    
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut line_idx: usize = 0;
    let mut errors: Vec<serde_json::Value> = Vec::new();
    let mut ok_count: usize = 0;

    for line_res in reader.lines() {
        line_idx += 1;
        let line = line_res.map_err(|e| e.to_string())?;
        if line.trim().is_empty() { continue; }
        if has_ansi(&line) {
            errors.push(serde_json::json!({"line": line_idx, "error": "ansi_codes_forbidden"}));
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                errors.push(serde_json::json!({"line": line_idx, "error": "invalid_json", "detail": e.to_string()}));
                continue;
            }
        };
        // Required fields
        let req = [
            "ts","project_id","agent_role","provider","session_id","direction","event"
        ];
        let obj = match v.as_object() {
            Some(o) => o,
            None => {
                errors.push(serde_json::json!({"line": line_idx, "error": "not_an_object"}));
                continue;
            }
        };
        for k in req {
            if !obj.contains_key(k) {
                errors.push(serde_json::json!({"line": line_idx, "error": "missing_field", "field": k}));
            }
        }
        if errors.last().map(|e| e["line"].as_u64().unwrap_or(0) == line_idx as u64).unwrap_or(false) {
            // had errors for this line
        } else {
            ok_count += 1;
        }
    }

    Ok(serde_json::json!({
        "ok_lines": ok_count,
        "errors": errors,
    }))
}
