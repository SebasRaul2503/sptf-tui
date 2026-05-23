# sptf-tui

A modern, keyboard-driven terminal music controller for Linux. Built on top of
**MPRIS/DBus** so it works with any compliant player (Spotify, mpv, VLC, …),
with first-class support for Spotify and in-terminal album art rendering.

> **Status:** v0.1 feature-complete. All eight iterations on the roadmap
> have landed: MPRIS, real-time sync, album art, configurable themes &
> keybindings, mock-player test harness, and UX polish.

---

## Screenshots

<!-- Replace with real screenshots once the v0.1 release is tagged. -->

```
╭ sptf-tui v0.1.0 ───────── space play/pause · n next · b prev · q quit ─╮
│                                                                        │
│ ╭─ Cover ─────────╮ ╭─ Now Playing ────────────────────────────────╮  │
│ │                 │ │                                                │  │
│ │   ▓▓▓▓▓▓▓▓▓▓    │ │           Sunflower, Vol. 6                   │  │
│ │   ▓▓░░░░░░▓▓    │ │           Harry Styles                        │  │
│ │   ▓▓░░██░░▓▓    │ │           Fine Line                           │  │
│ │   ▓▓░░░░░░▓▓    │ │                                                │  │
│ │   ▓▓▓▓▓▓▓▓▓▓    │ │                                                │  │
│ ╰─────────────────╯ ╰────────────────────────────────────────────────╯ │
│ ╭─ Controls ──────────────────────────────────────────────────────────╮│
│ │ ▶  Playing  ·  Spotify                                  1:24 / 3:45 ││
│ │ ████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  38%   ││
│ ╰─────────────────────────────────────────────────────────────────────╯│
│ ╭─ Volume ────────────────────────────────────────────────────────────╮│
│ │ Vol  ████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  80% ││
│ ╰─────────────────────────────────────────────────────────────────────╯│
│ ╭─────────────────────────────────────────────────────────────────────╮│
│ │ connected to Spotify                                                ││
│ ╰─────────────────────────────────────────────────────────────────────╯│
╰────────────────────────────────────────────────────────────────────────╯
```

## Features

Shipped:

- Cross-platform terminal bootstrap (alternate screen + raw mode + mouse + paste).
- Async event loop based on `tokio` + `crossterm` event stream.
- Layered TOML configuration via XDG paths with environment overrides.
- Structured tracing logs in `$XDG_STATE_HOME/sptf-tui/logs/` (TUI-safe).
- Keyboard bindings dispatched through a typed `Action` space (testable).
- Built-in `spotify-dark` theme.
- **MPRIS/DBus integration** with zbus 5: player discovery, metadata parsing,
  playback control, signal-driven real-time updates (`PropertiesChanged` +
  `NameOwnerChanged`).
- **Interactive TUI**: now-playing pane, status icon, progress bar, volume
  slider, footer status banner, keybinding cheatsheet.
- **Album cover rendering** via `ratatui-image` with automatic protocol
  selection (Kitty, iTerm2, Sixel, halfblock fallback), async fetch +
  decode + in-memory cache.

Coming next:

- **Configurable keybindings & themes** loaded from disk.
- **Mock player backend** for headless tests + integration-test hardening.
- **UX polish**: bounded LRU disk cache, loading states, smarter resize.

## Installation

Currently from source — package distribution comes once the feature set is stable.

```bash
git clone https://github.com/sebastiancastillo/sptf-tui
cd sptf-tui
cargo install --path .
sptf
```

Minimum Rust version: **1.80**. The pinned toolchain is `stable`.

## Supported terminals

The TUI works on any VT100-compatible emulator. Graphics support is detected
at runtime; non-supporting terminals get a unicode fallback.

| Terminal              | TUI | Album art |
| --------------------- | :-: | :-------: |
| kitty                 | ✅  | ✅ (planned: kitty graphics) |
| WezTerm               | ✅  | ✅ (planned: kitty graphics) |
| Foot (Wayland)        | ✅  | ✅ (planned: sixel) |
| Alacritty             | ✅  | ⚠️ (unicode fallback) |
| GNOME Terminal / KDE Konsole | ✅ | ⚠️ |

## Keybindings

Defaults — customizable in iteration 6.

| Key            | Action                  |
| -------------- | ----------------------- |
| `q` / `Esc`    | Quit                    |
| `Ctrl-C`       | Quit                    |
| `Space` / `p`  | Toggle play / pause     |
| `n` / `>`      | Next track              |
| `b` / `<`      | Previous track          |
| `+` / `=`      | Volume up               |
| `-` / `_`      | Volume down             |
| `→` / `l`      | Seek forward            |
| `←` / `h`      | Seek backward           |
| `r`            | Force redraw            |

## Configuration

Looked up at `$XDG_CONFIG_HOME/sptf-tui/config.toml` (or pass `--config FILE`).

```toml
[ui]
theme = "spotify-dark"  # spotify-dark | gruvbox-dark | monochrome
frame_rate = 30
show_album_art = true

[player]
# Substring matched against the MPRIS bus name; null = first active player.
preferred = "spotify"
position_poll_ms = 500

[logging]
level = "warn"  # any tracing-subscriber EnvFilter directive

# Custom keybindings extend (not replace) the defaults; ctrl-c always quits.
[keys]
quit          = ["q", "esc"]
play_pause    = ["space", "p"]
next          = ["n", ">"]
previous      = ["b", "<"]
volume_up     = ["+", "="]
volume_down   = ["-", "_"]
seek_forward  = ["right", "l"]
seek_backward = ["left", "h"]
redraw        = ["r"]
```

Anything in the file can be overridden by environment variables prefixed
`SPTF_` and using `__` for nesting (e.g. `SPTF_UI__FRAME_RATE=60`).

## Roadmap

| Iteration | Theme                                  | Status |
| --------- | -------------------------------------- | ------ |
| 1         | Bootstrap: cargo, layout, event loop, config, logging | ✅ done |
| 2         | MPRIS player discovery + metadata      | ✅ done |
| 3         | Interactive TUI: layout, controls, progress | ✅ done |
| 4         | Real-time sync (signal-driven updates) | ✅ done |
| 5         | Album cover rendering + cache          | ✅ done |
| 6         | Themes & keybinding configuration      | ✅ done |
| 7         | Test hardening + mock player backend   | ✅ done |
| 8         | UX polish + documentation              | ✅ done |

Long-term ideas (not committed):

- Lyrics view (synced + plain)
- Audio-spectrum visualizer
- Discord Rich Presence
- Last.fm scrobbling

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). The codebase intentionally separates
domain, services, and presentation — please read
[`docs/architecture.md`](docs/architecture.md) before submitting larger changes.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
