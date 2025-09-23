//! Kanban view implementation
//! 
//! Provides a Kanban board view with columns for ToDo, Doing, and Done tasks.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style, Stylize};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use super::super::components::{TaskCard, Task, TaskStatus, TaskPriority, render_task_card, ToastQueue, render_toasts, GlobalStatus, GlobalStateIcon, render_global_status};
use super::super::themes::{ThemePalette, Typography};

/// Kanban column data structure
#[derive(Debug, Clone)]
pub struct KanbanColumn {
    pub title: String,
    pub status: TaskStatus,
    pub tasks: Vec<Task>,
    pub selected_task: Option<usize>,
}

impl KanbanColumn {
    pub fn new(title: String, status: TaskStatus) -> Self {
        Self { title, status, tasks: Vec::new(), selected_task: None }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn remove_task(&mut self, index: usize) -> Option<Task> {
        if index < self.tasks.len() { Some(self.tasks.remove(index)) } else { None }
    }

    pub fn move_task_to(&mut self, from_index: usize, to_column: &mut KanbanColumn, to_index: Option<usize>) -> bool {
        if from_index >= self.tasks.len() { return false; }
        let task = self.tasks.remove(from_index);
        match to_index {
            Some(idx) if idx <= to_column.tasks.len() => to_column.tasks.insert(idx, task),
            _ => to_column.tasks.push(task),
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KanbanSort { Created = 0, Updated = 1, Priority = 2, Title = 3 }

#[derive(Debug, Clone)]
pub struct KanbanView {
    pub columns: Vec<KanbanColumn>,
    pub selected_column: usize,
    pub filter: String,
    pub show_completed: bool,
    pub sort_by: KanbanSort,
}

impl KanbanView {
    pub fn new() -> Self {
        let mut view = Self {
            columns: vec![
                KanbanColumn::new("To Do".to_string(), TaskStatus::Todo),
                KanbanColumn::new("Doing".to_string(), TaskStatus::Doing),
                KanbanColumn::new("Done".to_string(), TaskStatus::Done),
            ],
            selected_column: 0,
            filter: String::new(),
            show_completed: true,
            sort_by: KanbanSort::Priority,
        };
        // Seed demo tasks
        view.columns[0].tasks.push(Task { id: "1".into(), title: "Setup TUI".into(), description: None, status: TaskStatus::Todo, priority: TaskPriority::High, assignee: None, created_at: "".into(), updated_at: "".into()});
        view
    }

    pub fn move_to_column(&mut self, from_column: usize, from_task: usize, to_column: usize, to_task: Option<usize>) -> bool {
        if from_column >= self.columns.len() || to_column >= self.columns.len() { return false; }
        if from_column == to_column { return false; }

        if from_column < to_column {
            // Split before the destination column; left covers ..to_column, right starts at to_column
            let (left, right) = self.columns.split_at_mut(to_column);
            let from_col = &mut left[from_column];
            let to_col = &mut right[0];
            from_col.move_task_to(from_task, to_col, to_task)
        } else {
            // from_column > to_column: split before from_column; right starts at from_column
            let (left, right) = self.columns.split_at_mut(from_column);
            let to_col = &mut left[to_column];
            let from_col = &mut right[0];
            from_col.move_task_to(from_task, to_col, to_task)
        }
    }

    pub fn select_column(&mut self, index: usize) {
        if index < self.columns.len() { self.selected_column = index; }
    }

    pub fn select_task_in_column(&mut self, col: usize, task: Option<usize>) {
        if col < self.columns.len() {
            self.columns[col].selected_task = task;
        }
    }
}

pub fn render_kanban_view(f: &mut ratatui::Frame, area: Rect, kanban_view: &KanbanView, theme: &ThemePalette, typography: &Typography) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Board
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Header (global status)
    let status = GlobalStatus {
        project_name: "<project>".to_string(),
        view_name: "Kanban".to_string(),
        focus: "Body".to_string(),
        icon: GlobalStateIcon::Active,
        last_action: None,
    };
    render_global_status(f, chunks[0], &status, theme, typography);

    // Kanban board
    let board_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[1]);

    for (i, column) in kanban_view.columns.iter().enumerate() {
        if i < board_chunks.len() {
            render_kanban_column(f, board_chunks[i], column, i == kanban_view.selected_column, theme, typography, kanban_view);
        }
    }

    // Footer
    let footer_text = format!(
        "Use arrows to navigate, Enter to select, Filter: '{}'",
        kanban_view.filter
    );
    let footer = Paragraph::new(footer_text)
        .style(typography.caption.fg(theme.secondary))
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[2]);

    // Render toasts over the footer/body area (bottom-right)
    // For now, use an empty queue placeholder until wired with state
    let queue = ToastQueue::with_capacity(3);
    // Example (commented): queue.enqueue(Toast::new(ToastType::Success, "Saved", Some(2000)));
    render_toasts(f, chunks[1], &queue, theme, typography);
}

pub fn render_kanban_column(
    f: &mut ratatui::Frame,
    area: Rect,
    column: &KanbanColumn,
    is_selected: bool,
    theme: &ThemePalette,
    typography: &Typography,
    kanban_view: &KanbanView,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Tasks
        ])
        .split(area);

    let header_text = format!("{} {} ({})", column.status.icon(), column.title, column.tasks.len());
    let header_style = if is_selected {
        typography.subtitle.fg(theme.primary).add_modifier(Modifier::BOLD)
    } else {
        typography.subtitle.fg(theme.text)
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
            let card = TaskCard::new((*task).clone()).with_selection(is_selected).with_focus(is_selected);
            let text = format!("{} {} {}", card.task.status.icon(), card.task.priority.icon(), card.task.title);
            let style = if is_selected {
                typography.body.fg(theme.primary).add_modifier(Modifier::REVERSED)
            } else {
                typography.body.fg(theme.text)
            };
            ListItem::new(text).style(style)
        })
        .collect();
    
    let list = List::new(task_items)
        .block(Block::default().borders(Borders::ALL).border_style(theme.secondary))
        .highlight_style(typography.body.fg(theme.primary).add_modifier(Modifier::REVERSED));
    
    let mut list_state = ListState::default();
    list_state.select(column.selected_task);
    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

impl KanbanView {
    pub fn get_filtered_tasks(&self, _col_index: usize) -> Vec<&Task> {
        // For now, return all tasks of the column index
        let mut tasks: Vec<&Task> = Vec::new();
        for (i, col) in self.columns.iter().enumerate() {
            if i == _col_index { tasks.extend(col.tasks.iter()); }
        }
        if self.filter.is_empty() { tasks } else { tasks.into_iter().filter(|t| t.title.contains(&self.filter)).collect() }
    }

    pub fn get_total_tasks(&self) -> usize { self.columns.iter().map(|c| c.tasks.len()).sum() }
    pub fn get_completed_tasks(&self) -> usize { self.columns.iter().filter(|c| matches!(c.status, TaskStatus::Done)).map(|c| c.tasks.len()).sum() }
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
