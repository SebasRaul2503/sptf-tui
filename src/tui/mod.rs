//! Ratatui-based presentation layer.
//!
//! Currently exposes:
//! - [`terminal`]: setup / teardown of the terminal backend.
//! - [`view`]: stateless render functions that take an [`AppState`].
//! - [`theme`]: palette + style tokens used by widgets.

pub mod terminal;
pub mod theme;
pub mod view;

pub use terminal::{restore_terminal, setup_terminal, Tui};
