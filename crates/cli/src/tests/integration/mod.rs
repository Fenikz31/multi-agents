//! Integration tests

pub mod config_tests;
pub mod doctor_tests;
pub mod db_tests;
pub mod send_tests;
pub mod session_tests;
pub mod agent_tests;
pub mod tmux_tests;
pub mod broadcast_tests;
pub mod performance_tests;
pub mod tui_integration_tests;
pub mod tui_performance_tests;
pub mod tui_compatibility_tests;
pub mod tui_regression_tests;
pub mod supervisor_subscription_tests;
pub mod supervisor_aggregation_tests;
pub mod broadcast_supervisor_integration_tests;
pub mod m7_comprehensive_tests;
pub mod m7_m4_regression_tests;
pub mod m7_acceptance_tests;
pub mod m7_functional_validation_tests;

// Re-export all integration tests
