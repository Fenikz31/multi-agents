//! Toast notifications component
//!
//! Provides a minimal in-memory queue and rendering helpers for stacked toast
//! notifications in the bottom-right corner of a given area.

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::tui::themes::{ThemePalette, Typography};

/// Type of toast notification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Info,
    Success,
    Warn,
    Error,
}

impl ToastType {
    pub fn icon(&self) -> &'static str {
        match self {
            ToastType::Info => "ℹ",
            ToastType::Success => "✓",
            ToastType::Warn => "⚠",
            ToastType::Error => "✖",
        }
    }
}

/// A single toast entry
#[derive(Debug, Clone)]
pub struct Toast {
    pub kind: ToastType,
    pub message: String,
    /// Remaining time-to-live in milliseconds. None = persistent
    pub ttl_ms: Option<u64>,
}

impl Toast {
    pub fn new(kind: ToastType, message: impl Into<String>, ttl_ms: Option<u64>) -> Self {
        Self { kind, message: message.into(), ttl_ms }
    }
}

/// A simple queue to store pending toasts
#[derive(Debug, Default, Clone)]
pub struct ToastQueue {
    pub items: std::collections::VecDeque<Toast>,
    /// Maximum number of visible toasts at once
    pub max_visible: usize,
}

impl ToastQueue {
    pub fn with_capacity(max_visible: usize) -> Self {
        Self { items: std::collections::VecDeque::new(), max_visible }
    }

    pub fn enqueue(&mut self, toast: Toast) {
        self.items.push_back(toast);
    }

    /// Advance timers and drop expired toasts
    pub fn tick(&mut self, delta_ms: u64) {
        for item in self.items.iter_mut() {
            if let Some(ttl) = item.ttl_ms.as_mut() {
                if *ttl > delta_ms { *ttl -= delta_ms; } else { *ttl = 0; }
            }
        }
        self.items.retain(|t| t.ttl_ms.map(|ttl| ttl > 0).unwrap_or(true));
    }
}

/// Render up to `max_visible` toasts stacked from bottom to top in the right corner.
pub fn render_toasts(
    f: &mut ratatui::Frame,
    area: Rect,
    queue: &ToastQueue,
    theme: &ThemePalette,
    typography: &Typography,
) {
    if queue.items.is_empty() { return; }

    let visible: Vec<&Toast> = queue
        .items
        .iter()
        .rev() // start from the newest at the bottom
        .take(queue.max_visible.max(1))
        .collect();

    // Allocate a vertical stack in the right third of the provided area
    let container_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Percentage(40)].as_ref())
        .split(area);
    let right = container_chunks[1];

    let v_constraints: Vec<Constraint> = visible.iter().map(|_| Constraint::Length(3)).collect();
    let stack = Layout::default()
        .direction(Direction::Vertical)
        .constraints(v_constraints)
        .split(right);

    // Bottom-up stacking: newest (first in `visible`) goes to the last rect
    for (i, toast) in visible.into_iter().enumerate() {
        let idx_from_bottom = stack.len().saturating_sub(1).saturating_sub(i);
        let rect = stack.get(idx_from_bottom).copied().unwrap_or(right);
        let (fg, border) = match toast.kind {
            // Theme has no `info`; use `secondary` for informational toasts
            ToastType::Info => (theme.secondary, theme.secondary),
            ToastType::Success => (theme.success, theme.success),
            ToastType::Warn => (theme.warning, theme.warning),
            ToastType::Error => (theme.error, theme.error),
        };

        let text = format!("[ {} ] {}", toast.kind.icon(), toast.message);
        let para = Paragraph::new(Line::from(text))
            .style(typography.body.fg(fg).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Right)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border)
            );
        f.render_widget(para, rect);
    }
}


