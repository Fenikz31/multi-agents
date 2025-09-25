//! Unit tests for logging

use crate::logging::*;
use std::fs::File;
use std::io::Write;

#[cfg(test)]
mod tests {
    use super::*;

    fn write_tmp(contents: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("multi-agents-test-{}.ndjson", uuid_v4_like()));
        let mut f = File::create(&p).expect("create temp file");
        f.write_all(contents.as_bytes()).expect("write temp file");
        p.to_string_lossy().to_string()
    }

    fn uuid_v4_like() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        format!("{:x}", nanos)
    }

    #[test]
    fn test_ndjson_ok_single_line() {
        let line = r#"{"ts":"2025-09-15T14:03:21.123Z","project_id":"demo","agent_role":"backend","provider":"gemini","session_id":"s1","direction":"agent","event":"stdout_line"}"#;
        let path = write_tmp(&format!("{}\n", line));
        let rep = ndjson_self_check(&path).expect("self check");
        assert_eq!(rep["errors"].as_array().unwrap().len(), 0);
        assert_eq!(rep["ok_lines"].as_u64().unwrap(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_ndjson_detects_invalid_and_missing_fields() {
        let invalid = "not json\n";
        let missing = r#"{"ts":"2025-09-15T14:03:21.123Z","project_id":"demo","agent_role":"backend","provider":"gemini","session_id":"s1","direction":"agent"}"#; // missing event
        let path = write_tmp(&format!("{}{}\n", invalid, missing));
        let rep = ndjson_self_check(&path).expect("self check");
        let errs = rep["errors"].as_array().unwrap();
        assert!(errs.iter().any(|e| e["error"] == "invalid_json"));
        assert!(errs.iter().any(|e| e["error"] == "missing_field" && e["field"] == "event"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_ndjson_detects_ansi() {
        let ansi = "\u{1b}[31mred\u{1b}[0m\n"; // will not be valid JSON and also ANSI
        let path = write_tmp(ansi);
        let rep = ndjson_self_check(&path).expect("self check");
        let errs = rep["errors"].as_array().unwrap();
        assert!(errs.iter().any(|e| e["error"] == "ansi_codes_forbidden"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_has_ansi() {
        assert!(has_ansi("\u{1b}[31mred\u{1b}[0m"));
        assert!(has_ansi("normal \u{1b}[32mgreen\u{1b}[0m text"));
        assert!(!has_ansi("normal text"));
        assert!(!has_ansi(""));
    }

    #[test]
    fn test_remove_ansi_escape_sequences() {
        use crate::logging::ndjson::remove_ansi_escape_sequences;
        
        let ansi_text = "\u{1b}[31mred\u{1b}[0m text \u{1b}[32mgreen\u{1b}[0m";
        let clean_text = remove_ansi_escape_sequences(ansi_text);
        assert_eq!(clean_text, "red text green");
        
        let normal_text = "normal text";
        let clean_normal = remove_ansi_escape_sequences(normal_text);
        assert_eq!(clean_normal, "normal text");
    }

    #[test]
    fn test_ndjson_event_creation() {
        let start_event = NdjsonEvent::new_start("demo", "backend", "backend-agent", "claude");
        assert_eq!(start_event.event, "start");
        assert_eq!(start_event.project_id, "demo");
        assert_eq!(start_event.agent_role, "backend");
        assert_eq!(start_event.provider, "claude");
        assert!(start_event.text.is_none());
        assert!(start_event.dur_ms.is_none());

        let stdout_event = NdjsonEvent::new_stdout_line("demo", "backend", "backend-agent", "claude", "Hello World");
        assert_eq!(stdout_event.event, "stdout_line");
        assert_eq!(stdout_event.text, Some("Hello World".to_string()));

        let end_event = NdjsonEvent::new_end("demo", "backend", "backend-agent", "claude", 1500, "success");
        assert_eq!(end_event.event, "end");
        assert_eq!(end_event.dur_ms, Some(1500));
        assert_eq!(end_event.text, Some("success".to_string()));
    }

    #[test]
    fn test_ndjson_event_routed_contains_ids() {
        // Expect constructor NdjsonEvent::new_routed(project, role, agent, provider, broadcast_id, message_id)
        let ev = crate::logging::events::NdjsonEvent::new_routed(
            "demo",
            "backend",
            "backend1",
            "claude",
            Some("b-123".to_string()),
            Some("m-456".to_string()),
        );
        assert_eq!(ev.event, "routed");
        assert_eq!(ev.project_id, "demo");
        assert_eq!(ev.agent_role, "backend");
        assert_eq!(ev.agent_id, "backend1");
        assert_eq!(ev.provider, "claude");
        assert_eq!(ev.broadcast_id.as_deref(), Some("b-123"));
        assert_eq!(ev.message_id.as_deref(), Some("m-456"));
    }

    #[test]
    fn test_emit_routed_event_writes_line() {
        // Expect helper emit_routed_event(project, role, agent, provider, broadcast_id, message_id)
        let tmp = tempfile::tempdir().unwrap();
        let log_dir = tmp.path().join("logs/demo");
        std::fs::create_dir_all(&log_dir).unwrap();
        // Temporarily override logs dir via env or by calling the function that writes to ./logs/{project}
        // We simulate by running and then checking the file exists
        let res = crate::logging::ndjson::emit_routed_event(
            "demo",
            "backend",
            "backend1",
            "claude",
            Some("b-123"),
            Some("m-456"),
        );
        assert!(res.is_ok());

        let path = format!("./logs/{}/{}.ndjson", "demo", "backend");
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("\"event\":\"routed\""));
        assert!(content.contains("b-123"));
        assert!(content.contains("m-456"));
    }
}
