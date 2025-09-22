//! Kanban view implementation
//! 
//! Provides a Kanban board view with columns for ToDo, Doing, and Done tasks.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::super::components::{TaskCard, Task, TaskStatus, TaskPriority, render_task_card};
use super::super::themes::{ThemePalette, Typography};

/// Kanban column data structure
#[derive(Debug, Clone)]
pub struct KanbanColumn {
    pub title: String,
    pub tasks: Vec<Task>,
    pub selected_task: Option<usize>,
    pub status: TaskStatus,
}

impl KanbanColumn {
    pub fn new(title: String, status: TaskStatus) -> Self {
        Self {
            title,
            tasks: Vec::new(),
            selected_task: None,
            status,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn remove_task(&mut self, index: usize) -> Option<Task> {
        if index < self.tasks.len() {
            Some(self.tasks.remove(index))
        } else {
            None
        }
    }

    pub fn move_task_to(&mut self, from_index: usize, to_column: &mut KanbanColumn, to_index: Option<usize>) -> bool {
        if let Some(task) = self.remove_task(from_index) {
            let mut task = task;
            task.status = to_column.status;
            
            if let Some(index) = to_index {
                to_column.tasks.insert(index, task);
            } else {
                to_column.tasks.push(task);
            }
            true
        } else {
            false
        }
    }

    pub fn select_task(&mut self, index: Option<usize>) {
        self.selected_task = index;
    }

    pub fn get_selected_task(&self) -> Option<&Task> {
        self.selected_task.and_then(|i| self.tasks.get(i))
    }
}

/// Kanban view state
#[derive(Debug, Clone)]
pub struct KanbanView {
    pub columns: Vec<KanbanColumn>,
    pub selected_column: usize,
    pub filter: String,
    pub show_completed: bool,
    pub sort_by: KanbanSort,
}

/// Kanban sorting options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KanbanSort {
    Created,
    Updated,
    Priority,
    Title,
}

impl KanbanView {
    pub fn new() -> Self {
        let mut columns = vec![
            KanbanColumn::new("To Do".to_string(), TaskStatus::Todo),
            KanbanColumn::new("Doing".to_string(), TaskStatus::Doing),
            KanbanColumn::new("Done".to_string(), TaskStatus::Done),
        ];

        // Add some sample tasks for demonstration
        columns[0].add_task(Task {
            id: "task-1".to_string(),
            title: "Implement TUI components".to_string(),
            description: Some("Create reusable TUI components for the interface".to_string()),
            status: TaskStatus::Todo,
            priority: TaskPriority::High,
            assignee: Some("backend".to_string()),
            created_at: "2025-01-17T10:00:00Z".to_string(),
            updated_at: "2025-01-17T10:00:00Z".to_string(),
        });

        columns[0].add_task(Task {
            id: "task-2".to_string(),
            title: "Add tests for components".to_string(),
            description: Some("Write comprehensive tests for all TUI components".to_string()),
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            assignee: Some("frontend".to_string()),
            created_at: "2025-01-17T10:01:00Z".to_string(),
            updated_at: "2025-01-17T10:01:00Z".to_string(),
        });

        columns[1].add_task(Task {
            id: "task-3".to_string(),
            title: "Design system implementation".to_string(),
            description: Some("Implement the design system with themes and typography".to_string()),
            status: TaskStatus::Doing,
            priority: TaskPriority::High,
            assignee: Some("ui-ux".to_string()),
            created_at: "2025-01-17T09:00:00Z".to_string(),
            updated_at: "2025-01-17T10:30:00Z".to_string(),
        });

        columns[2].add_task(Task {
            id: "task-4".to_string(),
            title: "Repository pattern setup".to_string(),
            description: Some("Set up repository pattern for data access".to_string()),
            status: TaskStatus::Done,
            priority: TaskPriority::Medium,
            assignee: Some("backend".to_string()),
            created_at: "2025-01-17T08:00:00Z".to_string(),
            updated_at: "2025-01-17T09:30:00Z".to_string(),
        });

        Self {
            columns,
            selected_column: 0,
            filter: String::new(),
            show_completed: true,
            sort_by: KanbanSort::Priority,
        }
    }

    pub fn move_to_column(&mut self, from_column: usize, from_task: usize, to_column: usize, to_task: Option<usize>) -> bool {
        if from_column < self.columns.len() && to_column < self.columns.len() {
            self.columns[from_column].move_task_to(from_task, &mut self.columns[to_column], to_task)
        } else {
            false
        }
    }

    pub fn select_column(&mut self, column_index: usize) {
        if column_index < self.columns.len() {
            self.selected_column = column_index;
        }
    }

