//! End-to-end-ish tests for the iteration-1 surface.
//!
//! These exercise the library API without spinning up a real terminal so they
//! are safe to run in CI.

use sptf_tui::{
    config::Settings,
    domain::{PlaybackState, PlaybackStatus, PlayerSnapshot, Track},
    state::AppState,
};

#[test]
fn settings_load_with_defaults() {
    let settings = Settings::load(None).expect("default settings should always load");
    // Sanity-check a couple of values rather than the whole struct so this
    // test does not become a brittle snapshot.
    assert!(!settings.paths.config_dir.as_os_str().is_empty());
    assert!(settings.ui.frame_rate >= 1);
}

#[test]
fn app_state_round_trips_player_snapshot() {
    let mut state = AppState::default();
    assert!(!state.player.is_connected());

    state.set_player(PlayerSnapshot {
        bus_name: Some("org.mpris.MediaPlayer2.spotify".into()),
        identity: Some("Spotify".into()),
        track: Some(Track { title: "Test".into(), ..Track::default() }),
        playback: PlaybackState { status: PlaybackStatus::Playing, ..PlaybackState::stopped() },
    });

    assert!(state.player.is_connected());
    assert_eq!(state.player.identity.as_deref(), Some("Spotify"));
    assert_eq!(state.player.playback.status, PlaybackStatus::Playing);
}

#[test]
fn quit_intent_propagates() {
    let mut state = AppState::default();
    assert!(!state.should_quit);
    state.quit();
    assert!(state.should_quit);
}
