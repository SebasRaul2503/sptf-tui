//! Strongly-typed configuration structs.
//!
//! Each section is its own struct to keep [`super::Settings`] focused on
//! composition rather than knowing about every individual field.

use serde::{Deserialize, Serialize};

/// UI-related preferences.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct UiSettings {
    /// Name of the theme to load (looked up in `themes/`).
    pub theme: String,
    /// Frame rate cap for the render loop.
    pub frame_rate: u16,
    /// Whether to render album covers when supported by the terminal.
    pub show_album_art: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self { theme: "default".to_string(), frame_rate: 30, show_album_art: true }
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
