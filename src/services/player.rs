//! Abstraction over "the thing that controls a media player".
//!
//! Iteration 2 will provide an MPRIS-backed implementation; iteration 7 a
//! mock implementation for integration tests. The UI / App only know about
//! this trait so the two can be swapped freely.

use async_trait::async_trait;

use crate::core::error::PlayerError;
use crate::domain::PlayerSnapshot;

/// Operations the UI can invoke on the active player.
#[async_trait]
pub trait PlayerService: Send + Sync {
    /// Most recent observable state. Cheap; safe to call every frame.
    fn snapshot(&self) -> PlayerSnapshot;

    async fn play_pause(&self) -> Result<(), PlayerError>;
    async fn next(&self) -> Result<(), PlayerError>;
    async fn previous(&self) -> Result<(), PlayerError>;
    async fn set_volume(&self, volume: u8) -> Result<(), PlayerError>;
    async fn seek_relative(&self, delta_secs: i64) -> Result<(), PlayerError>;
}
