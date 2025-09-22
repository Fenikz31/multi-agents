//! TUI Views module

pub mod kanban;

// Re-export views for convenience
pub use kanban::{KanbanView, KanbanColumn, KanbanSort, render_kanban_view};
