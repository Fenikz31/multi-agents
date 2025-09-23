//! View state management for TUI
//! 
//! Handles individual view states (Kanban, Sessions, Detail) with
//! their specific data and interactions.

use std::error::Error;
use super::{TuiState, StateTransition};
use crate::repository::{RepositoryManager};
use db::open_or_create_db;

/// Kanban view state
pub struct KanbanState {
    pub tasks: Vec<TaskItem>,
    pub selected_column: usize,
    pub selected_task: Option<usize>,
    pub filter: String,
}

/// Task item for Kanban
#[derive(Debug, Clone)]
pub struct TaskItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub assignee: Option<String>,
    pub priority: String,
}

/// Kanban column
#[derive(Debug, Clone)]
pub struct KanbanColumn {
    pub name: String,
    pub status: String,
    pub tasks: Vec<TaskItem>,
}

impl KanbanState {
    /// Create new Kanban state
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            selected_column: 0,
            selected_task: None,
            filter: String::new(),
        }
    }

    /// Load tasks from SQLite for a given project id
    pub fn load_from_db(&mut self, db_path: &str, project_id: &str) -> Result<(), Box<dyn Error>> {
        let conn = open_or_create_db(db_path)?;
        let repo = RepositoryManager::new(conn);
        let rows = repo.tasks.list_by_project(project_id)?;
        self.tasks = rows.into_iter().map(|r| TaskItem {
            id: r.id,
            title: r.title,
            status: r.status,
            assignee: None,
            priority: "medium".to_string(),
        }).collect();
        Ok(())
    }
    
    /// Get columns
    pub fn get_columns(&self) -> Vec<KanbanColumn> {
        let mut columns = vec![
            KanbanColumn {
                name: "To Do".to_string(),
                status: "todo".to_string(),
                tasks: Vec::new(),
            },
            KanbanColumn {
                name: "Doing".to_string(),
                status: "doing".to_string(),
                tasks: Vec::new(),
            },
            KanbanColumn {
                name: "Done".to_string(),
                status: "done".to_string(),
                tasks: Vec::new(),
            },
        ];
        
        // Filter tasks by status
        for task in &self.tasks {
            if self.filter.is_empty() || task.title.to_lowercase().contains(&self.filter.to_lowercase()) {
                for column in &mut columns {
                    // Treat legacy/in-flight statuses as aliases
                    let task_status = match task.status.as_str() {
                        "in_progress" => "doing",
                        other => other,
                    };
                    if column.status == task_status {
                        column.tasks.push(task.clone());
                    }
                }
            }
        }
        
        columns
    }
    
    /// Move task to different status
    pub fn move_task(&mut self, task_id: &str, new_status: &str) -> Result<(), Box<dyn Error>> {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = new_status.to_string();
            Ok(())
        } else {
            Err(format!("Task with ID '{}' not found", task_id).into())
        }
    }
    
    /// Add new task
    pub fn add_task(&mut self, title: String, assignee: Option<String>) {
        let task = TaskItem {
            id: format!("task-{}", self.tasks.len() + 1),
            title,
            status: "todo".to_string(),
            assignee,
            priority: "medium".to_string(),
        };
        self.tasks.push(task);
    }
}

