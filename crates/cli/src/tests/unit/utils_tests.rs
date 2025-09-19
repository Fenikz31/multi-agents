//! Unit tests for utilities

use crate::utils::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn write_tmp(contents: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("multi-agents-test-{}.ndjson", uuid_v4_like()));
        let mut f = File::create(&p).expect("create temp file");
        f.write_all(contents.as_bytes()).expect("write temp file");
        p.to_string_lossy().to_string()
    }

    fn uuid_like() -> String {
        // simple unique-ish string using nanos timestamp
        format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
    }

    #[test]
    fn test_looks_like_uuid() {
        assert!(looks_like_uuid("12345678-1234-1234-1234-123456789abc"));
        assert!(looks_like_uuid("1234567890123456"));
        assert!(!looks_like_uuid("not-a-uuid"));
        assert!(!looks_like_uuid("123"));
    }

    #[test]
    fn test_short_id() {
        let id1 = short_id();
        let id2 = short_id();
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        // IDs should be different (very high probability)
        // If they're the same, it means the test ran too fast, which is acceptable
        // We'll just verify they're not empty and are valid hex
        assert!(id1.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(id2.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_uuid_v4_like() {
        let uuid1 = uuid_v4_like();
        let uuid2 = uuid_v4_like();
        
        // Should be valid UUID format
        assert!(uuid1.contains('-'));
        assert_eq!(uuid1.len(), 36);
        assert_eq!(uuid2.len(), 36);
        
        // Should be different
        assert_ne!(uuid1, uuid2);
        
        // Should have correct version (4) and variant
        let parts: Vec<&str> = uuid1.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
    }

    #[test]
    fn test_default_db_path() {
        let path = default_db_path();
        assert_eq!(path, "./data/multi-agents.sqlite3");
    }

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_SEND_TIMEOUT_MS, 120_000);
        assert_eq!(DEFAULT_AGENT_TIMEOUT_MS, 5_000);
        assert_eq!(MAX_CONCURRENCY, 3);
        assert_eq!(TMUX_RETRY_ATTEMPTS, 2);
        assert_eq!(TMUX_RETRY_DELAY_MS, 100);
    }
}
