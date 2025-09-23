//! Log Viewer component for Detail view
//! 
//! Provides a log viewer component with syntax highlighting,
//! filtering, and real-time updates.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget};

use super::super::themes::{ThemePalette, Typography};
use std::fs::File;
use std::io::Write;
use std::error::Error;
use serde_json::Value;

/// Log level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Trace,
}

impl LogLevel {
    pub fn icon(&self) -> &'static str {
        match self {
            LogLevel::Debug => "🐛",
            LogLevel::Info => "ℹ️",
            LogLevel::Warn => "⚠️",
            LogLevel::Error => "❌",
            LogLevel::Trace => "🔍",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Trace => "TRACE",
        }
    }

    pub fn color(&self, theme: &ThemePalette) -> Style {
        match self {
            LogLevel::Debug => Style::default().fg(theme.secondary),
            LogLevel::Info => Style::default().fg(theme.text),
            LogLevel::Warn => Style::default().fg(theme.warning),
            LogLevel::Error => Style::default().fg(theme.error),
            LogLevel::Trace => Style::default().fg(theme.secondary),
        }
    }

    pub fn from_str(s: &str) -> Option<LogLevel> {
        match s.to_ascii_uppercase().as_str() {
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARN" | "WARNING" => Some(LogLevel::Warn),
            "ERROR" => Some(LogLevel::Error),
            "TRACE" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

/// Log entry structure
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub source: Option<String>,
    pub metadata: Option<String>,
}

/// Log filter options
#[derive(Debug, Clone)]
pub struct LogFilter {
    pub levels: Vec<LogLevel>,
    pub search_term: Option<String>,
    pub source_filter: Option<String>,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            levels: vec![LogLevel::Info, LogLevel::Warn, LogLevel::Error],
            search_term: None,
            source_filter: None,
        }
    }
}

/// Log viewer component
#[derive(Debug, Clone)]
pub struct LogViewer {
    pub logs: Vec<LogEntry>,
    pub filter: LogFilter,
    pub scroll_position: usize,
    pub selected_line: Option<usize>,
    pub auto_scroll: bool,
    pub max_lines: usize,
}

