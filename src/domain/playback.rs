//! Playback-related value objects.

use std::time::Duration;

/// MPRIS-equivalent playback status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackStatus {
    Playing,
    Paused,
    #[default]
    Stopped,
}

impl PlaybackStatus {
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Playing | Self::Paused)
    }
}

/// Current playback position + status for a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlaybackState {
    pub status: PlaybackStatus,
    pub position: Duration,
    pub volume: u8, // 0..=100
}

impl PlaybackState {
    pub const fn stopped() -> Self {
        Self { status: PlaybackStatus::Stopped, position: Duration::ZERO, volume: 100 }
    }

    /// Compute progress as a 0.0–1.0 ratio, given a known total length.
    /// Returns `None` if the length is zero or unknown.
    pub fn progress_ratio(&self, length: Option<Duration>) -> Option<f64> {
        let total = length?.as_secs_f64();
        if total <= 0.0 {
            return None;
        }
        Some((self.position.as_secs_f64() / total).clamp(0.0, 1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_ratio_clamps() {
        let state = PlaybackState {
            status: PlaybackStatus::Playing,
            position: Duration::from_secs(120),
            volume: 50,
        };
        let ratio = state.progress_ratio(Some(Duration::from_secs(60))).unwrap();
        assert!((ratio - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn progress_ratio_handles_missing_length() {
        let state = PlaybackState::stopped();
        assert!(state.progress_ratio(None).is_none());
        assert!(state.progress_ratio(Some(Duration::ZERO)).is_none());
    }

    #[test]
    fn is_active_only_for_play_pause() {
        assert!(PlaybackStatus::Playing.is_active());
        assert!(PlaybackStatus::Paused.is_active());
        assert!(!PlaybackStatus::Stopped.is_active());
    }
}