impl TuiState for KanbanState {
    fn handle_input(&mut self, input: &str) -> Result<StateTransition, Box<dyn Error>> {
        match input.trim() {
            "q" | "quit" => Ok(StateTransition::Exit),
            "h" | "help" => Ok(StateTransition::Transition("help".to_string())),
            "s" => Ok(StateTransition::Transition("sessions".to_string())),
            "left" | "←" => {
                if self.selected_column > 0 {
                    self.selected_column -= 1;
                }
                Ok(StateTransition::Stay)
            }
            "right" | "→" => {
                let columns = self.get_columns();
                if self.selected_column < columns.len() - 1 {
                    self.selected_column += 1;
                }
                Ok(StateTransition::Stay)
            }
            "up" | "↑" => {
                let columns = self.get_columns();
                if let Some(column) = columns.get(self.selected_column) {
                    if let Some(selected) = self.selected_task {
                        if selected > 0 {
                            self.selected_task = Some(selected - 1);
                        }
                    } else if !column.tasks.is_empty() {
                        self.selected_task = Some(0);
                    }
                }
                Ok(StateTransition::Stay)
            }
            "down" | "↓" => {
                let columns = self.get_columns();
                if let Some(column) = columns.get(self.selected_column) {
                    if let Some(selected) = self.selected_task {
                        if selected < column.tasks.len() - 1 {
                            self.selected_task = Some(selected + 1);
                        }
                    } else if !column.tasks.is_empty() {
                        self.selected_task = Some(0);
                    }
                }
                Ok(StateTransition::Stay)
            }
            "tab" => {
                let columns = self.get_columns();
                if self.selected_column + 1 < columns.len() { self.selected_column += 1; }
                Ok(StateTransition::Stay)
            }
            "backtab" => {
                if self.selected_column > 0 { self.selected_column -= 1; }
                Ok(StateTransition::Stay)
            }
            ">" => {
                // Move selected task one step right in workflow
                if let Some(sel_idx) = self.selected_task {
                    let columns = self.get_columns();
                    if let Some(col) = columns.get(self.selected_column) {
                        if let Some(task) = col.tasks.get(sel_idx) {
                            let new_status = match task.status.as_str() {
                                "todo" => "doing",
                                "doing" => "done",
                                other => other,
                            };
                            let _ = self.move_task(&task.id, new_status);
                        }
                    }
                }
                Ok(StateTransition::Stay)
            }
            "<" => {
                // Move selected task one step left in workflow
                if let Some(sel_idx) = self.selected_task {
                    let columns = self.get_columns();
                    if let Some(col) = columns.get(self.selected_column) {
                        if let Some(task) = col.tasks.get(sel_idx) {
                            let new_status = match task.status.as_str() {
                                "done" => "doing",
                                "doing" => "todo",
                                other => other,
                            };
                            let _ = self.move_task(&task.id, new_status);
                        }
                    }
                }
                Ok(StateTransition::Stay)
            }
            "space" => {
                // Move selected task to next status
                let columns = self.get_columns();
                if let Some(column) = columns.get(self.selected_column) {
                    if let Some(selected) = self.selected_task {
                        if let Some(task) = column.tasks.get(selected) {
                            let new_status = match task.status.as_str() {
                                "todo" => "doing",
                                "doing" => "done",
                                _ => "todo",
                            };
                            self.move_task(&task.id, new_status)?;
                        }
                    }
                }
                Ok(StateTransition::Stay)
            }
            "n" | "new" => {
                // Add new task
                self.add_task("New Task".to_string(), None);
                Ok(StateTransition::Stay)
            }
            _ => {
                // Filter tasks
                self.filter = input.to_string();
                Ok(StateTransition::Stay)
            }
        }
    }
    
    fn render(&self) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        output.push_str("=== Kanban Board ===\n\n");
        
        let columns = self.get_columns();
        
        // Render column headers
        for (i, column) in columns.iter().enumerate() {
            let marker = if i == self.selected_column { "▶ " } else { "  " };
            output.push_str(&format!("{}{} ({})", marker, column.name, column.tasks.len()));
            if i < columns.len() - 1 {
                output.push_str(" | ");
            }
        }
        output.push_str("\n\n");
        
        // Render tasks
        for (i, column) in columns.iter().enumerate() {
            if i == self.selected_column {
                output.push_str(&format!("{}:\n", column.name));
                for (j, task) in column.tasks.iter().enumerate() {
                    let marker = if Some(j) == self.selected_task { "  ▶ " } else { "    " };
                    output.push_str(&format!("{}{}\n", marker, task.title));
                }
            }
        }
        
        output.push_str("\nCommands: ← → (navigate), ↑ ↓ (select), space (move), n (new), q (quit)\n");
        if !self.filter.is_empty() {
            output.push_str(&format!("Filter: {}\n", self.filter));
        }
        
