//! Tests unitaires pour les états TUI
//! 
//! Tests pour KanbanState, SessionsState, HelpState, ProjectSelectState et StateManager

use crate::tui::state::{
    TuiState, StateTransition, StateManager, StateContext,
    view_state::{KanbanState, TaskItem, KanbanColumn, SessionsState, SessionItem},
    navigation_state::{HelpState, ProjectSelectState, ProjectItem},
    selection_store
};

#[cfg(test)]
mod kanban_state_tests {
    use super::*;

    #[test]
    fn test_kanban_state_creation() {
        let state = KanbanState::new();
        
        assert!(state.tasks.is_empty());
        assert_eq!(state.selected_column, 0);
        assert!(state.selected_task.is_none());
        assert!(state.filter.is_empty());
        assert_eq!(state.col_page_size, 50);
        assert_eq!(state.col_page_index, 0);
    }

    #[test]
    fn test_kanban_state_add_task() {
        let mut state = KanbanState::new();
        
        state.add_task("Test Task".to_string(), Some("user1".to_string()));
        
        assert_eq!(state.tasks.len(), 1);
        assert_eq!(state.tasks[0].title, "Test Task");
        assert_eq!(state.tasks[0].assignee, Some("user1".to_string()));
        assert_eq!(state.tasks[0].status, "todo");
        assert_eq!(state.tasks[0].priority, "medium");
    }

    #[test]
    fn test_kanban_state_move_task() {
        let mut state = KanbanState::new();
        
        state.add_task("Test Task".to_string(), None);
        let task_id = state.tasks[0].id.clone();
        
        // Move task to doing status
        let result = state.move_task(&task_id, "doing");
        assert!(result.is_ok());
        assert_eq!(state.tasks[0].status, "doing");
        
        // Try to move non-existent task
        let result = state.move_task("non-existent", "done");
        assert!(result.is_err());
    }

    #[test]
    fn test_kanban_state_build_columns() {
        let mut state = KanbanState::new();
        
        // Create tasks manually with different statuses
        state.tasks.push(TaskItem {
            id: "task-1".to_string(),
            title: "Task 1".to_string(),
            status: "todo".to_string(),
            assignee: None,
            priority: "medium".to_string(),
        });
        
        state.tasks.push(TaskItem {
            id: "task-2".to_string(),
            title: "Task 2".to_string(),
            status: "in_progress".to_string(),
            assignee: None,
            priority: "medium".to_string(),
        });
        
        // Get columns (will build cache)
        let columns = state.get_columns();
        
        assert_eq!(columns.len(), 3);
        assert_eq!(columns[0].name, "To Do");
        assert_eq!(columns[1].name, "Doing");
        assert_eq!(columns[2].name, "Done");
        
        assert_eq!(columns[0].tasks.len(), 1);
        assert_eq!(columns[1].tasks.len(), 1);
        assert_eq!(columns[2].tasks.len(), 0);
    }

    #[test]
    fn test_kanban_state_handle_input_navigation() {
        let mut state = KanbanState::new();
        state.add_task("Task 1".to_string(), None);
        state.add_task("Task 2".to_string(), None);
        
        // Test left navigation
        let result = state.handle_input("left");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        
        // Test right navigation
        let result = state.handle_input("right");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        
        // Test up navigation
        let result = state.handle_input("up");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        
        // Test down navigation
        let result = state.handle_input("down");
        assert!(matches!(result, Ok(StateTransition::Stay)));
    }

    #[test]
    fn test_kanban_state_handle_input_actions() {
        let mut state = KanbanState::new();
        state.add_task("Test Task".to_string(), None);
        
        // Test new task
        let result = state.handle_input("n");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        assert_eq!(state.tasks.len(), 2);
        
        // Test quit
        let result = state.handle_input("q");
        assert!(matches!(result, Ok(StateTransition::Exit)));
        
        // Test help
        let result = state.handle_input("h");
        assert!(matches!(result, Ok(StateTransition::Transition(target)) if target == "help"));
    }

