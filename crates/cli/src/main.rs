use clap::{Parser, Subcommand, ValueEnum};
use config_model::{parse_project_yaml, parse_providers_yaml, validate_project_config, validate_providers_config};
use std::fs;

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
