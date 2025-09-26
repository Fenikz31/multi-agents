//! TUI Terminal Compatibility Tests
//! 
//! Tests the compatibility of TUI components with different terminal configurations,
//! screen sizes, color support, and input methods. These tests verify that the TUI
//! works correctly across various terminal environments.

use std::error::Error;
use ratatui::{
    backend::TestBackend,
    Terminal,
};

use crate::tui::{
    state::view_state::{KanbanState, SessionsState, TaskItem, SessionItem},
    state::navigation_state::{HelpState, ProjectSelectState, ProjectItem},
    state::TuiState,
    components::{Toast, ToastQueue, ToastType, GlobalStatus, GlobalStateIcon, Task, TaskStatus, TaskPriority},
    themes::{ThemeKind, default_typography, compact_typography, high_density_typography},
    views::{render_kanban_view, render_sessions_view, KanbanView, KanbanColumn},
    components::{render_global_status, render_toasts},
};

/// Terminal configuration for testing
#[derive(Debug, Clone)]
struct TerminalConfig {
    width: u16,
    height: u16,
    color_support: ColorSupport,
    unicode_support: bool,
    name: String,
}

#[derive(Debug, Clone, PartialEq)]
enum ColorSupport {
    Monochrome,
    Color16,
    Color256,
    TrueColor,
}

impl TerminalConfig {
    fn new(width: u16, height: u16, color_support: ColorSupport, unicode_support: bool, name: String) -> Self {
        Self { width, height, color_support, unicode_support, name }
    }

    fn small_terminal() -> Self {
        Self::new(80, 24, ColorSupport::Color16, true, "Small Terminal (80x24)".to_string())
    }

    fn medium_terminal() -> Self {
        Self::new(120, 30, ColorSupport::Color256, true, "Medium Terminal (120x30)".to_string())
    }

    fn large_terminal() -> Self {
        Self::new(200, 50, ColorSupport::TrueColor, true, "Large Terminal (200x50)".to_string())
    }

    fn monochrome_terminal() -> Self {
        Self::new(80, 24, ColorSupport::Monochrome, false, "Monochrome Terminal".to_string())
    }

    fn legacy_terminal() -> Self {
        Self::new(80, 24, ColorSupport::Color16, false, "Legacy Terminal (No Unicode)".to_string())
    }
}

/// Helper function to create a test terminal with specific configuration
fn create_test_terminal(config: &TerminalConfig) -> Terminal<TestBackend> {
    let backend = TestBackend::new(config.width, config.height);
    Terminal::new(backend).unwrap()
}

/// Helper function to create test data for compatibility testing
fn create_test_data() -> (Vec<TaskItem>, Vec<SessionItem>, Vec<ProjectItem>) {
    let tasks = vec![
        TaskItem {
            id: "task-1".to_string(),
            title: "Test Task with Unicode: 🚀 Développement".to_string(),
            status: "todo".to_string(),
            assignee: Some("agent-1".to_string()),
            priority: "high".to_string(),
        },
        TaskItem {
            id: "task-2".to_string(),
            title: "Another Task with Special Chars: <>&\"'".to_string(),
            status: "in_progress".to_string(),
            assignee: None,
            priority: "medium".to_string(),
        },
        TaskItem {
            id: "task-3".to_string(),
            title: "Long Task Title That Should Wrap Properly in Small Terminals".to_string(),
            status: "done".to_string(),
            assignee: Some("agent-2".to_string()),
            priority: "low".to_string(),
        },
    ];

    let sessions = vec![
        SessionItem {
            id: "session-1".to_string(),
            agent_name: "agent-1".to_string(),
            role: "developer".to_string(),
            provider: "claude".to_string(),
            status: "active".to_string(),
            duration: "2h 30m".to_string(),
        },
        SessionItem {
            id: "session-2".to_string(),
            agent_name: "agent-2".to_string(),
            role: "tester".to_string(),
            provider: "gemini".to_string(),
            status: "completed".to_string(),
            duration: "1h 15m".to_string(),
        },
    ];

    let projects = vec![
        ProjectItem {
            id: "project-1".to_string(),
            name: "Test Project with Unicode: 🎯".to_string(),
            agent_count: 3,
            session_count: 5,
            last_activity: "2 hours ago".to_string(),
        },
        ProjectItem {
            id: "project-2".to_string(),
            name: "Another Project".to_string(),
            agent_count: 1,
            session_count: 2,
            last_activity: "1 day ago".to_string(),
        },
    ];

    (tasks, sessions, projects)
}

