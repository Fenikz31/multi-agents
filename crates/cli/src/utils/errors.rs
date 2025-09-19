//! Error handling utilities

use crate::cli::commands::Format;

/// Exit with a specific error code and message
pub fn exit_with<T>(code: i32, msg: String) -> Result<T, Box<dyn std::error::Error>> {
    eprintln!("{}", msg);
    std::process::exit(code);
}

/// Format error message based on output format
pub fn format_error(format: Format, which: &str, err: &impl std::fmt::Display) -> String {
    match format {
        Format::Text => format!("{}: {}", which, err),
        Format::Json => serde_json::json!({"status":"error","scope":which,"error":err.to_string()}).to_string(),
    }
}

/// Generate first-run guidance message for missing configuration
pub fn generate_first_run_guidance() -> String {
    format!(
        "\n🚀 First-time setup detected! Follow these steps:\n\
         \n\
         1) Check your environment:\n\
            multi-agents doctor\n\
         \n\
         2) Initialize configuration:\n\
            multi-agents config init [--dir ./config]\n\
         \n\
         3) Initialize database:\n\
            multi-agents db init\n\
         \n\
         4) Add your project and agents:\n\
            multi-agents project add --name <project-name>\n\
            multi-agents agent add --project <project-name> --name <agent-name> --role <role> --provider <provider> --model <model>\n\
         \n\
         See docs/workflows.md for detailed examples."
    )
}

/// Handle missing configuration with first-run guidance
pub fn handle_missing_config<T>(error_msg: String) -> Result<T, Box<dyn std::error::Error>> {
    let guidance = generate_first_run_guidance();
    let full_message = format!("{}{}", error_msg, guidance);
    exit_with(6, full_message)
}
