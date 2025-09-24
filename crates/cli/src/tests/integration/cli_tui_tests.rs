//! Integration tests for CLI TUI subcommand parsing

use clap::CommandFactory;

use crate::cli::commands::{Cli, Commands};

#[test]
fn tui_subcommand_parses_project_and_refresh_rate() {
    let cli = Cli::parse_from([
        "multi-agents",
        "tui",
        "--project",
        "demo",
        "--refresh-rate",
        "250",
    ]);

    match cli.cmd {
        Commands::Tui { project, refresh_rate } => {
            assert_eq!(project.as_deref(), Some("demo"));
            assert_eq!(refresh_rate, Some(250));
        }
        other => panic!("expected Commands::Tui, got: {:?}", other),
    }
}

#[test]
fn tui_help_includes_expected_options() {
    let mut cmd = Cli::command();
    let mut help = Vec::new();
    cmd.write_long_help(&mut help).expect("help render");
    let help_str = String::from_utf8(help).expect("utf8");

    assert!(help_str.contains("tui"));
    assert!(help_str.contains("--project"));
    assert!(help_str.contains("--refresh-rate"));
}


