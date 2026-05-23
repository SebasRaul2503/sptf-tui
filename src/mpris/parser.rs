//! Pure functions that translate raw MPRIS metadata into [`Track`] values.
//!
//! All functions in this module are deterministic and free of I/O so they
//! can be exhaustively unit-tested by constructing the input HashMap by hand.
//!
//! MPRIS metadata uses [`zvariant::Value`]s, which can be wrapped in nested
//! containers; the helpers here unwrap one layer at a time defensively
//! because not every player conforms to the spec letter-by-letter (notably
//! Spotify sometimes ships `mpris:trackid` as a plain string instead of an
//! [`ObjectPath`](zbus::zvariant::ObjectPath)).

use std::collections::HashMap;
use std::time::Duration;

use zbus::zvariant::{OwnedValue, Value};

use crate::domain::{Album, Artist, PlaybackStatus, Track};

/// Parse a full MPRIS `Metadata` dictionary into a [`Track`].
pub fn parse_metadata(map: &HashMap<String, OwnedValue>) -> Track {
    let title = get_string(map, "xesam:title").unwrap_or_default();
    let artists =
        get_string_array(map, "xesam:artist").into_iter().map(|name| Artist { name }).collect();
    let length =
        get_i64(map, "mpris:length").filter(|v| *v > 0).map(|us| Duration::from_micros(us as u64));
    let track_id = get_string(map, "mpris:trackid");
    let url = get_string(map, "xesam:url");
    let album = build_album(map);

    Track { title, artists, album, length, track_id, url }
}

/// Convert MPRIS `PlaybackStatus` string into the domain enum.
pub fn parse_status(raw: &str) -> PlaybackStatus {
    match raw {
        "Playing" => PlaybackStatus::Playing,
        "Paused" => PlaybackStatus::Paused,
        _ => PlaybackStatus::Stopped,
    }
}

/// Normalize a 0.0..=1.0 (or, in practice, 0..=2.0) MPRIS volume into 0..=100.
pub fn parse_volume(raw: f64) -> u8 {
    let bounded = raw.clamp(0.0, 1.0);
    (bounded * 100.0).round() as u8
}

fn build_album(map: &HashMap<String, OwnedValue>) -> Option<Album> {
    let title = get_string(map, "xesam:album")?;
    if title.is_empty() {
        return None;
    }
    let artist = get_string_array(map, "xesam:albumArtist")
        .into_iter()
        .next()
        .or_else(|| get_string_array(map, "xesam:artist").into_iter().next())
        .unwrap_or_default();
    let art_url = get_string(map, "mpris:artUrl");
    Some(Album { title, artist, art_url })
}

fn get_string(map: &HashMap<String, OwnedValue>, key: &str) -> Option<String> {
    let v = map.get(key)?;
    string_from_value(v)
}

fn get_i64(map: &HashMap<String, OwnedValue>, key: &str) -> Option<i64> {
    let v = map.get(key)?;
    match &**v {
        Value::I64(n) => Some(*n),
        Value::I32(n) => Some(i64::from(*n)),
        Value::U64(n) => i64::try_from(*n).ok(),
        Value::U32(n) => Some(i64::from(*n)),
        _ => None,
    }
}

fn get_string_array(map: &HashMap<String, OwnedValue>, key: &str) -> Vec<String> {
    let Some(v) = map.get(key) else { return Vec::new() };
    match &**v {
        Value::Array(arr) => {
            arr.iter().filter_map(string_from_value).filter(|s| !s.is_empty()).collect()
        }
        _ => string_from_value(v).into_iter().collect(),
    }
}

fn string_from_value(v: &Value<'_>) -> Option<String> {
    match v {
        Value::Str(s) => Some(s.as_str().to_string()),
        Value::ObjectPath(p) => Some(p.as_str().to_string()),
        Value::Signature(s) => Some(s.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn val(v: impl Into<Value<'static>>) -> OwnedValue {
        OwnedValue::try_from(v.into()).expect("convert test value")
    }

    fn str_val(s: &'static str) -> OwnedValue {
        val(Value::from(s))
    }

    fn arr_val(items: &[&'static str]) -> OwnedValue {
        let owned: Vec<String> = items.iter().map(|s| (*s).to_string()).collect();
        let v: Value<'static> = owned.into();
        OwnedValue::try_from(v).unwrap()
    }

    #[test]
    fn parses_title_artist_album_length() {
        let mut map = HashMap::new();
        map.insert("xesam:title".into(), str_val("Test Track"));
        map.insert("xesam:artist".into(), arr_val(&["Alice", "Bob"]));
        map.insert("xesam:album".into(), str_val("Greatest Hits"));
        map.insert("xesam:albumArtist".into(), arr_val(&["Alice"]));
        map.insert("mpris:length".into(), val(180_000_000i64));
        map.insert("mpris:artUrl".into(), str_val("https://example.com/a.jpg"));
        map.insert("xesam:url".into(), str_val("spotify:track:abc"));

        let track = parse_metadata(&map);
        assert_eq!(track.title, "Test Track");
        assert_eq!(track.artists_joined(), "Alice, Bob");
        let album = track.album.expect("album present");
        assert_eq!(album.title, "Greatest Hits");
        assert_eq!(album.artist, "Alice");
        assert_eq!(album.art_url.as_deref(), Some("https://example.com/a.jpg"));
        assert_eq!(track.length, Some(Duration::from_secs(180)));
        assert_eq!(track.url.as_deref(), Some("spotify:track:abc"));
    }

    #[test]
    fn ignores_zero_length() {
        let mut map = HashMap::new();
        map.insert("xesam:title".into(), str_val("x"));
        map.insert("mpris:length".into(), val(0i64));
        let track = parse_metadata(&map);
        assert!(track.length.is_none());
    }

    #[test]
    fn missing_album_yields_none() {
        let mut map = HashMap::new();
        map.insert("xesam:title".into(), str_val("x"));
        let track = parse_metadata(&map);
        assert!(track.album.is_none());
    }

    #[test]
    fn falls_back_to_artist_when_album_artist_missing() {
        let mut map = HashMap::new();
        map.insert("xesam:title".into(), str_val("x"));
        map.insert("xesam:album".into(), str_val("y"));
        map.insert("xesam:artist".into(), arr_val(&["Solo"]));
        let track = parse_metadata(&map);
        assert_eq!(track.album.unwrap().artist, "Solo");
    }

    #[test]
    fn parse_status_handles_known_and_unknown() {
        assert_eq!(parse_status("Playing"), PlaybackStatus::Playing);
        assert_eq!(parse_status("Paused"), PlaybackStatus::Paused);
        assert_eq!(parse_status("Stopped"), PlaybackStatus::Stopped);
        assert_eq!(parse_status("Anything else"), PlaybackStatus::Stopped);
    }

    #[test]
    fn parse_volume_clamps_and_scales() {
        assert_eq!(parse_volume(0.0), 0);
        assert_eq!(parse_volume(0.5), 50);
        assert_eq!(parse_volume(1.0), 100);
        assert_eq!(parse_volume(2.0), 100);
        assert_eq!(parse_volume(-0.5), 0);
    }
}
