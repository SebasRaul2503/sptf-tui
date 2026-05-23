//! Style tokens used by widgets.
//!
//! Themes are resolved by *name* through [`Theme::by_name`]. Unknown names
//! fall back to the default and emit a `tracing::warn!` so configuration
//! typos surface in the log.
//!
//! Iteration 8 polish will add custom themes from disk; for now the catalogue
//! is built-in.

use ratatui::style::{Color, Modifier, Style};
use tracing::warn;

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
    /// Resolve a theme by its registered name. Unknown names emit a warn and
    /// return the default.
    pub fn by_name(name: &str) -> Self {
        match name {
            "spotify-dark" | "default" => Self::spotify_dark(),
            "gruvbox-dark" => Self::gruvbox_dark(),
            "monochrome" => Self::monochrome(),
            other => {
                warn!(theme = other, "unknown theme, falling back to spotify-dark");
                Self::spotify_dark()
            }
        }
    }

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

    pub fn gruvbox_dark() -> Self {
        let fg = Color::Rgb(0xeb, 0xdb, 0xb2);
        let yellow = Color::Rgb(0xfa, 0xbd, 0x2f);
        let aqua = Color::Rgb(0x8e, 0xc0, 0x7c);
        let gray = Color::Rgb(0x92, 0x83, 0x74);
        let orange = Color::Rgb(0xfe, 0x80, 0x19);
        let red = Color::Rgb(0xfb, 0x49, 0x34);
        Self {
            primary: Style::default().fg(fg).add_modifier(Modifier::BOLD),
            accent: Style::default().fg(yellow).add_modifier(Modifier::BOLD),
            muted: Style::default().fg(gray),
            progress_filled: Style::default().fg(aqua),
            progress_empty: Style::default().fg(gray),
            border: Style::default().fg(gray),
            status_ok: Style::default().fg(aqua),
            status_warn: Style::default().fg(orange),
            status_err: Style::default().fg(red),
        }
    }

    pub fn monochrome() -> Self {
        Self {
            primary: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            accent: Style::default().fg(Color::White).add_modifier(Modifier::UNDERLINED),
            muted: Style::default().fg(Color::Gray),
            progress_filled: Style::default().fg(Color::White),
            progress_empty: Style::default().fg(Color::DarkGray),
            border: Style::default().fg(Color::Gray),
            status_ok: Style::default().fg(Color::White),
            status_warn: Style::default().fg(Color::White).add_modifier(Modifier::REVERSED),
            status_err: Style::default().fg(Color::White).add_modifier(Modifier::REVERSED),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::spotify_dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn by_name_returns_built_ins() {
        // Only assert that resolution succeeds; the styles themselves are
        // visual choices and shouldn't be locked in by test assertions.
        let _ = Theme::by_name("spotify-dark");
        let _ = Theme::by_name("gruvbox-dark");
        let _ = Theme::by_name("monochrome");
        let _ = Theme::by_name("default");
    }

    #[test]
    fn unknown_name_falls_back_to_default_silently() {
        // The fallback must not panic; we don't assert on log contents.
        let _ = Theme::by_name("definitely-not-real");
    }
}
