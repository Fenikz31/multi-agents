//! Simple unit tests for TUI components
//! 
//! Tests for basic TUI functionality without complex dependencies

use crate::tui::components::{Toast, ToastType, ToastQueue, GlobalStatus, GlobalStateIcon};
use crate::tui::themes::{ThemeKind, ThemePalette, Typography, default_typography};

#[cfg(test)]
mod toast_tests {
    use super::*;

    #[test]
    fn test_toast_creation() {
        let toast = Toast::new(ToastType::Info, "Test message", Some(5000));
        assert_eq!(toast.kind, ToastType::Info);
        assert_eq!(toast.message, "Test message");
        assert_eq!(toast.ttl_ms, Some(5000));
    }

    #[test]
    fn test_toast_types() {
        let info_toast = Toast::new(ToastType::Info, "Info", None);
        let success_toast = Toast::new(ToastType::Success, "Success", None);
        let warn_toast = Toast::new(ToastType::Warn, "Warning", None);
        let error_toast = Toast::new(ToastType::Error, "Error", None);

        assert_eq!(info_toast.kind, ToastType::Info);
        assert_eq!(success_toast.kind, ToastType::Success);
        assert_eq!(warn_toast.kind, ToastType::Warn);
        assert_eq!(error_toast.kind, ToastType::Error);
    }

    #[test]
    fn test_toast_icons() {
        assert_eq!(ToastType::Info.icon(), "ℹ");
        assert_eq!(ToastType::Success.icon(), "✓");
        assert_eq!(ToastType::Warn.icon(), "⚠");
        assert_eq!(ToastType::Error.icon(), "✖");
    }

    #[test]
    fn test_toast_queue_creation() {
        let queue = ToastQueue::with_capacity(5);
        assert!(queue.items.is_empty());
        assert_eq!(queue.max_visible, 5);
    }

    #[test]
    fn test_toast_queue_enqueue() {
        let mut queue = ToastQueue::with_capacity(3);
        let toast = Toast::new(ToastType::Info, "Test", None);
        
        queue.enqueue(toast);
        assert_eq!(queue.items.len(), 1);
        assert_eq!(queue.items[0].message, "Test");
    }

    #[test]
    fn test_toast_queue_tick() {
        let mut queue = ToastQueue::with_capacity(3);
        let toast = Toast::new(ToastType::Info, "Test", Some(1000));
        queue.enqueue(toast);
        
        // Tick with 500ms
        queue.tick(500);
        assert_eq!(queue.items.len(), 1);
        assert_eq!(queue.items[0].ttl_ms, Some(500));
        
        // Tick with 600ms (should expire)
        queue.tick(600);
        assert!(queue.items.is_empty());
    }
}

#[cfg(test)]
mod status_tests {
    use super::*;

    #[test]
    fn test_global_status_creation() {
        let status = GlobalStatus {
            project_name: "test-project".to_string(),
            view_name: "kanban".to_string(),
            focus: "tasks".to_string(),
            icon: GlobalStateIcon::Active,
            last_action: Some("loaded".to_string()),
        };
        
        assert_eq!(status.project_name, "test-project");
        assert_eq!(status.view_name, "kanban");
        assert_eq!(status.focus, "tasks");
        assert_eq!(status.icon, GlobalStateIcon::Active);
        assert_eq!(status.last_action, Some("loaded".to_string()));
    }

    #[test]
    fn test_global_status_icons() {
        assert_eq!(GlobalStateIcon::Active.symbol(), "●");
        assert_eq!(GlobalStateIcon::Busy.symbol(), "◐");
        assert_eq!(GlobalStateIcon::Warn.symbol(), "⚠");
        assert_eq!(GlobalStateIcon::Error.symbol(), "✖");
    }

    #[test]
    fn test_global_status_header_text() {
        let status = GlobalStatus {
            project_name: "test-project".to_string(),
            view_name: "kanban".to_string(),
            focus: "tasks".to_string(),
            icon: GlobalStateIcon::Active,
            last_action: Some("loaded".to_string()),
        };
        
        let header = status.header_text();
        assert!(header.contains("test-project"));
        assert!(header.contains("kanban"));
        assert!(header.contains("tasks"));
        assert!(header.contains("●"));
        assert!(header.contains("loaded"));
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn test_theme_kind_palettes() {
        let light_palette = ThemeKind::Light.palette();
        assert_eq!(light_palette.background, ratatui::style::Color::White);
        assert_eq!(light_palette.text, ratatui::style::Color::Black);
        
        let dark_palette = ThemeKind::Dark.palette();
        assert_eq!(dark_palette.background, ratatui::style::Color::Black);
        assert_eq!(dark_palette.text, ratatui::style::Color::White);
        
        let high_contrast_palette = ThemeKind::HighContrast.palette();
        assert_eq!(high_contrast_palette.background, ratatui::style::Color::Black);
        assert_eq!(high_contrast_palette.text, ratatui::style::Color::White);
        assert_eq!(high_contrast_palette.primary, ratatui::style::Color::Yellow);
    }

    #[test]
    fn test_typography_creation() {
        let palette = ThemeKind::Dark.palette();
        let typography = default_typography(&palette);
        
        assert_eq!(typography.title.fg, Some(palette.text));
        assert_eq!(typography.subtitle.fg, Some(palette.secondary));
        assert_eq!(typography.body.fg, Some(palette.text));
        assert_eq!(typography.caption.fg, Some(palette.secondary));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_toast_and_status_integration() {
        let mut queue = ToastQueue::with_capacity(3);
        let toast = Toast::new(ToastType::Info, "Operation started", Some(5000));
        queue.enqueue(toast);
        
        let status = GlobalStatus {
            project_name: "test-project".to_string(),
            view_name: "kanban".to_string(),
            focus: "tasks".to_string(),
            icon: GlobalStateIcon::Busy,
            last_action: Some("processing".to_string()),
        };
        
        // Verify both components work together
        assert_eq!(queue.items.len(), 1);
        assert_eq!(status.icon, GlobalStateIcon::Busy);
        assert_eq!(status.last_action, Some("processing".to_string()));
    }

    #[test]
    fn test_theme_and_components_integration() {
        let theme = ThemeKind::Dark.palette();
        let typography = default_typography(&theme);
        
        let toast = Toast::new(ToastType::Success, "Operation completed", None);
        let status = GlobalStatus {
            project_name: "test-project".to_string(),
            view_name: "sessions".to_string(),
            focus: "list".to_string(),
            icon: GlobalStateIcon::Active,
            last_action: Some("completed".to_string()),
        };
        
        // Verify theme colors
        assert_eq!(theme.background, ratatui::style::Color::Black);
        assert_eq!(theme.text, ratatui::style::Color::White);
        
        // Verify typography
        assert_eq!(typography.title.fg, Some(theme.text));
        assert_eq!(typography.body.fg, Some(theme.text));
        
        // Verify components
        assert_eq!(toast.kind, ToastType::Success);
        assert_eq!(status.icon, GlobalStateIcon::Active);
    }
}
