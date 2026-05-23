//! Map raw key events to [`Action`]s.
//!
//! Iteration 1 ships a hard-coded default; iteration 6 will load this from
//! the config file. The lookup logic itself stays here so callers don't care
//! where the mappings came from.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::action::Action;

#[derive(Debug, Clone)]
pub struct Keymap;

impl Keymap {
    pub fn new() -> Self {
        Self
    }

    /// Look up an action for a raw key event. Returns `None` for unknown
    /// bindings so the event loop can simply skip them.
    pub fn lookup(&self, event: KeyEvent) -> Option<Action> {
        if event.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(event.code, KeyCode::Char('c'))
        {
            return Some(Action::Quit);
        }

        match event.code {
            KeyCode::Esc | KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char(' ' | 'p') => Some(Action::TogglePlayPause),
            KeyCode::Char('n' | '>') => Some(Action::Next),
            KeyCode::Char('b' | '<') => Some(Action::Previous),
            KeyCode::Char('+' | '=') => Some(Action::VolumeUp),
            KeyCode::Char('-' | '_') => Some(Action::VolumeDown),
            KeyCode::Right | KeyCode::Char('l') => Some(Action::SeekForward),
            KeyCode::Left | KeyCode::Char('h') => Some(Action::SeekBackward),
            KeyCode::Char('r') => Some(Action::Redraw),
            _ => None,
        }
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn q_and_esc_quit() {
        let km = Keymap::new();
        assert_eq!(km.lookup(key(KeyCode::Char('q'))), Some(Action::Quit));
        assert_eq!(km.lookup(key(KeyCode::Esc)), Some(Action::Quit));
    }

    #[test]
    fn ctrl_c_quits() {
        let km = Keymap::new();
        let ev = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(km.lookup(ev), Some(Action::Quit));
    }

    #[test]
    fn space_toggles_playback() {
        let km = Keymap::new();
        assert_eq!(km.lookup(key(KeyCode::Char(' '))), Some(Action::TogglePlayPause));
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let km = Keymap::new();
        assert!(km.lookup(key(KeyCode::F(12))).is_none());
    }
}
