//! MPRIS player discovery, metadata parsing, and a concrete
//! [`PlayerService`](crate::services::PlayerService) implementation.
//!
//! Submodules:
//! - [`proxy`]: zbus-generated DBus proxies for `org.mpris.MediaPlayer2` and
//!   its `.Player` sub-interface.
//! - [`discovery`]: scan the session bus for MPRIS-compatible names and pick
//!   one based on user preference.
//! - [`parser`]: pure functions that turn DBus dictionaries into
//!   [`crate::domain::Track`] values — fully unit-testable, no DBus required.
//! - [`service`]: holds the connection + proxies and implements the
//!   `PlayerService` trait. Also exposes a `watch` helper that pushes
//!   snapshots into an `AppSender`.

pub mod discovery;
mod pactl;
pub mod parser;
pub mod proxy;
pub mod service;

pub use service::{spawn_realtime_watcher, MprisPlayerService};
