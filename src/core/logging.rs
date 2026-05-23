//! Tracing initialization.
//!
//! When the application is running inside the alternate screen, writing logs
//! to stderr would corrupt the TUI. We therefore default to a *file* sink
//! placed in the XDG state directory and only emit to stderr when the user
//! explicitly asks for verbose output via `-v`.

use std::fs;

use anyhow::{Context, Result};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use crate::config::Settings;

/// Initialize the global tracing subscriber.
///
/// The returned [`LogGuard`] **must** be kept alive for the lifetime of the
/// program; dropping it flushes any buffered log lines.
pub fn init(settings: &Settings, verbose: u8) -> Result<LogGuard> {
    let log_dir = settings.paths.log_dir.clone();
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("creating log directory at {}", log_dir.display()))?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "sptf.log");
    let (file_writer, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_env("SPTF_LOG").unwrap_or_else(|_| {
        let level = match verbose {
            0 => &settings.logging.level,
            1 => "info",
            2 => "debug",
            _ => "trace",
        };
        EnvFilter::new(format!("sptf_tui={level},warn"))
    });

    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    // Only attach the stderr layer when the user explicitly asked for it.
    // Otherwise the TUI alt-screen would be corrupted by log output.
    let stderr_layer = (verbose > 0).then(|| {
        fmt::layer()
            .with_writer(std::io::stderr)
            .with_ansi(true)
            .with_target(false)
            .with_filter(EnvFilter::new("sptf_tui=debug"))
    });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(stderr_layer)
        .try_init()
        .map_err(|err| anyhow::anyhow!("tracing init failed: {err}"))?;

    Ok(LogGuard { _file: guard })
}

/// Drop-guard that keeps the non-blocking log writer alive.
pub struct LogGuard {
    _file: WorkerGuard,
}
