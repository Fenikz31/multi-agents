//! Timeout handling utilities

use std::time::{Duration, Instant};
use std::process::{Command, Stdio};
use std::io::{Read, BufRead, BufReader};
use std::thread;
use std::sync::mpsc;

/// Run a command with timeout and return (exit_code, stdout, stderr)
pub fn run_with_timeout(bin: &str, args: &[&str], timeout: Duration) -> Result<(i32, String, String), String> {
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

/// Line event for streaming operations
#[derive(Debug)]
pub enum LineEvent { 
    Stdout(String), 
    Stderr(String), 
    Exit(i32) 
}

/// Run a command with timeout and streaming output
pub fn run_with_timeout_streaming(
    bin: &str,
    args: &[&str],
    timeout: Duration,
    _project: &str,
    _agent_role: &str,
    _provider_key: &str,
    _session_id: &str,
    pb_opt: Option<&indicatif::ProgressBar>,
    parse_cursor_stream: bool,
) -> Result<i32, String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let (tx, rx) = mpsc::channel::<LineEvent>();

    // stdout reader
    if let Some(so) = child.stdout.take() {
        let txo = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(so);
            for line_res in reader.lines() {
                if let Ok(line) = line_res { let _ = txo.send(LineEvent::Stdout(line)); } else { break; }
            }
        });
    }
    // stderr reader
    if let Some(se) = child.stderr.take() {
        let txe = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(se);
            for line_res in reader.lines() {
                if let Ok(line) = line_res { let _ = txe.send(LineEvent::Stderr(line)); } else { break; }
            }
        });
    }
    // wait thread
    let txw = tx.clone();
    thread::spawn(move || {
        match child.wait() {
            Ok(status) => { let _ = txw.send(LineEvent::Exit(status.code().unwrap_or(-1))); }
            Err(_) => { let _ = txw.send(LineEvent::Exit(-1)); }
        }
    });

    let start = Instant::now();
    let mut exit_code: Option<i32> = None;
    let mut saw_final_result: bool = false;
    loop {
        let remaining = if start.elapsed() >= timeout { 0 } else { (timeout - start.elapsed()).as_millis() as u64 };
        if remaining == 0 { return Err("timeout".into()); }
        match rx.recv_timeout(Duration::from_millis(remaining)) {
            Ok(LineEvent::Stdout(line)) => {
                if parse_cursor_stream {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&line) {
                        // Parse cursor stream-json according to official spec
                        let mut text_to_print = None;
                        
                        if let Some(event_type) = v.get("type").and_then(|t| t.as_str()) {
                            match event_type {
                                "assistant" => {
                                    // Extract text from assistant.message.content[].text
                                    if let Some(message) = v.get("message") {
                                        if let Some(content) = message.get("content").and_then(|c| c.as_array()) {
                                            for item in content {
                                                if let Some(item_type) = item.get("type").and_then(|t| t.as_str()) {
                                                    if item_type == "text" {
                                                        if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                                            text_to_print = Some(text.to_string());
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                "result" => {
                                    // Final result event - extract complete text
                                    if let Some(result) = v.get("result").and_then(|r| r.as_str()) {
                                        text_to_print = Some(result.to_string());
                                        saw_final_result = true;
                                    }
                                }
                                "tool_call" => {
                                    // Optional: could extract tool call info, but skip for now
                                    continue;
                                }
                                _ => {
                                    // system, user events - skip
                                    continue;
                                }
                            }
                        } else {
                            // Fallback: try legacy flat fields for compatibility
                            text_to_print = v.get("text").and_then(|x| x.as_str()).map(|s| s.to_string())
                                .or_else(|| v.get("content").and_then(|x| x.as_str()).map(|s| s.to_string()))
                                .or_else(|| v.get("message").and_then(|x| x.as_str()).map(|s| s.to_string()))
                                .or_else(|| v.get("delta").and_then(|x| x.as_str()).map(|s| s.to_string()))
                                .or_else(|| v.get("data").and_then(|x| x.as_str()).map(|s| s.to_string()));
                        }
                        
                        if let Some(text) = text_to_print {
                            println!("{}", text);
                            // Log to NDJSON (would need logging module)
                            // log_ndjson(project, agent_role, provider_key, Some(session_id), "agent", "stdout_line", Some(&text), None, None);
                            // If we've seen the final result, we can return success immediately
                            if saw_final_result {
                                exit_code = Some(0);
                                break;
                            }
                        }
                    }
                } else {
                    println!("{}", line);
                    // Log to NDJSON (would need logging module)
                    // log_ndjson(project, agent_role, provider_key, Some(session_id), "agent", "stdout_line", Some(&line), None, None);
                }
                if let Some(pb) = pb_opt { pb.tick(); }
            }
            Ok(LineEvent::Stderr(line)) => {
                eprintln!("{}", line);
                // Log to NDJSON (would need logging module)
                // log_ndjson(project, agent_role, provider_key, Some(session_id), "agent", "stderr_line", Some(&line), None, None);
                if let Some(pb) = pb_opt { pb.tick(); }
            }
            Ok(LineEvent::Exit(code)) => { exit_code = Some(code); break; }
            Err(mpsc::RecvTimeoutError::Timeout) => { return Err("timeout".into()); }
            Err(_e) => { break; }
        }
    }
    Ok(exit_code.unwrap_or(-1))
}
