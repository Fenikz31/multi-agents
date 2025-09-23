//! TUI application runtime backed by ratatui + crossterm
//! 
//! This provides the event loop, terminal setup/teardown, and delegates
//! rendering to the current `StateManager` by wrapping its string output
//! into a simple Paragraph for now. This establishes the infrastructure
//! required by M6.2; dedicated views/components will evolve hereafter.

use std::error::Error;
use std::io;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
// use ratatui::style::{Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

use super::state::{StateManager, StateTransition};
use super::TuiError;
use super::themes::{Theme, ThemeKind, Typography, default_typography, compact_typography, high_density_typography};

/// TUI App using ratatui/crossterm
pub struct TuiRuntime {
    state_manager: StateManager,
    tick_rate: Duration,
    running: bool,
    current_theme: ThemeKind,
    prefix_g: bool,
    current_mode: DisplayMode,
}

impl TuiRuntime {
    /// Create a new runtime with a default tick of 200ms
    pub fn new(state_manager: StateManager) -> Self {
        Self { state_manager, tick_rate: Duration::from_millis(200), running: true, current_theme: ThemeKind::Dark, prefix_g: false, current_mode: DisplayMode::Normal }
    }

    /// Initialize app states and set initial state
    fn initialize_states(&mut self) -> Result<(), Box<dyn Error>> {
        // Add initial states
        self.state_manager.add_state("help".to_string(), Box::new(super::state::navigation_state::HelpState::new()));
        self.state_manager.add_state("project_select".to_string(), Box::new(super::state::navigation_state::ProjectSelectState::new()));
        let mut kanban = super::state::view_state::KanbanState::new();
        // Best-effort load from default DB and first project (to be refined later)
        let _ = kanban.load_from_db("./data/multi-agents.sqlite3", "default-project");
        self.state_manager.add_state("kanban".to_string(), Box::new(kanban));
        self.state_manager.add_state("sessions".to_string(), Box::new(super::state::view_state::SessionsState::new()));

        // Initial state
        self.state_manager.set_current_state("project_select".to_string())?;
        Ok(())
    }

    /// Run the event/render loop
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.initialize_states()?;

        // Terminal setup
        enable_raw_mode().map_err(|e| TuiError::InputError(format!("enable_raw_mode: {}", e)))?;
        let mut stdout = io::stdout();
        execute!(&mut stdout, EnterAlternateScreen).map_err(|e| TuiError::RenderError(format!("enter alt screen: {}", e)))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).map_err(|e| TuiError::RenderError(format!("terminal: {}", e)))?;
        terminal.hide_cursor().ok();

        let mut last_tick = Instant::now();
        let tick_rate = self.tick_rate;

        let res = (|| -> Result<(), Box<dyn Error>> {
            while self.running {
                // Render current state as text for now
                let output = self.state_manager.render()?;
                terminal.draw(|f| {
                    let size = f.area();
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(100)].as_ref())
                        .split(size);

                    // Apply display mode to typography
                    let base_palette = self.current_theme.palette();
                    let typography: Typography = match self.current_mode {
                        DisplayMode::Normal => default_typography(&base_palette),
                        DisplayMode::Compact => compact_typography(&base_palette),
                        DisplayMode::HighDensity => high_density_typography(&base_palette),
                    };
                    let theme = Theme::with_typography(self.current_theme, typography);
                    let block = Block::default().title(Line::from(vec![Span::raw("Multi-Agents TUI")])).borders(Borders::ALL);
                    let para = Paragraph::new(output).block(block).style(theme.type_scale.body);
                    f.render_widget(para, chunks[0]);
                })?;

                let timeout = tick_rate.saturating_sub(last_tick.elapsed());
                if event::poll(timeout)? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            match key.code {
                                KeyCode::Char('q') => {
                                    self.running = false;
                                }
                                KeyCode::Char('g') => { self.prefix_g = true; }
                                KeyCode::Char('T') => {
                                    if self.prefix_g { self.cycle_theme(); }
                                    self.prefix_g = false;
                                }
                                KeyCode::Char('M') => {
                                    if self.prefix_g { self.cycle_mode(); }
                                    self.prefix_g = false;
                                }
                                KeyCode::Char('h') => {
                                    self.process_input("h")?;
                                    self.prefix_g = false;
                                }
                                KeyCode::Char('k') => {
                                    self.process_input("k")?;
                                    self.prefix_g = false;
                                }
                                KeyCode::Char('s') => {
                                    self.process_input("s")?;
                                    self.prefix_g = false;
                                }
                                KeyCode::Up => { self.process_input("up")?; }
                                KeyCode::Down => { self.process_input("down")?; }
                                KeyCode::Left => { self.process_input("left")?; }
                                KeyCode::Right => { self.process_input("right")?; }
                                KeyCode::Home => { self.process_input("home")?; }
                                KeyCode::End => { self.process_input("end")?; }
                                KeyCode::PageUp => { self.process_input("pageup")?; }
                                KeyCode::PageDown => { self.process_input("pagedown")?; }
                                KeyCode::Tab => { self.process_input("tab")?; }
                                KeyCode::BackTab => { self.process_input("backtab")?; }
                                KeyCode::Enter => { self.process_input("enter")?; }
                                _ => {}
                            }
                        }
                    }
                }
                if last_tick.elapsed() >= tick_rate {
                    last_tick = Instant::now();
                }
            }
            Ok(())
        })();

        // Teardown
        terminal.show_cursor().ok();
        // Leave alternate screen without consuming the terminal backend
        let mut stdout = io::stdout();
        execute!(&mut stdout, LeaveAlternateScreen).ok();
        disable_raw_mode().ok();

        // bubble up any error after teardown
        res
    }

    fn process_input(&mut self, input: &str) -> Result<(), Box<dyn Error>> {
        let transition = self.state_manager.handle_input(input)?;
        match transition {
            StateTransition::Exit => { self.running = false; }
            other => { self.state_manager.process_transition(other)?; }
        }
        Ok(())
    }

    fn cycle_theme(&mut self) {
        self.current_theme = match self.current_theme {
            ThemeKind::Light => ThemeKind::Dark,
            ThemeKind::Dark => ThemeKind::HighContrast,
            ThemeKind::HighContrast => ThemeKind::Light,
        };
    }

    fn cycle_mode(&mut self) {
        self.current_mode = match self.current_mode {
            DisplayMode::Normal => DisplayMode::Compact,
            DisplayMode::Compact => DisplayMode::HighDensity,
            DisplayMode::HighDensity => DisplayMode::Normal,
        };
    }
}

#[derive(Clone, Copy, Debug)]
enum DisplayMode { Normal, Compact, HighDensity }


