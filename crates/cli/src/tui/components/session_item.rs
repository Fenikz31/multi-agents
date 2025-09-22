//! Session Item component for Sessions view
//! 
//! Provides a reusable session item component with status indicators,
//! provider badges, and action buttons.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use super::super::themes::{ThemePalette, Typography};

/// Session status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Inactive,
    Error,
    Starting,
    Stopping,
}

impl SessionStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            SessionStatus::Active => "🟢",
            SessionStatus::Inactive => "⚪",
            SessionStatus::Error => "🔴",
            SessionStatus::Starting => "🟡",
            SessionStatus::Stopping => "🟠",
        }
    }

    pub fn color(&self, theme: &ThemePalette) -> Style {
        match self {
            SessionStatus::Active => Style::default().fg(theme.success),
            SessionStatus::Inactive => Style::default().fg(theme.secondary),
            SessionStatus::Error => Style::default().fg(theme.error),
            SessionStatus::Starting => Style::default().fg(theme.warning),
            SessionStatus::Stopping => Style::default().fg(theme.warning),
        }
    }
}

/// Provider enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Gemini,
    Claude,
    Cursor,
}

impl Provider {
    pub fn icon(&self) -> &'static str {
        match self {
            Provider::Gemini => "🤖",
            Provider::Claude => "🧠",
            Provider::Cursor => "🎯",
        }
    }

    pub fn color(&self, theme: &ThemePalette) -> Style {
        match self {
            Provider::Gemini => Style::default().fg(theme.primary),
            Provider::Claude => Style::default().fg(theme.secondary),
            Provider::Cursor => Style::default().fg(theme.warning),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Provider::Gemini => "Gemini",
            Provider::Claude => "Claude",
            Provider::Cursor => "Cursor",
        }
    }
}

/// Session data structure
#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub role: String,
    pub provider: Provider,
    pub model: String,
    pub status: SessionStatus,
    pub created_at: String,
    pub last_activity: Option<String>,
    pub duration: Option<String>,
}

/// Session item component
#[derive(Debug, Clone)]
pub struct SessionItem {
    pub session: Session,
    pub selected: bool,
    pub focused: bool,
    pub hovered: bool,
}

impl SessionItem {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            selected: false,
            focused: false,
            hovered: false,
        }
    }

    pub fn with_selection(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn with_focus(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_hover(mut self, hovered: bool) -> Self {
        self.hovered = hovered;
        self
    }
}

/// Renders a detailed session item.
pub fn render_session_item(f: &mut ratatui::Frame, area: Rect, session_item: &SessionItem, theme: &ThemePalette, typography: &Typography) {
    let border_style = if session_item.selected { theme.primary } else { theme.secondary };
    let bg_color = if session_item.hovered { theme.surface } else { theme.background };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Line::from(vec![
            Span::styled(session_item.session.status.icon(), session_item.session.status.color(theme)),
            Span::raw(" "),
            Span::styled(&session_item.session.agent_name, typography.body.fg(theme.text)),
        ]))
        .style(Style::default().bg(bg_color));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Provider & Model
            Constraint::Length(1), // Role & Duration
            Constraint::Min(0),    // Last Activity
        ])
        .split(inner_area);

    // Status icon
    let status_text = session_item.session.status.icon();
    let status_style = typography.body.fg(session_item.session.status.color(theme).fg.unwrap_or(theme.text));
    let status = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(status, chunks[0]);

    // Provider & Model
    let provider_model_text = format!("{} {}", session_item.session.provider.icon(), session_item.session.model);
    let provider_model_paragraph = Paragraph::new(provider_model_text)
        .style(typography.caption.fg(theme.secondary));
    f.render_widget(provider_model_paragraph, chunks[0]);

    // Role & Duration
    let duration_text = session_item.session.duration.as_deref().unwrap_or("N/A");
    let role_duration_text = format!("Role: {} | Duration: {}", session_item.session.role, duration_text);
    let role_duration_paragraph = Paragraph::new(role_duration_text)
        .style(typography.caption.fg(theme.secondary));
    f.render_widget(role_duration_paragraph, chunks[1]);

    // Last Activity
    if let Some(last_activity) = &session_item.session.last_activity {
        let last_activity_text = format!("Last Activity: {}", last_activity);
        let last_activity_paragraph = Paragraph::new(last_activity_text)
            .style(typography.caption.fg(theme.text));
        f.render_widget(last_activity_paragraph, chunks[2]);
    }
}

