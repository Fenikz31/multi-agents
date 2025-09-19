//! Multi-Agents CLI - Main entry point

use clap::Parser;
use multi_agents_cli::Cli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();
    let cli = Cli::parse();
    cli.execute()
}