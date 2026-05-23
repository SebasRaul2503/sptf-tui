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
    /// Marks the state as needing a redraw on the next loop tick.
    pub dirty: bool,
}

impl AppState {
    pub fn quit(&mut self) {
        self.should_quit = true;
        self.dirty = true;
    }

    /// Replace the player snapshot. Marks state dirty *only if* the new
    /// snapshot differs from the current one — saves a redraw on every
    /// 500 ms position tick when nothing actually changed.
    pub fn set_player(&mut self, snapshot: PlayerSnapshot) {
        if !player_eq(&self.player, &snapshot) {
            self.dirty = true;
        }
        self.player = snapshot;
    }

    pub fn set_banner(&mut self, banner: impl Into<String>) {
        let new = banner.into();
        if self.status_banner.as_deref() != Some(new.as_str()) {
            self.dirty = true;
        }
        self.status_banner = Some(new);
    }

    pub fn clear_banner(&mut self) {
        if self.status_banner.is_some() {
            self.dirty = true;
        }
        self.status_banner = None;
    }

    /// Force a redraw on the next tick (e.g. terminal resize).
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Consume the dirty flag and return whether a redraw is needed.
    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// Current art URL from the active track, if any.
    pub fn current_art_url(&self) -> Option<&str> {
        self.player.track.as_ref().and_then(|t| t.album.as_ref()).and_then(|a| a.art_url.as_deref())
    }
}

fn player_eq(a: &PlayerSnapshot, b: &PlayerSnapshot) -> bool {
    // We intentionally compare *every* field — including playback.position,
    // because the position-tick is the whole reason this exists. That works
    // because `Duration::eq` is precise to nanoseconds and adjacent position
    // ticks always differ (real players never report the exact same micros
    // twice in a row).
    a.bus_name == b.bus_name
        && a.identity == b.identity
        && a.track == b.track
        && a.playback == b.playback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_identical_snapshot_does_not_dirty() {
        let mut s = AppState::default();
        s.set_player(PlayerSnapshot::default()); // first time → dirty? actually identical to default
                                                 // The first set with the default value is a no-op (since current
                                                 // PlayerSnapshot is already default). Take and confirm not dirty.
        assert!(!s.take_dirty());
    }

    #[test]
    fn setting_changed_snapshot_dirties() {
        let mut s = AppState::default();
        let new = PlayerSnapshot { bus_name: Some("x".into()), ..Default::default() };
        s.set_player(new);
        assert!(s.take_dirty());
        // Take consumes it.
        assert!(!s.take_dirty());
    }

    #[test]
    fn banner_changes_dirty() {
        let mut s = AppState::default();
        s.set_banner("hello");
        assert!(s.take_dirty());
        s.set_banner("hello"); // same text
        assert!(!s.take_dirty());
        s.clear_banner();
        assert!(s.take_dirty());
    }
}
