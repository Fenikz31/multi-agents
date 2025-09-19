//! Doctor command implementation

use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use config_model::parse_providers_yaml;
use crate::cli::commands::Format;
use crate::utils::{resolve_config_paths, DEFAULT_TIMEOUT_PER_PROVIDER_MS, DEFAULT_TIMEOUT_GLOBAL_MS, exit_with};
use crate::utils::timeouts::run_with_timeout;
use crate::logging::ndjson_self_check;

/// Probe result structure
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub name: String,
    pub present: bool,
    pub version: Option<String>,
    pub supports: BTreeMap<String, bool>,
    pub timed_out: bool,
    pub error: Option<String>,
}

/// Run doctor command
pub fn run_doctor(format: Format, ndjson_sample: Option<&str>, snapshot_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let per_timeout = DEFAULT_TIMEOUT_PER_PROVIDER_MS;
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::with_template("{spinner} doctor").unwrap());
    pb.enable_steady_tick(Duration::from_millis(120));
    let global_cap: u64 = DEFAULT_TIMEOUT_GLOBAL_MS; // 20s global cap

    // Try to read providers.yaml to get cmd/help/version args; fallback to built-in probes
    let mut results: Vec<ProbeResult> = Vec::new();
    let providers_cfg = resolve_config_paths(None, None)
        .ok()
        .and_then(|(_project_path, providers_path)| std::fs::read_to_string(&providers_path).ok())
        .and_then(|s| parse_providers_yaml(&s).ok());

    let started = Instant::now();
    if let Some(cfg) = providers_cfg {
        let empty: Vec<String> = Vec::new();
        let gem_bin = cfg.providers.get("gemini").map(|p| p.cmd.clone()).unwrap_or_else(|| "gemini".into());
        let cla_bin = cfg.providers.get("claude").map(|p| p.cmd.clone()).unwrap_or_else(|| "claude".into());
        let cur_bin = cfg.providers.get("cursor-agent").map(|p| p.cmd.clone()).unwrap_or_else(|| "cursor-agent".into());
        let handles = vec![
            std::thread::spawn({ let gem_bin = gem_bin.clone(); let empty = empty.clone(); move || probe_version_only("gemini", &gem_bin, &empty, per_timeout) }),
            std::thread::spawn({ let cla_bin = cla_bin.clone(); let empty = empty.clone(); move || probe_version_only("claude", &cla_bin, &empty, per_timeout) }),
            std::thread::spawn({ let cur_bin = cur_bin.clone(); let empty = empty.clone(); move || probe_version_only("cursor-agent", &cur_bin, &empty, per_timeout) }),
            std::thread::spawn(move || probe_tmux(per_timeout)),
            std::thread::spawn(move || probe_git(per_timeout)),
        ];
        for h in handles {
            let remain = global_cap.saturating_sub(started.elapsed().as_millis() as u64);
            if remain == 0 { break; }
            let r = h.join().unwrap_or_else(|_| ProbeResult { name: "unknown".into(), present: false, version: None, supports: BTreeMap::new(), timed_out: true, error: Some("thread_panic".into()) });
            results.push(r);
        }
    } else {
        let empty: Vec<String> = Vec::new();
        let handles = vec![
            std::thread::spawn({ let empty = empty.clone(); move || probe_version_only("gemini", "gemini", &empty, per_timeout) }),
            std::thread::spawn({ let empty = empty.clone(); move || probe_version_only("claude", "claude", &empty, per_timeout) }),
            std::thread::spawn({ let empty = empty.clone(); move || probe_version_only("cursor-agent", "cursor-agent", &empty, per_timeout) }),
            std::thread::spawn(move || probe_tmux(per_timeout)),
            std::thread::spawn(move || probe_git(per_timeout)),
        ];
        for h in handles {
            let remain = global_cap.saturating_sub(started.elapsed().as_millis() as u64);
            if remain == 0 { break; }
            let r = h.join().unwrap_or_else(|_| ProbeResult { name: "unknown".into(), present: false, version: None, supports: BTreeMap::new(), timed_out: true, error: Some("thread_panic".into()) });
            results.push(r);
        }
    }

    // Derive status and worst error code according to spec
    let mut any_timeout = false;
    let mut any_missing = false;
    let degraded = false;

    for r in &results {
        if r.timed_out { any_timeout = true; }
        if !r.present { any_missing = true; }
    }

    // Relaxed policy: if version is obtained and not timed out, consider OK.
    // Reserve DEGRADE for real timeouts (handled via any_timeout) or explicit probe errors in future.

    let status_text = if any_missing {
        "KO"
    } else if any_timeout || degraded {
        "DEGRADE"
    } else {
        "OK"
    };

    // NDJSON self-check if requested
    let mut ndjson_report: Option<Value> = None;
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
            pb.finish_and_clear();
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
            pb.finish_and_clear();
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

