//! Detail view implementation (logs NDJSON)

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::super::themes::{ThemePalette, Typography};
use crate::tui::components::{ToastQueue, render_toasts};
use crate::tui::components::log_viewer::{LogViewer, render_log_viewer};
use crate::tui::components::{GlobalStatus, GlobalStateIcon, render_global_status};

pub fn render_detail_view(
    f: &mut ratatui::Frame,
    area: Rect,
    log_viewer: &LogViewer,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Logs
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Header (global status)
    let status = GlobalStatus {
        project_name: "<project>".to_string(),
        view_name: "Detail".to_string(),
        focus: "Body".to_string(),
        icon: GlobalStateIcon::Active,
        last_action: None,
    };
    render_global_status(f, chunks[0], &status, theme, typography);

    // Use existing component to render the logs
    render_log_viewer(f, chunks[1], log_viewer, theme, typography);

    let footer = Paragraph::new("↑ ↓ scroll  g/G home/end  F follow  / search  e export")
        .style(typography.caption.fg(theme.secondary))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[2]);

    // Render toasts placeholder
    let queue = ToastQueue::with_capacity(3);
    render_toasts(f, chunks[1], &queue, theme, typography);
}


