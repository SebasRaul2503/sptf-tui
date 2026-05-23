//! Strongly-typed configuration structs.
//!
//! Each section is its own struct to keep [`super::Settings`] focused on
//! composition rather than knowing about every individual field.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// UI-related preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UiSettings {
    /// Name of the theme to load. Must match one of the built-in theme names
    /// (`spotify-dark`, `gruvbox-dark`, `monochrome`).
    pub theme: String,
    /// Frame rate cap for the render loop.
    pub frame_rate: u16,
    /// Whether to render album covers when supported by the terminal.
    pub show_album_art: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self { theme: "spotify-dark".to_string(), frame_rate: 30, show_album_art: true }
    }
}

/// Preferences for selecting/handling MPRIS players.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PlayerSettings {
    /// Preferred player name (substring match on the MPRIS bus name).
    /// `None` means: pick the first active player.
    pub preferred: Option<String>,
    /// Poll interval for position updates when the player does not push them.
    pub position_poll_ms: u64,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self { preferred: Some("spotify".to_string()), position_poll_ms: 500 }
    }
}

/// Logging preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LoggingSettings {
    /// Default tracing filter applied when `SPTF_LOG` is unset.
    pub level: String,
}

impl Default for LoggingSettings {
    fn default() -> Self {
        Self { level: "warn".to_string() }
    }
}

/// User keybinding overrides.
///
/// Map of *action name* to a list of key strings. Missing actions retain
/// their default binding(s); custom bindings *extend* rather than replace
/// the defaults so users don't have to repeat them.
///
/// Action names: `quit`, `toggle_play_pause` (alias `play_pause`), `next`,
/// `previous`, `volume_up`, `volume_down`, `seek_forward`, `seek_backward`,
/// `redraw`.
///
/// Key syntax: `(modifier-)*key` — e.g. `ctrl-c`, `shift-tab`, `esc`,
/// `space`, `q`, `right`, `f1`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields, transparent)]
pub struct KeymapSettings(pub HashMap<String, Vec<String>>);

impl KeymapSettings {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.0.iter()
    }
}
