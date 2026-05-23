//! Cross-layer integration tests using the public `MockPlayerService`.
//!
//! These verify the contract the rest of the app relies on: snapshot reads
//! are infallible, control methods produce observable state changes, and
//! errors surface as `PlayerError`.

use sptf_tui::core::error::PlayerError;
use sptf_tui::domain::{PlaybackState, PlaybackStatus, PlayerSnapshot, Track};
use sptf_tui::services::{MockCall, MockPlayerService, PlayerService};

fn snapshot_with(title: &str) -> PlayerSnapshot {
    PlayerSnapshot {
        bus_name: Some("mock".into()),
        identity: Some("Mock".into()),
        track: Some(Track { title: title.into(), ..Track::default() }),
        playback: PlaybackState { status: PlaybackStatus::Playing, ..PlaybackState::stopped() },
    }
}

#[tokio::test]
async fn snapshot_reflects_setter_changes() {
    let svc = MockPlayerService::new(snapshot_with("Initial"));
    assert_eq!(svc.snapshot().track.unwrap().title, "Initial");

    svc.set_snapshot(snapshot_with("Changed"));
    assert_eq!(svc.snapshot().track.unwrap().title, "Changed");
}

#[tokio::test]
async fn full_control_sequence_records_each_call() {
    let svc = MockPlayerService::new(snapshot_with("x"));
    svc.play_pause().await.unwrap();
    svc.next().await.unwrap();
    svc.previous().await.unwrap();
    svc.set_volume(42).await.unwrap();
    svc.seek_relative(-15).await.unwrap();

    assert_eq!(
        svc.calls(),
        vec![
            MockCall::PlayPause,
            MockCall::Next,
            MockCall::Previous,
            MockCall::SetVolume(42),
            MockCall::SeekRelative(-15),
        ]
    );
}

#[tokio::test]
async fn errors_propagate_with_correct_variant() {
    let svc = MockPlayerService::new(snapshot_with("x"));
    svc.fail_next(PlayerError::PlayerDisconnected);
    let err = svc.next().await.unwrap_err();
    assert!(matches!(err, PlayerError::PlayerDisconnected));
}

#[tokio::test]
async fn snapshot_returns_default_when_unset() {
    // The default snapshot must be safe to render — that's the invariant
    // the UI relies on when no player is connected.
    let svc = MockPlayerService::new(PlayerSnapshot::default());
    let snap = svc.snapshot();
    assert!(!snap.is_connected());
    assert!(snap.track.is_none());
}
