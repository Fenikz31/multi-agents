use clap::{Parser, Subcommand, ValueEnum};
use config_model::{
    parse_project_yaml, parse_providers_yaml, validate_project_config, validate_providers_config,
};
use db::{open_or_create_db, insert_project, insert_agent, find_project_id, IdOrName};
use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(name = "multi-agents", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Configuration commands
    Config {
        #[command(subcommand)]
        cmd: ConfigCmd,
    },
    /// Environment checks (CLIs, flags, timeouts)
    Doctor {
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
        /// Optional: path to NDJSON sample to self-check parsing
        #[arg(long, value_name = "PATH")]
        ndjson_sample: Option<String>,
        /// Optional: write JSON snapshot of detected capabilities to file
        #[arg(long, value_name = "PATH")]
        snapshot: Option<String>,
    },
    /// Database commands
    Db {
        #[command(subcommand)]
        cmd: DbCmd,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCmd {
    /// Validate configuration files (YAML schemas + semantic rules)
    Validate {
        #[arg(long, value_name = "PATH")] project_file: String,
        #[arg(long, value_name = "PATH")] providers_file: String,
        #[arg(long, value_enum, default_value_t = Format::Text)] format: Format,
    },
}

#[derive(Subcommand, Debug)]
enum DbCmd {
    /// Initialize the SQLite database (idempotent)
    Init {
        #[arg(long, value_name = "PATH")]
        db_path: Option<String>,
    },
    /// Add a new project
    ProjectAdd {
        #[arg(long)] name: String,
        #[arg(long, value_name = "PATH")] db_path: Option<String>,
    },
    /// Add a new agent to a project
    AgentAdd {
        /// Project id or name
        #[arg(long)] project: String,
        #[arg(long)] name: String,
        #[arg(long)] role: String,
        #[arg(long)] provider: String,
        #[arg(long)] model: String,
        /// Repeatable flag for allowed tools
        #[arg(long = "allowed-tool")] allowed_tool: Vec<String>,
        #[arg(long = "system-prompt")] system_prompt: String,
        #[arg(long, value_name = "PATH")] db_path: Option<String>,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Format { Text, Json }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let cli = Cli::parse();
    match cli.cmd {
        Commands::Config { cmd } => match cmd {
            ConfigCmd::Validate { project_file, providers_file, format } => {
                run_config_validate(&project_file, &providers_file, format)
            }
        },
        Commands::Doctor { format, ndjson_sample, snapshot } => run_doctor(format, ndjson_sample.as_deref(), snapshot.as_deref()),
        Commands::Db { cmd } => match cmd {
            DbCmd::Init { db_path } => run_db_init(db_path.as_deref()),
            DbCmd::ProjectAdd { name, db_path } => run_project_add(&name, db_path.as_deref()),
            DbCmd::AgentAdd { project, name, role, provider, model, allowed_tool, system_prompt, db_path } =>
                run_agent_add(&project, &name, &role, &provider, &model, &allowed_tool, &system_prompt, db_path.as_deref()),
        },
    }
}

fn run_config_validate(project_path: &str, providers_path: &str, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let proj_s = fs::read_to_string(project_path)?;
    let prov_s = fs::read_to_string(providers_path)?;

    let project = match parse_project_yaml(&proj_s) {
        Ok(p) => p,
        Err(e) => return exit_with(2, format_error(format, "project", &e)),
    };
    let providers = match parse_providers_yaml(&prov_s) {
        Ok(p) => p,
        Err(e) => return exit_with(2, format_error(format, "providers", &e)),
    };

    if let Err(e) = validate_providers_config(&providers) {
        return exit_with(2, format_error(format, "providers", &e));
    }
    if let Err(e) = validate_project_config(&project, &providers) {
        return exit_with(2, format_error(format, "project", &e));
    }

    match format {
        Format::Text => println!("OK: configuration valid"),
        Format::Json => println!("{}", serde_json::json!({"status":"ok"})),
    }
    Ok(())
}

fn format_error(format: Format, which: &str, err: &impl std::fmt::Display) -> String {
    match format {
        Format::Text => format!("{}: {}", which, err),
        Format::Json => serde_json::json!({"status":"error","scope":which,"error":err.to_string()}).to_string(),
    }
}

fn exit_with<T>(code: i32, msg: String) -> Result<T, Box<dyn std::error::Error>> {
    eprintln!("{}", msg);
    std::process::exit(code);
}

// ---- db commands ----

fn default_db_path() -> String { "./data/multi-agents.sqlite3".into() }

fn run_db_init(db_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let binding;
    let path = match db_path { Some(p) => p, None => { binding = default_db_path(); &binding } };
    match open_or_create_db(path) {
        Ok(_) => { println!("OK: db initialized"); Ok(()) }
        Err(e) => exit_with(7, format!("db: {}", e)),
    }
}

fn run_project_add(name: &str, db_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let binding;
    let path = match db_path { Some(p) => p, None => { binding = default_db_path(); &binding } };
    let conn = match open_or_create_db(path) { Ok(c) => c, Err(e) => return exit_with(7, format!("db: {}", e)) };
    match insert_project(&conn, name) {
        Ok(p) => { println!("project_id={} name={}", p.id, p.name); Ok(()) }
        Err(db::DbError::InvalidInput(e)) => exit_with(2, format!("project: {}", e)),
        Err(e) => exit_with(7, format!("project: {}", e)),
    }
}

fn run_agent_add(project_sel: &str, name: &str, role: &str, provider: &str, model: &str, allowed_tool: &[String], system_prompt: &str, db_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let binding;
    let path = match db_path { Some(p) => p, None => { binding = default_db_path(); &binding } };
    let conn = match open_or_create_db(path) { Ok(c) => c, Err(e) => return exit_with(7, format!("db: {}", e)) };
    let project_id = match find_project_id(&conn, if looks_like_uuid(project_sel) { IdOrName::Id(project_sel) } else { IdOrName::Name(project_sel) })? {
        Some(id) => id,
        None => return exit_with(2, format!("project not found: {}", project_sel)),
    };
    match insert_agent(&conn, &project_id, name, role, provider, model, allowed_tool, system_prompt) {
        Ok(a) => { println!("agent_id={} project_id={} name={}", a.id, a.project_id, a.name); Ok(()) }
        Err(db::DbError::InvalidInput(e)) => exit_with(2, format!("agent: {}", e)),
        Err(e) => exit_with(7, format!("agent: {}", e)),
    }
}

fn looks_like_uuid(s: &str) -> bool { s.len() >= 16 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-') }

// ---- doctor implementation ----

const DEFAULT_TIMEOUT_PER_PROVIDER_MS: u64 = 2000; // docs/specs/errors-and-timeouts.md
const DEFAULT_TIMEOUT_GLOBAL_MS: u64 = 10000;

#[derive(Debug, Clone)]
struct ProbeResult {
    name: String,
    present: bool,
    version: Option<String>,
    supports: BTreeMap<String, bool>,
    timed_out: bool,
    error: Option<String>,
}

fn run_with_timeout(bin: &str, args: &[&str], timeout: Duration) -> Result<(i32, String, String), String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
            let mut out = String::new();
            let mut err = String::new();
            if let Some(mut so) = child.stdout.take() {
                let _ = so.read_to_string(&mut out);
            }
            if let Some(mut se) = child.stderr.take() {
                let _ = se.read_to_string(&mut err);
            }
            let code = status.code().unwrap_or(-1);
            return Ok((code, out, err));
        }
        if start.elapsed() >= timeout {
            // best-effort kill
            let _ = child.kill();
            return Err("timeout".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

fn probe_help(bin: &str, help_args: &[&str], timeout_ms: u64) -> Result<String, String> {
    match run_with_timeout(bin, help_args, Duration::from_millis(timeout_ms)) {
        Ok((_code, out, err)) => {
            // Some CLIs print help to stderr
            let text = if !out.trim().is_empty() { out } else { err };
            Ok(text)
        }
        Err(e) => Err(e),
    }
}

fn probe_version(bin: &str, candidates: &[&[&str]], timeout_ms: u64) -> Option<String> {
    for args in candidates {
        if let Ok((_code, out, err)) = run_with_timeout(bin, args, Duration::from_millis(timeout_ms)) {
            let text = if !out.trim().is_empty() { out } else { err };
            let line = text.lines().next().unwrap_or("").trim().to_string();
            if !line.is_empty() {
                return Some(line);
            }
        }
    }
    None
}

fn parse_gemini_supports(help: &str) -> BTreeMap<String, bool> {
    let mut supports = BTreeMap::new();
    supports.insert("allowed_tools".into(), help.contains("--allowed-tools"));
    supports.insert("interactive".into(), help.contains("-i") || help.to_lowercase().contains("interactive"));
    supports
}

fn probe_gemini(timeout_ms: u64) -> ProbeResult {
    let mut timed_out = false;
    let mut error = None;
    let help = match probe_help("gemini", &["--help"], timeout_ms) {
        Ok(h) => h,
        Err(e) => {
            if e == "timeout" { timed_out = true; }
            error = Some(e);
            return ProbeResult { name: "gemini".into(), present: false, version: None, supports: BTreeMap::new(), timed_out, error };
        }
    };
    let supports = parse_gemini_supports(&help);
    let version = probe_version("gemini", &[&["--version"], &["version"], &["-v"]], timeout_ms);
    ProbeResult { name: "gemini".into(), present: true, version, supports, timed_out, error }
}

fn parse_claude_supports(help: &str) -> BTreeMap<String, bool> {
    let mut supports = BTreeMap::new();
    supports.insert("output_format".into(), help.contains("--output-format"));
    supports.insert("session_id".into(), help.contains("--session-id"));
    supports.insert("allowed_tools".into(), help.contains("--allowed-tools"));
    supports.insert("permission_mode".into(), help.contains("--permission-mode"));
    supports.insert("resume".into(), help.contains("-r") || help.contains("--resume"));
    supports
}

fn probe_claude(timeout_ms: u64) -> ProbeResult {
    let mut timed_out = false;
    let mut error = None;
    let help = match probe_help("claude", &["--help"], timeout_ms) {
        Ok(h) => h,
        Err(e) => {
            if e == "timeout" { timed_out = true; }
            error = Some(e);
            return ProbeResult { name: "claude".into(), present: false, version: None, supports: BTreeMap::new(), timed_out, error };
        }
    };
    let supports = parse_claude_supports(&help);
    let version = probe_version("claude", &[&["--version"], &["version"], &["-v"]], timeout_ms);
    ProbeResult { name: "claude".into(), present: true, version, supports, timed_out, error }
}

fn parse_cursor_supports(help: &str) -> BTreeMap<String, bool> {
    let mut supports = BTreeMap::new();
    supports.insert("print".into(), help.contains("-p"));
    supports.insert("output_format".into(), help.contains("--output-format"));
    supports.insert("create_chat".into(), help.contains("create-chat"));
    supports.insert("resume".into(), help.contains("--resume"));
    supports
}

fn probe_cursor(timeout_ms: u64) -> ProbeResult {
    let mut timed_out = false;
    let mut error = None;
    let help = match probe_help("cursor-agent", &["--help"], timeout_ms) {
        Ok(h) => h,
        Err(e) => {
            if e == "timeout" { timed_out = true; }
            error = Some(e);
            return ProbeResult { name: "cursor-agent".into(), present: false, version: None, supports: BTreeMap::new(), timed_out, error };
        }
    };
    let supports = parse_cursor_supports(&help);
    let version = probe_version("cursor-agent", &[&["--version"], &["version"], &["-v"]], timeout_ms);
    ProbeResult { name: "cursor-agent".into(), present: true, version, supports, timed_out, error }
}

fn parse_tmux_list_commands(list_cmds: &str) -> BTreeMap<String, bool> {
    let mut supports = BTreeMap::new();
    supports.insert("pipe_pane".into(), list_cmds.contains("pipe-pane"));
    supports
}

fn probe_tmux(timeout_ms: u64) -> ProbeResult {
    let mut timed_out = false;
    let mut error = None;
    let version = probe_version("tmux", &[&["-V"], &["--version"]], timeout_ms);
    if version.is_none() {
        // Not present or failed
        // Attempt to see if binary exists via help, else mark not present
        match probe_help("tmux", &["-h"], timeout_ms) {
            Ok(_) => {},
            Err(e) => {
                if e == "timeout" { timed_out = true; }
                error = Some(e);
                return ProbeResult { name: "tmux".into(), present: false, version: None, supports: BTreeMap::new(), timed_out, error };
            }
        }
    }
    // pipe-pane support via list-commands
    let list = probe_help("tmux", &["list-commands"], timeout_ms).unwrap_or_default();
    let supports = parse_tmux_list_commands(&list);
    ProbeResult { name: "tmux".into(), present: true, version, supports, timed_out, error }
}

fn probe_git(timeout_ms: u64) -> ProbeResult {
    let supports = BTreeMap::new();
    let mut timed_out = false;
    let mut error = None;
    let version = probe_version("git", &[&["--version"], &["version"]], timeout_ms);
    if version.is_none() {
        match probe_help("git", &["--help"], timeout_ms) {
            Ok(_) => {},
            Err(e) => {
                if e == "timeout" { timed_out = true; }
                error = Some(e);
                return ProbeResult { name: "git".into(), present: false, version: None, supports: BTreeMap::new(), timed_out, error };
            }
        }
    }
    ProbeResult { name: "git".into(), present: true, version, supports, timed_out, error }
}

#[allow(dead_code)]
fn extract_version_line(text: &str) -> Option<String> {
    let line = text.lines().next().unwrap_or("").trim();
    if line.is_empty() { None } else { Some(line.to_string()) }
}

fn run_doctor(format: Format, ndjson_sample: Option<&str>, snapshot_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let per_timeout = DEFAULT_TIMEOUT_PER_PROVIDER_MS;
    let _global_timeout = DEFAULT_TIMEOUT_GLOBAL_MS; // reserved for future aggregation

    let results = vec![
        probe_gemini(per_timeout),
        probe_claude(per_timeout),
        probe_cursor(per_timeout),
        probe_tmux(per_timeout),
        probe_git(per_timeout),
    ];

    // Derive status and worst error code according to spec
    let mut any_timeout = false;
    let mut any_missing = false;
    let mut degraded = false;

    for r in &results {
        if r.timed_out { any_timeout = true; }
        if !r.present { any_missing = true; }
    }

    // Degraded if provider present but a key flag appears missing (heuristic)
    for r in &results {
        if r.present {
            match r.name.as_str() {
                "claude" => {
                    // require output_format and session_id for OK
                    if !r.supports.get("output_format").copied().unwrap_or(false) || !r.supports.get("session_id").copied().unwrap_or(false) {
                        degraded = true;
                    }
                }
                "cursor-agent" => {
                    if !r.supports.get("resume").copied().unwrap_or(false) || !r.supports.get("create_chat").copied().unwrap_or(false) {
                        degraded = true;
                    }
                }
                "gemini" => {
                    if !r.supports.get("interactive").copied().unwrap_or(false) {
                        degraded = true;
                    }
                }
                "tmux" | "git" => {}
                _ => {}
            }
        }
    }

    let status_text = if any_missing {
        "KO"
    } else if any_timeout || degraded {
        "DEGRADE"
    } else {
        "OK"
    };

    // NDJSON self-check if requested
    let mut ndjson_report: Option<serde_json::Value> = None;
    let mut ndjson_invalid = false;
    if let Some(path) = ndjson_sample {
        match ndjson_self_check(path) {
            Ok(report) => {
                ndjson_invalid = report.get("errors").and_then(|e| e.as_array()).map(|a| !a.is_empty()).unwrap_or(false);
                ndjson_report = Some(report);
            }
            Err(e) => return exit_with(2, format!("ndjson: {}", e)),
        }
    }

    // Build JSON root for snapshot/printing
    let root_json = build_doctor_json(status_text, &results, ndjson_report.clone());

    // Write snapshot if requested (even if status is KO/DEGRADE)
    if let Some(path) = snapshot_path {
        let parent = std::path::Path::new(path).parent();
        if let Some(dir) = parent { if !dir.as_os_str().is_empty() { let _ = std::fs::create_dir_all(dir); } }
        std::fs::write(path, serde_json::to_vec_pretty(&root_json)?)?;
    }

    match format {
        Format::Text => {
            println!("doctor: {}", status_text);
            for r in &results {
                let ver = r.version.clone().unwrap_or_else(|| "(unknown)".into());
                let mut feats: Vec<String> = r
                    .supports
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, if *v { "true" } else { "false" }))
                    .collect();
                feats.sort();
                println!(
                    "- {}: present={} version={}{}{}",
                    r.name,
                    if r.present { "true" } else { "false" },
                    ver,
                    if feats.is_empty() { "".into() } else { format!(" supports: {}", feats.join(", ")) },
                    if r.timed_out { " (timeout)" } else { "" }
                );
            }
            if let Some(rep) = ndjson_report {
                println!("ndjson: {}", rep);
            }
        }
        Format::Json => {
            println!("{}", root_json);
        }
    }

    // Exit codes: 0 OK; 2 invalid input (ndjson invalid); 3 provider unavailable; 5 timeout; 1 degraded
    if ndjson_invalid {
        return exit_with(2, "doctor: ndjson sample invalid".into());
    }
    if any_missing {
        return exit_with(3, "doctor: missing required providers".into());
    }
    if any_timeout {
        return exit_with(5, "doctor: timed out while probing providers".into());
    }
    if degraded {
        return exit_with(1, "doctor: environment degraded (missing key flags)".into());
    }
    Ok(())
}

fn build_doctor_json(status_text: &str, results: &Vec<ProbeResult>, ndjson_report: Option<Value>) -> Value {
    let arr: Vec<_> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "present": r.present,
                "version": r.version,
                "supports": r.supports,
                "timed_out": r.timed_out,
                "error": r.error,
            })
        })
        .collect();
    let mut root = serde_json::json!({
        "status": status_text,
        "results": arr
    });
    if let Some(rep) = ndjson_report {
        if let Some(obj) = root.as_object_mut() {
            obj.insert("ndjson".into(), rep);
        }
    }
    root
}