    #[test]
    fn test_kanban_state_render() {
        let mut state = KanbanState::new();
        state.add_task("Test Task".to_string(), None);
        
        let result = state.render();
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("=== Kanban Board ==="));
        assert!(output.contains("To Do"));
        assert!(output.contains("Doing"));
        assert!(output.contains("Done"));
        assert!(output.contains("Test Task"));
    }

    #[test]
    fn test_kanban_state_state_name() {
        let state = KanbanState::new();
        assert_eq!(state.state_name(), "kanban");
    }

    #[test]
    fn test_kanban_state_can_transition_to() {
        let state = KanbanState::new();
        assert!(state.can_transition_to("sessions"));
        assert!(state.can_transition_to("help"));
        assert!(!state.can_transition_to("invalid"));
    }
}

#[cfg(test)]
mod sessions_state_tests {
    use super::*;

    #[test]
    fn test_sessions_state_creation() {
        let state = SessionsState::new();
        
        assert!(state.sessions.is_empty());
        assert!(state.selected_session.is_none());
        assert!(state.filter.is_empty());
        assert!(!state.sort_by_agent);
    }

    #[test]
    fn test_sessions_state_add_session() {
        let mut state = SessionsState::new();
        
        let session = SessionItem {
            id: "session1".to_string(),
            agent_name: "agent1".to_string(),
            role: "backend".to_string(),
            provider: "claude".to_string(),
            status: "active".to_string(),
            duration: "2h 30m".to_string(),
        };
        
        state.add_session(session);
        
        assert_eq!(state.sessions.len(), 1);
        assert_eq!(state.sessions[0].id, "session1");
        assert_eq!(state.sessions[0].agent_name, "agent1");
        assert_eq!(state.sessions[0].role, "backend");
    }

    #[test]
    fn test_sessions_state_get_filtered_sessions() {
        let mut state = SessionsState::new();
        
        let session1 = SessionItem {
            id: "session1".to_string(),
            agent_name: "agent1".to_string(),
            role: "backend".to_string(),
            provider: "claude".to_string(),
            status: "active".to_string(),
            duration: "2h 30m".to_string(),
        };
        
        let session2 = SessionItem {
            id: "session2".to_string(),
            agent_name: "agent2".to_string(),
            role: "frontend".to_string(),
            provider: "gemini".to_string(),
            status: "idle".to_string(),
            duration: "1h 15m".to_string(),
        };
        
        state.add_session(session1);
        state.add_session(session2);
        
        // Test without filter
        let filtered = state.get_filtered_sessions();
        assert_eq!(filtered.len(), 2);
        
        // Test with filter
        state.filter = "agent1".to_string();
        let filtered = state.get_filtered_sessions();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].agent_name, "agent1");
    }

    #[test]
    fn test_sessions_state_handle_input_navigation() {
        let mut state = SessionsState::new();
        
        let session = SessionItem {
            id: "session1".to_string(),
            agent_name: "agent1".to_string(),
            role: "backend".to_string(),
            provider: "claude".to_string(),
            status: "active".to_string(),
            duration: "2h 30m".to_string(),
        };
        
        state.add_session(session);
        
        // Test up navigation
        let result = state.handle_input("up");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        
        // Test down navigation
        let result = state.handle_input("down");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        
        // Test home
        let result = state.handle_input("home");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        
        // Test end
        let result = state.handle_input("end");
        assert!(matches!(result, Ok(StateTransition::Stay)));
    }

    #[test]
    fn test_sessions_state_handle_input_actions() {
        let mut state = SessionsState::new();
        
        // Test quit
        let result = state.handle_input("q");
        assert!(matches!(result, Ok(StateTransition::Exit)));
        
        // Test help
        let result = state.handle_input("h");
        assert!(matches!(result, Ok(StateTransition::Transition(target)) if target == "help"));
        
        // Test kanban
        let result = state.handle_input("k");
        assert!(matches!(result, Ok(StateTransition::Transition(target)) if target == "kanban"));
        
        // Test sort toggle
        let result = state.handle_input("t");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        assert!(state.sort_by_agent);
    }

    #[test]
    fn test_sessions_state_render() {
        let mut state = SessionsState::new();
        
        let session = SessionItem {
            id: "session1".to_string(),
            agent_name: "agent1".to_string(),
            role: "backend".to_string(),
            provider: "claude".to_string(),
            status: "active".to_string(),
            duration: "2h 30m".to_string(),
        };
        
        state.add_session(session);
        
        let result = state.render();
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("=== Sessions ==="));
        assert!(output.contains("agent1"));
        assert!(output.contains("backend"));
        assert!(output.contains("claude"));
    }

    #[test]
    fn test_sessions_state_state_name() {
        let state = SessionsState::new();
        assert_eq!(state.state_name(), "sessions");
    }

    #[test]
    fn test_sessions_state_can_transition_to() {
        let state = SessionsState::new();
        assert!(state.can_transition_to("kanban"));
        assert!(state.can_transition_to("help"));
        assert!(!state.can_transition_to("invalid"));
    }
}

