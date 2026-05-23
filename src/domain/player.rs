//! A consolidated, immutable snapshot of a player's observable state.

use super::playback::PlaybackState;
use super::track::Track;

/// Single value the UI / services exchange to describe "what is playing".
///
/// This is intentionally a snapshot (no interior mutability, no callbacks):
/// services produce a new one on each update, and the UI diff renders it.
#[derive(Debug, Clone, Default)]
pub struct PlayerSnapshot {
    /// MPRIS bus name (e.g. `org.mpris.MediaPlayer2.spotify`).
    pub bus_name: Option<String>,
    /// User-friendly identity (`Identity` property), e.g. "Spotify".
    pub identity: Option<String>,
    pub track: Option<Track>,
    pub playback: PlaybackState,
}

impl PlayerSnapshot {
    pub fn is_connected(&self) -> bool {
        self.bus_name.is_some()
    }
}
