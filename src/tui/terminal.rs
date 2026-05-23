//! Terminal setup / teardown helpers.
//!
//! Centralizing these means callers do not need to remember the correct
//! pairing of `enable_raw_mode` / `EnterAlternateScreen` and their inverses
//! — and tests can avoid the side effects entirely by skipping this module.

use std::io::{self, Stdout};

use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, Show},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Convenience type alias used throughout the UI layer.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Put the terminal in alternate screen + raw mode and return a ratatui
/// [`Terminal`] ready for drawing.
pub fn setup_terminal() -> Result<Tui> {
    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)
        .context("enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).context("create ratatui terminal")
}

/// Restore the terminal to its original state. Safe to call multiple times.
pub fn restore_terminal(terminal: &mut Tui) -> Result<()> {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture, Show);
    let _ = terminal.show_cursor();
    Ok(())
}
