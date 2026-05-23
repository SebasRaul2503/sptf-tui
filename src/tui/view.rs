//! Top-level layout. The composition root for what one frame looks like.
//!
//! Keeping `draw` free of a UI struct means tests can drive it against a
//! [`ratatui::backend::TestBackend`] without constructing the full app.
//! Each pane is delegated to a stateless widget in [`crate::widgets`].

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use super::theme::Theme;
use crate::state::AppState;
use crate::widgets;

/// Minimum terminal width that we'll dedicate space to album cover. Below
/// this we drop the art and give the now-playing pane the full width.
const ART_MIN_WIDTH: u16 = 60;
const ART_PANE_WIDTH: u16 = 24;

/// Render one frame of the UI from the given state.
pub fn draw(frame: &mut Frame<'_>, state: &mut AppState, theme: &Theme) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    widgets::help_bar::render(frame, chunks[0], theme);
    draw_body(frame, chunks[1], state, theme);
    widgets::banner::render(frame, chunks[2], state, theme);
}

fn draw_body(frame: &mut Frame<'_>, area: Rect, state: &mut AppState, theme: &Theme) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(7),    // top: art + now playing
            Constraint::Length(4), // controls
            Constraint::Length(3), // volume
        ])
        .split(area);

    let top = rows[0];
    let has_art_url = state.current_art_url().is_some();

    if top.width >= ART_MIN_WIDTH && state.player.is_connected() {
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(ART_PANE_WIDTH), Constraint::Min(10)])
            .split(top);
        widgets::album_art::render(frame, columns[0], state.art.as_mut(), has_art_url, theme);
        widgets::now_playing::render(frame, columns[1], &state.player, theme);
    } else {
        widgets::now_playing::render(frame, top, &state.player, theme);
    }

    widgets::controls::render(frame, rows[1], &state.player, theme);
    widgets::volume::render(frame, rows[2], &state.player, theme);
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    use super::*;
    use crate::domain::{Album, Artist, PlaybackState, PlaybackStatus, PlayerSnapshot, Track};

    fn render_to_string(state: &mut AppState) -> String {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let theme = Theme::default();
        terminal.draw(|f| draw(f, state, &theme)).unwrap();
        terminal.backend().buffer().content().iter().map(ratatui::buffer::Cell::symbol).collect()
    }

    #[test]
    fn renders_when_disconnected() {
        let mut state = AppState::default();
        let dump = render_to_string(&mut state);
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
        let dump = render_to_string(&mut state);
        assert!(dump.contains("Title-Here"));
        assert!(dump.contains("ArtistA"));
        assert!(dump.contains("Album-Here"));
        assert!(dump.contains("Spotify"));
        assert!(dump.contains("0:50"));
        assert!(dump.contains("3:20"));
        assert!(dump.contains("75%"));
    }

    #[test]
    fn renders_album_art_placeholder_when_url_present_but_unloaded() {
        let mut state = AppState::default();
        state.set_player(PlayerSnapshot {
            bus_name: Some("b".into()),
            identity: Some("P".into()),
            track: Some(Track {
                title: "T".into(),
                album: Some(Album {
                    title: "A".into(),
                    artist: "x".into(),
                    art_url: Some("https://example.com/x.png".into()),
                }),
                ..Track::default()
            }),
            playback: PlaybackState::stopped(),
        });
        let dump = render_to_string(&mut state);
        assert!(dump.contains("Cover"));
        assert!(dump.contains("loading"));
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
        let dump = render_to_string(&mut state);
        assert!(dump.contains("Paused"));
    }
}
