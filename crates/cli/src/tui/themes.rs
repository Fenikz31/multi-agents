//! TUI themes (light/dark/high-contrast)
//! Provides color palettes and helpers to style ratatui widgets consistently.

use ratatui::style::{Color, Style, Modifier};

#[derive(Clone, Debug)]
pub struct ThemePalette {
    pub primary: Color,
    pub secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub background: Color,
    pub surface: Color,
    pub text: Color,
}

#[derive(Clone, Copy, Debug)]
pub enum ThemeKind {
    Light,
    Dark,
    HighContrast,
}

impl ThemeKind {
    pub fn palette(self) -> ThemePalette {
        match self {
            ThemeKind::Light => ThemePalette {
                primary: Color::Blue,
                secondary: Color::Gray,
                success: Color::Green,
                warning: Color::Yellow,
                error: Color::Red,
                background: Color::White,
                surface: Color::Rgb(248, 250, 252),
                text: Color::Black,
            },
            ThemeKind::Dark => ThemePalette {
                primary: Color::LightBlue,
                secondary: Color::DarkGray,
                success: Color::LightGreen,
                warning: Color::LightYellow,
                error: Color::LightRed,
                background: Color::Black,
                surface: Color::Gray,
                text: Color::White,
            },
            ThemeKind::HighContrast => ThemePalette {
                primary: Color::Yellow,
                secondary: Color::White,
                success: Color::Green,
                warning: Color::Magenta,
                error: Color::Red,
                background: Color::Black,
                surface: Color::Black,
                text: Color::White,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Typography {
    pub title: Style,
    pub subtitle: Style,
    pub body: Style,
    pub caption: Style,
}

pub fn default_typography(palette: &ThemePalette) -> Typography {
    Typography {
        title: Style::default().fg(palette.text).add_modifier(Modifier::BOLD),
        subtitle: Style::default().fg(palette.secondary),
        body: Style::default().fg(palette.text),
        caption: Style::default().fg(palette.secondary),
    }
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub kind: ThemeKind,
    pub palette: ThemePalette,
    pub type_scale: Typography,
}

impl Theme {
    pub fn new(kind: ThemeKind) -> Self {
        let palette = kind.palette();
        let type_scale = default_typography(&palette);
        Self { kind, palette, type_scale }
    }

    pub fn button_primary(&self) -> Style {
        Style::default().fg(self.palette.text).bg(self.palette.primary).add_modifier(Modifier::BOLD)
    }

    pub fn button_surface(&self) -> Style {
        Style::default().fg(self.palette.text).bg(self.palette.surface)
    }

    pub fn badge_success(&self) -> Style {
        Style::default().fg(self.palette.text).bg(self.palette.success).add_modifier(Modifier::BOLD)
    }

    pub fn badge_warning(&self) -> Style { Style::default().fg(self.palette.text).bg(self.palette.warning) }
    pub fn badge_error(&self) -> Style { Style::default().fg(self.palette.text).bg(self.palette.error) }
}


