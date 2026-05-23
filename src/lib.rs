//! sptf-tui — a terminal music controller for Linux.
//!
//! This crate is organized around a small set of layers:
//!
//! - [`domain`]: pure data structures describing music playback.
//! - [`config`]: typed configuration loaded from disk (XDG).
//! - [`core`]: cross-cutting infrastructure (errors, logging).
//! - [`infrastructure`]: adapters to external systems (`DBus`, HTTP, …).
//! - [`services`]: orchestrators that combine infrastructure + domain.
//! - [`state`]: in-memory application state model.
//! - [`tui`], [`widgets`], [`rendering`], [`input`]: presentation layer.
//! - [`app`]: composition root that wires everything together.
//!
//! The intent is to keep UI, domain logic, and external integrations
//! independent and individually testable.

pub mod app;
pub mod cli;
pub mod config;
pub mod core;
pub mod domain;
pub mod infrastructure;
pub mod input;
pub mod mpris;
pub mod rendering;
pub mod services;
pub mod state;
pub mod tui;
pub mod utils;
pub mod widgets;

use anyhow::Result;

pub use crate::app::App;
pub use crate::cli::Cli;

/// Top-level entry point used by the binary and integration tests.
///
/// Performs early bootstrap (logging + config), constructs the [`App`],
/// runs the event loop, and ensures the terminal is restored on exit.
pub async fn run(args: Cli) -> Result<()> {
    // Load configuration before logging so the logging layer can honor it.
    let settings = config::Settings::load(args.config.as_deref())?;

    let _log_guard = core::logging::init(&settings, args.verbose)?;

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "starting sptf-tui");

    let mut app = App::new(settings);
    let result = app.run().await;

    if let Err(err) = &result {
        tracing::error!(error = ?err, "app exited with error");
    } else {
        tracing::info!("app exited cleanly");
    }
    result
}
