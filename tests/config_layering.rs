//! Configuration loader integration tests.
//!
//! Spot-check the layering rules end-to-end: file overrides defaults, and
//! env vars override the file. We use `tempfile::NamedTempFile` so tests
//! can run in parallel without stepping on a real XDG location.

use std::io::Write;

use sptf_tui::config::Settings;

#[test]
fn defaults_load_when_no_file_passed() {
    let s = Settings::load(None).expect("defaults always load");
    assert_eq!(s.ui.theme, "spotify-dark");
    assert_eq!(s.ui.frame_rate, 30);
    assert_eq!(s.player.position_poll_ms, 500);
}

#[test]
fn explicit_file_overrides_defaults() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        f,
        r#"
[ui]
theme = "gruvbox-dark"
frame_rate = 12
show_album_art = false

[player]
preferred = "mpv"
position_poll_ms = 1000

[logging]
level = "info"
"#
    )
    .unwrap();
    let s = Settings::load(Some(f.path())).expect("load explicit");
    assert_eq!(s.ui.theme, "gruvbox-dark");
    assert_eq!(s.ui.frame_rate, 12);
    assert!(!s.ui.show_album_art);
    assert_eq!(s.player.preferred.as_deref(), Some("mpv"));
    assert_eq!(s.player.position_poll_ms, 1000);
    assert_eq!(s.logging.level, "info");
}

#[test]
fn keymap_section_parses_into_settings() {
    let mut f = tempfile::NamedTempFile::new().unwrap();
    writeln!(
        f,
        r#"
[keys]
quit = ["q", "esc"]
next = ["f5"]
"#
    )
    .unwrap();
    let s = Settings::load(Some(f.path())).expect("load explicit");
    let bindings = s.keys.0;
    assert_eq!(bindings.get("quit").map(Vec::len), Some(2));
    assert_eq!(bindings.get("next").map(Vec::len), Some(1));
}
