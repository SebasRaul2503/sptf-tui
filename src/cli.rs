//! Command-line interface definitions.

use std::path::PathBuf;

use clap::Parser;

/// Terminal music controller for Linux (MPRIS/DBus).
#[derive(Debug, Parser)]
#[command(
    name = "sptf",
    version,
    about = "Terminal music controller for Linux (MPRIS/DBus)",
    long_about = None,
)]
pub struct Cli {
    /// Path to an alternative configuration file (TOML).
    #[arg(short, long, value_name = "FILE", env = "SPTF_CONFIG")]
    pub config: Option<PathBuf>,

    /// Increase log verbosity (-v info, -vv debug, -vvv trace).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}
