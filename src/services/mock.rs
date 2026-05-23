//! In-memory [`PlayerService`] for tests.
//!
//! Records every call so assertions can verify *what* the UI tried to do.
//! Volume is clamped to 0..=100 the same way the real implementation
//! enforces it. `next` / `previous` advance through a pre-loaded playlist
//! when one is provided.
//!
//! This type lives in the main crate so both unit tests and `tests/`
//! integration tests can share it; it is intentionally not feature-gated.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use super::PlayerService;
use crate::core::error::PlayerError;
use crate::domain::{PlaybackStatus, PlayerSnapshot, Track};

/// Record of an action performed on the mock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockCall {
    PlayPause,
    Next,
    Previous,
    SetVolume(u8),
    SeekRelative(i64),
}

#[derive(Default)]
struct Inner {
    snapshot: PlayerSnapshot,
    calls: Vec<MockCall>,
    fail_next: Option<PlayerError>,
    playlist: Vec<Track>,
    cursor: usize,
}

pub struct MockPlayerService {
    inner: Mutex<Inner>,
}

impl MockPlayerService {
    pub fn new(snapshot: PlayerSnapshot) -> Arc<Self> {
        Arc::new(Self { inner: Mutex::new(Inner { snapshot, ..Inner::default() }) })
    }

    pub fn with_playlist(snapshot: PlayerSnapshot, playlist: Vec<Track>) -> Arc<Self> {
        Arc::new(Self { inner: Mutex::new(Inner { snapshot, playlist, ..Inner::default() }) })
    }

    /// Replace the snapshot the mock will return on the next call.
    pub fn set_snapshot(&self, snapshot: PlayerSnapshot) {
        if let Ok(mut g) = self.inner.lock() {
            g.snapshot = snapshot;
        }
    }

    /// Cause the *next* action call to fail with this error. One-shot.
    pub fn fail_next(&self, err: PlayerError) {
        if let Ok(mut g) = self.inner.lock() {
            g.fail_next = Some(err);
        }
    }

    pub fn calls(&self) -> Vec<MockCall> {
        self.inner.lock().map(|g| g.calls.clone()).unwrap_or_default()
    }

    fn record(&self, call: MockCall) -> Result<(), PlayerError> {
        let mut guard = self.inner.lock().map_err(|_| PlayerError::Dbus("poisoned".into()))?;
        if let Some(err) = guard.fail_next.take() {
            return Err(err);
        }
        guard.calls.push(call);
        Ok(())
    }
}

#[async_trait]
impl PlayerService for MockPlayerService {
    fn snapshot(&self) -> PlayerSnapshot {
        self.inner.lock().map(|g| g.snapshot.clone()).unwrap_or_default()
    }

    async fn play_pause(&self) -> Result<(), PlayerError> {
        self.record(MockCall::PlayPause)?;
        let mut g = self.inner.lock().unwrap();
        g.snapshot.playback.status = match g.snapshot.playback.status {
            PlaybackStatus::Playing => PlaybackStatus::Paused,
            _ => PlaybackStatus::Playing,
        };
        Ok(())
    }

    async fn next(&self) -> Result<(), PlayerError> {
        self.record(MockCall::Next)?;
        let mut g = self.inner.lock().unwrap();
        if !g.playlist.is_empty() {
            g.cursor = (g.cursor + 1) % g.playlist.len();
            let track = g.playlist[g.cursor].clone();
            g.snapshot.track = Some(track);
        }
        Ok(())
    }

    async fn previous(&self) -> Result<(), PlayerError> {
        self.record(MockCall::Previous)?;
        let mut g = self.inner.lock().unwrap();
        if !g.playlist.is_empty() {
            g.cursor = g.cursor.checked_sub(1).unwrap_or(g.playlist.len() - 1);
            let track = g.playlist[g.cursor].clone();
            g.snapshot.track = Some(track);
        }
        Ok(())
    }

    async fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        let v = volume.min(100);
        self.record(MockCall::SetVolume(v))?;
        if let Ok(mut g) = self.inner.lock() {
            g.snapshot.playback.volume = v;
        }
        Ok(())
    }

    async fn seek_relative(&self, delta_secs: i64) -> Result<(), PlayerError> {
        self.record(MockCall::SeekRelative(delta_secs))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{PlaybackState, Track};

    fn playing() -> PlayerSnapshot {
        PlayerSnapshot {
            bus_name: Some("mock".into()),
            identity: Some("Mock".into()),
            track: Some(Track { title: "First".into(), ..Track::default() }),
            playback: PlaybackState { status: PlaybackStatus::Playing, ..PlaybackState::stopped() },
        }
    }

    #[tokio::test]
    async fn play_pause_toggles_and_records() {
        let svc = MockPlayerService::new(playing());
        svc.play_pause().await.unwrap();
        svc.play_pause().await.unwrap();
        assert_eq!(svc.calls(), vec![MockCall::PlayPause, MockCall::PlayPause]);
        assert_eq!(svc.snapshot().playback.status, PlaybackStatus::Playing);
    }

    #[tokio::test]
    async fn next_and_previous_cycle_through_playlist() {
        let playlist = vec![
            Track { title: "A".into(), ..Track::default() },
            Track { title: "B".into(), ..Track::default() },
            Track { title: "C".into(), ..Track::default() },
        ];
        let svc = MockPlayerService::with_playlist(playing(), playlist);
        svc.next().await.unwrap();
        assert_eq!(svc.snapshot().track.unwrap().title, "B");
        svc.next().await.unwrap();
        assert_eq!(svc.snapshot().track.unwrap().title, "C");
        svc.next().await.unwrap(); // wraps
        assert_eq!(svc.snapshot().track.unwrap().title, "A");
        svc.previous().await.unwrap(); // wraps backward
        assert_eq!(svc.snapshot().track.unwrap().title, "C");
    }

    #[tokio::test]
    async fn set_volume_is_clamped_and_recorded() {
        let svc = MockPlayerService::new(playing());
        svc.set_volume(255).await.unwrap();
        svc.set_volume(0).await.unwrap();
        assert_eq!(svc.snapshot().playback.volume, 0);
        assert_eq!(svc.calls(), vec![MockCall::SetVolume(100), MockCall::SetVolume(0)]);
    }

    #[tokio::test]
    async fn fail_next_is_one_shot() {
        let svc = MockPlayerService::new(playing());
        svc.fail_next(PlayerError::Dbus("boom".into()));
        let err = svc.play_pause().await.unwrap_err();
        assert!(matches!(err, PlayerError::Dbus(_)));
        // No longer fails on the next call.
        assert!(svc.play_pause().await.is_ok());
    }
}
