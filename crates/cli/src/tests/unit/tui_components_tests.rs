//! Tests unitaires pour les composants TUI
//! 
//! Tests pour Toast, ToastQueue, GlobalStatus, ThemePalette et Typography

use crate::tui::components::{Toast, ToastQueue, ToastType, GlobalStatus, GlobalStateIcon};
use crate::tui::themes::{ThemePalette, ThemeKind, Typography, default_typography, compact_typography, high_density_typography};
use ratatui::style::Color;

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
        let info_toast = Toast::new(ToastType::Info, "Info", Some(1000));
        let success_toast = Toast::new(ToastType::Success, "Success", Some(1000));
        let warn_toast = Toast::new(ToastType::Warn, "Warning", Some(1000));
        let error_toast = Toast::new(ToastType::Error, "Error", Some(1000));
        
        assert!(matches!(info_toast.kind, ToastType::Info));
        assert!(matches!(success_toast.kind, ToastType::Success));
        assert!(matches!(warn_toast.kind, ToastType::Warn));
        assert!(matches!(error_toast.kind, ToastType::Error));
    }

    #[test]
    fn test_toast_queue_creation() {
        let queue = ToastQueue::with_capacity(10);
        assert_eq!(queue.max_visible, 10);
        assert!(queue.items.is_empty());
    }

    #[test]
    fn test_toast_queue_enqueue() {
        let mut queue = ToastQueue::with_capacity(5);
        
        let toast = Toast::new(ToastType::Info, "Test message", Some(1000));
        queue.enqueue(toast);
        
        assert_eq!(queue.items.len(), 1);
        assert_eq!(queue.items[0].message, "Test message");
        assert_eq!(queue.items[0].ttl_ms, Some(1000));
    }

    #[test]
    fn test_toast_queue_tick() {
        let mut queue = ToastQueue::with_capacity(5);
        
        let toast = Toast::new(ToastType::Info, "Test message", Some(1000));
        queue.enqueue(toast);
        assert_eq!(queue.items.len(), 1);
        
        // Tick should not remove toast with ttl > 0
        queue.tick(500);
        assert_eq!(queue.items.len(), 1);
        assert_eq!(queue.items[0].ttl_ms, Some(500));
        
        // Tick should remove toast when ttl reaches 0
        queue.tick(500);
        assert_eq!(queue.items.len(), 0);
    }

    #[test]
    fn test_toast_queue_max_visible() {
        let mut queue = ToastQueue::with_capacity(2);
        
        let toast1 = Toast::new(ToastType::Info, "Message 1", Some(1000));
        let toast2 = Toast::new(ToastType::Success, "Message 2", Some(1000));
        let toast3 = Toast::new(ToastType::Warn, "Message 3", Some(1000));
        
        queue.enqueue(toast1);
        queue.enqueue(toast2);
        queue.enqueue(toast3);
        
        // Should keep all messages (max_visible is just for rendering)
        assert_eq!(queue.items.len(), 3);
        assert_eq!(queue.items[0].message, "Message 1");
        assert_eq!(queue.items[1].message, "Message 2");
        assert_eq!(queue.items[2].message, "Message 3");
    }
}

#[cfg(test)]
mod global_status_tests {
    use super::*;

    #[test]
    fn test_global_status_creation() {
        let status = GlobalStatus {
            project_name: "Test Project".to_string(),
            view_name: "kanban".to_string(),
            focus: "tasks".to_string(),
            icon: GlobalStateIcon::Active,
            last_action: Some("navigate".to_string()),
        };
        
        assert_eq!(status.project_name, "Test Project");
        assert_eq!(status.view_name, "kanban");
        assert_eq!(status.focus, "tasks");
        assert!(matches!(status.icon, GlobalStateIcon::Active));
        assert_eq!(status.last_action, Some("navigate".to_string()));
        assert!(status.header_text().contains("Test Project"));
    }

