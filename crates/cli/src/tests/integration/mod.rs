//! Integration tests

pub mod config_tests;
pub mod doctor_tests;
pub mod db_tests;
pub mod send_tests;
pub mod session_tests;
pub mod agent_tests;
pub mod tmux_tests;

// Re-export all integration tests
pub use config_tests::*;
pub use doctor_tests::*;
pub use db_tests::*;
pub use send_tests::*;
pub use session_tests::*;
pub use agent_tests::*;
pub use tmux_tests::*;