#[cfg(test)]
mod help_state_tests {
    use super::*;

    #[test]
    fn test_help_state_creation() {
        let state = HelpState::new();
        
        assert_eq!(state.current_section, 0);
        assert_eq!(state.sections.len(), 5);
        assert_eq!(state.sections[0].title, "General / Global");
        assert_eq!(state.sections[1].title, "Navigation");
        assert_eq!(state.sections[2].title, "Kanban View");
        assert_eq!(state.sections[3].title, "Sessions View");
        assert_eq!(state.sections[4].title, "Detail View");
    }

    #[test]
    fn test_help_state_get_current_section() {
        let state = HelpState::new();
        
        let section = state.get_current_section();
        assert!(section.is_some());
        assert_eq!(section.unwrap().title, "General / Global");
    }

    #[test]
    fn test_help_state_handle_input_navigation() {
        let mut state = HelpState::new();
        
        // Test up navigation
        let result = state.handle_input("up");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        assert_eq!(state.current_section, 0); // Should stay at 0
        
        // Test down navigation
        let result = state.handle_input("down");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        assert_eq!(state.current_section, 1);
        
        // Test direct section navigation
        let result = state.handle_input("3");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        assert_eq!(state.current_section, 2);
    }

    #[test]
    fn test_help_state_handle_input_actions() {
        let mut state = HelpState::new();
        
        // Test quit
        let result = state.handle_input("q");
        assert!(matches!(result, Ok(StateTransition::Exit)));
        
        // Test back
        let result = state.handle_input("b");
        assert!(matches!(result, Ok(StateTransition::Transition(target)) if target == "project_select"));
    }

    #[test]
    fn test_help_state_render() {
        let state = HelpState::new();
        
        let result = state.render();
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("=== Help ==="));
        assert!(output.contains("General / Global"));
        assert!(output.contains("Navigation:"));
    }

    #[test]
    fn test_help_state_state_name() {
        let state = HelpState::new();
        assert_eq!(state.state_name(), "help");
    }

    #[test]
    fn test_help_state_can_transition_to() {
        let state = HelpState::new();
        assert!(state.can_transition_to("kanban"));
        assert!(state.can_transition_to("sessions"));
        assert!(state.can_transition_to("project_select"));
        assert!(!state.can_transition_to("invalid"));
    }
}

#[cfg(test)]
mod project_select_state_tests {
    use super::*;

    #[test]
    fn test_project_select_state_creation() {
        let state = ProjectSelectState::new();
        
        assert!(state.projects.is_empty());
        assert!(state.selected_project.is_none());
        assert!(state.filter.is_empty());
    }

    #[test]
    fn test_project_select_state_add_project() {
        let mut state = ProjectSelectState::new();
        
        let project = ProjectItem {
            id: "project1".to_string(),
            name: "Test Project".to_string(),
            agent_count: 3,
            session_count: 5,
            last_activity: "2 hours ago".to_string(),
        };
        
        state.add_project(project);
        
        assert_eq!(state.projects.len(), 1);
        assert_eq!(state.projects[0].name, "Test Project");
        assert_eq!(state.projects[0].agent_count, 3);
    }

