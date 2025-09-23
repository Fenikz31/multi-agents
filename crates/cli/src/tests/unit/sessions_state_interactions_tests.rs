//! Unit tests for SessionsState interactions (sorting, navigation, actions stubs)

use crate::tui::state::view_state::{SessionsState, SessionItem};
use crate::tui::state::{TuiState, StateTransition};

#[test]
fn test_sessions_state_sort_toggle_and_navigation() {
    let mut state = SessionsState::new();
    state.sessions = vec![
        SessionItem { id: "s1".into(), agent_name: "b".into(), role: "".into(), provider: "claude".into(), status: "Active".into(), duration: "2025-01-17T10:00:01Z".into() },
        SessionItem { id: "s2".into(), agent_name: "a".into(), role: "".into(), provider: "gemini".into(), status: "Inactive".into(), duration: "2025-01-17T10:00:02Z".into() },
    ];

    // Default: sort by last activity desc (duration string here)
    let filtered = state.get_filtered_sessions();
    assert_eq!(filtered.len(), 2);

    // Toggle sort (t) and ensure call does not error
    let res = state.handle_input("t");
    assert!(res.is_ok());

    // Navigation down/up
    let _ = state.handle_input("down");
    assert_eq!(state.selected_session, Some(0));
    let _ = state.handle_input("down");
    assert_eq!(state.selected_session, Some(1));
    let _ = state.handle_input("up");
    assert_eq!(state.selected_session, Some(0));
}

#[test]
fn test_sessions_state_actions_stubs() {
    let mut state = SessionsState::new();
    state.sessions = vec![SessionItem { id: "s1".into(), agent_name: "a".into(), role: "".into(), provider: "claude".into(), status: "Active".into(), duration: "".into() }];
    state.selected_session = Some(0);

    // Resume
    match state.handle_input("r").unwrap() {
        StateTransition::Error(msg) => assert!(msg.contains("not implemented")),
        _ => panic!("expected Error transition"),
    }

    // Stop
    match state.handle_input("x").unwrap() {
        StateTransition::Error(msg) => assert!(msg.contains("not implemented")),
        _ => panic!("expected Error transition"),
    }

    // Start
    match state.handle_input("S").unwrap() {
        StateTransition::Error(msg) => assert!(msg.contains("not implemented")),
        _ => panic!("expected Error transition"),
    }
}


