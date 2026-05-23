//! Metadata models for the currently playing media.

use std::time::Duration;

/// An album an artist released.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Album {
    pub title: String,
    pub artist: String,
    pub art_url: Option<String>,
}

/// A single contributing artist.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Artist {
    pub name: String,
}

/// A single track currently exposed by a media player.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Track {
    pub title: String,
    pub artists: Vec<Artist>,
    pub album: Option<Album>,
    pub length: Option<Duration>,
    /// MPRIS `mpris:trackid`, when supplied; used for identity / cache keys.
    pub track_id: Option<String>,
    /// `xesam:url` when supplied (e.g. `spotify:track:...`).
    pub url: Option<String>,
}

impl Track {
    /// Human-readable single-line artist list.
    pub fn artists_joined(&self) -> String {
        self.artists.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
    }
}
