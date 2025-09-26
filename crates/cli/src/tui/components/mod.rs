//! TUI Components module

pub mod task_card;
pub mod session_item;
pub mod log_viewer;
pub mod toast;
pub mod status;

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, List, ListItem, ListState, Table, Row, Cell};

use super::themes::{ThemePalette, Typography};

// Re-export components for convenience
pub use task_card::{TaskCard, Task, TaskStatus, TaskPriority, render_task_card, render_task_card_compact};
pub use session_item::{SessionItem, Session, SessionStatus, Provider, render_session_item, render_session_item_compact, render_session_status_badge};
pub use log_viewer::{LogViewer, LogEntry, LogLevel, LogFilter, render_log_viewer, render_log_entry};
pub use toast::{Toast, ToastType, ToastQueue, render_toasts};
pub use status::{GlobalStatus, GlobalStateIcon, render_global_status};

/// Renders a styled button.
pub fn render_button(f: &mut ratatui::Frame, area: Rect, text: &str, is_selected: bool, theme: &ThemePalette, typography: &Typography) {
    let style = if is_selected {
        typography.body.fg(theme.primary).add_modifier(Modifier::REVERSED)
    } else {
        typography.body.fg(theme.text).bg(theme.surface)
    };
    let button = Paragraph::new(text).style(style).block(Block::default().borders(Borders::ALL).border_style(theme.secondary));
    f.render_widget(button, area);
}

/// Renders a simple list.
pub fn render_simple_list(f: &mut ratatui::Frame, area: Rect, title: &str, items: &[String], selected_index: Option<usize>, theme: &ThemePalette, typography: &Typography) {
    let block = Block::default().borders(Borders::ALL).title(Line::from(vec![Span::raw(title).style(typography.subtitle.fg(theme.text))])).border_style(theme.secondary);
    let list_items: Vec<ListItem> = items.iter().enumerate().map(|(i, item)| {
        let style = if selected_index == Some(i) {
            typography.body.fg(theme.primary).add_modifier(Modifier::REVERSED)
        } else {
            typography.body.fg(theme.text)
        };
        ListItem::new(item.clone()).style(style)
    }).collect();
    let list = List::new(list_items).block(block).highlight_style(typography.body.fg(theme.primary).add_modifier(Modifier::REVERSED));
    
    let mut state = ListState::default();
    state.select(selected_index);
    
    f.render_stateful_widget(list, area, &mut state);
}

/// Renders a simple table.
pub fn render_simple_table(f: &mut ratatui::Frame, area: Rect, title: &str, headers: &[&str], rows: Vec<Vec<String>>, column_widths: &[Constraint], theme: &ThemePalette, typography: &Typography) {
    let block = Block::default().borders(Borders::ALL).title(Line::from(vec![Span::raw(title).style(typography.subtitle.fg(theme.text))])).border_style(theme.secondary);
    let header_cells = headers.iter().map(|h| Cell::from(*h).style(typography.caption.fg(theme.primary)));
    let header = Row::new(header_cells).height(1).bottom_margin(1);
    
    let table_rows = rows.iter().map(|item| {
        let height = item.iter().map(|content| content.chars().filter(|c| *c == '\n').count()).max().unwrap_or(0) + 1;
        let cells = item.iter().map(|c| Cell::from(c.clone()).style(typography.body.fg(theme.text)));
        Row::new(cells).height(height as u16).bottom_margin(1)
    });
    
    let table = Table::new(table_rows, column_widths)
        .header(header)
        .block(block)
        .highlight_style(typography.body.fg(theme.primary).add_modifier(Modifier::REVERSED))
        .widths(column_widths);
    
    f.render_widget(table, area);
}

/// Renders a styled badge.
pub fn render_badge(f: &mut ratatui::Frame, area: Rect, text: &str, style: Style) {
    let badge = Paragraph::new(text).style(style).block(Block::default().borders(Borders::NONE));
    f.render_widget(badge, area);
}
