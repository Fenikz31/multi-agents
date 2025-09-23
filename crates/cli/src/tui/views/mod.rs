//! TUI Views module

pub mod kanban;
pub mod sessions;
pub mod detail;

// Re-export views for convenience
pub use kanban::{KanbanView, KanbanColumn, KanbanSort, render_kanban_view};
pub use sessions::render_sessions_view;
pub use detail::render_detail_view;
