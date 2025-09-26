//! Task Card component for Kanban view
//! 
//! Provides a reusable task card component with status indicators,
//! priority badges, and interactive elements.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::super::themes::{ThemePalette, Typography};

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

    pub fn color(&self, theme: &ThemePalette) -> Style {
        match self {
            TaskStatus::Todo => Style::default().fg(theme.text),
            TaskStatus::Doing => Style::default().fg(theme.warning),
            TaskStatus::Done => Style::default().fg(theme.success),
        }
    }
}

/// Task priority enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

    pub fn color(&self, theme: &ThemePalette) -> Style {
        match self {
            TaskPriority::Low => Style::default().fg(theme.success),
            TaskPriority::Medium => Style::default().fg(theme.warning),
            TaskPriority::High => Style::default().fg(theme.error),
            TaskPriority::Critical => Style::default().fg(theme.error),
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

/// Task card component
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

/// Renders a detailed task card.
pub fn render_task_card(f: &mut ratatui::Frame, area: Rect, task_card: &TaskCard, theme: &ThemePalette, typography: &Typography) {
    let border_style = if task_card.selected { theme.primary } else { theme.secondary };
    let bg_color = if task_card.hovered { theme.surface } else { theme.background };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Line::from(vec![
            Span::styled(task_card.task.status.icon(), task_card.task.status.color(theme)),
            Span::raw(" "),
            Span::styled(&task_card.task.title, typography.body.fg(theme.text)),
        ]))
        .style(Style::default().bg(bg_color));

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Priority
            Constraint::Length(1), // Assignee
            Constraint::Min(0),    // Description
        ])
        .split(inner_area);

    // Determine card style based on state
    let _card_style = if task_card.selected {
        typography.body.fg(theme.primary).add_modifier(Modifier::REVERSED)
    } else if task_card.focused {
        typography.body.fg(theme.primary).add_modifier(Modifier::BOLD)
    } else if task_card.hovered {
        typography.body.fg(theme.secondary)
    } else {
        typography.body.fg(theme.text)
    };

    // Priority
    let priority_text = format!("Priority: {} {:?}", task_card.task.priority.icon(), task_card.task.priority);
    let priority_paragraph = Paragraph::new(priority_text)
        .style(typography.caption.fg(theme.secondary));
    f.render_widget(priority_paragraph, chunks[0]);

    // Assignee
    if let Some(assignee) = &task_card.task.assignee {
        let assignee_text = format!("Assignee: {}", assignee);
        let assignee_paragraph = Paragraph::new(assignee_text)
            .style(typography.caption.fg(theme.secondary));
        f.render_widget(assignee_paragraph, chunks[1]);
    }

    // Description
    if let Some(description) = &task_card.task.description {
        let desc = Paragraph::new(description.as_str())
            .style(typography.caption.fg(theme.text))
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(desc, chunks[2]);
    }

    // Footer: Assignee and Date
    let footer_text = format!(
        "{} {}",
        task_card.task.assignee.as_deref().unwrap_or("Unassigned"),
        task_card.task.updated_at
    );
    let footer = Paragraph::new(footer_text)
        .style(typography.caption.fg(theme.secondary))
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
        typography.body.fg(theme.primary).add_modifier(Modifier::REVERSED)
    } else if task_card.focused {
        typography.body.fg(theme.primary).add_modifier(Modifier::BOLD)
    } else {
        typography.body.fg(theme.text)
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
