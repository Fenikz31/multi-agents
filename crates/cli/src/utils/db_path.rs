//! Database path resolution utilities
//! 
//! Provides centralized database path resolution following best practices:
//! 1. MULTI_AGENTS_DB (explicit override)
//! 2. MULTI_AGENTS_HOME (app-specific home)
//! 3. XDG_DATA_HOME (Linux standard)
//! 4. $HOME/.local/share (XDG fallback)
//! 5. ./data (development fallback)

use std::path::Path;

/// Resolve database path following best practices (env → XDG → HOME → dev fallback)
/// 
/// Priority order:
/// 1. MULTI_AGENTS_DB (explicit override)
/// 2. MULTI_AGENTS_HOME/multi-agents.sqlite3 (app-specific home)
/// 3. XDG_DATA_HOME/multi-agents/multi-agents.sqlite3 (Linux standard)
/// 4. $HOME/.local/share/multi-agents/multi-agents.sqlite3 (XDG fallback)
/// 5. ./data/multi-agents.sqlite3 (development fallback)
pub fn resolve_db_path() -> String {
    // 1) Hard override via explicit DB path
    if let Ok(p) = std::env::var("MULTI_AGENTS_DB") {
        return p;
    }

    // 2) Base home override for the app
    if let Ok(home) = std::env::var("MULTI_AGENTS_HOME") {
        let path = format!("{}/multi-agents.sqlite3", home.trim_end_matches('/'));
        ensure_parent_dir(&path);
        return path;
    }

    // 3) XDG data home (Linux standard)
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        let base = format!("{}/multi-agents", xdg.trim_end_matches('/'));
        let path = format!("{}/multi-agents.sqlite3", base);
        std::fs::create_dir_all(&base).ok();
        return path;
    }

    // 4) HOME/.local/share as default XDG-like
    if let Ok(home) = std::env::var("HOME") {
        let base = format!("{}/.local/share/multi-agents", home.trim_end_matches('/'));
        let path = format!("{}/multi-agents.sqlite3", base);
        std::fs::create_dir_all(&base).ok();
        return path;
    }

    // 5) Dev fallback (repo local)
    let fallback = "./data/multi-agents.sqlite3".to_string();
    ensure_parent_dir(&fallback);
    fallback
}

/// Resolve config directory path following best practices
/// 
/// Priority order:
/// 1. MULTI_AGENTS_CONFIG_DIR (explicit override)
/// 2. MULTI_AGENTS_HOME (app-specific home)
/// 3. XDG_CONFIG_HOME/multi-agents (Linux standard)
/// 4. $HOME/.config/multi-agents (XDG fallback)
/// 5. ./config (development fallback)
pub fn resolve_config_dir() -> String {
    // 1) Hard override via explicit config dir
    if let Ok(p) = std::env::var("MULTI_AGENTS_CONFIG_DIR") {
        return p;
    }

    // 2) Base home override for the app
    if let Ok(home) = std::env::var("MULTI_AGENTS_HOME") {
        let path = format!("{}/config", home.trim_end_matches('/'));
        std::fs::create_dir_all(&path).ok();
        return path;
    }

    // 3) XDG config home (Linux standard)
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let path = format!("{}/multi-agents", xdg.trim_end_matches('/'));
        std::fs::create_dir_all(&path).ok();
        return path;
    }

    // 4) HOME/.config as default XDG-like
    if let Ok(home) = std::env::var("HOME") {
        let path = format!("{}/.config/multi-agents", home.trim_end_matches('/'));
        std::fs::create_dir_all(&path).ok();
        return path;
    }

    // 5) Dev fallback (repo local)
    let fallback = "./config".to_string();
    std::fs::create_dir_all(&fallback).ok();
    fallback
}

/// Resolve logs directory path following best practices
/// 
/// Priority order:
/// 1. MULTI_AGENTS_LOGS_DIR (explicit override)
/// 2. MULTI_AGENTS_HOME/logs (app-specific home)
/// 3. XDG_DATA_HOME/multi-agents/logs (Linux standard)
/// 4. $HOME/.local/share/multi-agents/logs (XDG fallback)
/// 5. ./logs (development fallback)
pub fn resolve_logs_dir() -> String {
    // 1) Hard override via explicit logs dir
    if let Ok(p) = std::env::var("MULTI_AGENTS_LOGS_DIR") {
        return p;
    }

    // 2) Base home override for the app
    if let Ok(home) = std::env::var("MULTI_AGENTS_HOME") {
        let path = format!("{}/logs", home.trim_end_matches('/'));
        std::fs::create_dir_all(&path).ok();
        return path;
    }

    // 3) XDG data home (Linux standard)
    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        let path = format!("{}/multi-agents/logs", xdg.trim_end_matches('/'));
        std::fs::create_dir_all(&path).ok();
        return path;
    }

    // 4) HOME/.local/share as default XDG-like
    if let Ok(home) = std::env::var("HOME") {
        let path = format!("{}/.local/share/multi-agents/logs", home.trim_end_matches('/'));
        std::fs::create_dir_all(&path).ok();
        return path;
    }

    // 5) Dev fallback (repo local)
    let fallback = "./logs".to_string();
    std::fs::create_dir_all(&fallback).ok();
    fallback
}

/// Ensure parent directory exists for a given path
fn ensure_parent_dir(path: &str) {
    if let Some(parent) = Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_resolve_db_path_override() {
        env::set_var("MULTI_AGENTS_DB", "/custom/db.sqlite3");
        let path = resolve_db_path();
        assert_eq!(path, "/custom/db.sqlite3");
        env::remove_var("MULTI_AGENTS_DB");
    }

    #[test]
    fn test_resolve_db_path_home() {
        env::set_var("MULTI_AGENTS_HOME", "/home/user/.multi-agents");
        let path = resolve_db_path();
        assert_eq!(path, "/home/user/.multi-agents/multi-agents.sqlite3");
        env::remove_var("MULTI_AGENTS_HOME");
    }

    #[test]
    fn test_resolve_db_path_xdg() {
        env::set_var("XDG_DATA_HOME", "/home/user/.local/share");
        let path = resolve_db_path();
        assert_eq!(path, "/home/user/.local/share/multi-agents/multi-agents.sqlite3");
        env::remove_var("XDG_DATA_HOME");
    }

    #[test]
    fn test_resolve_db_path_home_fallback() {
        env::set_var("HOME", "/home/user");
        env::remove_var("MULTI_AGENTS_DB");
        env::remove_var("MULTI_AGENTS_HOME");
        env::remove_var("XDG_DATA_HOME");
        let path = resolve_db_path();
        assert_eq!(path, "/home/user/.local/share/multi-agents/multi-agents.sqlite3");
    }

    #[test]
    fn test_resolve_config_dir() {
        env::set_var("MULTI_AGENTS_CONFIG_DIR", "/custom/config");
        let path = resolve_config_dir();
        assert_eq!(path, "/custom/config");
        env::remove_var("MULTI_AGENTS_CONFIG_DIR");
    }

    #[test]
    fn test_resolve_logs_dir() {
        env::set_var("MULTI_AGENTS_LOGS_DIR", "/custom/logs");
        let path = resolve_logs_dir();
        assert_eq!(path, "/custom/logs");
        env::remove_var("MULTI_AGENTS_LOGS_DIR");
    }
}
