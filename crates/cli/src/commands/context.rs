//! Context commands implementation
use crate::cli::commands::{Format, GitKind};
use crate::utils::constants::DEFAULT_TIMEOUT_GLOBAL_MS;
use std::process::Command;
use std::time::{Duration, Instant};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_function() {
        // Test fonction de redaction
        let input = "Email: user@example.com and token=abc123 and Authorization: Bearer xyz789";
        let redacted = redact_sensitive(input);
        
        assert!(redacted.contains("[redacted:email]"));
        assert!(redacted.contains("[redacted:token]"));
        assert!(redacted.contains("[redacted]"));
        assert!(!redacted.contains("user@example.com"));
        assert!(!redacted.contains("abc123"));
        assert!(!redacted.contains("xyz789"));
    }

    #[test]
    fn test_redaction_multiple_emails() {
        // Test redaction multiple emails
        let input = "Contact: user1@example.com and user2@test.org";
        let redacted = redact_sensitive(input);
        
        assert!(redacted.contains("[redacted:email]"));
        assert!(!redacted.contains("user1@example.com"));
        assert!(!redacted.contains("user2@test.org"));
    }

    #[test]
    fn test_redaction_bearer_token() {
        // Test redaction Bearer token
        let input = "Authorization: Bearer sk-1234567890abcdef";
        let redacted = redact_sensitive(input);
        
        assert!(redacted.contains("Authorization: Bearer [redacted:token]"));
        assert!(!redacted.contains("sk-1234567890abcdef"));
    }
}

/// Run the `context git` subcommand
pub fn run_context_git(
    kind: GitKind,
    format: Format,
    max_bytes: Option<usize>,
    max_lines: Option<usize>,
    pathspec: Option<&str>,
    no_color: bool,
    strict: bool,
    staged: bool,
    since: Option<&str>,
    until: Option<&str>,
    limit: Option<usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Timeout budget: use global default from constants (20s)
    let timeout = Duration::from_millis(DEFAULT_TIMEOUT_GLOBAL_MS);
    let started = Instant::now();

    // Verify git availability
    let git_ok = Command::new("git").arg("--version").output();
    if git_ok.is_err() {
        // exit code 3 (provider unavailable)
        std::process::exit(3);
    }

    // Detect repository
    let repo_chk = Command::new("git").arg("rev-parse").arg("--is-inside-work-tree").output()?;
    let inside_repo = repo_chk.status.success() && String::from_utf8_lossy(&repo_chk.stdout).trim() == "true";
    if !inside_repo {
        if strict {
            // exit code 1 generic
            eprintln!("No Git repository detected (strict mode)");
            std::process::exit(1);
        } else {
            match format {
                Format::Text => {
                    println!("No Git repository detected");
                }
                Format::Json => {
                    println!("{}", r#"{"kind":"none","truncated":false,"notes":["no_git_repo"]}"#);
                }
            }
            return Ok(());
        }
    }

    // Build git command
    let mut cmd = Command::new("git");
    match kind {
        GitKind::Status => {
            cmd.arg("status").arg("--porcelain=v2").arg("--branch");
        }
        GitKind::Diff => {
            cmd.arg("diff");
            if staged { cmd.arg("--cached"); }
            if no_color { cmd.arg("--no-color"); }
        }
        GitKind::Log => {
            cmd.arg("log");
            // Pretty compact one-line JSON-friendly
            cmd.arg("--pretty=format:%h %ad %s").arg("--date=iso-strict");
            let lim = limit.unwrap_or(5);
            cmd.arg(format!("-n{}", lim));
            if let Some(s) = since { cmd.arg(format!("--since={}", s)); }
            if let Some(u) = until { cmd.arg(format!("--until={}", u)); }
        }
    }
    if let Some(ps) = pathspec { if !ps.trim().is_empty() { cmd.arg("--").arg(ps); } }

    // Execute with coarse timeout control
    let output = cmd.output()?;
    if started.elapsed() > timeout {
        // exit code 5 timeout
        std::process::exit(5);
    }

    let mut content = String::from_utf8_lossy(if output.status.success() { &output.stdout } else { &output.stderr }).to_string();

    // Normalize/cleanup for diff: replace binary lines with placeholder
    if matches!(kind, GitKind::Diff) {
        let mut rebuilt = String::with_capacity(content.len());
        for line in content.lines() {
            if line.starts_with("Binary files ") || line.contains("GIT binary patch") {
                rebuilt.push_str("[[binary content omitted]]\n");
            } else {
                rebuilt.push_str(line);
                rebuilt.push('\n');
            }
        }
        content = rebuilt;
    }

    // Redaction: emails and simple token patterns
    content = redact_sensitive(&content);

    // Truncation by lines
    let mut truncated = false;
    if let Some(maxl) = max_lines { 
        let mut lines: Vec<&str> = content.lines().collect();
        if lines.len() > maxl { 
            lines.truncate(maxl);
            content = lines.join("\n");
            truncated = true;
        }
    }
    // Truncation by bytes
    if let Some(maxb) = max_bytes { 
        if content.as_bytes().len() > maxb {
            content.truncate(maxb);
            truncated = true;
        }
    }

    match format {
        Format::Text => {
            if truncated { println!("{}\n(truncated)", content); } else { println!("{}", content); }
        }
        Format::Json => {
            // minimal JSON wrapper
            let kind_str = match kind { GitKind::Status => "status", GitKind::Diff => "diff", GitKind::Log => "log" };
            let escaped = content.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
            println!("{{\"kind\":\"{}\",\"truncated\":{},\"bytes\":{},\"lines\":{},\"data\":\"{}\"}}",
                kind_str,
                if truncated { "true" } else { "false" },
                content.as_bytes().len(),
                content.lines().count(),
                escaped,
            );
        }
    }

    Ok(())
}

pub fn redact_sensitive(input: &str) -> String {
    // Redact emails
    let email_re = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b").unwrap();
    let mut out = email_re.replace_all(input, "[redacted:email]").to_string();
    // Redact Authorization: Bearer <token>
    let bearer_re = regex::Regex::new(r"(?i)(Authorization:\s*Bearer)\s+[A-Za-z0-9._\-]+").unwrap();
    out = bearer_re.replace_all(&out, "$1 [redacted:token]").to_string();
    // Redact token=... in URLs/query strings
    let token_param_re = regex::Regex::new(r"(?i)(token=)[^&\s]+").unwrap();
    out = token_param_re.replace_all(&out, "$1[redacted]").to_string();
    out
}


