//! PulseAudio/PipeWire `pactl` fallback for per-application volume.
//!
//! Some MPRIS clients accept `SetVolume` over D-Bus but silently ignore it
//! (the Spotify Linux client is the canonical offender). When that happens
//! we control the player's sink-input volume directly via `pactl`, which
//! works regardless of the player's MPRIS compliance because it operates
//! one layer below — on the audio-server stream.
//!
//! Caveat for callers to surface if useful: pactl adjusts the *sink-input*
//! attenuation, not the player's internal volume. The number shown in the
//! player's own UI will diverge from what the user hears.

use std::process::Stdio;

use tokio::process::Command;

/// Set the volume of the sink-input owned by `pid` to `volume` (0..=100).
///
/// Returns `true` only when pactl reported success on a matching
/// sink-input. Returns `false` for every "fallback did not apply" case —
/// pactl missing, no sink-input owned by the PID, or non-zero exit —
/// because callers only need to know whether to fall through to an error.
pub(super) async fn try_set_volume_for_pid(pid: u32, volume: u8) -> bool {
    let Some(index) = find_sink_input_for_pid(pid).await else {
        return false;
    };
    let percent = format!("{}%", volume.min(100));
    Command::new("pactl")
        .args(["set-sink-input-volume", &index.to_string(), &percent])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .is_ok_and(|s| s.success())
}

async fn find_sink_input_for_pid(pid: u32) -> Option<u32> {
    let output = Command::new("pactl")
        .args(["list", "sink-inputs"])
        .stdin(Stdio::null())
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_sink_input_for_pid(&stdout, pid)
}

fn parse_sink_input_for_pid(text: &str, pid: u32) -> Option<u32> {
    // Walk the textual `pactl list sink-inputs` output: each record begins
    // with `Sink Input #N`, and the PID appears inside its `Properties:`
    // block as `application.process.id = "<pid>"`. We track the current
    // record's index and return it the first time the PID line matches.
    let pid_marker = format!("application.process.id = \"{pid}\"");
    let mut current: Option<u32> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("Sink Input #") {
            current = rest.trim().parse().ok();
        } else if current.is_some() && trimmed == pid_marker {
            return current;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::parse_sink_input_for_pid;

    #[test]
    fn parses_matching_pid() {
        let sample = "Sink Input #42\n\
            \tDriver: PipeWire\n\
            \tProperties:\n\
            \t\tapplication.process.id = \"9999\"\n\
            Sink Input #43\n\
            \tDriver: PipeWire\n\
            \tProperties:\n\
            \t\tapplication.process.id = \"12345\"\n\
            \t\tapplication.name = \"Spotify\"\n";
        assert_eq!(parse_sink_input_for_pid(sample, 12345), Some(43));
        assert_eq!(parse_sink_input_for_pid(sample, 9999), Some(42));
    }

    #[test]
    fn returns_none_when_pid_absent() {
        let sample = "Sink Input #1\n\
            \tProperties:\n\
            \t\tapplication.process.id = \"7\"\n";
        assert_eq!(parse_sink_input_for_pid(sample, 99), None);
    }

    #[test]
    fn returns_none_for_empty_input() {
        assert_eq!(parse_sink_input_for_pid("", 1), None);
    }
}
