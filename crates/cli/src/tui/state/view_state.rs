//! View state management for TUI
//! 
//! Handles individual view states (Kanban, Sessions, Detail) with
//! their specific data and interactions.

use std::error::Error;
use super::{TuiState, StateTransition, StateContext};
use crate::repository::{RepositoryManager};
use db::open_or_create_db;

/// Kanban view state
pub struct KanbanState {
    pub tasks: Vec<TaskItem>,
    pub selected_column: usize,
    pub selected_task: Option<usize>,
    pub filter: String,
    // cache
    cached_columns: Option<Box<[KanbanColumn]>>,
    // simple pagination for visible tasks in current column
    pub col_page_size: usize,
    pub col_page_index: usize,
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
            cached_columns: None,
            col_page_size: 50,
            col_page_index: 0,
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
        self.cached_columns = None; // invalidate cache
        self.ensure_columns_cache();
        Ok(())
    }

    fn build_columns(&self) -> Vec<KanbanColumn> {
        let mut columns = vec![
            KanbanColumn { name: "To Do".to_string(), status: "todo".to_string(), tasks: Vec::new() },
            KanbanColumn { name: "Doing".to_string(), status: "doing".to_string(), tasks: Vec::new() },
            KanbanColumn { name: "Done".to_string(), status: "done".to_string(), tasks: Vec::new() },
        ];
        for task in &self.tasks {
            if self.filter.is_empty() || task.title.to_lowercase().contains(&self.filter.to_lowercase()) {
                for column in &mut columns {
                    let task_status = match task.status.as_str() { "in_progress" => "doing", other => other };
                    if column.status == task_status { column.tasks.push(task.clone()); }
                }
            }
        }
        columns
    }

    fn ensure_columns_cache(&mut self) {
        let columns = self.build_columns();
        self.cached_columns = Some(columns.into_boxed_slice());
    }

    /// Get columns
    pub fn get_columns(&self) -> Vec<KanbanColumn> {
        if let Some(cols) = &self.cached_columns { return cols.to_vec(); }
        let columns = self.build_columns();
        // Apply simple pagination on the selected column
        let mut columns_paginated = columns.clone();
        if let Some(col) = columns_paginated.get_mut(self.selected_column) {
            let start = self.col_page_index.saturating_mul(self.col_page_size);
            let end = (start + self.col_page_size).min(col.tasks.len());
            if start < end { col.tasks = col.tasks[start..end].to_vec(); } else { col.tasks.clear(); }
        }
        columns_paginated
    }
    
    /// Move task to different status
    pub fn move_task(&mut self, task_id: &str, new_status: &str) -> Result<(), Box<dyn Error>> {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.status = new_status.to_string();
            self.cached_columns = None; // invalidate cache
            self.ensure_columns_cache();
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
        self.cached_columns = None; // invalidate cache
        self.ensure_columns_cache();
    }
}

