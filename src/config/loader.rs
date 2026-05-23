//! Configuration loader.
//!
//! Builds a [`Settings`] value from defaults, an optional TOML file, and
//! environment variables. The resulting struct also carries the resolved
//! [`Paths`] so callers don't have to re-derive them.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use config::{Config, Environment, File, FileFormat};
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::paths::Paths;
use super::settings::{LoggingSettings, PlayerSettings, UiSettings};

/// Fully resolved application settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub ui: UiSettings,
    pub player: PlayerSettings,
    pub logging: LoggingSettings,

    /// Filled in by [`Settings::load`] from the runtime environment; not read
    /// from the TOML file. Kept here so downstream code can borrow it.
    #[serde(skip)]
    pub paths: Paths,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui: UiSettings::default(),
            player: PlayerSettings::default(),
            logging: LoggingSettings::default(),
            paths: Paths {
                config_file: PathBuf::new(),
                config_dir: PathBuf::new(),
                cache_dir: PathBuf::new(),
                log_dir: PathBuf::new(),
            },
        }
    }
}

impl Settings {
    /// Load settings from disk (and env). If `override_path` is `Some`, that
    /// file is required to exist; otherwise the standard XDG file is
    /// consulted and missing files are treated as "use defaults".
    pub fn load(override_path: Option<&Path>) -> Result<Self> {
        let paths = Paths::resolve()?;

        // Defaults are supplied via `#[serde(default)]` on every settings struct
        // — the loader only needs to layer optional file + env sources on top.
        let mut builder = Config::builder();

        let file_to_load = override_path
            .map(Path::to_path_buf)
            .filter(|p| !p.as_os_str().is_empty())
            .or_else(|| paths.config_file.exists().then(|| paths.config_file.clone()));

        if let Some(path) = file_to_load {
            debug!(file = %path.display(), "loading config file");
            builder = builder.add_source(File::from(path).format(FileFormat::Toml).required(true));
        }

        builder =
            builder.add_source(Environment::with_prefix("SPTF").separator("__").try_parsing(true));

        let mut settings: Self = builder
            .build()
            .context("building configuration")?
            .try_deserialize()
            .context("deserializing configuration")?;

        settings.paths = paths;
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let settings = Settings::default();
        assert_eq!(settings.ui.theme, "default");
        assert_eq!(settings.ui.frame_rate, 30);
        assert!(settings.ui.show_album_art);
        assert_eq!(settings.player.preferred.as_deref(), Some("spotify"));
        assert_eq!(settings.player.position_poll_ms, 500);
        assert_eq!(settings.logging.level, "warn");
    }

    #[test]
    fn load_uses_defaults_when_no_file_is_present() {
        // Point the loader at a non-existent file via the override; load() should
        // refuse because we required it. Confirm by also passing None.
        let s = Settings::load(None).expect("load with defaults");
        assert_eq!(s.ui.frame_rate, 30);
    }

    #[test]
    fn load_reads_explicit_file() {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "[ui]\nframe_rate = 12\nshow_album_art = false").unwrap();
        let s = Settings::load(Some(f.path())).expect("load explicit");
        assert_eq!(s.ui.frame_rate, 12);
        assert!(!s.ui.show_album_art);
    }
}
