//! TUI (Terminal User Interface) module
//! 
//! This module provides the terminal user interface for the multi-agents CLI,
//! implementing the TUI functionality using ratatui.

pub mod state;
pub mod app;
pub mod views;
pub mod components;
pub mod navigation;
pub mod themes;

use std::error::Error;
use state::{StateManager, TuiState, StateTransition};

/// Main TUI application
pub struct TuiApp {
    state_manager: StateManager,
    running: bool,
}

impl TuiApp {
    /// Create new TUI application
    pub fn new() -> Self {
        Self {
            state_manager: StateManager::new(),
            running: true,
        }
    }
    
    /// Initialize the TUI application
    pub fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        // Add initial states
        self.state_manager.add_state("help".to_string(), Box::new(state::navigation_state::HelpState::new()));
        self.state_manager.add_state("project_select".to_string(), Box::new(state::navigation_state::ProjectSelectState::new()));
        self.state_manager.add_state("kanban".to_string(), Box::new(state::view_state::KanbanState::new()));
        self.state_manager.add_state("sessions".to_string(), Box::new(state::view_state::SessionsState::new()));
        
        // Set initial state
        self.state_manager.set_current_state("project_select".to_string())?;
        
        Ok(())
    }
    
    /// Run the TUI application
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.initialize()?;
        
        while self.running {
            // Render current state
            let output = self.state_manager.render()?;
            println!("{}", output);
            
            // Get user input
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            // Handle input
            let transition = self.state_manager.handle_input(&input)?;
            
            // Process transition
            match transition {
                StateTransition::Exit => {
                    self.running = false;
                }
                _ => {
                    self.state_manager.process_transition(transition)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Stop the TUI application
    pub fn stop(&mut self) {
        self.running = false;
    }
}

/// TUI error types
#[derive(Debug)]
pub enum TuiError {
    StateNotFound(String),
    InvalidTransition(String),
    RenderError(String),
    InputError(String),
}

impl std::fmt::Display for TuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TuiError::StateNotFound(state) => write!(f, "State not found: {}", state),
            TuiError::InvalidTransition(transition) => write!(f, "Invalid transition: {}", transition),
            TuiError::RenderError(msg) => write!(f, "Render error: {}", msg),
            TuiError::InputError(msg) => write!(f, "Input error: {}", msg),
        }
    }
}

impl Error for TuiError {}