fn has_ansi(s: &str) -> bool {
    // Quick heuristic: ESC [ ... m  (CSI SGR)
    s.contains("\u{1b}[")
}

fn ndjson_self_check(path: &str) -> Result<Value, String> {
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut line_idx: usize = 0;
    let mut errors: Vec<Value> = Vec::new();
    let mut ok_count: usize = 0;

    for line_res in reader.lines() {
        line_idx += 1;
        let line = line_res.map_err(|e| e.to_string())?;
        if line.trim().is_empty() { continue; }
        if has_ansi(&line) {
            errors.push(serde_json::json!({"line": line_idx, "error": "ansi_codes_forbidden"}));
            continue;
        }
        let v: Value = match serde_json::from_str(&line) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn write_tmp(contents: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("multi-agents-test-{}.ndjson", uuid_like()));
        let mut f = File::create(&p).expect("create temp file");
        f.write_all(contents.as_bytes()).expect("write temp file");
        p.to_string_lossy().to_string()
    }

    fn uuid_like() -> String {
        // simple unique-ish string using nanos timestamp
        format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())
    }

    #[test]
    fn ndjson_ok_single_line() {
        let line = r#"{"ts":"2025-09-15T14:03:21.123Z","project_id":"demo","agent_role":"backend","provider":"gemini","session_id":"s1","direction":"agent","event":"stdout_line"}"#;
        let path = write_tmp(&format!("{}\n", line));
        let rep = ndjson_self_check(&path).expect("self check");
        assert_eq!(rep["errors"].as_array().unwrap().len(), 0);
        assert_eq!(rep["ok_lines"].as_u64().unwrap(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ndjson_detects_invalid_and_missing_fields() {
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
    fn ndjson_detects_ansi() {
        let ansi = "\u{1b}[31mred\u{1b}[0m\n"; // will not be valid JSON and also ANSI
        let path = write_tmp(ansi);
        let rep = ndjson_self_check(&path).expect("self check");
        let errs = rep["errors"].as_array().unwrap();
        assert!(errs.iter().any(|e| e["error"] == "ansi_codes_forbidden"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn parse_supports_from_help_texts() {
        let claude_help = "--output-format --session-id --allowed-tools --permission-mode -r";
        let s = parse_claude_supports(claude_help);
        assert!(s.get("output_format").copied().unwrap_or(false));
        assert!(s.get("session_id").copied().unwrap_or(false));
        assert!(s.get("allowed_tools").copied().unwrap_or(false));
        assert!(s.get("permission_mode").copied().unwrap_or(false));
        assert!(s.get("resume").copied().unwrap_or(false));

        let cursor_help = "-p --output-format create-chat --resume";
        let s2 = parse_cursor_supports(cursor_help);
        assert!(s2.get("print").copied().unwrap_or(false));
        assert!(s2.get("output_format").copied().unwrap_or(false));
        assert!(s2.get("create_chat").copied().unwrap_or(false));
        assert!(s2.get("resume").copied().unwrap_or(false));

        let gemini_help = "-i something --allowed-tools";
        let s3 = parse_gemini_supports(gemini_help);
        assert!(s3.get("interactive").copied().unwrap_or(false));
        assert!(s3.get("allowed_tools").copied().unwrap_or(false));

        let list_cmds = "list-commands\npipe-pane\nresize-pane";
        let s4 = parse_tmux_list_commands(list_cmds);
        assert!(s4.get("pipe_pane").copied().unwrap_or(false));
    }
}
