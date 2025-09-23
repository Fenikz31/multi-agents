//! Global status indicators (header)

use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::themes::{ThemePalette, Typography};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalStateIcon {
    Active,
    Busy,
    Warn,
    Error,
}

impl GlobalStateIcon {
    pub fn symbol(&self) -> &'static str {
        match self {
            GlobalStateIcon::Active => "●",
            GlobalStateIcon::Busy => "◐",
            GlobalStateIcon::Warn => "⚠",
            GlobalStateIcon::Error => "✖",
        }
    }
}

#[derive(Debug, Clone)]
pub struct GlobalStatus {
    pub project_name: String,
    pub view_name: String,
    pub focus: String,
    pub icon: GlobalStateIcon,
    pub last_action: Option<String>,
}

impl GlobalStatus {
    pub fn header_text(&self) -> String {
        let action = self.last_action.as_deref().unwrap_or("");
        format!(
            "Multi-Agents TUI — Project:{}  {}  View:{}  Focus:{}  {}",
            self.project_name,
            self.icon.symbol(),
            self.view_name,
            self.focus,
            action
        )
    }
}

pub fn render_global_status(
    f: &mut ratatui::Frame,
    area: Rect,
    status: &GlobalStatus,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let header = Paragraph::new(status.header_text())
        .style(typography.subtitle.fg(theme.primary))
        .block(Block::default().borders(Borders::ALL).border_style(theme.primary));
    f.render_widget(header, area);
}


