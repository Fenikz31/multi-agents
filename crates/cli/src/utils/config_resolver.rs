//! Configuration path resolution utilities

use std::path::Path;

/// Resolve config paths from (flags -> env -> defaults)
/// ENV: MULTI_AGENTS_PROJECT_FILE, MULTI_AGENTS_PROVIDERS_FILE, MULTI_AGENTS_CONFIG_DIR
pub fn resolve_config_paths(project_flag: Option<&str>, providers_flag: Option<&str>) -> Result<(String, String), String> {
    let resolve_one = |kind: &str, flag_opt: Option<&str>| -> Result<String, String> {
        // 1) explicit flag
        if let Some(p) = flag_opt { if Path::new(p).exists() { return Ok(p.to_string()); } }
        // 2) file-by-file env var
        let env_key = if kind == "project" { "MULTI_AGENTS_PROJECT_FILE" } else { "MULTI_AGENTS_PROVIDERS_FILE" };
        if let Ok(p) = std::env::var(env_key) { if Path::new(&p).exists() { return Ok(p); } }
        // 3) config dir env var or default ./config
        let base = std::env::var("MULTI_AGENTS_CONFIG_DIR").unwrap_or_else(|_| "./config".into());
        let candidates = if kind == "project" {
            vec![format!("{}/project.yaml", base), format!("{}/project.yml", base)]
        } else {
            vec![format!("{}/providers.yaml", base), format!("{}/providers.yml", base)]
        };
        for c in &candidates { if Path::new(c).exists() { return Ok(c.clone()); } }
        Err(format!(
            "{} config not found. Provide --{}-file, or set {} / MULTI_AGENTS_CONFIG_DIR. Tried: {}",
            kind,
            kind,
            env_key,
            candidates.join(", ")
        ))
    };

    let pr = resolve_one("project", project_flag)?;
    let pv = resolve_one("providers", providers_flag)?;
    Ok((pr, pv))
}

/// Check if a string looks like a UUID
pub fn looks_like_uuid(s: &str) -> bool { 
    s.len() >= 16 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-') 
}

/// Generate a short ID based on current time
pub fn short_id() -> String { 
    format!("{:x}", std::time::Instant::now().elapsed().as_nanos()) 
}

/// Generate a UUID v4-like string
pub fn uuid_v4_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut s = format!("{:032x}", nanos);
    // Set version (v4)
    s.replace_range(12..13, "4");
    // Set variant (10xx)
    let variants = ['8','9','a','b'];
    let idx = (nanos & 0x3) as usize;
    s.replace_range(16..17, &variants[idx].to_string());
    format!(
        "{}-{}-{}-{}-{}",
        &s[0..8], &s[8..12], &s[12..16], &s[16..20], &s[20..32]
    )
}