    #[test]
    fn test_project_select_state_get_filtered_projects() {
        let mut state = ProjectSelectState::new();
        
        let project1 = ProjectItem {
            id: "project1".to_string(),
            name: "Test Project".to_string(),
            agent_count: 3,
            session_count: 5,
            last_activity: "2 hours ago".to_string(),
        };
        
        let project2 = ProjectItem {
            id: "project2".to_string(),
            name: "Another Project".to_string(),
            agent_count: 1,
            session_count: 2,
            last_activity: "1 hour ago".to_string(),
        };
        
        state.add_project(project1);
        state.add_project(project2);
        
        // Test without filter
        let filtered = state.get_filtered_projects();
        assert_eq!(filtered.len(), 2);
        
        // Test with filter
        state.filter = "Test".to_string();
        let filtered = state.get_filtered_projects();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Test Project");
    }

    #[test]
    fn test_project_select_state_get_selected_project() {
        let mut state = ProjectSelectState::new();
        
        let project = ProjectItem {
            id: "project1".to_string(),
            name: "Test Project".to_string(),
            agent_count: 3,
            session_count: 5,
            last_activity: "2 hours ago".to_string(),
        };
        
        state.add_project(project);
        state.selected_project = Some(0);
        
        let selected = state.get_selected_project();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "Test Project");
    }

    #[test]
    fn test_project_select_state_handle_input_navigation() {
        let mut state = ProjectSelectState::new();
        
        let project = ProjectItem {
            id: "project1".to_string(),
            name: "Test Project".to_string(),
            agent_count: 3,
            session_count: 5,
            last_activity: "2 hours ago".to_string(),
        };
        
        state.add_project(project);
        
        // Test up navigation
        let result = state.handle_input("up");
        assert!(matches!(result, Ok(StateTransition::Stay)));
        
        // Test down navigation
        let result = state.handle_input("down");
        assert!(matches!(result, Ok(StateTransition::Stay)));
    }

    #[test]
    fn test_project_select_state_handle_input_actions() {
        let mut state = ProjectSelectState::new();
        
        let project = ProjectItem {
            id: "project1".to_string(),
            name: "Test Project".to_string(),
            agent_count: 3,
            session_count: 5,
            last_activity: "2 hours ago".to_string(),
        };
        
        state.add_project(project);
        state.selected_project = Some(0);
        
        // Test quit
        let result = state.handle_input("q");
        assert!(matches!(result, Ok(StateTransition::Exit)));
        
        // Test help
        let result = state.handle_input("h");
        assert!(matches!(result, Ok(StateTransition::Transition(target)) if target == "help"));
        
        // Test kanban
        let result = state.handle_input("k");
        assert!(matches!(result, Ok(StateTransition::Transition(target)) if target == "kanban"));
        
        // Test sessions
        let result = state.handle_input("s");
        assert!(matches!(result, Ok(StateTransition::Transition(target)) if target == "sessions"));
    }

    #[test]
    fn test_project_select_state_render() {
        let mut state = ProjectSelectState::new();
        
        let project = ProjectItem {
            id: "project1".to_string(),
            name: "Test Project".to_string(),
            agent_count: 3,
            session_count: 5,
            last_activity: "2 hours ago".to_string(),
        };
        
        state.add_project(project);
        
        let result = state.render();
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("=== Project Selection ==="));
        assert!(output.contains("Test Project"));
        assert!(output.contains("3 agents"));
        assert!(output.contains("5 sessions"));
    }

    #[test]
    fn test_project_select_state_state_name() {
        let state = ProjectSelectState::new();
        assert_eq!(state.state_name(), "project_select");
    }

    #[test]
    fn test_project_select_state_can_transition_to() {
        let state = ProjectSelectState::new();
        assert!(state.can_transition_to("kanban"));
        assert!(state.can_transition_to("help"));
        assert!(state.can_transition_to("sessions"));
        assert!(!state.can_transition_to("invalid"));
    }
}

#[cfg(test)]
mod state_manager_tests {
    use super::*;

