//! Reusable widgets.
//!
//! Each widget is a free function that takes a [`ratatui::Frame`], an area,
//! the data it needs, and the current [`Theme`](crate::tui::theme::Theme).
//! They are intentionally stateless: the caller decides where to place them
//! and the [`crate::state::AppState`] is the single source of truth.

pub mod album_art;
pub mod banner;
pub mod big_title;
pub mod controls;
pub mod help_bar;
pub mod now_playing;
pub mod volume;