    pub fn select_task_in_column(&mut self, column_index: usize, task_index: Option<usize>) {
        if column_index < self.columns.len() {
            self.columns[column_index].select_task(task_index);
        }
    }

    pub fn get_filtered_tasks(&self, column_index: usize) -> Vec<&Task> {
        if column_index >= self.columns.len() {
            return Vec::new();
        }

        let column = &self.columns[column_index];
        let mut tasks: Vec<&Task> = column.tasks.iter().collect();

        // Apply filter
        if !self.filter.is_empty() {
            tasks.retain(|task| {
                task.title.to_lowercase().contains(&self.filter.to_lowercase()) ||
                task.description.as_ref().map_or(false, |desc| desc.to_lowercase().contains(&self.filter.to_lowercase())) ||
                task.assignee.as_ref().map_or(false, |assignee| assignee.to_lowercase().contains(&self.filter.to_lowercase()))
            });
        }

        // Apply completed filter
        if !self.show_completed && column.status == TaskStatus::Done {
            tasks.clear();
        }

        // Sort tasks
        match self.sort_by {
            KanbanSort::Created => tasks.sort_by(|a, b| a.created_at.cmp(&b.created_at)),
            KanbanSort::Updated => tasks.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
            KanbanSort::Priority => tasks.sort_by(|a, b| {
                let priority_order = |p: TaskPriority| match p {
                    TaskPriority::Critical => 0,
                    TaskPriority::High => 1,
                    TaskPriority::Medium => 2,
                    TaskPriority::Low => 3,
                };
                priority_order(a.priority).cmp(&priority_order(b.priority))
            }),
            KanbanSort::Title => tasks.sort_by(|a, b| a.title.cmp(&b.title)),
        }

        tasks
    }

    pub fn get_total_tasks(&self) -> usize {
        self.columns.iter().map(|col| col.tasks.len()).sum()
    }

    pub fn get_completed_tasks(&self) -> usize {
        self.columns.iter()
            .find(|col| col.status == TaskStatus::Done)
            .map(|col| col.tasks.len())
            .unwrap_or(0)
    }
}

/// Render the Kanban view
pub fn render_kanban_view(
    f: &mut ratatui::Frame,
    area: Rect,
    kanban_view: &KanbanView,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(1),    // Kanban board
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Header
    let header_text = format!(
        "📋 Kanban Board - Total: {} | Completed: {} | Filter: {}",
        kanban_view.get_total_tasks(),
        kanban_view.get_completed_tasks(),
        if kanban_view.filter.is_empty() { "None" } else { &kanban_view.filter }
    );
    let header = Paragraph::new(header_text)
        .style(typography.subtitle.style(theme.primary))
        .block(Block::default().borders(Borders::ALL).border_style(theme.primary));
    f.render_widget(header, chunks[0]);

    // Kanban board
    let board_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[1]);

    for (i, column) in kanban_view.columns.iter().enumerate() {
        render_kanban_column(f, board_chunks[i], column, i == kanban_view.selected_column, kanban_view, theme, typography);
    }

    // Footer
    let footer_text = format!(
        "Sort: {:?} | Show completed: {} | Use ←→ to navigate columns, ↑↓ to navigate tasks",
        kanban_view.sort_by,
        kanban_view.show_completed
    );
    let footer = Paragraph::new(footer_text)
        .style(typography.small.style(theme.secondary))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[2]);
}

