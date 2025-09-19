//! Multi-Agents CLI Library
//! 
//! This library provides the core functionality for the multi-agents CLI tool,
//! including command parsing, provider management, tmux operations, and logging.

pub mod cli;
pub mod commands;
pub mod providers;
pub mod tmux;
pub mod logging;
pub mod utils;
pub mod broadcast;

// Re-export main types for convenience
pub use cli::Cli;
pub use commands::*;

#[cfg(test)]
mod tests;