        Ok(output)
    }
    
    fn state_name(&self) -> &'static str {
        "kanban"
    }
    
    fn can_transition_to(&self, target_state: &str) -> bool {
        matches!(target_state, "sessions" | "help")
    }
}

/// Sessions view state
pub struct SessionsState {
    pub sessions: Vec<SessionItem>,
    pub selected_session: Option<usize>,
    pub filter: String,
}

/// Session item for Sessions view
#[derive(Debug, Clone)]
pub struct SessionItem {
    pub id: String,
    pub agent_name: String,
    pub role: String,
    pub provider: String,
    pub status: String,
    pub duration: String,
}

impl SessionsState {
    /// Create new Sessions state
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected_session: None,
            filter: String::new(),
        }
    }
    
    /// Add session
    pub fn add_session(&mut self, session: SessionItem) {
        self.sessions.push(session);
    }
    
    /// Get filtered sessions
    pub fn get_filtered_sessions(&self) -> Vec<&SessionItem> {
        if self.filter.is_empty() {
            self.sessions.iter().collect()
        } else {
            self.sessions.iter()
                .filter(|s| s.agent_name.to_lowercase().contains(&self.filter.to_lowercase()) ||
                           s.role.to_lowercase().contains(&self.filter.to_lowercase()))
                .collect()
        }
    }
}

impl TuiState for SessionsState {
    fn handle_input(&mut self, input: &str) -> Result<StateTransition, Box<dyn Error>> {
        match input.trim() {
            "q" | "quit" => Ok(StateTransition::Exit),
            "h" | "help" => Ok(StateTransition::Transition("help".to_string())),
            "k" => Ok(StateTransition::Transition("kanban".to_string())),
            "up" | "↑" => {
                if let Some(selected) = self.selected_session {
                    if selected > 0 {
                        self.selected_session = Some(selected - 1);
                    }
                } else if !self.sessions.is_empty() {
                    self.selected_session = Some(0);
                }
                Ok(StateTransition::Stay)
            }
            "down" | "↓" => {
                let filtered = self.get_filtered_sessions();
                if let Some(selected) = self.selected_session {
                    if selected < filtered.len() - 1 {
                        self.selected_session = Some(selected + 1);
                    }
                } else if !filtered.is_empty() {
                    self.selected_session = Some(0);
                }
                Ok(StateTransition::Stay)
            }
            "enter" | "return" => {
                // Attach to selected session
                if let Some(selected) = self.selected_session {
                    let filtered = self.get_filtered_sessions();
                    if let Some(session) = filtered.get(selected) {
                        // TODO: Implement session attachment
                        return Ok(StateTransition::Error("Session attachment not implemented yet".to_string()));
                    }
                }
                Ok(StateTransition::Stay)
            }
            "s" | "start" => {
                // Start new session
                Ok(StateTransition::Error("Start session not implemented yet".to_string()))
            }
            _ => {
                // Filter sessions
                self.filter = input.to_string();
                self.selected_session = None;
                Ok(StateTransition::Stay)
            }
        }
    }
    
    fn render(&self) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        output.push_str("=== Sessions ===\n\n");
        
        let filtered = self.get_filtered_sessions();
        
        if filtered.is_empty() {
            output.push_str("No sessions found\n");
        }
        for (i, session) in filtered.iter().enumerate() {
            let marker = if Some(i) == self.selected_session { "▶ " } else { "  " };
            output.push_str(&format!("{}{}:{} ({}) - {} - {}\n", 
                marker, session.role, session.agent_name, session.provider, session.status, session.duration));
        }
        
        output.push_str("\nCommands: ↑ ↓ (navigate), enter (attach), s (start), q (quit)\n");
        if !self.filter.is_empty() {
            output.push_str(&format!("Filter: {}\n", self.filter));
        }
        
        Ok(output)
    }
    
    fn state_name(&self) -> &'static str {
        "sessions"
    }
    
    fn can_transition_to(&self, target_state: &str) -> bool {
        matches!(target_state, "kanban" | "help")
    }
}