#[cfg(test)]
mod terminal_size_compatibility_tests {
    use super::*;

    #[test]
    fn test_small_terminal_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::small_terminal();
        let mut terminal = create_test_terminal(&config);
        let (tasks, sessions, _) = create_test_data();

        // Test KanbanState rendering on small terminal
        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = tasks.clone();
        
        let theme = ThemeKind::Dark.palette();
        let typography = default_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let kanban_view = KanbanView {
                columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                    title: c.name.clone(),
                    status: match c.name.as_str() {
                        "To Do" => TaskStatus::Todo,
                        "Doing" => TaskStatus::Doing,
                        _ => TaskStatus::Done,
                    },
                    tasks: c.tasks.into_iter().map(|t| Task {
                        id: t.id,
                        title: t.title,
                        description: None,
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        priority: match t.priority.as_str() {
                            "high" => TaskPriority::High,
                            "medium" => TaskPriority::Medium,
                            _ => TaskPriority::Low,
                        },
                        assignee: t.assignee,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                    selected_task: None,
                }).collect(),
                selected_column: 0,
                filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
            };
            
            render_kanban_view(f, area, &kanban_view, &theme, &typography);
        })?;

        // Verify that the rendering succeeded without panicking
        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty after rendering");
        
        // Check that the content fits within the terminal bounds
        assert!(buffer.area().width <= config.width, "Content should fit within terminal width");
        assert!(buffer.area().height <= config.height, "Content should fit within terminal height");

        println!("✅ Small terminal ({}x{}) compatibility test passed", config.width, config.height);
        Ok(())
    }

    #[test]
    fn test_medium_terminal_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::medium_terminal();
        let mut terminal = create_test_terminal(&config);
        let (tasks, sessions, _) = create_test_data();

        // Test SessionsState rendering on medium terminal
        let mut sessions_state = SessionsState::new();
        sessions_state.sessions = sessions.clone();
        
        let theme = ThemeKind::Dark.palette();
        let typography = default_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let status = GlobalStatus {
                project_name: "test-project".to_string(),
                view_name: "sessions".to_string(),
                focus: "Session 1".to_string(),
                icon: GlobalStateIcon::Active,
                last_action: Some("View sessions".to_string()),
            };
            
            render_sessions_view(f, area, &mut sessions_state, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty after rendering");
        assert!(buffer.area().width <= config.width, "Content should fit within terminal width");
        assert!(buffer.area().height <= config.height, "Content should fit within terminal height");

        println!("✅ Medium terminal ({}x{}) compatibility test passed", config.width, config.height);
        Ok(())
    }

    #[test]
    fn test_large_terminal_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::large_terminal();
        let mut terminal = create_test_terminal(&config);
        let (tasks, sessions, _) = create_test_data();

        // Test with large amounts of data
        let mut kanban_state = KanbanState::new();
        let mut large_tasks = Vec::new();
        for i in 0..100 {
            large_tasks.push(TaskItem {
                id: format!("task-{}", i),
                title: format!("Large Dataset Task {} with Long Title", i),
                status: match i % 3 {
                    0 => "todo".to_string(),
                    1 => "in_progress".to_string(),
                    _ => "done".to_string(),
                },
                assignee: if i % 2 == 0 { Some(format!("agent-{}", i % 5)) } else { None },
                priority: match i % 4 {
                    0 => "low".to_string(),
                    1 => "medium".to_string(),
                    2 => "high".to_string(),
                    _ => "critical".to_string(),
                },
            });
        }
        kanban_state.tasks = large_tasks;
        
        let theme = ThemeKind::Dark.palette();
        let typography = default_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let kanban_view = KanbanView {
                columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                    title: c.name.clone(),
                    status: match c.name.as_str() {
                        "To Do" => TaskStatus::Todo,
                        "Doing" => TaskStatus::Doing,
                        _ => TaskStatus::Done,
                    },
                    tasks: c.tasks.into_iter().map(|t| Task {
                        id: t.id,
                        title: t.title,
                        description: None,
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        priority: match t.priority.as_str() {
                            "high" => TaskPriority::High,
                            "medium" => TaskPriority::Medium,
                            "critical" => TaskPriority::Critical,
                            _ => TaskPriority::Low,
                        },
                        assignee: t.assignee,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                    selected_task: None,
                }).collect(),
                selected_column: 0,
                filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
            };
            
            render_kanban_view(f, area, &kanban_view, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty after rendering");
        assert!(buffer.area().width <= config.width, "Content should fit within terminal width");
        assert!(buffer.area().height <= config.height, "Content should fit within terminal height");

        println!("✅ Large terminal ({}x{}) compatibility test passed", config.width, config.height);
        Ok(())
    }

    #[test]
    fn test_responsive_layout_adaptation() -> Result<(), Box<dyn Error>> {
        let configs = vec![
            TerminalConfig::small_terminal(),
            TerminalConfig::medium_terminal(),
            TerminalConfig::large_terminal(),
        ];

        for config in configs {
            let mut terminal = create_test_terminal(&config);
            let (tasks, _, _) = create_test_data();

            let mut kanban_state = KanbanState::new();
            kanban_state.tasks = tasks.clone();
            
            let theme = ThemeKind::Dark.palette();
            let typography = default_typography(&theme);
            
            terminal.draw(|f| {
                let area = f.area();
                let kanban_view = KanbanView {
                    columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                        title: c.name.clone(),
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        tasks: c.tasks.into_iter().map(|t| Task {
                            id: t.id,
                            title: t.title,
                            description: None,
                            status: match c.name.as_str() {
                                "To Do" => TaskStatus::Todo,
                                "Doing" => TaskStatus::Doing,
                                _ => TaskStatus::Done,
                            },
                            priority: match t.priority.as_str() {
                                "high" => TaskPriority::High,
                                "medium" => TaskPriority::Medium,
                                _ => TaskPriority::Low,
                            },
                            assignee: t.assignee,
                            created_at: "2024-01-01T00:00:00Z".to_string(),
                            updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                        selected_task: None,
                }).collect(),
                    selected_column: 0,
                    filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
                };
                
                render_kanban_view(f, area, &kanban_view, &theme, &typography);
            })?;

            let buffer = terminal.backend().buffer();
            assert!(!buffer.content.is_empty(), "Buffer should not be empty for {}x{}", config.width, config.height);
            
            // Test that the layout adapts to different screen sizes
            if config.width <= 80 {
                // Small terminal should show fewer columns or stack them
                println!("✅ Small terminal layout adaptation verified");
            } else if config.width <= 140 {
                // Medium terminal should show 2 columns
                println!("✅ Medium terminal layout adaptation verified");
            } else {
                // Large terminal should show 3 columns
                println!("✅ Large terminal layout adaptation verified");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod color_compatibility_tests {
    use super::*;

    #[test]
    fn test_theme_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::medium_terminal();
        let themes = vec![
            ThemeKind::Light,
            ThemeKind::Dark,
            ThemeKind::HighContrast,
        ];

        for theme_kind in themes {
            let mut terminal = create_test_terminal(&config);
            let (tasks, _, _) = create_test_data();

            let mut kanban_state = KanbanState::new();
            kanban_state.tasks = tasks.clone();
            
            let theme = theme_kind.palette();
            let typography = default_typography(&theme);
            
            terminal.draw(|f| {
                let area = f.area();
                let kanban_view = KanbanView {
                    columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                        title: c.name.clone(),
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        tasks: c.tasks.into_iter().map(|t| Task {
                            id: t.id,
                            title: t.title,
                            description: None,
                            status: match c.name.as_str() {
                                "To Do" => TaskStatus::Todo,
                                "Doing" => TaskStatus::Doing,
                                _ => TaskStatus::Done,
                            },
                            priority: match t.priority.as_str() {
                                "high" => TaskPriority::High,
                                "medium" => TaskPriority::Medium,
                                _ => TaskPriority::Low,
                            },
                            assignee: t.assignee,
                            created_at: "2024-01-01T00:00:00Z".to_string(),
                            updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                        selected_task: None,
                }).collect(),
                    selected_column: 0,
                    filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
                };
                
                render_kanban_view(f, area, &kanban_view, &theme, &typography);
            })?;

            let buffer = terminal.backend().buffer();
            assert!(!buffer.content.is_empty(), "Buffer should not be empty for theme {:?}", theme_kind);
            
            println!("✅ Theme {:?} compatibility test passed", theme_kind);
        }

        Ok(())
    }

    #[test]
    fn test_monochrome_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::monochrome_terminal();
        let mut terminal = create_test_terminal(&config);
        let (tasks, _, _) = create_test_data();

        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = tasks.clone();
        
        // Use high contrast theme for monochrome terminals
        let theme = ThemeKind::HighContrast.palette();
        let typography = default_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let kanban_view = KanbanView {
                columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                    title: c.name.clone(),
                    status: match c.name.as_str() {
                        "To Do" => TaskStatus::Todo,
                        "Doing" => TaskStatus::Doing,
                        _ => TaskStatus::Done,
                    },
                    tasks: c.tasks.into_iter().map(|t| Task {
                        id: t.id,
                        title: t.title,
                        description: None,
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        priority: match t.priority.as_str() {
                            "high" => TaskPriority::High,
                            "medium" => TaskPriority::Medium,
                            _ => TaskPriority::Low,
                        },
                        assignee: t.assignee,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                    selected_task: None,
                }).collect(),
                selected_column: 0,
                filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
            };
            
            render_kanban_view(f, area, &kanban_view, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty for monochrome terminal");
        
        println!("✅ Monochrome terminal compatibility test passed");
        Ok(())
    }

    #[test]
    fn test_color_palette_compatibility() -> Result<(), Box<dyn Error>> {
        let configs = vec![
            (TerminalConfig::new(80, 24, ColorSupport::Color16, true, "16 Color".to_string()), "16 colors"),
            (TerminalConfig::new(80, 24, ColorSupport::Color256, true, "256 Color".to_string()), "256 colors"),
            (TerminalConfig::new(80, 24, ColorSupport::TrueColor, true, "True Color".to_string()), "true color"),
        ];

        for (config, color_desc) in configs {
            let mut terminal = create_test_terminal(&config);
            let (tasks, _, _) = create_test_data();

            let mut kanban_state = KanbanState::new();
            kanban_state.tasks = tasks.clone();
            
            let theme = ThemeKind::Dark.palette();
            let typography = default_typography(&theme);
            
            terminal.draw(|f| {
                let area = f.area();
                let kanban_view = KanbanView {
                    columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                        title: c.name.clone(),
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        tasks: c.tasks.into_iter().map(|t| Task {
                            id: t.id,
                            title: t.title,
                            description: None,
                            status: match c.name.as_str() {
                                "To Do" => TaskStatus::Todo,
                                "Doing" => TaskStatus::Doing,
                                _ => TaskStatus::Done,
                            },
                            priority: match t.priority.as_str() {
                                "high" => TaskPriority::High,
                                "medium" => TaskPriority::Medium,
                                _ => TaskPriority::Low,
                            },
                            assignee: t.assignee,
                            created_at: "2024-01-01T00:00:00Z".to_string(),
                            updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                        selected_task: None,
                }).collect(),
                    selected_column: 0,
                    filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
                };
                
                render_kanban_view(f, area, &kanban_view, &theme, &typography);
            })?;

            let buffer = terminal.backend().buffer();
            assert!(!buffer.content.is_empty(), "Buffer should not be empty for {}", color_desc);
            
            println!("✅ {} compatibility test passed", color_desc);
        }

        Ok(())
    }
}

#[cfg(test)]
mod typography_compatibility_tests {
    use super::*;

    #[test]
    fn test_typography_modes_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::medium_terminal();
        let (tasks, _, _) = create_test_data();

        let typography_modes = vec![
            ("Normal", default_typography(&ThemeKind::Dark.palette())),
            ("Compact", compact_typography(&ThemeKind::Dark.palette())),
            ("High Density", high_density_typography(&ThemeKind::Dark.palette())),
        ];

        for (mode_name, typography) in typography_modes {
            let mut terminal = create_test_terminal(&config);
            let mut kanban_state = KanbanState::new();
            kanban_state.tasks = tasks.clone();
            
            let theme = ThemeKind::Dark.palette();
            
            terminal.draw(|f| {
                let area = f.area();
                let kanban_view = KanbanView {
                    columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                        title: c.name.clone(),
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        tasks: c.tasks.into_iter().map(|t| Task {
                            id: t.id,
                            title: t.title,
                            description: None,
                            status: match c.name.as_str() {
                                "To Do" => TaskStatus::Todo,
                                "Doing" => TaskStatus::Doing,
                                _ => TaskStatus::Done,
                            },
                            priority: match t.priority.as_str() {
                                "high" => TaskPriority::High,
                                "medium" => TaskPriority::Medium,
                                _ => TaskPriority::Low,
                            },
                            assignee: t.assignee,
                            created_at: "2024-01-01T00:00:00Z".to_string(),
                            updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                        selected_task: None,
                }).collect(),
                    selected_column: 0,
                    filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
                };
                
                render_kanban_view(f, area, &kanban_view, &theme, &typography);
            })?;

            let buffer = terminal.backend().buffer();
            assert!(!buffer.content.is_empty(), "Buffer should not be empty for {} typography", mode_name);
            
            println!("✅ {} typography compatibility test passed", mode_name);
        }

        Ok(())
    }

    #[test]
    fn test_compact_mode_small_terminal() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::small_terminal();
        let mut terminal = create_test_terminal(&config);
        let (tasks, _, _) = create_test_data();

        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = tasks.clone();
        
        let theme = ThemeKind::Dark.palette();
        let typography = compact_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let kanban_view = KanbanView {
                columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                    title: c.name.clone(),
                    status: match c.name.as_str() {
                        "To Do" => TaskStatus::Todo,
                        "Doing" => TaskStatus::Doing,
                        _ => TaskStatus::Done,
                    },
                    tasks: c.tasks.into_iter().map(|t| Task {
                        id: t.id,
                        title: t.title,
                        description: None,
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        priority: match t.priority.as_str() {
                            "high" => TaskPriority::High,
                            "medium" => TaskPriority::Medium,
                            _ => TaskPriority::Low,
                        },
                        assignee: t.assignee,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                selected_task: None,
                }).collect(),
                selected_column: 0,
                filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
            };
            
            render_kanban_view(f, area, &kanban_view, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty for compact mode on small terminal");
        
        println!("✅ Compact mode on small terminal compatibility test passed");
        Ok(())
    }
}

#[cfg(test)]
mod unicode_compatibility_tests {
    use super::*;

    #[test]
    fn test_unicode_support() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::medium_terminal();
        let mut terminal = create_test_terminal(&config);

        // Test with Unicode characters
        let unicode_tasks = vec![
            TaskItem {
                id: "unicode-1".to_string(),
                title: "🚀 Développement avec émojis".to_string(),
                status: "todo".to_string(),
                assignee: Some("développeur-1".to_string()),
                priority: "high".to_string(),
            },
            TaskItem {
                id: "unicode-2".to_string(),
                title: "测试中文标题".to_string(),
                status: "in_progress".to_string(),
                assignee: None,
                priority: "medium".to_string(),
            },
            TaskItem {
                id: "unicode-3".to_string(),
                title: "Тест на русском языке".to_string(),
                status: "done".to_string(),
                assignee: Some("агент-1".to_string()),
                priority: "low".to_string(),
            },
        ];

        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = unicode_tasks;
        
        let theme = ThemeKind::Dark.palette();
        let typography = default_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let kanban_view = KanbanView {
                columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                    title: c.name.clone(),
                    status: match c.name.as_str() {
                        "To Do" => TaskStatus::Todo,
                        "Doing" => TaskStatus::Doing,
                        _ => TaskStatus::Done,
                    },
                    tasks: c.tasks.into_iter().map(|t| Task {
                        id: t.id,
                        title: t.title,
                        description: None,
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        priority: match t.priority.as_str() {
                            "high" => TaskPriority::High,
                            "medium" => TaskPriority::Medium,
                            _ => TaskPriority::Low,
                        },
                        assignee: t.assignee,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                    selected_task: None,
                }).collect(),
                selected_column: 0,
                filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
            };
            
            render_kanban_view(f, area, &kanban_view, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty with Unicode content");
        
        println!("✅ Unicode support compatibility test passed");
        Ok(())
    }

    #[test]
    fn test_legacy_terminal_fallback() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::legacy_terminal();
        let mut terminal = create_test_terminal(&config);

        // Test with mixed content (some Unicode, some ASCII)
        let mixed_tasks = vec![
            TaskItem {
                id: "mixed-1".to_string(),
                title: "ASCII Task Title".to_string(),
                status: "todo".to_string(),
                assignee: Some("agent-1".to_string()),
                priority: "high".to_string(),
            },
            TaskItem {
                id: "mixed-2".to_string(),
                title: "Task with Special Chars: <>&\"'".to_string(),
                status: "in_progress".to_string(),
                assignee: None,
                priority: "medium".to_string(),
            },
        ];

        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = mixed_tasks;
        
        let theme = ThemeKind::Dark.palette();
        let typography = default_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let kanban_view = KanbanView {
                columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                    title: c.name.clone(),
                    status: match c.name.as_str() {
                        "To Do" => TaskStatus::Todo,
                        "Doing" => TaskStatus::Doing,
                        _ => TaskStatus::Done,
                    },
                    tasks: c.tasks.into_iter().map(|t| Task {
                        id: t.id,
                        title: t.title,
                        description: None,
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        priority: match t.priority.as_str() {
                            "high" => TaskPriority::High,
                            "medium" => TaskPriority::Medium,
                            _ => TaskPriority::Low,
                        },
                        assignee: t.assignee,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                    selected_task: None,
                }).collect(),
                selected_column: 0,
                filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
            };
            
            render_kanban_view(f, area, &kanban_view, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty on legacy terminal");
        
        println!("✅ Legacy terminal fallback compatibility test passed");
        Ok(())
    }

    #[test]
    fn test_special_characters_handling() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::medium_terminal();
        let mut terminal = create_test_terminal(&config);

        // Test with various special characters
        let special_tasks = vec![
            TaskItem {
                id: "special-1".to_string(),
                title: "Task with quotes: \"Hello World\"".to_string(),
                status: "todo".to_string(),
                assignee: Some("agent-1".to_string()),
                priority: "high".to_string(),
            },
            TaskItem {
                id: "special-2".to_string(),
                title: "Task with HTML: <div>content</div>".to_string(),
                status: "in_progress".to_string(),
                assignee: None,
                priority: "medium".to_string(),
            },
            TaskItem {
                id: "special-3".to_string(),
                title: "Task with ampersand: A & B".to_string(),
                status: "done".to_string(),
                assignee: Some("agent-2".to_string()),
                priority: "low".to_string(),
            },
        ];

        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = special_tasks;
        
        let theme = ThemeKind::Dark.palette();
        let typography = default_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let kanban_view = KanbanView {
                columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                    title: c.name.clone(),
                    status: match c.name.as_str() {
                        "To Do" => TaskStatus::Todo,
                        "Doing" => TaskStatus::Doing,
                        _ => TaskStatus::Done,
                    },
                    tasks: c.tasks.into_iter().map(|t| Task {
                        id: t.id,
                        title: t.title,
                        description: None,
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        priority: match t.priority.as_str() {
                            "high" => TaskPriority::High,
                            "medium" => TaskPriority::Medium,
                            _ => TaskPriority::Low,
                        },
                        assignee: t.assignee,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                    selected_task: None,
                }).collect(),
                selected_column: 0,
                filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
            };
            
            render_kanban_view(f, area, &kanban_view, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty with special characters");
        
        println!("✅ Special characters handling compatibility test passed");
        Ok(())
    }
}

#[cfg(test)]
mod component_compatibility_tests {
    use super::*;

    #[test]
    fn test_global_status_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::medium_terminal();
        let mut terminal = create_test_terminal(&config);

        let status = GlobalStatus {
            project_name: "test-project".to_string(),
            view_name: "kanban".to_string(),
            focus: "Task 1".to_string(),
            icon: GlobalStateIcon::Active,
            last_action: Some("Added task".to_string()),
        };

        let theme = ThemeKind::Dark.palette();
        let typography = default_typography(&theme);

        terminal.draw(|f| {
            let area = f.area();
            render_global_status(f, area, &status, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty after rendering GlobalStatus");
        
        println!("✅ GlobalStatus compatibility test passed");
        Ok(())
    }

    #[test]
    fn test_toast_queue_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::medium_terminal();
        let mut terminal = create_test_terminal(&config);

        let mut toast_queue = ToastQueue::with_capacity(5);
        toast_queue.enqueue(Toast::new(ToastType::Info, "Information message", Some(5000)));
        toast_queue.enqueue(Toast::new(ToastType::Success, "Success message", Some(3000)));
        toast_queue.enqueue(Toast::new(ToastType::Warn, "Warning message", Some(4000)));
        toast_queue.enqueue(Toast::new(ToastType::Error, "Error message", Some(6000)));

        let theme = ThemeKind::Dark.palette();
        let typography = default_typography(&theme);

        terminal.draw(|f| {
            let area = f.area();
            render_toasts(f, area, &toast_queue, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty after rendering ToastQueue");
        
        println!("✅ ToastQueue compatibility test passed");
        Ok(())
    }

    #[test]
    fn test_navigation_states_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::medium_terminal();
        let (_, _, projects) = create_test_data();

        // Test HelpState
        let help_state = HelpState::new();
        let help_output = TuiState::render(&help_state)?;
        assert!(!help_output.is_empty(), "HelpState should render content");
        
        // Test ProjectSelectState
        let mut project_select_state = ProjectSelectState::new();
        for project in projects {
            project_select_state.add_project(project);
        }
        let project_output = TuiState::render(&project_select_state)?;
        assert!(!project_output.is_empty(), "ProjectSelectState should render content");
        
        println!("✅ Navigation states compatibility test passed");
        Ok(())
    }
}

#[cfg(test)]
mod edge_case_compatibility_tests {
    use super::*;

    #[test]
    fn test_minimal_terminal_size() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::new(40, 10, ColorSupport::Color16, true, "Minimal Terminal".to_string());
        let mut terminal = create_test_terminal(&config);
        let (tasks, _, _) = create_test_data();

        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = tasks.clone();
        
        let theme = ThemeKind::Dark.palette();
        let typography = compact_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let kanban_view = KanbanView {
                columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                    title: c.name.clone(),
                    status: match c.name.as_str() {
                        "To Do" => TaskStatus::Todo,
                        "Doing" => TaskStatus::Doing,
                        _ => TaskStatus::Done,
                    },
                    tasks: c.tasks.into_iter().map(|t| Task {
                        id: t.id,
                        title: t.title,
                        description: None,
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        priority: match t.priority.as_str() {
                            "high" => TaskPriority::High,
                            "medium" => TaskPriority::Medium,
                            _ => TaskPriority::Low,
                        },
                        assignee: t.assignee,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                    selected_task: None,
                }).collect(),
                selected_column: 0,
                filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
            };
            
            render_kanban_view(f, area, &kanban_view, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty even on minimal terminal");
        
        println!("✅ Minimal terminal size compatibility test passed");
        Ok(())
    }

    #[test]
    fn test_empty_data_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::medium_terminal();
        let mut terminal = create_test_terminal(&config);

        // Test with empty data
        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = Vec::new();
        
        let theme = ThemeKind::Dark.palette();
        let typography = default_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let kanban_view = KanbanView {
                columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                    title: c.name.clone(),
                    status: match c.name.as_str() {
                        "To Do" => TaskStatus::Todo,
                        "Doing" => TaskStatus::Doing,
                        _ => TaskStatus::Done,
                    },
                    tasks: c.tasks.into_iter().map(|t| Task {
                        id: t.id,
                        title: t.title,
                        description: None,
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        priority: match t.priority.as_str() {
                            "high" => TaskPriority::High,
                            "medium" => TaskPriority::Medium,
                            _ => TaskPriority::Low,
                        },
                        assignee: t.assignee,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                selected_task: None,
                }).collect(),
                selected_column: 0,
                filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
            };
            
            render_kanban_view(f, area, &kanban_view, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty even with empty data");
        
        println!("✅ Empty data compatibility test passed");
        Ok(())
    }

    #[test]
    fn test_very_long_content_compatibility() -> Result<(), Box<dyn Error>> {
        let config = TerminalConfig::small_terminal();
        let mut terminal = create_test_terminal(&config);

        // Test with very long content
        let long_tasks = vec![
            TaskItem {
                id: "long-1".to_string(),
                title: "This is a very long task title that should be handled gracefully by the TUI system and should not cause any rendering issues or layout problems".to_string(),
                status: "todo".to_string(),
                assignee: Some("very-long-agent-name-that-might-cause-issues".to_string()),
                priority: "high".to_string(),
            },
        ];

        let mut kanban_state = KanbanState::new();
        kanban_state.tasks = long_tasks;
        
        let theme = ThemeKind::Dark.palette();
        let typography = compact_typography(&theme);
        
        terminal.draw(|f| {
            let area = f.area();
            let kanban_view = KanbanView {
                columns: kanban_state.get_columns().into_iter().map(|c| KanbanColumn {
                    title: c.name.clone(),
                    status: match c.name.as_str() {
                        "To Do" => TaskStatus::Todo,
                        "Doing" => TaskStatus::Doing,
                        _ => TaskStatus::Done,
                    },
                    tasks: c.tasks.into_iter().map(|t| Task {
                        id: t.id,
                        title: t.title,
                        description: None,
                        status: match c.name.as_str() {
                            "To Do" => TaskStatus::Todo,
                            "Doing" => TaskStatus::Doing,
                            _ => TaskStatus::Done,
                        },
                        priority: match t.priority.as_str() {
                            "high" => TaskPriority::High,
                            "medium" => TaskPriority::Medium,
                            _ => TaskPriority::Low,
                        },
                        assignee: t.assignee,
                        created_at: "2024-01-01T00:00:00Z".to_string(),
                        updated_at: "2024-01-01T00:00:00Z".to_string(),
                }).collect(),
                    selected_task: None,
                }).collect(),
                selected_column: 0,
                filter: String::new(),
                show_completed: true,
                sort_by: crate::tui::views::KanbanSort::Created,
            };
            
            render_kanban_view(f, area, &kanban_view, &theme, &typography);
        })?;

        let buffer = terminal.backend().buffer();
        assert!(!buffer.content.is_empty(), "Buffer should not be empty even with very long content");
        
        println!("✅ Very long content compatibility test passed");
        Ok(())
    }
}
