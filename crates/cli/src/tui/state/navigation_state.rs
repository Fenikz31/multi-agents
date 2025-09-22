//! Navigation state management for TUI
//! 
//! Handles navigation between different views and application states.

use std::error::Error;
use super::{TuiState, StateTransition};

/// Help state for showing help information
pub struct HelpState {
    pub current_section: usize,
    pub sections: Vec<HelpSection>,
}

/// Help section
#[derive(Debug, Clone)]
pub struct HelpSection {
    pub title: String,
    pub content: String,
}

impl HelpState {
    /// Create new help state
    pub fn new() -> Self {
        Self {
            current_section: 0,
            sections: vec![
                HelpSection {
                    title: "General Commands".to_string(),
                    content: "q, quit, exit - Quit application\nh, help - Show this help\np, project - Select project".to_string(),
                },
                HelpSection {
                    title: "Navigation".to_string(),
                    content: "Arrow keys - Navigate\nTab - Switch between views\nEnter - Select/Activate".to_string(),
                },
                HelpSection {
                    title: "Kanban View".to_string(),
                    content: "← → - Navigate columns\n↑ ↓ - Select tasks\nSpace - Move task to next status\nn - New task".to_string(),
                },
                HelpSection {
                    title: "Sessions View".to_string(),
                    content: "↑ ↓ - Navigate sessions\nEnter - Attach to session\ns - Start new session".to_string(),
                },
                HelpSection {
                    title: "Detail View".to_string(),
                    content: "↑ ↓ - Scroll logs\nf - Filter logs\nr - Refresh\nCtrl+F - Search".to_string(),
                },
            ],
        }
    }
    
    /// Get current section
    pub fn get_current_section(&self) -> Option<&HelpSection> {
        self.sections.get(self.current_section)
    }
}

impl TuiState for HelpState {
    fn handle_input(&mut self, input: &str) -> Result<StateTransition, Box<dyn Error>> {
        match input.trim() {
            "q" | "quit" | "exit" => Ok(StateTransition::Exit),
            "b" | "back" => Ok(StateTransition::Transition("project_select".to_string())),
            "up" | "↑" => {
                if self.current_section > 0 {
                    self.current_section -= 1;
                }
                Ok(StateTransition::Stay)
            }
            "down" | "↓" => {
                if self.current_section < self.sections.len() - 1 {
                    self.current_section += 1;
                }
                Ok(StateTransition::Stay)
            }
            "1" => {
                self.current_section = 0;
                Ok(StateTransition::Stay)
            }
            "2" => {
                self.current_section = 1;
                Ok(StateTransition::Stay)
            }
            "3" => {
                self.current_section = 2;
                Ok(StateTransition::Stay)
            }
            "4" => {
                self.current_section = 3;
                Ok(StateTransition::Stay)
            }
            "5" => {
                self.current_section = 4;
                Ok(StateTransition::Stay)
            }
            _ => Ok(StateTransition::Stay),
        }
    }
    
    fn render(&self) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        output.push_str("=== Help ===\n\n");
        
        if let Some(section) = self.get_current_section() {
            output.push_str(&format!("{}. {}\n\n", self.current_section + 1, section.title));
            output.push_str(&section.content);
        }
        
        output.push_str("\n\nNavigation:\n");
        output.push_str("  ↑ ↓ - Navigate sections\n");
        output.push_str("  1-5 - Jump to section\n");
        output.push_str("  b, back - Return to main\n");
        output.push_str("  q, quit - Exit application\n");
        
        Ok(output)
    }
    
    fn state_name(&self) -> &'static str {
        "help"
    }
    
    fn can_transition_to(&self, target_state: &str) -> bool {
        matches!(target_state, "kanban" | "sessions" | "project_select")
    }
}

/// Project selection state
pub struct ProjectSelectState {
    pub projects: Vec<ProjectItem>,
    pub selected_project: Option<usize>,
    pub filter: String,
}