/// Probe help command
fn probe_help(bin: &str, help_args: &[&str], timeout_ms: u64) -> Result<String, String> {
    let timeout = Duration::from_millis(timeout_ms);
    let debug = std::env::var("DOCTOR_DEBUG").ok().map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false);
    if debug { eprintln!("[doctor] help probe: {} {:?}", bin, help_args); }
    match run_with_timeout(bin, help_args, timeout) {
        Ok((_code, out, err)) => {
            let text = if !out.trim().is_empty() { out } else { err };
            return Ok(text);
        }
        Err(e) => {
            if debug { eprintln!("[doctor] help direct failed: {} {:?} => {}", bin, help_args, e); }
            // Fallback via login shell to inherit PATH managers (e.g. NVM)
            let joined = std::iter::once(bin).chain(help_args.iter().copied()).collect::<Vec<_>>().join(" ");
            let shell_cmd = format!("bash -lc '{}'", joined.replace("'", "'\\''"));
            if debug { eprintln!("[doctor] help via shell: {}", shell_cmd); }
            match run_with_timeout("bash", &["-lc", &joined], timeout) {
                Ok((_code, out, err)) => {
                    let text = if !out.trim().is_empty() { out } else { err };
                    Ok(text)
                }
                Err(e2) => Err(e2),
            }
        }
    }
}

/// Probe version command
fn probe_version(bin: &str, candidates: &[&[&str]], timeout_ms: u64) -> Option<String> {
    for args in candidates {
        let timeout = Duration::from_millis(timeout_ms);
        let debug = std::env::var("DOCTOR_DEBUG").ok().map(|v| v == "1" || v.to_lowercase() == "true").unwrap_or(false);
        if debug { eprintln!("[doctor] version probe: {} {:?}", bin, args); }
        match run_with_timeout(bin, args, timeout) {
            Ok((_code, out, err)) => {
                let text = if !out.trim().is_empty() { out } else { err };
                let line = text.lines().next().unwrap_or("").trim().to_string();
                if !line.is_empty() { return Some(line); }
            }
            Err(e) => {
                if debug { eprintln!("[doctor] version direct failed: {} {:?} => {}", bin, args, e); }
                // shell fallback
                let joined = std::iter::once(bin).chain(args.iter().copied()).collect::<Vec<_>>().join(" ");
                if let Ok((_code, out, err)) = run_with_timeout("bash", &["-lc", &joined], timeout) {
                    let text = if !out.trim().is_empty() { out } else { err };
                    let line = text.lines().next().unwrap_or("").trim().to_string();
                    if !line.is_empty() { return Some(line); }
                }
            }
        }
    }
    None
}

/// Probe version only
fn probe_version_only(name: &str, cmd: &str, version_args: &[String], timeout_ms: u64) -> ProbeResult {
    let supports = BTreeMap::new();
    let version_candidates: Vec<Vec<&str>> = if version_args.is_empty() {
        vec![vec!["--version"], vec!["version"], vec!["-v"]]
    } else {
        vec![version_args.iter().map(|s| s.as_str()).collect()]
    };
    let version = probe_version(cmd, &version_candidates.iter().map(|v| v.as_slice()).collect::<Vec<_>>(), timeout_ms);
    if let Some(v) = version {
        ProbeResult { name: name.into(), present: true, version: Some(v), supports, timed_out: false, error: None }
    } else {
        ProbeResult { name: name.into(), present: false, version: None, supports, timed_out: false, error: Some("version_probe_failed".into()) }
    }
}

/// Parse tmux list commands
fn parse_tmux_list_commands(list_cmds: &str) -> BTreeMap<String, bool> {
    let mut supports = BTreeMap::new();
    supports.insert("pipe_pane".into(), list_cmds.contains("pipe-pane"));
    supports
}

/// Probe tmux
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

/// Probe git
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

/// Build doctor JSON output
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
