//! Task Card component for Kanban view
//! 
//! Provides a reusable task card component with status indicators,
//! priority badges, and interactive elements.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use super::super::themes::{ThemePalette, Typography, ThemeKind, default_typography};

/// Task status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Todo,
    Doing,
    Done,
}

impl TaskStatus {
    pub fn icon(&self) -> &'static str {
        match self {
            TaskStatus::Todo => "📝",
            TaskStatus::Doing => "🔄",
            TaskStatus::Done => "✅",
        }
    }

    pub fn color(&self, theme: &ThemePalette) -> ratatui::style::Color {
        match self {
            TaskStatus::Todo => theme.text,
            TaskStatus::Doing => theme.warning,
            TaskStatus::Done => theme.success,
        }
    }
}

/// Task priority enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl TaskPriority {
    pub fn icon(&self) -> &'static str {
        match self {
            TaskPriority::Low => "🔵",
            TaskPriority::Medium => "🟡",
            TaskPriority::High => "🟠",
            TaskPriority::Critical => "🔴",
        }
    }

    pub fn color(&self, theme: &ThemePalette) -> ratatui::style::Color {
        match self {
            TaskPriority::Low => theme.success,
            TaskPriority::Medium => theme.warning,
            TaskPriority::High => theme.error,
            TaskPriority::Critical => theme.error,
        }
    }
}

/// Task data structure
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub assignee: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Task card component state
#[derive(Debug, Clone)]
pub struct TaskCard {
    pub task: Task,
    pub selected: bool,
    pub focused: bool,
    pub hovered: bool,
}

impl TaskCard {
    pub fn new(task: Task) -> Self {
        Self {
            task,
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

impl Widget for TaskCard {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // This is a placeholder implementation
        // The actual rendering will be handled by the render_task_card function
    }
}

/// Render a task card with proper styling
pub fn render_task_card(
    f: &mut ratatui::Frame,
    area: Rect,
    task_card: &TaskCard,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with title and priority
            Constraint::Min(1),    // Description (if available)
            Constraint::Length(1), // Footer with assignee and date
        ])
        .split(area);

    // Determine card style based on state
    let card_style = if task_card.selected {
        typography.body.add_modifier(Modifier::REVERSED)
    } else if task_card.focused {
        typography.body.add_modifier(Modifier::BOLD)
    } else if task_card.hovered {
        typography.body
    } else {
        typography.body
    };

    // Header: Title and Priority
    let header_text = format!(
        "{} {} {}",
        task_card.task.status.icon(),
        task_card.task.priority.icon(),
        task_card.task.title
    );
    let header = Paragraph::new(header_text)
        .style(card_style)
        .block(Block::default().borders(Borders::ALL).border_style(theme.secondary));
    f.render_widget(header, chunks[0]);

    // Description (if available)
    if let Some(description) = &task_card.task.description {
        let desc = Paragraph::new(description.as_str())
            .style(typography.caption)
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(desc, chunks[1]);
    }

    // Footer: Assignee and Date
    let footer_text = format!(
        "{} {}",
        task_card.task.assignee.as_deref().unwrap_or("Unassigned"),
        task_card.task.updated_at
    );
    let footer = Paragraph::new(footer_text)
        .style(typography.caption)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[2]);
}

/// Render a task card in compact mode (single line)
pub fn render_task_card_compact(
    f: &mut ratatui::Frame,
    area: Rect,
    task_card: &TaskCard,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let text = format!(
        "{} {} {} - {}",
        task_card.task.status.icon(),
        task_card.task.priority.icon(),
        task_card.task.title,
        task_card.task.assignee.as_deref().unwrap_or("Unassigned")
    );

    let style = if task_card.selected {
        typography.body.add_modifier(Modifier::REVERSED)
    } else if task_card.focused {
        typography.body.add_modifier(Modifier::BOLD)
    } else {
        typography.body
    };

    let paragraph = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_icon() {
        assert_eq!(TaskStatus::Todo.icon(), "📝");
        assert_eq!(TaskStatus::Doing.icon(), "🔄");
        assert_eq!(TaskStatus::Done.icon(), "✅");
    }

    #[test]
    fn test_task_priority_icon() {
        assert_eq!(TaskPriority::Low.icon(), "🔵");
        assert_eq!(TaskPriority::Medium.icon(), "🟡");
        assert_eq!(TaskPriority::High.icon(), "🟠");
        assert_eq!(TaskPriority::Critical.icon(), "🔴");
    }

    #[test]
    fn test_task_card_creation() {
        let task = Task {
            id: "task-1".to_string(),
            title: "Test Task".to_string(),
            description: Some("Test description".to_string()),
            status: TaskStatus::Todo,
            priority: TaskPriority::High,
            assignee: Some("backend".to_string()),
            created_at: "2025-01-17T10:00:00Z".to_string(),
            updated_at: "2025-01-17T10:00:00Z".to_string(),
        };

        let task_card = TaskCard::new(task.clone());
        assert_eq!(task_card.task.id, "task-1");
        assert_eq!(task_card.task.title, "Test Task");
        assert!(!task_card.selected);
        assert!(!task_card.focused);
        assert!(!task_card.hovered);
    }

    #[test]
    fn test_task_card_with_selection() {
        let task = Task {
            id: "task-1".to_string(),
            title: "Test Task".to_string(),
            description: None,
            status: TaskStatus::Doing,
            priority: TaskPriority::Medium,
            assignee: None,
            created_at: "2025-01-17T10:00:00Z".to_string(),
            updated_at: "2025-01-17T10:00:00Z".to_string(),
        };

        let task_card = TaskCard::new(task)
            .with_selection(true)
            .with_focus(true)
            .with_hover(true);

        assert!(task_card.selected);
        assert!(task_card.focused);
        assert!(task_card.hovered);
    }
}
