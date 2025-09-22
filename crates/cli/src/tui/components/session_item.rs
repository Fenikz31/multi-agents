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

    pub fn color(&self, theme: &ThemePalette) -> ratatui::style::Color {
        match self {
            SessionStatus::Active => theme.success,
            SessionStatus::Inactive => theme.secondary,
            SessionStatus::Error => theme.error,
            SessionStatus::Starting => theme.warning,
            SessionStatus::Stopping => theme.warning,
        }
    }

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

    pub fn color(&self, theme: &ThemePalette) -> ratatui::style::Color {
        match self {
            Provider::Gemini => theme.primary,
            Provider::Claude => theme.secondary,
            Provider::Cursor => theme.warning,
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

/// Session item component state
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

impl Widget for SessionItem {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // This is a placeholder implementation
        // The actual rendering will be handled by the render_session_item function
    }
}

/// Render a session item with proper styling
pub fn render_session_item(
    f: &mut ratatui::Frame,
    area: Rect,
    session_item: &SessionItem,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(3),  // Status icon
            Constraint::Min(20),    // Main content
            Constraint::Length(8),  // Provider
            Constraint::Length(10), // Duration
            Constraint::Length(20), // Actions
        ])
        .split(area);

    // Status icon
    let status_text = session_item.session.status.icon();
    let status_style = typography.body.style(session_item.session.status.color(theme));
    let status = Paragraph::new(status_text)
        .style(status_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(status, chunks[0]);

    // Main content: Agent name and role
    let main_text = format!(
        "{}:{}",
        session_item.session.role,
        session_item.session.agent_name
    );
    let main_style = if session_item.selected {
        typography.body.style(theme.primary).add_modifier(Modifier::REVERSED)
    } else if session_item.focused {
        typography.body.style(theme.primary).add_modifier(Modifier::BOLD)
    } else if session_item.hovered {
        typography.body.style(theme.secondary)
    } else {
        typography.body.style(theme.text)
    };
    let main = Paragraph::new(main_text)
        .style(main_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(main, chunks[1]);

    // Provider
    let provider_text = format!(
        "{} {}",
        session_item.session.provider.icon(),
        session_item.session.provider.label()
    );
    let provider_style = typography.caption.style(session_item.session.provider.color(theme));
    let provider = Paragraph::new(provider_text)
        .style(provider_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(provider, chunks[2]);

    // Duration
    let duration_text = session_item.session.duration.as_deref().unwrap_or("N/A");
    let duration_style = typography.small.style(theme.secondary);
    let duration = Paragraph::new(duration_text)
        .style(duration_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(duration, chunks[3]);

    // Actions
    let actions_text = match session_item.session.status {
        SessionStatus::Active => "[Attach] [Stop]",
        SessionStatus::Inactive => "[Start] [Delete]",
        SessionStatus::Error => "[Restart] [Delete]",
        SessionStatus::Starting => "[Starting...]",
        SessionStatus::Stopping => "[Stopping...]",
    };
    let actions_style = typography.small.style(theme.primary);
    let actions = Paragraph::new(actions_text)
        .style(actions_style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(actions, chunks[4]);
}

/// Render a session item in compact mode (single line)
pub fn render_session_item_compact(
    f: &mut ratatui::Frame,
    area: Rect,
    session_item: &SessionItem,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let text = format!(
        "{} {}:{} {} {}",
        session_item.session.status.icon(),
        session_item.session.role,
        session_item.session.agent_name,
        session_item.session.provider.icon(),
        session_item.session.duration.as_deref().unwrap_or("N/A")
    );

    let style = if session_item.selected {
        typography.body.style(theme.primary).add_modifier(Modifier::REVERSED)
    } else if session_item.focused {
        typography.body.style(theme.primary).add_modifier(Modifier::BOLD)
    } else {
        typography.body.style(theme.text)
    };

    let paragraph = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}

/// Render session status badge
pub fn render_session_status_badge(
    f: &mut ratatui::Frame,
    area: Rect,
    status: SessionStatus,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let text = format!("{} {}", status.icon(), status.label());
    let style = typography.caption.style(status.color(theme));
    let badge = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::ALL).border_style(status.color(theme)));
    f.render_widget(badge, area);
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
