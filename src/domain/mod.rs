//! Pure domain models used across layers.
//!
//! Nothing in this module depends on `DBus`, ratatui, or any I/O. Types here
//! are intentionally trivial (`Copy` / `Clone` / `PartialEq`) so they can be
//! tested in isolation and serialized for snapshots.

pub mod playback;
pub mod player;
pub mod track;

pub use playback::{PlaybackState, PlaybackStatus};
pub use player::PlayerSnapshot;
pub use track::{Album, Artist, Track};
