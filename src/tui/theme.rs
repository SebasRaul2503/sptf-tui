//! Style tokens used by widgets.
//!
//! Iteration 1 ships a single built-in theme. Iteration 6 introduces theme
//! loading from disk, at which point this becomes the *resolved* theme that
//! the widgets actually consume.

use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Style,
    pub accent: Style,
    pub muted: Style,
    pub progress_filled: Style,
    pub progress_empty: Style,
    pub border: Style,
    pub status_ok: Style,
    pub status_warn: Style,
    pub status_err: Style,
}

impl Theme {
    /// Built-in default palette inspired by Spotify's brand colors but kept
    /// terminal-friendly (no truecolor required).
    pub fn spotify_dark() -> Self {
        Self {
            primary: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            accent: Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            muted: Style::default().fg(Color::DarkGray),
            progress_filled: Style::default().fg(Color::Green),
            progress_empty: Style::default().fg(Color::DarkGray),
            border: Style::default().fg(Color::DarkGray),
            status_ok: Style::default().fg(Color::Green),
            status_warn: Style::default().fg(Color::Yellow),
            status_err: Style::default().fg(Color::Red),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::spotify_dark()
    }
}
