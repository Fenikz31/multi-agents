//! TUI State management using State Pattern
//! 
//! This module implements the State Pattern for managing TUI application states,
//! providing a clean separation between different UI modes and their behaviors.

pub mod view_state;
pub mod navigation_state;
pub mod selection_store;

use std::error::Error;

/// Generic state trait for TUI states
pub trait TuiState {
    /// Handle user input for this state
    fn handle_input(&mut self, input: &str) -> Result<StateTransition, Box<dyn Error>>;
    
    /// Render the current state
    fn render(&self) -> Result<String, Box<dyn Error>>;
    
    /// Get the state name
    fn state_name(&self) -> &'static str;
    
    /// Check if this state can transition to another state
    fn can_transition_to(&self, target_state: &str) -> bool;
}

/// State transition result
#[derive(Debug, Clone)]
pub enum StateTransition {
    /// Stay in current state
    Stay,
    /// Transition to another state
    Transition(String),
    /// Exit the application
    Exit,
    /// Show error message
    Error(String),
}

/// State manager for coordinating state transitions
pub struct StateManager {
    current_state: String,
    states: std::collections::HashMap<String, Box<dyn TuiState>>,
}

impl StateManager {
    /// Create new state manager
    pub fn new() -> Self {
        Self {
            current_state: "initial".to_string(),
            states: std::collections::HashMap::new(),
        }
    }
    
    /// Add a state to the manager
    pub fn add_state(&mut self, name: String, state: Box<dyn TuiState>) {
        self.states.insert(name, state);
    }
    
    /// Set the current state
    pub fn set_current_state(&mut self, state_name: String) -> Result<(), Box<dyn Error>> {
        if self.states.contains_key(&state_name) {
            self.current_state = state_name;
            Ok(())
        } else {
            Err(format!("State '{}' not found", state_name).into())
        }
    }
    
    /// Get current state name
    pub fn current_state_name(&self) -> &str {
        &self.current_state
    }
    
    /// Handle input in current state
    pub fn handle_input(&mut self, input: &str) -> Result<StateTransition, Box<dyn Error>> {
        if let Some(state) = self.states.get_mut(&self.current_state) {
            state.handle_input(input)
        } else {
            Err(format!("Current state '{}' not found", self.current_state).into())
        }
    }
    
    /// Render current state
    pub fn render(&self) -> Result<String, Box<dyn Error>> {
        if let Some(state) = self.states.get(&self.current_state) {
            state.render()
        } else {
            Err(format!("Current state '{}' not found", self.current_state).into())
        }
    }
    
    /// Process state transition
    pub fn process_transition(&mut self, transition: StateTransition) -> Result<(), Box<dyn Error>> {
        match transition {
            StateTransition::Stay => Ok(()),
            StateTransition::Transition(target_state) => {
                if let Some(current_state) = self.states.get(&self.current_state) {
                    if current_state.can_transition_to(&target_state) {
                        // If transitioning to kanban, attempt to load tasks for selected project
                        let res = self.set_current_state(target_state.clone());
                        if res.is_ok() && target_state == "kanban" {
                            if let Some(project_id) = selection_store::get_project_id() {
                                if let Some(state) = self.states.get_mut("kanban") {
                                    if let Some(kanban) = state.downcast_mut::<view_state::KanbanState>() {
                                        let _ = kanban.load_from_db("./data/multi-agents.sqlite3", &project_id);
                                    }
                                }
                            }
                        }
                        res
                    } else {
                        Err(format!("Cannot transition from '{}' to '{}'", self.current_state, target_state).into())
                    }
                } else {
                    Err(format!("Current state '{}' not found", self.current_state).into())
                }
            }
            StateTransition::Exit => {
                self.current_state = "initial".to_string();
                Ok(())
            }
            StateTransition::Error(msg) => {
                Err(msg.into())
            }
        }
    }
}