/// Renders a compact session item.
pub fn render_session_item_compact(f: &mut ratatui::Frame, area: Rect, session_item: &SessionItem, theme: &ThemePalette, typography: &Typography) {
    let text = format!(
        "{} {} {}:{} {} - {}",
        session_item.session.status.icon(),
        session_item.session.provider.icon(),
        session_item.session.role,
        session_item.session.agent_name,
        session_item.session.model,
        session_item.session.duration.as_deref().unwrap_or("N/A")
    );

    let style = if session_item.selected {
        typography.body.fg(theme.primary).add_modifier(Modifier::REVERSED)
    } else if session_item.focused {
        typography.body.fg(theme.primary).add_modifier(Modifier::BOLD)
    } else {
        typography.body.fg(theme.text)
    };

    let paragraph = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}

/// Renders a session status badge.
pub fn render_session_status_badge(
    f: &mut ratatui::Frame,
    area: Rect,
    status: SessionStatus,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let text = format!("{} {}", status.icon(), status.label());
    let style = typography.caption.fg(status.color(theme).fg.unwrap_or(theme.text));
    let badge = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::ALL).border_style(status.color(theme)));
    f.render_widget(badge, area);
}

impl SessionStatus {
    pub fn label(&self) -> &'static str {
        match self {
            SessionStatus::Active => "Active",
            SessionStatus::Inactive => "Inactive",
            SessionStatus::Error => "Error",
            SessionStatus::Starting => "Starting",
            SessionStatus::Stopping => "Stopping",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_icon() {
        assert_eq!(SessionStatus::Active.icon(), "🟢");
        assert_eq!(SessionStatus::Inactive.icon(), "⚪");
        assert_eq!(SessionStatus::Error.icon(), "🔴");
        assert_eq!(SessionStatus::Starting.icon(), "🟡");
        assert_eq!(SessionStatus::Stopping.icon(), "🟠");
    }

    #[test]
    fn test_provider_icon() {
        assert_eq!(Provider::Gemini.icon(), "🤖");
        assert_eq!(Provider::Claude.icon(), "🧠");
        assert_eq!(Provider::Cursor.icon(), "🎯");
    }

    #[test]
    fn test_session_item_creation() {
        let session = Session {
            id: "session-1".to_string(),
            project_id: "project-1".to_string(),
            agent_id: "agent-1".to_string(),
            agent_name: "backend".to_string(),
            role: "dev".to_string(),
            provider: Provider::Claude,
            model: "3.5-sonnet".to_string(),
            status: SessionStatus::Active,
            created_at: "2025-01-17T10:00:00Z".to_string(),
            last_activity: Some("2m ago".to_string()),
            duration: Some("5m".to_string()),
        };

        let session_item = SessionItem::new(session.clone());
        assert_eq!(session_item.session.id, "session-1");
        assert_eq!(session_item.session.agent_name, "backend");
        assert!(!session_item.selected);
        assert!(!session_item.focused);
        assert!(!session_item.hovered);
    }

    #[test]
    fn test_session_item_with_selection() {
        let session = Session {
            id: "session-1".to_string(),
            project_id: "project-1".to_string(),
            agent_id: "agent-1".to_string(),
            agent_name: "frontend".to_string(),
            role: "dev".to_string(),
            provider: Provider::Gemini,
            model: "2.0".to_string(),
            status: SessionStatus::Inactive,
            created_at: "2025-01-17T10:00:00Z".to_string(),
            last_activity: None,
            duration: None,
        };

        let session_item = SessionItem::new(session)
            .with_selection(true)
            .with_focus(true)
            .with_hover(true);

        assert!(session_item.selected);
        assert!(session_item.focused);
        assert!(session_item.hovered);
    }

    #[test]
    fn test_session_status_label() {
        assert_eq!(SessionStatus::Active.label(), "Active");
        assert_eq!(SessionStatus::Inactive.label(), "Inactive");
        assert_eq!(SessionStatus::Error.label(), "Error");
        assert_eq!(SessionStatus::Starting.label(), "Starting");
        assert_eq!(SessionStatus::Stopping.label(), "Stopping");
    }

    #[test]
    fn test_provider_label() {
        assert_eq!(Provider::Gemini.label(), "Gemini");
        assert_eq!(Provider::Claude.label(), "Claude");
        assert_eq!(Provider::Cursor.label(), "Cursor");
    }
}
