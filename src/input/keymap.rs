//! Key event → [`Action`] lookup table.
//!
//! Defaults are encoded here. User overrides arrive via
//! [`crate::config::KeymapSettings`] and *extend* (rather than replace) the
//! defaults so partial config files still leave the app usable.

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tracing::warn;

use super::action::Action;
use super::key_parser::{self, KeyCombo};
use crate::config::KeymapSettings;

#[derive(Debug, Clone)]
pub struct Keymap {
    bindings: HashMap<KeyCombo, Action>,
}

impl Keymap {
    /// Built-in default bindings.
    pub fn defaults() -> Self {
        let mut bindings: HashMap<KeyCombo, Action> = HashMap::new();
        let none = KeyModifiers::empty();
        for (combo, action) in default_pairs() {
            bindings.insert(combo, action);
        }
        // Ctrl-C always quits, no matter what the user does in their config.
        bindings.insert(KeyCombo::new(KeyCode::Char('c'), KeyModifiers::CONTROL), Action::Quit);
        debug_assert!(!none.contains(KeyModifiers::CONTROL));
        Self { bindings }
    }

    /// Build from user settings, layered over the defaults.
    pub fn from_settings(settings: &KeymapSettings) -> Self {
        let mut km = Self::defaults();
        for (action_name, keys) in settings.iter() {
            let Some(action) = parse_action(action_name) else {
                warn!(action = action_name, "unknown action in [keys] config, ignoring");
                continue;
            };
            for key in keys {
                match key_parser::parse(key) {
                    Some(combo) => {
                        km.bindings.insert(combo, action);
                    }
                    None => warn!(key, "could not parse keybinding"),
                }
            }
        }
        km
    }

    /// Compatibility shim — equivalent to [`Self::defaults`].
    pub fn new() -> Self {
        Self::defaults()
    }

    pub fn lookup(&self, event: KeyEvent) -> Option<Action> {
        self.bindings.get(&KeyCombo::from(event)).copied()
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Self::defaults()
    }
}

fn parse_action(name: &str) -> Option<Action> {
    Some(match name {
        "quit" => Action::Quit,
        "toggle_play_pause" | "play_pause" => Action::TogglePlayPause,
        "next" => Action::Next,
        "previous" | "prev" => Action::Previous,
        "volume_up" | "vol_up" => Action::VolumeUp,
        "volume_down" | "vol_down" => Action::VolumeDown,
        "seek_forward" => Action::SeekForward,
        "seek_backward" => Action::SeekBackward,
        "redraw" | "refresh" => Action::Redraw,
        _ => return None,
    })
}

fn default_pairs() -> Vec<(KeyCombo, Action)> {
    let none = KeyModifiers::empty();
    vec![
        (KeyCombo::new(KeyCode::Char('q'), none), Action::Quit),
        (KeyCombo::new(KeyCode::Esc, none), Action::Quit),
        (KeyCombo::new(KeyCode::Char(' '), none), Action::TogglePlayPause),
        (KeyCombo::new(KeyCode::Char('p'), none), Action::TogglePlayPause),
        (KeyCombo::new(KeyCode::Char('n'), none), Action::Next),
        (KeyCombo::new(KeyCode::Char('>'), none), Action::Next),
        (KeyCombo::new(KeyCode::Char('b'), none), Action::Previous),
        (KeyCombo::new(KeyCode::Char('<'), none), Action::Previous),
        (KeyCombo::new(KeyCode::Char('+'), none), Action::VolumeUp),
        (KeyCombo::new(KeyCode::Char('='), none), Action::VolumeUp),
        (KeyCombo::new(KeyCode::Char('-'), none), Action::VolumeDown),
        (KeyCombo::new(KeyCode::Char('_'), none), Action::VolumeDown),
        (KeyCombo::new(KeyCode::Right, none), Action::SeekForward),
        (KeyCombo::new(KeyCode::Char('l'), none), Action::SeekForward),
        (KeyCombo::new(KeyCode::Left, none), Action::SeekBackward),
        (KeyCombo::new(KeyCode::Char('h'), none), Action::SeekBackward),
        (KeyCombo::new(KeyCode::Char('r'), none), Action::Redraw),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn q_and_esc_quit() {
        let km = Keymap::defaults();
        assert_eq!(km.lookup(key(KeyCode::Char('q'))), Some(Action::Quit));
        assert_eq!(km.lookup(key(KeyCode::Esc)), Some(Action::Quit));
    }

    #[test]
    fn ctrl_c_quits() {
        let km = Keymap::defaults();
        let ev = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(km.lookup(ev), Some(Action::Quit));
    }

    #[test]
    fn space_toggles_playback() {
        let km = Keymap::defaults();
        assert_eq!(km.lookup(key(KeyCode::Char(' '))), Some(Action::TogglePlayPause));
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let km = Keymap::defaults();
        assert!(km.lookup(key(KeyCode::F(12))).is_none());
    }

    #[test]
    fn user_override_extends_defaults() {
        let mut overrides = std::collections::HashMap::new();
        overrides.insert("next".to_string(), vec!["f5".to_string()]);
        let settings = KeymapSettings(overrides);
        let km = Keymap::from_settings(&settings);
        // New binding is active...
        assert_eq!(km.lookup(key(KeyCode::F(5))), Some(Action::Next));
        // ...and old ones survive.
        assert_eq!(km.lookup(key(KeyCode::Char('q'))), Some(Action::Quit));
        assert_eq!(km.lookup(key(KeyCode::Char('n'))), Some(Action::Next));
    }

    #[test]
    fn unknown_actions_and_keys_are_logged_and_skipped() {
        let mut overrides = std::collections::HashMap::new();
        overrides.insert("nuke".to_string(), vec!["k".to_string()]);
        overrides.insert("quit".to_string(), vec!["nope-not-a-key".to_string()]);
        let settings = KeymapSettings(overrides);
        let km = Keymap::from_settings(&settings);
        assert!(km.lookup(key(KeyCode::Char('k'))).is_none());
        assert_eq!(km.lookup(key(KeyCode::Char('q'))), Some(Action::Quit));
    }
}