/// Project item
#[derive(Debug, Clone)]
pub struct ProjectItem {
    pub id: String,
    pub name: String,
    pub agent_count: usize,
    pub session_count: usize,
    pub last_activity: String,
}

impl ProjectSelectState {
    /// Create new project selection state
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
            selected_project: None,
            filter: String::new(),
        }
    }
    
    /// Add project
    pub fn add_project(&mut self, project: ProjectItem) {
        self.projects.push(project);
    }
    
    /// Get filtered projects
    pub fn get_filtered_projects(&self) -> Vec<&ProjectItem> {
        if self.filter.is_empty() {
            self.projects.iter().collect()
        } else {
            self.projects.iter()
                .filter(|p| p.name.to_lowercase().contains(&self.filter.to_lowercase()))
                .collect()
        }
    }
    
    /// Get selected project
    pub fn get_selected_project(&self) -> Option<&ProjectItem> {
        if let Some(selected) = self.selected_project {
            self.get_filtered_projects().get(selected).copied()
        } else {
            None
        }
    }
}

impl TuiState for ProjectSelectState {
    fn handle_input(&mut self, input: &str) -> Result<StateTransition, Box<dyn Error>> {
        match input.trim() {
            "q" | "quit" | "exit" => Ok(StateTransition::Exit),
            "b" | "back" => Ok(StateTransition::Transition("project_select".to_string())),
            "h" | "help" => Ok(StateTransition::Transition("help".to_string())),
            "k" => Ok(StateTransition::Transition("kanban".to_string())),
            "s" => Ok(StateTransition::Transition("sessions".to_string())),
            "up" | "↑" => {
                if let Some(selected) = self.selected_project {
                    if selected > 0 {
                        self.selected_project = Some(selected - 1);
                    }
                } else if !self.projects.is_empty() {
                    self.selected_project = Some(0);
                }
                Ok(StateTransition::Stay)
            }
            "down" | "↓" => {
                let filtered = self.get_filtered_projects();
                if let Some(selected) = self.selected_project {
                    if selected < filtered.len() - 1 {
                        self.selected_project = Some(selected + 1);
                    }
                } else if !filtered.is_empty() {
                    self.selected_project = Some(0);
                }
                Ok(StateTransition::Stay)
            }
            "enter" | "return" => {
                if let Some(_project) = self.get_selected_project() {
                    Ok(StateTransition::Transition("kanban".to_string()))
                } else {
                    Ok(StateTransition::Error("No project selected".to_string()))
                }
            }
            "n" | "new" => {
                Ok(StateTransition::Error("Create project not implemented yet".to_string()))
            }
            _ => {
                self.filter = input.to_string();
                self.selected_project = None;
                Ok(StateTransition::Stay)
            }
        }
    }
    
    fn render(&self) -> Result<String, Box<dyn Error>> {
        let mut output = String::new();
        output.push_str("=== Project Selection ===\n\n");
        
        let filtered = self.get_filtered_projects();
        
        if filtered.is_empty() {
            output.push_str("No projects found.\n");
            output.push_str("Use 'n' to create a new project.\n");
        } else {
            for (i, project) in filtered.iter().enumerate() {
                let marker = if Some(i) == self.selected_project { "▶ " } else { "  " };
                output.push_str(&format!("{}{} ({} agents, {} sessions) - {}\n", 
                    marker, project.name, project.agent_count, project.session_count, project.last_activity));
            }
        }
        
        output.push_str("\nCommands: ↑ ↓ (navigate), enter (select), n (new), h (help), k (kanban), s (sessions), q (quit)\n");
        if !self.filter.is_empty() {
            output.push_str(&format!("Filter: {}\n", self.filter));
        }
        
        Ok(output)
    }
    
    fn state_name(&self) -> &'static str {
        "project_select"
    }
    
    fn can_transition_to(&self, target_state: &str) -> bool {
        matches!(target_state, "kanban" | "help" | "sessions")
    }
}

