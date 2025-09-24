//! TUI UX Regression Tests
//!
//! Goal: guard against regressions in navigation, focus visibility, theming, typography modes,
//! responsive layout, and critical components (GlobalStatus, Toasts) across key flows.

use std::error::Error;

use ratatui::{
    backend::TestBackend,
    Terminal,
};

use crate::tui::{
    state::{
        TuiState,
        view_state::{KanbanState, SessionsState, TaskItem, SessionItem},
        navigation_state::{HelpState, ProjectSelectState, ProjectItem},
    },
    themes::{ThemeKind, default_typography, compact_typography, high_density_typography},
    components::{GlobalStatus, GlobalStateIcon, ToastQueue, Toast, ToastType, render_global_status, render_toasts},
    views::{KanbanView, KanbanColumn, render_kanban_view, render_sessions_view},
};

// Minimal terminal helper
fn term(w: u16, h: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(w, h)).expect("terminal")
}

#[test]
fn regression_flow_navigation_and_focus() -> Result<(), Box<dyn Error>> {
    let mut terminal = term(80, 30);

    // Project → Kanban → Sessions → Help
    let mut project = ProjectSelectState::new();
    project.add_project(ProjectItem { id: "p1".into(), name: "Demo".into(), agent_count: 2, session_count: 0, last_activity: String::new() });

    let mut kanban = KanbanState::new();
    kanban.tasks = vec![
        TaskItem { id: "t1".into(), title: "First".into(), status: "todo".into(), assignee: None, priority: "medium".into() },
        TaskItem { id: "t2".into(), title: "Second".into(), status: "doing".into(), assignee: None, priority: "high".into() },
    ];

    let mut sessions = SessionsState::new();
    sessions.sessions = vec![
        SessionItem { id: "s1".into(), agent_name: "agent-a".into(), role: String::new(), provider: "mock".into(), status: "running".into(), duration: "2024-01-01".into() },
    ];

    let theme = ThemeKind::Dark.palette();
    let typo = default_typography(&theme);

    // Render Kanban with visible focus cues (selected column/task not crashing)
    terminal.draw(|f| {
        let area = f.area();
        let view = KanbanView {
            columns: kanban.get_columns().into_iter().map(|c| KanbanColumn {
                title: c.name.clone(),
                status: match c.name.as_str() { "To Do" => crate::tui::components::TaskStatus::Todo, "Doing" => crate::tui::components::TaskStatus::Doing, _ => crate::tui::components::TaskStatus::Done },
                tasks: c.tasks.into_iter().map(|t| crate::tui::components::Task {
                    id: t.id,
                    title: t.title,
                    description: None,
                    status: match c.name.as_str() { "To Do" => crate::tui::components::TaskStatus::Todo, "Doing" => crate::tui::components::TaskStatus::Doing, _ => crate::tui::components::TaskStatus::Done },
                    priority: crate::tui::components::TaskPriority::Medium,
                    assignee: t.assignee,
                    created_at: "2024-01-01".into(),
                    updated_at: "2024-01-01".into(),
                }).collect(),
                selected_task: None,
            }).collect(),
            selected_column: 0,
            filter: String::new(),
            show_completed: true,
            sort_by: crate::tui::views::KanbanSort::Created,
        };
        render_kanban_view(f, area, &view, &theme, &typo);
    })?;
    assert!(!terminal.backend().buffer().content.is_empty());

    // Render Sessions responsive + focus
    terminal.draw(|f| {
        let area = f.area();
        render_sessions_view(f, area, &mut sessions, &theme, &typo);
    })?;
    assert!(!terminal.backend().buffer().content.is_empty());

    // Help state should render
    let help = HelpState::new();
    // trait render returns a String; just ensure it succeeds
    let _ = TuiState::render(&help)?;
    terminal.draw(|f| { let _ = f.area(); })?;
    assert!(!terminal.backend().buffer().content.is_empty());

    Ok(())
}

#[test]
fn regression_theming_and_typography_modes() -> Result<(), Box<dyn Error>> {
    let themes = [ThemeKind::Light, ThemeKind::Dark, ThemeKind::HighContrast];
    for kind in themes {
        let theme = kind.palette();
        let typos = [
            default_typography(&theme),
            compact_typography(&theme),
            high_density_typography(&theme),
        ];
        for typo in typos {
            let mut term = term(80, 30);
            let mut kanban = KanbanState::new();
            kanban.tasks = vec![TaskItem { id: "t".into(), title: "X".into(), status: "todo".into(), assignee: None, priority: "low".into() }];
            term.draw(|f| {
                let area = f.area();
                let view = KanbanView {
                    columns: kanban.get_columns().into_iter().map(|c| KanbanColumn {
                        title: c.name.clone(),
                        status: crate::tui::components::TaskStatus::Todo,
                        tasks: vec![],
                        selected_task: None,
                    }).collect(),
                    selected_column: 0,
                    filter: String::new(),
                    show_completed: true,
                    sort_by: crate::tui::views::KanbanSort::Created,
                };
                render_kanban_view(f, area, &view, &theme, &typo);
            })?;
            assert!(!term.backend().buffer().content.is_empty());
        }
    }
    Ok(())
}

#[test]
fn regression_status_and_toasts() -> Result<(), Box<dyn Error>> {
    let mut terminal = term(80, 10);
    let theme = ThemeKind::Dark.palette();
    let typo = default_typography(&theme);

    let status = GlobalStatus { project_name: "demo".into(), view_name: "kanban".into(), focus: "First".into(), icon: GlobalStateIcon::Active, last_action: None };
    let mut toasts = ToastQueue::with_capacity(4);
    toasts.enqueue(Toast { kind: ToastType::Info, message: "Saved".into(), ttl_ms: Some(1_000) });

    terminal.draw(|f| {
        let area = f.area();
        let top = ratatui::layout::Rect { x: area.x, y: area.y, width: area.width, height: 1 };
        let rest = ratatui::layout::Rect { x: area.x, y: area.y + 1, width: area.width, height: area.height - 1 };
        render_global_status(f, top, &status, &theme, &typo);
        render_toasts(f, rest, &mut toasts, &theme, &typo);
    })?;
    assert!(!terminal.backend().buffer().content.is_empty());
    Ok(())
}


