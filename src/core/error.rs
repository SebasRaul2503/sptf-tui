//! Common error types used across the application.
//!
//! We expose a small, semantic enum so adapters (MPRIS, HTTP, image, …) can
//! map their failures to a recoverable category. The binary itself uses
//! [`anyhow::Result`] for ergonomics; this enum is intended for service-level
//! APIs where callers want to *react* to specific failure classes (e.g. the
//! UI showing "Spotify not running" vs. a generic error banner).

use thiserror::Error;

/// Errors that may occur while talking to an MPRIS player.
#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("no MPRIS-compatible player is currently running")]
    NoPlayerAvailable,
    #[error("requested player {0:?} was not found")]
    PlayerNotFound(String),
    #[error("DBus is not available on this session")]
    DbusUnavailable,
    #[error("DBus call failed: {0}")]
    Dbus(String),
    #[error("received malformed metadata from player: {0}")]
    InvalidMetadata(String),
}

/// Errors that may occur while fetching or decoding album art.
#[derive(Debug, Error)]
pub enum ArtError {
    #[error("no album art available for current track")]
    Unavailable,
    #[error("network request for album art failed: {0}")]
    Network(String),
    #[error("failed to decode image: {0}")]
    Decode(String),
    #[error("cache I/O error: {0}")]
    Cache(String),
}

/// Errors that may occur while loading or parsing configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("configuration file at {path} is invalid: {source}")]
    Parse {
        path: String,
        #[source]
        source: anyhow::Error,
    },
    #[error("could not determine configuration directory for the current user")]
    NoConfigDir,
    #[error("I/O error while reading configuration: {0}")]
    Io(#[from] std::io::Error),
}
