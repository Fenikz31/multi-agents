//! Minimal global selection store for current project id
//! Used to pass selected project from ProjectSelectState to KanbanState.

use std::sync::{Mutex, OnceLock};

static PROJECT_ID: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn store() -> &'static Mutex<Option<String>> {
    PROJECT_ID.get_or_init(|| Mutex::new(None))
}

/// Set current project id
pub fn set_project_id(project_id: String) {
    if let Ok(mut slot) = store().lock() {
        *slot = Some(project_id);
    }
}

/// Get current project id
pub fn get_project_id() -> Option<String> {
    store().lock().ok().and_then(|g| g.clone())
}


