//! Unit tests for Detail View wiring using LogViewer component

use crate::tui::components::log_viewer::{LogViewer, LogEntry, LogLevel};

#[test]
fn test_log_viewer_filters_levels() {
    let mut viewer = LogViewer::new();
    let logs = vec![
        LogEntry { timestamp: "t1".into(), level: LogLevel::Info, message: "hello".into(), source: None, metadata: None },
        LogEntry { timestamp: "t2".into(), level: LogLevel::Warn, message: "warn".into(), source: None, metadata: None },
        LogEntry { timestamp: "t3".into(), level: LogLevel::Error, message: "error".into(), source: None, metadata: None },
        LogEntry { timestamp: "t4".into(), level: LogLevel::Debug, message: "debug".into(), source: None, metadata: None },
    ];
    for l in logs { viewer.add_log(l); }

    // default filter excludes Debug
    let filtered = viewer.get_filtered_logs();
    assert_eq!(filtered.len(), 3);

    // include only errors
    viewer.filter.levels = vec![LogLevel::Error];
    let filtered = viewer.get_filtered_logs();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].level, LogLevel::Error);
}


