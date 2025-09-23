//! Sessions view implementation
//!
//! Renders a simple sessions list with header/footer using ratatui.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::super::themes::{ThemePalette, Typography};
use crate::tui::state::view_state::SessionsState;

pub fn render_sessions_view(
    f: &mut ratatui::Frame,
    area: Rect,
    sessions_state: &SessionsState,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // List
            Constraint::Length(1), // Footer
        ])
        .split(area);

    let header_text = format!(
        "Sessions | Total: {} | Filter: {} | Sort: {}",
        sessions_state.sessions.len(),
        if sessions_state.filter.is_empty() { "None" } else { &sessions_state.filter },
        if sessions_state.sort_by_agent { "agent" } else { "last-activity" }
    );
    let header = Paragraph::new(header_text)
        .style(typography.subtitle.fg(theme.primary))
        .block(Block::default().borders(Borders::ALL).border_style(theme.primary));
    f.render_widget(header, chunks[0]);

    let filtered = sessions_state.get_filtered_sessions();
    let items: Vec<ListItem> = filtered
        .iter()
        .map(|s| {
            let text = format!("{} [{}] {}", s.agent_name, s.provider, s.status);
            ListItem::new(text).style(typography.body.fg(theme.text))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_style(theme.secondary));
    let mut list_state = ListState::default();
    list_state.select(sessions_state.selected_session);
    f.render_stateful_widget(list, chunks[1], &mut list_state);

    let footer = Paragraph::new("↑ ↓ navigate  |  t sort  |  / filter  |  r resume  X stop  S start")
        .style(typography.caption.fg(theme.secondary))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[2]);
}


