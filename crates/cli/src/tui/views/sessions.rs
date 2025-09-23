//! Sessions view implementation
//!
//! Renders a simple sessions list with header/footer using ratatui.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::super::themes::{ThemePalette, Typography};
use crate::tui::components::{ToastQueue, render_toasts, GlobalStatus, GlobalStateIcon, render_global_status};
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

    let status = GlobalStatus {
        project_name: "<project>".to_string(),
        view_name: "Sessions".to_string(),
        focus: "Body".to_string(),
        icon: GlobalStateIcon::Active,
        last_action: None,
    };
    render_global_status(f, chunks[0], &status, theme, typography);

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

    // Render toasts placeholder
    let queue = ToastQueue::with_capacity(3);
    render_toasts(f, chunks[1], &queue, theme, typography);
}


