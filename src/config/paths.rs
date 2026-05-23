//! Filesystem paths derived from XDG base directories.

use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::core::error::ConfigError;

const QUALIFIER: &str = "io.github";
const ORG: &str = "sptf-tui";
const APP: &str = "sptf-tui";

/// Resolved filesystem locations the app reads or writes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paths {
    pub config_file: PathBuf,
    pub config_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl Paths {
    /// Resolve XDG-based paths for the current user.
    pub fn resolve() -> Result<Self> {
        let dirs = ProjectDirs::from(QUALIFIER, ORG, APP).ok_or(ConfigError::NoConfigDir)?;

        let config_dir = dirs.config_dir().to_path_buf();
        let cache_dir = dirs.cache_dir().to_path_buf();
        // `state_dir` is only available on Linux; fall back to data dir elsewhere.
        let log_dir =
            dirs.state_dir().map_or_else(|| dirs.data_local_dir().join("logs"), |p| p.join("logs"));

        Ok(Self { config_file: config_dir.join("config.toml"), config_dir, cache_dir, log_dir })
    }
}