impl TuiState for KanbanState {
    fn on_enter(&mut self, ctx: &StateContext) -> Result<(), Box<dyn Error>> {
        if let Some(project_id) = &ctx.selected_project_id {
            let _ = self.load_from_db("./data/multi-agents.sqlite3", project_id);
        }
        Ok(())
    }
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
            "home" => {
                let columns = self.get_columns();
                if let Some(column) = columns.get(self.selected_column) {
                    if !column.tasks.is_empty() {
                        self.selected_task = Some(0);
                    }
                }
                Ok(StateTransition::Stay)
            }
            "end" => {
                let columns = self.get_columns();
                if let Some(column) = columns.get(self.selected_column) {
                    if !column.tasks.is_empty() {
                        self.selected_task = Some(column.tasks.len().saturating_sub(1));
                    }
                }
                Ok(StateTransition::Stay)
            }
            "pageup" => {
                let columns = self.get_columns();
                if let Some(column) = columns.get(self.selected_column) {
                    if let Some(selected) = self.selected_task {
                        let new_idx = selected.saturating_sub(5);
                        self.selected_task = Some(new_idx);
                    } else if !column.tasks.is_empty() {
                        self.selected_task = Some(0);
                    }
                }
                Ok(StateTransition::Stay)
            }
            "pagedown" => {
                let columns = self.get_columns();
                if let Some(column) = columns.get(self.selected_column) {
                    if let Some(selected) = self.selected_task {
                        let max_last = column.tasks.len().saturating_sub(1);
                        let new_idx = (selected + 5).min(max_last);
                        self.selected_task = Some(new_idx);
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
                    // Resolve current selected task id before mutation
                    let current_columns = self.get_columns();
                    if let Some(col) = current_columns.get(self.selected_column) {
                        if let Some(task) = col.tasks.get(sel_idx) {
                            let new_status = match task.status.as_str() {
                                "todo" => "doing",
                                "doing" => "done",
                                other => other,
                            };
                            let _ = self.move_task(&task.id, new_status);
                            // Move focus to the next column since task moved right
                            let cols_len = current_columns.len();
                            if self.selected_column + 1 < cols_len { self.selected_column += 1; }
                            // Reset selection to first item in the new column
                            self.selected_task = Some(0);
                        }
                    }
                }
                Ok(StateTransition::Stay)
            }
            "<" => {
                // Move selected task one step left in workflow
                if let Some(sel_idx) = self.selected_task {
                    // Resolve current selected task id before mutation
                    let current_columns = self.get_columns();
                    if let Some(col) = current_columns.get(self.selected_column) {
                        if let Some(task) = col.tasks.get(sel_idx) {
                            let new_status = match task.status.as_str() {
                                "done" => "doing",
                                "doing" => "todo",
                                other => other,
                            };
                            let _ = self.move_task(&task.id, new_status);
                            // Move focus to the previous column since task moved left
                            if self.selected_column > 0 { self.selected_column -= 1; }
                            // Reset selection to first item in the new column
                            self.selected_task = Some(0);
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
    pub sort_by_agent: bool,
    // caching & lazy display
    cache_filter: String,
    cache_sort_by_agent: bool,
    cache_indices: Option<Vec<usize>>, // indices into sessions matching current filter/sort
    page_size: usize,
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
            sort_by_agent: false,
            cache_filter: String::new(),
            cache_sort_by_agent: false,
            cache_indices: None,
            page_size: 200,
        }
    }
    /// Load sessions from SQLite
    pub fn load_from_db_with_filters(&mut self, db_path: &str, project_id: Option<String>, agent_id: Option<String>) -> Result<(), Box<dyn Error>> {
        let conn = db::open_or_create_db(db_path)?;
        let mut sql = String::from("SELECT id, agent_id, provider, status, created_at FROM sessions");
        let mut clauses: Vec<&str> = Vec::new();
        if project_id.is_some() { clauses.push("project_id = ?1"); }
        if agent_id.is_some() { clauses.push("agent_id = ?2"); }
        if !clauses.is_empty() { sql.push_str(" WHERE "); sql.push_str(&clauses.join(" AND ")); }
        sql.push_str(" ORDER BY created_at DESC");

        let mut stmt = conn.prepare(&sql)?;

        let mut collected: Vec<(String, String, String, String, String)> = Vec::new();
        if let (Some(p), Some(a)) = (project_id.as_ref(), agent_id.as_ref()) {
            let mapped = stmt.query_map((p, a), |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?)))?;
            for r in mapped { collected.push(r?); }
        } else if let (Some(p), None) = (project_id.as_ref(), agent_id.as_ref()) {
            let mapped = stmt.query_map([p], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?)))?;
            for r in mapped { collected.push(r?); }
        } else if let (None, Some(a)) = (project_id.as_ref(), agent_id.as_ref()) {
            let mapped = stmt.query_map([a], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?)))?;
            for r in mapped { collected.push(r?); }
        } else {
            let mapped = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?, row.get::<_, String>(3)?, row.get::<_, String>(4)?)))?;
            for r in mapped { collected.push(r?); }
        }

        self.sessions.clear();
        for (id, agent_id, provider, status, created_at) in collected.into_iter() {
            self.sessions.push(SessionItem { id, agent_name: agent_id, role: String::new(), provider, status, duration: created_at });
        }
        // Invalidate cache on data reload
        self.cache_indices = None;
        Ok(())
    }
    
    /// Add session
    pub fn add_session(&mut self, session: SessionItem) {
        self.sessions.push(session);
        self.cache_indices = None; // invalidate cache
    }
    
    /// Get filtered sessions
    pub fn get_filtered_sessions(&self) -> Vec<&SessionItem> {
        // Prefer cached indices when valid
        if let Some(indices) = &self.cache_indices {
            if self.cache_filter == self.filter && self.cache_sort_by_agent == self.sort_by_agent {
                return indices.iter().take(self.page_size).map(|&i| &self.sessions[i]).collect();
            }
        }
        // Fallback compute without mutating state
        let mut idx: Vec<usize> = if self.filter.is_empty() {
            (0..self.sessions.len()).collect()
        } else {
            let fl = self.filter.to_lowercase();
            self.sessions
                .iter()
                .enumerate()
                .filter(|(_, s)| s.agent_name.to_lowercase().contains(&fl) || s.role.to_lowercase().contains(&fl))
                .map(|(i, _)| i)
                .collect()
        };
        if self.sort_by_agent {
            idx.sort_by(|&ia, &ib| self.sessions[ia].agent_name.cmp(&self.sessions[ib].agent_name));
        } else {
            idx.sort_by(|&ia, &ib| self.sessions[ib].duration.cmp(&self.sessions[ia].duration));
        }
        idx.into_iter().take(self.page_size).map(|i| &self.sessions[i]).collect()
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
                let len = self.get_filtered_sessions().len();
                if let Some(selected) = self.selected_session {
                    if selected + 1 < len {
                        self.selected_session = Some(selected + 1);
                    }
                } else if len > 0 {
                    self.selected_session = Some(0);
                }
                Ok(StateTransition::Stay)
            }
            "home" => {
                let filtered = self.get_filtered_sessions();
                if !filtered.is_empty() {
                    self.selected_session = Some(0);
                }
                Ok(StateTransition::Stay)
            }
            "end" => {
                let len = self.get_filtered_sessions().len();
                if len > 0 {
                    self.selected_session = Some(len.saturating_sub(1));
                }
                Ok(StateTransition::Stay)
            }
            "pageup" => {
                if let Some(selected) = self.selected_session {
                    self.selected_session = Some(selected.saturating_sub(5));
                } else if !self.sessions.is_empty() {
                    self.selected_session = Some(0);
                }
                Ok(StateTransition::Stay)
            }
            "pagedown" => {
                let filtered = self.get_filtered_sessions();
                if let Some(selected) = self.selected_session {
                    let last = filtered.len().saturating_sub(1);
                    self.selected_session = Some((selected + 5).min(last));
                } else if !filtered.is_empty() {
                    self.selected_session = Some(0);
                }
                Ok(StateTransition::Stay)
            }
            "enter" | "return" => {
                // Attach to selected session
                if let Some(selected) = self.selected_session {
                    let filtered = self.get_filtered_sessions();
                    if let Some(_session) = filtered.get(selected) {
                        // TODO: Implement session attachment
                        return Ok(StateTransition::Error("Session attachment not implemented yet".to_string()));
                    }
                }
                Ok(StateTransition::Stay)
            }
            "t" => {
                // Toggle sort
                self.sort_by_agent = !self.sort_by_agent;
                Ok(StateTransition::Stay)
            }
            "r" => Ok(StateTransition::Error("Resume session not implemented yet".to_string())),
            "X" => Ok(StateTransition::Error("Stop session not implemented yet".to_string())),
            "S" => Ok(StateTransition::Error("Start session not implemented yet".to_string())),
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

