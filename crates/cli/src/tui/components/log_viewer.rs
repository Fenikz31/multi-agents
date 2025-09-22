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

    pub fn color(&self, theme: &ThemePalette) -> ratatui::style::Color {
        match self {
            LogLevel::Debug => theme.secondary,
            LogLevel::Info => theme.primary,
            LogLevel::Warn => theme.warning,
            LogLevel::Error => theme.error,
            LogLevel::Trace => theme.secondary,
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

    pub fn from_str(level: &str) -> Option<Self> {
        match level.to_uppercase().as_str() {
            "DEBUG" => Some(LogLevel::Debug),
            "INFO" => Some(LogLevel::Info),
            "WARN" | "WARNING" => Some(LogLevel::Warn),
            "ERROR" => Some(LogLevel::Error),
            "TRACE" => Some(LogLevel::Trace),
            _ => None,
        }
    }
}

/// Log entry data structure
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub source: Option<String>,
    pub metadata: Option<String>,
}

/// Log filter configuration
#[derive(Debug, Clone)]
pub struct LogFilter {
    pub levels: Vec<LogLevel>,
    pub search_term: Option<String>,
    pub source_filter: Option<String>,
    pub show_timestamps: bool,
    pub show_metadata: bool,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            levels: vec![LogLevel::Info, LogLevel::Warn, LogLevel::Error],
            search_term: None,
            source_filter: None,
            show_timestamps: true,
            show_metadata: false,
        }
    }
}

/// Log viewer component state
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

    pub fn with_logs(mut self, logs: Vec<LogEntry>) -> Self {
        self.logs = logs;
        self
    }

    pub fn with_filter(mut self, filter: LogFilter) -> Self {
        self.filter = filter;
        self
    }

    pub fn with_max_lines(mut self, max_lines: usize) -> Self {
        self.max_lines = max_lines;
        self
    }

    pub fn add_log(&mut self, log: LogEntry) {
        self.logs.push(log);
        
        // Maintain max_lines limit
        if self.logs.len() > self.max_lines {
            self.logs.remove(0);
        }

        // Auto-scroll to bottom if enabled
        if self.auto_scroll {
            self.scroll_to_bottom();
        }
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
        self.scroll_position = 0;
        self.selected_line = None;
    }

    pub fn scroll_up(&mut self, lines: usize) {
        if self.scroll_position > lines {
            self.scroll_position -= lines;
        } else {
            self.scroll_position = 0;
        }
        self.auto_scroll = false;
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let filtered_logs = self.get_filtered_logs();
        let max_scroll = filtered_logs.len().saturating_sub(1);
        
        if self.scroll_position + lines < max_scroll {
            self.scroll_position += lines;
        } else {
            self.scroll_position = max_scroll;
        }
        self.auto_scroll = false;
    }

    pub fn scroll_to_bottom(&mut self) {
        let filtered_logs = self.get_filtered_logs();
        self.scroll_position = filtered_logs.len().saturating_sub(1);
        self.auto_scroll = true;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_position = 0;
        self.auto_scroll = false;
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

    pub fn get_visible_logs(&self, height: usize) -> Vec<&LogEntry> {
        let filtered_logs = self.get_filtered_logs();
        let start = self.scroll_position;
        let end = (start + height).min(filtered_logs.len());
        
        if start < filtered_logs.len() {
            filtered_logs[start..end].to_vec()
        } else {
            Vec::new()
        }
    }
}

impl Default for LogViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for LogViewer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // This is a placeholder implementation
        // The actual rendering will be handled by the render_log_viewer function
    }
}

/// Render a log viewer with proper styling
pub fn render_log_viewer(
    f: &mut ratatui::Frame,
    area: Rect,
    log_viewer: &LogViewer,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Filter bar
            Constraint::Min(1),    // Log content
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
    let filter_style = typography.small.style(theme.secondary);
    let filter_bar = Paragraph::new(filter_text)
        .style(filter_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(filter_bar, chunks[0]);

    // Log content
    let visible_logs = log_viewer.get_visible_logs(chunks[1].height as usize);
    let log_items: Vec<ListItem> = visible_logs
        .iter()
        .enumerate()
        .map(|(i, log)| {
            let line_number = log_viewer.scroll_position + i + 1;
            let timestamp = if log_viewer.filter.show_timestamps {
                format!("[{}] ", log.timestamp)
            } else {
                String::new()
            };
            
            let source = if let Some(ref source) = log.source {
                format!("[{}] ", source)
            } else {
                String::new()
            };

            let metadata = if log_viewer.filter.show_metadata {
                log.metadata.as_deref().unwrap_or("")
            } else {
                ""
            };

            let text = format!(
                "{}{}{} {}: {}{}",
                timestamp,
                source,
                log.level.icon(),
                log.level.label(),
                log.message,
                metadata
            );

            let style = typography.body.style(log.level.color(theme));
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(log_items)
        .block(Block::default().borders(Borders::ALL).border_style(theme.secondary));
    
    let mut list_state = ListState::default();
    if let Some(selected) = log_viewer.selected_line {
        list_state.select(Some(selected));
    }
    
    f.render_stateful_widget(list, chunks[1], &mut list_state);

    // Status bar
    let status_text = format!(
        "Scroll: {}/{} | Auto-scroll: {} | Selected: {}",
        log_viewer.scroll_position + 1,
        log_viewer.get_filtered_logs().len(),
        if log_viewer.auto_scroll { "ON" } else { "OFF" },
        log_viewer.selected_line.map_or("None".to_string(), |i| (i + 1).to_string())
    );
    let status_style = typography.small.style(theme.secondary);
    let status_bar = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(status_bar, chunks[2]);
}

/// Render a log entry as a single line
pub fn render_log_entry(
    f: &mut ratatui::Frame,
    area: Rect,
    log_entry: &LogEntry,
    show_timestamp: bool,
    show_source: bool,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let timestamp = if show_timestamp {
        format!("[{}] ", log_entry.timestamp)
    } else {
        String::new()
    };
    
    let source = if show_source {
        log_entry.source.as_deref().map(|s| format!("[{}] ", s)).unwrap_or_default()
    } else {
        String::new()
    };

    let text = format!(
        "{}{}{} {}: {}",
        timestamp,
        source,
        log_entry.level.icon(),
        log_entry.level.label(),
        log_entry.message
    );

    let style = typography.body.style(log_entry.level.color(theme));
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
        assert_eq!(log_viewer.scroll_position, 5);
        assert!(!log_viewer.auto_scroll);

        log_viewer.scroll_up(2);
        assert_eq!(log_viewer.scroll_position, 3);

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
