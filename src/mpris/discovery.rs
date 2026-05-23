//! Find MPRIS players on the current session bus.

use zbus::fdo::DBusProxy;
use zbus::names::OwnedBusName;
use zbus::Connection;

use crate::core::error::PlayerError;

const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";

/// List every well-known bus name on the session bus that implements MPRIS.
pub async fn list_mpris_players(conn: &Connection) -> Result<Vec<String>, PlayerError> {
    let proxy = DBusProxy::new(conn).await.map_err(map_zbus)?;
    let names = proxy.list_names().await.map_err(map_fdo)?;

    Ok(names
        .into_iter()
        .map(OwnedBusName::into_inner)
        .map(|n| n.to_string())
        .filter(|n| n.starts_with(MPRIS_PREFIX))
        .collect())
}

/// Pick the best candidate from a list, honoring an optional case-insensitive
/// substring preference (e.g. `"spotify"`).
pub fn select_player<'a>(names: &'a [String], preferred: Option<&str>) -> Option<&'a str> {
    if let Some(pref) = preferred {
        let needle = pref.to_lowercase();
        if let Some(found) = names.iter().find(|n| n.to_lowercase().contains(&needle)) {
            return Some(found.as_str());
        }
    }
    names.first().map(String::as_str)
}

fn map_fdo(err: zbus::fdo::Error) -> PlayerError {
    PlayerError::Dbus(format!("{err}"))
}

fn map_zbus(err: zbus::Error) -> PlayerError {
    PlayerError::Dbus(format!("{err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_prefers_substring_match() {
        let names = vec![
            "org.mpris.MediaPlayer2.mpv".to_string(),
            "org.mpris.MediaPlayer2.spotify".to_string(),
            "org.mpris.MediaPlayer2.VLC".to_string(),
        ];
        assert_eq!(select_player(&names, Some("spotify")), Some("org.mpris.MediaPlayer2.spotify"));
    }

    #[test]
    fn select_is_case_insensitive() {
        let names = vec!["org.mpris.MediaPlayer2.VLC".to_string()];
        assert_eq!(select_player(&names, Some("vlc")), Some("org.mpris.MediaPlayer2.VLC"));
    }

    #[test]
    fn select_falls_back_to_first_when_preference_missing() {
        let names = vec!["org.mpris.MediaPlayer2.mpv".to_string()];
        assert_eq!(select_player(&names, Some("spotify")), Some("org.mpris.MediaPlayer2.mpv"));
    }

    #[test]
    fn select_returns_none_when_empty() {
        assert_eq!(select_player(&[], Some("spotify")), None);
        assert_eq!(select_player(&[], None), None);
    }
}