    #[test]
    fn test_state_manager_creation() {
        let manager = StateManager::new();
        
        assert_eq!(manager.current_state_name(), "initial");
    }

    #[test]
    fn test_state_manager_add_state() {
        let mut manager = StateManager::new();
        let state = Box::new(HelpState::new());
        
        manager.add_state("help".to_string(), state);
        
        // Should be able to set current state to added state
        let result = manager.set_current_state("help".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.current_state_name(), "help");
    }

    #[test]
    fn test_state_manager_set_current_state() {
        let mut manager = StateManager::new();
        let state = Box::new(HelpState::new());
        
        manager.add_state("help".to_string(), state);
        
        // Test valid state
        let result = manager.set_current_state("help".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.current_state_name(), "help");
        
        // Test invalid state
        let result = manager.set_current_state("invalid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_state_manager_handle_input() {
        let mut manager = StateManager::new();
        let state = Box::new(HelpState::new());
        
        manager.add_state("help".to_string(), state);
        manager.set_current_state("help".to_string()).unwrap();
        
        // Test input handling
        let result = manager.handle_input("q");
        assert!(matches!(result, Ok(StateTransition::Exit)));
    }

    #[test]
    fn test_state_manager_render() {
        let mut manager = StateManager::new();
        let state = Box::new(HelpState::new());
        
        manager.add_state("help".to_string(), state);
        manager.set_current_state("help".to_string()).unwrap();
        
        // Test rendering
        let result = manager.render();
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.contains("=== Help ==="));
    }

    #[test]
    fn test_state_manager_process_transition() {
        let mut manager = StateManager::new();
        let help_state = Box::new(HelpState::new());
        let kanban_state = Box::new(KanbanState::new());
        
        manager.add_state("help".to_string(), help_state);
        manager.add_state("kanban".to_string(), kanban_state);
        manager.set_current_state("help".to_string()).unwrap();
        
        // Test valid transition
        let transition = StateTransition::Transition("kanban".to_string());
        let result = manager.process_transition(transition);
        assert!(result.is_ok());
        assert_eq!(manager.current_state_name(), "kanban");
        
        // Test invalid transition
        let transition = StateTransition::Transition("invalid".to_string());
        let result = manager.process_transition(transition);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod state_transition_tests {
    use super::*;

    #[test]
    fn test_state_transition_variants() {
        let stay = StateTransition::Stay;
        let transition = StateTransition::Transition("kanban".to_string());
        let exit = StateTransition::Exit;
        let error = StateTransition::Error("Test error".to_string());
        
        // Test pattern matching
        match stay {
            StateTransition::Stay => assert!(true),
            _ => assert!(false),
        }
        
        match transition {
            StateTransition::Transition(target) => assert_eq!(target, "kanban"),
            _ => assert!(false),
        }
        
        match exit {
            StateTransition::Exit => assert!(true),
            _ => assert!(false),
        }
        
        match error {
            StateTransition::Error(msg) => assert_eq!(msg, "Test error"),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_state_transition_clone() {
        let transition = StateTransition::Transition("kanban".to_string());
        let cloned = transition.clone();
        
        match (transition, cloned) {
            (StateTransition::Transition(target1), StateTransition::Transition(target2)) => {
                assert_eq!(target1, target2);
            }
            _ => assert!(false),
        }
    }
}

#[cfg(test)]
mod selection_store_tests {
    use super::*;

    #[test]
    fn test_selection_store_set_get_project_id() {
        // Clear any existing project id
        selection_store::set_project_id("".to_string());
        
        // Test setting and getting project id
        selection_store::set_project_id("project1".to_string());
        let project_id = selection_store::get_project_id();
        
        assert!(project_id.is_some());
        assert_eq!(project_id.unwrap(), "project1");
    }

    #[test]
    fn test_selection_store_clear_project_id() {
        // Set a project id
        selection_store::set_project_id("project1".to_string());
        
        // Clear it by setting empty string
        selection_store::set_project_id("".to_string());
        let project_id = selection_store::get_project_id();
        
        assert!(project_id.is_some());
        assert_eq!(project_id.unwrap(), "");
    }
}
