//! TUI reusable components (widgets)

use ratatui::layout::{Constraint, Rect};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table};
use ratatui::{Frame};

use crate::tui::themes::Theme;

pub fn button(f: &mut Frame, area: Rect, label: &str, theme: &Theme, primary: bool, focused: bool) {
    let style = if primary { theme.button_primary() } else { theme.button_surface() };
    let label = if focused { format!("> {} <", label) } else { label.to_string() };
    let w = Paragraph::new(label).block(Block::default().borders(Borders::ALL).border_style(style)).style(style);
    f.render_widget(w, area);
}

pub fn list_simple<'a>(f: &mut Frame, area: Rect, title: &str, items: Vec<Span<'a>>) {
    let block = Block::default().borders(Borders::ALL).title(title);
    let para = Paragraph::new(Text::from(Line::from(items))).block(block);
    f.render_widget(para, area);
}

pub fn table_simple<'a>(f: &mut Frame, area: Rect, title: &str, headers: Vec<&'a str>, rows: Vec<Vec<&'a str>>) {
    let header = Row::new(headers.clone());
    let rows = rows.into_iter().map(|r| Row::new(r));
    let cols = headers.len().max(1) as u16;
    let width = 100 / cols;
    let widths: Vec<Constraint> = (0..cols).map(|_| Constraint::Percentage(width)).collect();
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(title));
    f.render_widget(table, area);
}

pub fn badge_success(f: &mut Frame, area: Rect, label: &str, theme: &Theme) {
    let style = theme.badge_success();
    let w = Paragraph::new(label).style(style).block(Block::default().borders(Borders::ALL));
    f.render_widget(w, area);
}


