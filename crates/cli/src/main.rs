use clap::{Parser, Subcommand, ValueEnum};
use config_model::{
    parse_project_yaml, parse_providers_yaml, validate_project_config, validate_providers_config,
};
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
        Commands::Doctor { format, ndjson_sample } => run_doctor(format, ndjson_sample.as_deref()),
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

fn probe_gemini(timeout_ms: u64) -> ProbeResult {
    let mut supports = BTreeMap::new();
    let mut timed_out = false;
    let mut error = None;
    // help
    let help = match probe_help("gemini", &["--help"], timeout_ms) {
        Ok(h) => h,
        Err(e) => {
            if e == "timeout" { timed_out = true; }
            error = Some(e);
            return ProbeResult { name: "gemini".into(), present: false, version: None, supports, timed_out, error };
        }
    };
    supports.insert("allowed_tools".into(), help.contains("--allowed-tools"));
    supports.insert("interactive".into(), help.contains("-i") || help.to_lowercase().contains("interactive"));
    let version = probe_version("gemini", &[&["--version"], &["version"], &["-v"]], timeout_ms);
    ProbeResult { name: "gemini".into(), present: true, version, supports, timed_out, error }
}

fn probe_claude(timeout_ms: u64) -> ProbeResult {
    let mut supports = BTreeMap::new();
    let mut timed_out = false;
    let mut error = None;
    let help = match probe_help("claude", &["--help"], timeout_ms) {
        Ok(h) => h,
        Err(e) => {
            if e == "timeout" { timed_out = true; }
            error = Some(e);
            return ProbeResult { name: "claude".into(), present: false, version: None, supports, timed_out, error };
        }
    };
    supports.insert("output_format".into(), help.contains("--output-format"));
    supports.insert("session_id".into(), help.contains("--session-id"));
    supports.insert("allowed_tools".into(), help.contains("--allowed-tools"));
    supports.insert("permission_mode".into(), help.contains("--permission-mode"));
    supports.insert("resume".into(), help.contains("-r") || help.contains("--resume"));
    let version = probe_version("claude", &[&["--version"], &["version"], &["-v"]], timeout_ms);
    ProbeResult { name: "claude".into(), present: true, version, supports, timed_out, error }
}

fn probe_cursor(timeout_ms: u64) -> ProbeResult {
    let mut supports = BTreeMap::new();
    let mut timed_out = false;
    let mut error = None;
    let help = match probe_help("cursor-agent", &["--help"], timeout_ms) {
        Ok(h) => h,
        Err(e) => {
            if e == "timeout" { timed_out = true; }
            error = Some(e);
            return ProbeResult { name: "cursor-agent".into(), present: false, version: None, supports, timed_out, error };
        }
    };
    supports.insert("print".into(), help.contains("-p"));
    supports.insert("output_format".into(), help.contains("--output-format"));
    supports.insert("create_chat".into(), help.contains("create-chat"));
    supports.insert("resume".into(), help.contains("--resume"));
    let version = probe_version("cursor-agent", &[&["--version"], &["version"], &["-v"]], timeout_ms);
    ProbeResult { name: "cursor-agent".into(), present: true, version, supports, timed_out, error }
}

fn probe_tmux(timeout_ms: u64) -> ProbeResult {
    let mut supports = BTreeMap::new();
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
                return ProbeResult { name: "tmux".into(), present: false, version: None, supports, timed_out, error };
            }
        }
    }
    // pipe-pane support via list-commands
    let list = probe_help("tmux", &["list-commands"], timeout_ms).unwrap_or_default();
    supports.insert("pipe_pane".into(), list.contains("pipe-pane"));
    ProbeResult { name: "tmux".into(), present: true, version, supports, timed_out, error }
}

fn probe_git(timeout_ms: u64) -> ProbeResult {
    let mut supports = BTreeMap::new();
    let mut timed_out = false;
    let mut error = None;
    let version = probe_version("git", &[&["--version"], &["version"]], timeout_ms);
    if version.is_none() {
        match probe_help("git", &["--help"], timeout_ms) {
            Ok(_) => {},
            Err(e) => {
                if e == "timeout" { timed_out = true; }
                error = Some(e);
                return ProbeResult { name: "git".into(), present: false, version: None, supports, timed_out, error };
            }
        }
    }
    ProbeResult { name: "git".into(), present: true, version, supports, timed_out, error }
}

fn run_doctor(format: Format, ndjson_sample: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let per_timeout = DEFAULT_TIMEOUT_PER_PROVIDER_MS;
    let _global_timeout = DEFAULT_TIMEOUT_GLOBAL_MS; // reserved for future aggregation

    let mut results = vec![
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
    if let Some(path) = ndjson_sample {
        match ndjson_self_check(path) {
            Ok(report) => ndjson_report = Some(report),
            Err(e) => return exit_with(2, format!("ndjson: {}", e)),
        }
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
            let arr: Vec<_> = results
                .into_iter()
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
            println!("{}", root);
        }
    }

    // Exit codes: 0 OK; 3 provider unavailable; 5 timeout; 1 degraded
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
