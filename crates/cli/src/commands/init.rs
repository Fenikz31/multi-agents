//! Project initialization command

use std::fs;
use std::path::Path;
use config_model::parse_project_yaml;
use db::{open_or_create_db, sync_project_from_config};
use crate::utils::resolve_db_path;
use crate::utils::errors::exit_with;

/// Run project initialization command
pub fn run_init(config_dir: Option<&str>, force: bool, skip_db: bool) -> Result<(), Box<dyn std::error::Error>> {
    let base = config_dir.unwrap_or("./config");
    
    println!("🚀 Initializing multi-agents project...");
    
    // 1. Initialize database (if not skipped)
    if !skip_db {
        println!("📊 Initializing database...");
        let db_path = resolve_db_path();
        match open_or_create_db(&db_path) {
            Ok(_) => println!("✅ Database initialized"),
            Err(e) => return exit_with(7, format!("Database initialization failed: {}", e)),
        }
    } else {
        println!("⏭️  Skipping database initialization");
    }
    
    // 2. Create config files (if not exist or force)
    println!("📝 Creating configuration files...");
    let proj_path = format!("{}/project.yaml", base);
    let prov_path = format!("{}/providers.yaml", base);
    
    let project_yaml = r#"schema_version: 1
project: demo
agents:
  - name: backend
    role: backend
    provider: cursor-agent
    model: auto
    allowed_tools: [Bash, Edit]
    system_prompt: >
      Backend engineer. Respond in up to 5 bullet points
  - name: frontend
    role: frontend
    provider: gemini
    model: auto
    allowed_tools: [Bash, Edit]
    system_prompt: >
      Frontend engineer. Respond in up to 5 bullet points
  - name: devops
    role: devops
    provider: cursor-agent
    model: auto
    allowed_tools: [Bash, Edit]
    system_prompt: >
      DevOps engineer. Respond in up to 5 bullet points
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
    oneshot_args: ["-p","--output-format","stream-json","--resume","{chat_id}","{prompt}"]
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
            println!("⏭️  SKIP: {} exists (use --force to overwrite)", path);
            return Ok(());
        }
        std::fs::create_dir_all(Path::new(path).parent().unwrap())?;
        std::fs::write(path, contents)?;
        println!("✅ WROTE: {}", path);
        Ok(())
    };

    write_file(&proj_path, project_yaml)?;
    write_file(&prov_path, providers_yaml)?;
    
    // 3. Synchronize project and agents to database
    println!("🔄 Synchronizing project and agents...");
    let db_path = resolve_db_path();
    let conn = open_or_create_db(&db_path)?;
    
    let proj_s = fs::read_to_string(&proj_path)?;
    let project_config = parse_project_yaml(&proj_s).map_err(|e| format!("Invalid project config: {}", e))?;
    
    match sync_project_from_config(&conn, &project_config) {
        Ok(_) => println!("✅ Project synchronized successfully"),
        Err(e) => return exit_with(7, format!("Synchronization failed: {}", e)),
    }
    
    // 4. Validate configuration
    println!("🔍 Validating configuration...");
    let prov_s = fs::read_to_string(&prov_path)?;
    let providers_config = config_model::parse_providers_yaml(&prov_s).map_err(|e| format!("Invalid providers config: {}", e))?;
    
    match config_model::validate_project_config(&project_config, &providers_config) {
        Ok(_) => println!("✅ Project configuration valid"),
        Err(e) => return exit_with(6, format!("Project validation failed: {}", e)),
    }
    
    match config_model::validate_providers_config(&providers_config) {
        Ok(_) => println!("✅ Providers configuration valid"),
        Err(e) => return exit_with(6, format!("Providers validation failed: {}", e)),
    }
    
    println!("\n🎉 Project initialized successfully!");
    println!("📁 Config directory: {}", base);
    println!("💾 Database: {}", db_path);
    println!("\n🚀 Next steps:");
    println!("  • multi-agents send --to @all --message \"Hello world!\"");
    println!("  • multi-agents session start --agent backend");
    println!("  • multi-agents session list");
    
    Ok(())
}