/// Render a single Kanban column
fn render_kanban_column(
    f: &mut ratatui::Frame,
    area: Rect,
    column: &KanbanColumn,
    is_selected: bool,
    kanban_view: &KanbanView,
    theme: &ThemePalette,
    typography: &Typography,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Column header
            Constraint::Min(1),    // Tasks list
        ])
        .split(area);

    // Column header
    let header_text = format!("{} {} ({})", column.status.icon(), column.title, column.tasks.len());
    let header_style = if is_selected {
        typography.subtitle.style(theme.primary).add_modifier(Modifier::BOLD)
    } else {
        typography.subtitle.style(theme.text)
    };
    let header = Paragraph::new(header_text)
        .style(header_style)
        .block(Block::default().borders(Borders::ALL).border_style(if is_selected { theme.primary } else { theme.secondary }));
    f.render_widget(header, chunks[0]);

    // Tasks list
    let filtered_tasks = kanban_view.get_filtered_tasks(column.status as usize);
    let task_items: Vec<ListItem> = filtered_tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let is_selected = column.selected_task == Some(i);
            let task_card = TaskCard::new(task.clone())
                .with_selection(is_selected)
                .with_focus(is_selected);
            
            let text = format!(
                "{} {} {} - {}",
                task.status.icon(),
                task.priority.icon(),
                task.title,
                task.assignee.as_deref().unwrap_or("Unassigned")
            );
            
            let style = if is_selected {
                typography.body.style(theme.primary).add_modifier(Modifier::REVERSED)
            } else {
                typography.body.style(theme.text)
            };
            
            ListItem::new(text).style(style)
        })
        .collect();

    let list = List::new(task_items)
        .block(Block::default().borders(Borders::ALL).border_style(theme.secondary))
        .highlight_style(typography.body.style(theme.primary).add_modifier(Modifier::REVERSED));
    
    let mut list_state = ListState::default();
    list_state.select(column.selected_task);
    
    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kanban_column_creation() {
        let column = KanbanColumn::new("Test Column".to_string(), TaskStatus::Todo);
        assert_eq!(column.title, "Test Column");
        assert_eq!(column.status, TaskStatus::Todo);
        assert!(column.tasks.is_empty());
        assert_eq!(column.selected_task, None);
    }

    #[test]
    fn test_kanban_column_add_task() {
        let mut column = KanbanColumn::new("Test Column".to_string(), TaskStatus::Todo);
        let task = Task {
            id: "task-1".to_string(),
            title: "Test Task".to_string(),
            description: None,
            status: TaskStatus::Todo,
            priority: TaskPriority::High,
            assignee: None,
            created_at: "2025-01-17T10:00:00Z".to_string(),
            updated_at: "2025-01-17T10:00:00Z".to_string(),
        };

        column.add_task(task);
        assert_eq!(column.tasks.len(), 1);
        assert_eq!(column.tasks[0].title, "Test Task");
    }

    #[test]
    fn test_kanban_column_remove_task() {
        let mut column = KanbanColumn::new("Test Column".to_string(), TaskStatus::Todo);
        let task = Task {
            id: "task-1".to_string(),
            title: "Test Task".to_string(),
            description: None,
            status: TaskStatus::Todo,
            priority: TaskPriority::High,
            assignee: None,
            created_at: "2025-01-17T10:00:00Z".to_string(),
            updated_at: "2025-01-17T10:00:00Z".to_string(),
        };

        column.add_task(task);
        let removed_task = column.remove_task(0);
        assert!(removed_task.is_some());
        assert_eq!(removed_task.unwrap().title, "Test Task");
        assert!(column.tasks.is_empty());
    }

    #[test]
    fn test_kanban_view_creation() {
        let kanban_view = KanbanView::new();
        assert_eq!(kanban_view.columns.len(), 3);
        assert_eq!(kanban_view.selected_column, 0);
        assert!(kanban_view.filter.is_empty());
        assert!(kanban_view.show_completed);
        assert_eq!(kanban_view.sort_by, KanbanSort::Priority);
    }

    #[test]
    fn test_kanban_view_move_task() {
        let mut kanban_view = KanbanView::new();
        let initial_todo_count = kanban_view.columns[0].tasks.len();
        let initial_doing_count = kanban_view.columns[1].tasks.len();

        // Move first task from ToDo to Doing
        let success = kanban_view.move_to_column(0, 0, 1, None);
        assert!(success);
        assert_eq!(kanban_view.columns[0].tasks.len(), initial_todo_count - 1);
        assert_eq!(kanban_view.columns[1].tasks.len(), initial_doing_count + 1);
    }

    #[test]
    fn test_kanban_view_filter() {
        let mut kanban_view = KanbanView::new();
        kanban_view.filter = "TUI".to_string();
        
        let filtered_tasks = kanban_view.get_filtered_tasks(0);
        assert!(!filtered_tasks.is_empty());
        assert!(filtered_tasks.iter().any(|task| task.title.contains("TUI")));
    }

    #[test]
    fn test_kanban_view_totals() {
        let kanban_view = KanbanView::new();
        let total = kanban_view.get_total_tasks();
        let completed = kanban_view.get_completed_tasks();
        
        assert!(total > 0);
        assert!(completed >= 0);
        assert!(completed <= total);
    }

    #[test]
    fn test_kanban_view_column_selection() {
        let mut kanban_view = KanbanView::new();
        assert_eq!(kanban_view.selected_column, 0);
        
        kanban_view.select_column(1);
        assert_eq!(kanban_view.selected_column, 1);
        
        // Test invalid column selection
        kanban_view.select_column(10);
        assert_eq!(kanban_view.selected_column, 1); // Should remain unchanged
    }

    #[test]
    fn test_kanban_view_task_selection() {
        let mut kanban_view = KanbanView::new();
        kanban_view.select_task_in_column(0, Some(0));
        assert_eq!(kanban_view.columns[0].selected_task, Some(0));
        
        kanban_view.select_task_in_column(0, None);
        assert_eq!(kanban_view.columns[0].selected_task, None);
    }
}
