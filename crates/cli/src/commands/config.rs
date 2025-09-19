//! Configuration commands implementation

use std::fs;
use std::path::Path;
use config_model::{
    parse_project_yaml, parse_providers_yaml, validate_project_config, validate_providers_config,
};
use crate::cli::commands::Format;
use crate::utils::{resolve_config_paths, handle_missing_config, format_error, exit_with};

/// Run config validation command
pub fn run_config_validate(project_path_opt: Option<&str>, providers_path_opt: Option<&str>, format: Format) -> Result<(), Box<dyn std::error::Error>> {
    let (project_path, providers_path) = match resolve_config_paths(project_path_opt, providers_path_opt) {
        Ok(p) => p,
        Err(msg) => return handle_missing_config(msg),
    };
    let proj_s = fs::read_to_string(&project_path)?;
    let prov_s = fs::read_to_string(&providers_path)?;

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

/// Run config initialization command
pub fn run_config_init(dir_opt: Option<&str>, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let base = dir_opt.unwrap_or("./config");
    let _ = std::fs::create_dir_all(base);
    let proj_path = format!("{}/project.yaml", base);
    let prov_path = format!("{}/providers.yaml", base);

    let project_yaml = r#"schema_version: 1
project: demo
agents:
  - name: backend
    role: backend
    provider: claude
    model: fill-me
    allowed_tools: [Edit]
    system_prompt: |
      You are a backend agent.
"#;

    let providers_yaml = r#"schema_version: 1
providers:
  claude:
    cmd: "claude"
    oneshot_args: ["-p","--print","--output-format","text","{prompt}","--session-id","{session_id}","--allowed-tools","{allowed_tools}","--permission-mode","plan"]
    repl_args: ["repl"]
    allowlist_flag: "--allowed-tools"
  cursor-agent:
    cmd: "cursor-agent"
    oneshot_args: ["-p","--output-format","text","--resume","{chat_id}","{prompt}"]
    repl_args: ["agent","--resume","{chat_id}"]
    create_chat_args: ["create-chat"]
    forbid_flags: ["--force"]
  gemini:
    cmd: "gemini"
    oneshot_args: ["{prompt}"]
    repl_args: ["-i","{system_prompt}","--allowed-tools","{allowed_tools}"]
    allowlist_flag: "--allowed-tools"
"#;

    let write_file = |path: &str, contents: &str| -> Result<(), Box<dyn std::error::Error>> {
        if Path::new(path).exists() && !force {
            println!("SKIP: {} exists (use --force to overwrite)", path);
            return Ok(());
        }
        std::fs::write(path, contents)?;
        println!("WROTE: {}", path);
        Ok(())
    };

    write_file(&proj_path, project_yaml)?;
    write_file(&prov_path, providers_yaml)?;
    println!("OK: config initialized under {}", base);
    Ok(())
}
