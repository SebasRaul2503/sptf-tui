//! Keyboard input handling.
//!
//! [`Action`] is the abstract command space the rest of the app understands;
//! [`Keymap`] turns raw [`crossterm`] events into actions. Tests can construct
//! key events directly to exercise dispatch logic without a terminal.

pub mod action;
pub mod keymap;

pub use action::Action;
pub use keymap::Keymap;
