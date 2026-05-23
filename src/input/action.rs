//! High-level user intents.
//!
//! Actions are decoupled from key codes so we can later support keybinding
//! customization and alternative input sources (mouse, MPRIS hotkeys, …).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    TogglePlayPause,
    Next,
    Previous,
    VolumeUp,
    VolumeDown,
    SeekForward,
    SeekBackward,
    Redraw,
}