impl LogViewer {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            filter: LogFilter::default(),
            scroll_position: 0,
            selected_line: None,
            auto_scroll: true,
            max_lines: 1000,
        }
    }

    pub fn with_max_lines(mut self, max: usize) -> Self {
        self.max_lines = max;
        self
    }

    pub fn add_log(&mut self, log: LogEntry) {
        self.logs.push(log);
        if self.logs.len() > self.max_lines {
            self.logs.remove(0);
        }
        if self.auto_scroll {
            self.scroll_position = self.logs.len().saturating_sub(1);
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.auto_scroll = false;
        self.scroll_position = self.scroll_position.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_position = (self.scroll_position + lines).min(self.logs.len().saturating_sub(1));
        if self.scroll_position + 1 >= self.logs.len() {
            self.auto_scroll = true;
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.auto_scroll = false;
        self.scroll_position = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_position = self.logs.len().saturating_sub(1);
        self.auto_scroll = true;
    }

    pub fn get_filtered_logs(&self) -> Vec<&LogEntry> {
        self.logs
            .iter()
            .filter(|log| {
                // Filter by level
                if !self.filter.levels.contains(&log.level) {
                    return false;
                }

                // Filter by search term
                if let Some(ref search) = self.filter.search_term {
                    if !log.message.to_lowercase().contains(&search.to_lowercase()) {
                        return false;
                    }
                }

                // Filter by source
                if let Some(ref source) = self.filter.source_filter {
                    if let Some(ref log_source) = log.source {
                        if !log_source.contains(source) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Export current filtered logs to a file (one line per log)
    pub fn export_to_file(&self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(path)?;
        for log in self.get_filtered_logs() {
            let line = format!(
                "{} [{}] {}\n",
                log.timestamp,
                log.level.label(),
                log.message
            );
            file.write_all(line.as_bytes())?;
        }
        Ok(())
    }

    /// Ingest a single NDJSON line (lenient). Unknown/missing fields are ignored.
    pub fn ingest_ndjson_line(&mut self, line: &str) {
        if let Ok(v) = serde_json::from_str::<Value>(line) {
            let timestamp = v.get("timestamp").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let level = v.get("level").and_then(|x| x.as_str()).and_then(LogLevel::from_str).unwrap_or(LogLevel::Info);
            let message = v.get("message").and_then(|x| x.as_str()).unwrap_or("").to_string();
            let source = v.get("source").and_then(|x| x.as_str()).map(|s| s.to_string());
            let metadata = v.get("metadata").and_then(|x| x.as_str()).map(|s| s.to_string());
            if !message.is_empty() {
                self.add_log(LogEntry { timestamp, level, message, source, metadata });
            }
        }
    }
}

pub fn render_log_viewer(f: &mut ratatui::Frame, area: Rect, log_viewer: &LogViewer, theme: &ThemePalette, typography: &Typography) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Filter bar
            Constraint::Min(0),    // Log list
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Filter bar
    let filter_text = format!(
        "Filter: {} | Search: {} | Lines: {}",
        log_viewer.filter.levels.iter().map(|l| l.label()).collect::<Vec<_>>().join(","),
        log_viewer.filter.search_term.as_deref().unwrap_or("None"),
        log_viewer.logs.len()
    );
    let filter_style = typography.caption.fg(theme.secondary);
    let filter_bar = Paragraph::new(filter_text)
        .style(filter_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(filter_bar, chunks[0]);

    // Log list
    let visible_logs = log_viewer.get_filtered_logs();
    let max_index = visible_logs.len().saturating_sub(1);
    let start = log_viewer.scroll_position.min(max_index);

    let log_items: Vec<ListItem> = visible_logs
        .iter()
        .enumerate()
        .skip(start)
        .map(|(i, log)| {
            let line_number = start + i + 1;
            let metadata = log.metadata.clone().unwrap_or_default();
            let text = format!(
                "{} [{}] {} - {}",
                log.timestamp,
                log.level.label(),
                log.message,
                metadata
            );

            let style = typography.body.fg(log.level.color(theme).fg.unwrap_or(theme.text));
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).border_style(theme.secondary));

    let mut state = ListState::default();
    state.select(log_viewer.selected_line);

    f.render_stateful_widget(list, chunks[1], &mut state);

    // Status bar
    let status_text = format!(
        "Line {}/{} | AutoScroll: {} | Selected: {}",
        start + 1,
        visible_logs.len(),
        if log_viewer.auto_scroll { "ON" } else { "OFF" },
        log_viewer.selected_line.map_or("None".to_string(), |i| (i + 1).to_string())
    );
    let status_style = typography.caption.fg(theme.secondary);
    let status_bar = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(status_bar, chunks[2]);
}

pub fn render_log_entry(f: &mut ratatui::Frame, area: Rect, log_entry: &LogEntry, theme: &ThemePalette, typography: &Typography) {
    let text = format!(
        "{} [{}] {}",
        log_entry.timestamp,
        log_entry.level.icon(),
        log_entry.message
    );

    let style = typography.body.fg(log_entry.level.color(theme).fg.unwrap_or(theme.text));
    let paragraph = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_icon() {
        assert_eq!(LogLevel::Debug.icon(), "🐛");
        assert_eq!(LogLevel::Info.icon(), "ℹ️");
        assert_eq!(LogLevel::Warn.icon(), "⚠️");
        assert_eq!(LogLevel::Error.icon(), "❌");
        assert_eq!(LogLevel::Trace.icon(), "🔍");
    }

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("WARN"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("WARNING"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("ERROR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("DEBUG"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("TRACE"), Some(LogLevel::Trace));
        assert_eq!(LogLevel::from_str("UNKNOWN"), None);
    }

    #[test]
    fn test_log_viewer_creation() {
        let log_viewer = LogViewer::new();
        assert!(log_viewer.logs.is_empty());
        assert_eq!(log_viewer.scroll_position, 0);
        assert_eq!(log_viewer.selected_line, None);
        assert!(log_viewer.auto_scroll);
        assert_eq!(log_viewer.max_lines, 1000);
    }

    #[test]
    fn test_log_viewer_add_log() {
        let mut log_viewer = LogViewer::new();
        let log = LogEntry {
            timestamp: "2025-01-17T10:00:00Z".to_string(),
            level: LogLevel::Info,
            message: "Test message".to_string(),
            source: Some("test".to_string()),
            metadata: None,
        };

        log_viewer.add_log(log);
        assert_eq!(log_viewer.logs.len(), 1);
        assert_eq!(log_viewer.logs[0].message, "Test message");
    }

    #[test]
    fn test_log_viewer_scroll() {
        let mut log_viewer = LogViewer::new();
        
        // Add some test logs
        for i in 0..10 {
            let log = LogEntry {
                timestamp: format!("2025-01-17T10:00:{:02}Z", i),
                level: LogLevel::Info,
                message: format!("Message {}", i),
                source: None,
                metadata: None,
            };
            log_viewer.add_log(log);
        }

        // Test scrolling
        log_viewer.scroll_down(5);
        assert_eq!(log_viewer.scroll_position, 9);
        assert!(log_viewer.auto_scroll);

        log_viewer.scroll_up(2);
        assert_eq!(log_viewer.scroll_position, 7);

        log_viewer.scroll_to_top();
        assert_eq!(log_viewer.scroll_position, 0);

        log_viewer.scroll_to_bottom();
        assert_eq!(log_viewer.scroll_position, 9);
        assert!(log_viewer.auto_scroll);
    }

    #[test]
    fn test_log_viewer_filter() {
        let mut log_viewer = LogViewer::new();
        
        // Add logs with different levels
        let logs = vec![
            LogEntry {
                timestamp: "2025-01-17T10:00:00Z".to_string(),
                level: LogLevel::Info,
                message: "Info message".to_string(),
                source: Some("app".to_string()),
                metadata: None,
            },
            LogEntry {
                timestamp: "2025-01-17T10:00:01Z".to_string(),
                level: LogLevel::Error,
                message: "Error message".to_string(),
                source: Some("app".to_string()),
                metadata: None,
            },
            LogEntry {
                timestamp: "2025-01-17T10:00:02Z".to_string(),
                level: LogLevel::Debug,
                message: "Debug message".to_string(),
                source: Some("debug".to_string()),
                metadata: None,
            },
        ];

        for log in logs {
            log_viewer.add_log(log);
        }

        // Test level filtering
        let filtered = log_viewer.get_filtered_logs();
        assert_eq!(filtered.len(), 2); // Info and Error (Debug is filtered out by default)

        // Test search filtering
        log_viewer.filter.search_term = Some("Error".to_string());
        let filtered = log_viewer.get_filtered_logs();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].message, "Error message");

        // Test source filtering
        log_viewer.filter.search_term = None;
        log_viewer.filter.source_filter = Some("debug".to_string());
        let filtered = log_viewer.get_filtered_logs();
        assert_eq!(filtered.len(), 0); // Debug level is filtered out

        // Include debug level
        log_viewer.filter.levels = vec![LogLevel::Debug];
        let filtered = log_viewer.get_filtered_logs();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].message, "Debug message");
    }

    #[test]
    fn test_log_viewer_max_lines() {
        let mut log_viewer = LogViewer::new().with_max_lines(3);
        
        // Add more logs than max_lines
        for i in 0..5 {
            let log = LogEntry {
                timestamp: format!("2025-01-17T10:00:{:02}Z", i),
                level: LogLevel::Info,
                message: format!("Message {}", i),
                source: None,
                metadata: None,
            };
            log_viewer.add_log(log);
        }

        assert_eq!(log_viewer.logs.len(), 3);
        assert_eq!(log_viewer.logs[0].message, "Message 2"); // First two were removed
        assert_eq!(log_viewer.logs[2].message, "Message 4");
    }
}