    #[test]
    fn test_global_state_icons() {
        let active_status = GlobalStatus {
            project_name: "".to_string(),
            view_name: "".to_string(),
            focus: "".to_string(),
            icon: GlobalStateIcon::Active,
            last_action: None,
        };
        
        let busy_status = GlobalStatus {
            project_name: "".to_string(),
            view_name: "".to_string(),
            focus: "".to_string(),
            icon: GlobalStateIcon::Busy,
            last_action: None,
        };
        
        let warn_status = GlobalStatus {
            project_name: "".to_string(),
            view_name: "".to_string(),
            focus: "".to_string(),
            icon: GlobalStateIcon::Warn,
            last_action: None,
        };
        
        let error_status = GlobalStatus {
            project_name: "".to_string(),
            view_name: "".to_string(),
            focus: "".to_string(),
            icon: GlobalStateIcon::Error,
            last_action: None,
        };
        
        assert!(matches!(active_status.icon, GlobalStateIcon::Active));
        assert!(matches!(busy_status.icon, GlobalStateIcon::Busy));
        assert!(matches!(warn_status.icon, GlobalStateIcon::Warn));
        assert!(matches!(error_status.icon, GlobalStateIcon::Error));
    }
}

#[cfg(test)]
mod theme_tests {
    use super::*;

    #[test]
    fn test_theme_kind_palettes() {
        let light_palette = ThemeKind::Light.palette();
        let dark_palette = ThemeKind::Dark.palette();
        let high_contrast_palette = ThemeKind::HighContrast.palette();
        
        // Test that each theme has a valid palette
        assert!(matches!(light_palette.primary, Color::Blue));
        assert!(matches!(dark_palette.primary, Color::LightBlue));
        assert!(matches!(high_contrast_palette.primary, Color::Yellow));
        
        // Test that palettes are different
        assert_ne!(light_palette.primary, dark_palette.primary);
        assert_ne!(dark_palette.primary, high_contrast_palette.primary);
    }

    #[test]
    fn test_theme_palette_structure() {
        let palette = ThemeKind::Light.palette();
        
        // Test that all required colors are present
        assert!(matches!(palette.primary, Color::Blue));
        assert!(matches!(palette.secondary, Color::Gray));
        assert!(matches!(palette.success, Color::Green));
        assert!(matches!(palette.warning, Color::Yellow));
        assert!(matches!(palette.error, Color::Red));
        assert!(matches!(palette.background, Color::White));
        assert!(matches!(palette.surface, Color::Rgb(248, 250, 252)));
        assert!(matches!(palette.text, Color::Black));
    }

    #[test]
    fn test_typography_defaults() {
        let palette = ThemeKind::Light.palette();
        let default = default_typography(&palette);
        let compact = compact_typography(&palette);
        let high_density = high_density_typography(&palette);
        
        // Test that typographies are different (they might be similar for some fields)
        // The main difference is in caption modifiers between default and compact
        assert_ne!(default.caption, compact.caption);
        // And subtitle modifiers between compact and high_density
        assert_ne!(compact.subtitle, high_density.subtitle);
        
        // Test that all typography fields are present
        assert!(default.title.fg.is_some());
        assert!(default.subtitle.fg.is_some());
        assert!(default.body.fg.is_some());
        assert!(default.caption.fg.is_some());
    }

    #[test]
    fn test_typography_consistency() {
        let palette = ThemeKind::Light.palette();
        let typography = default_typography(&palette);
        
        // Test that typography has consistent structure
        assert!(typography.title.fg.is_some());
        assert!(typography.subtitle.fg.is_some());
        assert!(typography.body.fg.is_some());
        assert!(typography.caption.fg.is_some());
        
        // Test that different typography levels are different
        assert_ne!(typography.title, typography.subtitle);
        assert_ne!(typography.subtitle, typography.body);
        assert_ne!(typography.body, typography.caption);
    }
}
