//! Top-level layout. The composition root for what one frame looks like.
//!
//! Keeping `draw` free of `&mut self` (no methods on a UI struct) means we
//! can call it from tests with a [`ratatui::backend::TestBackend`] without
//! constructing the full app. Each pane is delegated to a stateless widget
//! in [`crate::widgets`] so widgets can be unit-tested in isolation.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use super::theme::Theme;
use crate::state::AppState;
use crate::widgets;

/// Render one frame of the UI from the given state.
pub fn draw(frame: &mut Frame<'_>, state: &AppState, theme: &Theme) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    widgets::help_bar::render(frame, chunks[0], theme);
    draw_body(frame, chunks[1], state, theme);
    widgets::banner::render(frame, chunks[2], state, theme);
}

fn draw_body(frame: &mut Frame<'_>, area: Rect, state: &AppState, theme: &Theme) {
    // Layout: now-playing on top, controls below, volume at the bottom.
    // Heights are chosen to gracefully degrade on small terminals — Min(0)
    // makes the now-playing pane shrinkable rather than vanishing controls.
    let panes = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // now playing
            Constraint::Length(4), // controls
            Constraint::Length(3), // volume
        ])
        .split(area);

    widgets::now_playing::render(frame, panes[0], &state.player, theme);
    widgets::controls::render(frame, panes[1], &state.player, theme);
    widgets::volume::render(frame, panes[2], &state.player, theme);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;
    use crate::domain::{Album, Artist, PlaybackState, PlaybackStatus, PlayerSnapshot, Track};

    fn render_to_string(state: &AppState) -> String {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::default();
        terminal.draw(|f| draw(f, state, &theme)).unwrap();
        terminal.backend().buffer().content().iter().map(ratatui::buffer::Cell::symbol).collect()
    }

    #[test]
    fn renders_when_disconnected() {
        let state = AppState::default();
        let dump = render_to_string(&state);
        assert!(dump.contains("no player connected"));
        assert!(dump.contains("sptf-tui"));
    }

    #[test]
    fn renders_track_when_connected() {
        let mut state = AppState::default();
        state.set_player(PlayerSnapshot {
            bus_name: Some("org.mpris.MediaPlayer2.spotify".into()),
            identity: Some("Spotify".into()),
            track: Some(Track {
                title: "Title-Here".into(),
                artists: vec![Artist { name: "ArtistA".into() }, Artist { name: "ArtistB".into() }],
                album: Some(Album { title: "Album-Here".into(), ..Album::default() }),
                length: Some(Duration::from_secs(200)),
                ..Track::default()
            }),
            playback: PlaybackState {
                status: PlaybackStatus::Playing,
                position: Duration::from_secs(50),
                volume: 75,
            },
        });
        let dump = render_to_string(&state);
        assert!(dump.contains("Title-Here"));
        assert!(dump.contains("ArtistA"));
        assert!(dump.contains("Album-Here"));
        assert!(dump.contains("Spotify"));
        assert!(dump.contains("0:50"));
        assert!(dump.contains("3:20"));
        assert!(dump.contains("75%"));
    }

    #[test]
    fn renders_paused_status() {
        let mut state = AppState::default();
        state.set_player(PlayerSnapshot {
            bus_name: Some("x".into()),
            identity: Some("Player".into()),
            track: Some(Track { title: "T".into(), ..Track::default() }),
            playback: PlaybackState { status: PlaybackStatus::Paused, ..PlaybackState::stopped() },
        });
        let dump = render_to_string(&state);
        assert!(dump.contains("Paused"));
    }
}
