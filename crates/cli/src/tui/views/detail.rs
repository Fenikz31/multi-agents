//! Detail view implementation (logs NDJSON)

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::super::themes::{ThemePalette, Typography};
use crate::tui::components::log_viewer::LogViewer;

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

    // Header with simple status
    let header = Paragraph::new("Logs — follow:on  level:info|warn|error  filter:<term>")
        .style(typography.subtitle.fg(theme.primary))
        .block(Block::default().borders(Borders::ALL).border_style(theme.primary));
    f.render_widget(header, chunks[0]);

    // Use existing component to render the logs
    log_viewer.render(f, chunks[1], theme, typography);

    let footer = Paragraph::new("↑ ↓ scroll  g/G home/end  F follow  / search  e export")
        .style(typography.caption.fg(theme.secondary))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[2]);
}


