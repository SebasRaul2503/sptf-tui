//! In-memory application state.
//!
//! [`AppState`] is the single source of truth for what the UI renders. Mutations
//! happen through small intent-style methods so the event loop never reaches
//! inside private fields. Future iterations will add reducers / message types.

use crate::domain::PlayerSnapshot;

#[derive(Debug, Default)]
pub struct AppState {
    pub should_quit: bool,
    pub player: PlayerSnapshot,
    /// UI-only ephemeral status banner ("Connecting…", error messages, …).
    pub status_banner: Option<String>,
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
}
