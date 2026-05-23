//! Stateless rendering of [`AppState`] onto a [`Frame`].
//!
//! Keeping `draw` free of `&mut self` (no methods on a UI struct) means we can
//! call it from tests with a [`ratatui::backend::TestBackend`] without
//! constructing the full app.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use super::theme::Theme;
use crate::state::AppState;

/// Render one frame of the UI from the given state.
pub fn draw(frame: &mut Frame<'_>, state: &AppState, theme: &Theme) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    draw_header(frame, chunks[0], theme);
    draw_body(frame, chunks[1], state, theme);
    draw_footer(frame, chunks[2], state, theme);
}

fn draw_header(frame: &mut Frame<'_>, area: Rect, theme: &Theme) {
    let title = format!(" sptf-tui v{} ", env!("CARGO_PKG_VERSION"));
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(Span::styled(title, theme.accent))
        .title_alignment(Alignment::Left);

    let hint = Paragraph::new(Line::from(vec![
        Span::styled("Press ", theme.muted),
        Span::styled("q", theme.accent),
        Span::styled(" to quit", theme.muted),
    ]))
    .block(block)
    .alignment(Alignment::Right);

    frame.render_widget(hint, area);
}

fn draw_body(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border)
        .title(Span::styled(" Now Playing ", theme.primary));

    let inner_text = if state.player.is_connected() {
        let track = state.player.track.clone().unwrap_or_default();
        vec![
            Line::from(Span::styled(track.title.clone(), theme.primary)),
            Line::from(Span::styled(track.artists_joined(), theme.accent)),
            Line::from(Span::styled(
                track.album.as_ref().map_or(String::new(), |a| a.title.clone()),
                theme.muted,
            )),
        ]
    } else {
        vec![
            Line::from(Span::styled("No MPRIS player connected.", theme.status_warn)),
            Line::from(""),
            Line::from(Span::styled(
                "Open Spotify (or any MPRIS-compatible player) and start playing.",
                theme.muted,
            )),
        ]
    };

    let paragraph = Paragraph::new(inner_text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(paragraph, area);
}

fn draw_footer(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    let status = state
        .status_banner
        .clone()
        .unwrap_or_else(|| "ready · iteration 1 (bootstrap)".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border);

    let p =
        Paragraph::new(Span::styled(status, theme.muted)).block(block).alignment(Alignment::Left);
    frame.render_widget(p, area);
}

#[cfg(test)]
mod tests {
    use ratatui::{backend::TestBackend, Terminal};

    use super::*;

    #[test]
    fn draw_renders_without_panic_for_default_state() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = AppState::default();
        let theme = Theme::default();
        terminal.draw(|f| draw(f, &state, &theme)).unwrap();
    }

    #[test]
    fn draw_shows_no_player_message_when_disconnected() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = AppState::default();
        let theme = Theme::default();
        terminal.draw(|f| draw(f, &state, &theme)).unwrap();
        let buffer = terminal.backend().buffer();
        let dump: String = buffer.content().iter().map(ratatui::buffer::Cell::symbol).collect();
        assert!(dump.contains("No MPRIS player connected"));
    }
}
