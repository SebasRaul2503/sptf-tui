//! In-memory application state.
//!
//! [`AppState`] is the single source of truth for what the UI renders.
//! Mutations happen through small intent-style methods so the event loop
//! never reaches inside private fields.

use ratatui_image::protocol::StatefulProtocol;

use crate::domain::PlayerSnapshot;

#[derive(Default)]
pub struct AppState {
    pub should_quit: bool,
    pub player: PlayerSnapshot,
    /// UI-only ephemeral status banner ("Connecting…", error messages, …).
    pub status_banner: Option<String>,
    /// Currently rendered album-cover protocol.
    pub art: Option<StatefulProtocol>,
    /// URL of the art currently loaded into `art` (or being fetched).
    /// Used to short-circuit redundant loads and to detect staleness when
    /// a fetch task returns *after* the user moved to a different track.
    pub art_url: Option<String>,
}

impl AppState {
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn set_player(&mut self, snapshot: PlayerSnapshot) {
        self.player = snapshot;
    }

    pub fn set_banner(&mut self, banner: impl Into<String>) {
        self.status_banner = Some(banner.into());
    }

    pub fn clear_banner(&mut self) {
        self.status_banner = None;
    }

    /// Current art URL from the active track, if any.
    pub fn current_art_url(&self) -> Option<&str> {
        self.player.track.as_ref().and_then(|t| t.album.as_ref()).and_then(|a| a.art_url.as_deref())
    }
}
