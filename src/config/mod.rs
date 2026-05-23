//! Application configuration.
//!
//! Settings are loaded from a layered source:
//!
//! 1. Built-in defaults (always available).
//! 2. `~/.config/sptf-tui/config.toml` (or an explicit path via `--config`).
//! 3. `SPTF_` prefixed environment variables (e.g. `SPTF_LOGGING__LEVEL=debug`).
//!
//! XDG directories are resolved through the `directories` crate so this also
//! works on systems that don't set `XDG_*` environment variables.

mod loader;
mod paths;
mod settings;

pub use loader::Settings;
pub use paths::Paths;
pub use settings::{KeymapSettings, LoggingSettings, PlayerSettings, UiSettings};
