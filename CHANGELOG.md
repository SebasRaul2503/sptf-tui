# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- **Bootstrap** — cargo project, layered architecture
  (`domain → state → services ↔ infrastructure ↔ presentation`), tokio +
  ratatui + crossterm event loop, XDG-aware layered TOML configuration with
  environment overrides, TUI-safe file-based tracing.
- **MPRIS integration** — zbus 5 proxies for `org.mpris.MediaPlayer2` and
  `.Player`; bus discovery with substring-match preference; metadata parser
  that handles common MPRIS quirks (Spotify `mpris:trackid` as plain string,
  optional fields).
- **Interactive TUI** — now-playing pane, status icon (Playing / Paused /
  Stopped), progress gauge, volume gauge, footer status banner, compact
  keybinding cheatsheet.
- **Real-time sync** — DBus signal-driven watcher (`PropertiesChanged` +
  `NameOwnerChanged`) plus a low-rate position tick.
- **Album cover rendering** — ratatui-image based pipeline with kitty /
  iTerm2 / sixel / halfblocks fallback; async fetch (`http(s)://`,
  `file://`); bounded LRU in-memory cache (32 entries).
- **Themes** — built-in `spotify-dark`, `gruvbox-dark`, `monochrome`.
- **Configurable keybindings** — `[keys]` config section with action
  aliases (`prev` / `previous`, `play_pause` / `toggle_play_pause`, …) and
  a key string parser (`ctrl-c`, `shift-tab`, `f1`, …). Defaults always
  apply; user bindings extend them. `Ctrl-C` → Quit is hard-wired.
- **Resize handling** — minimum-size guard message for tiny terminals;
  dirty-flag redraws skip frames when nothing has changed.
- **Reconnect** — `r` retries the discovery / connect cycle when no player
  is attached.
- **Test infrastructure** — public `MockPlayerService`, integration tests
  for player control, config layering, and parser edge cases.

### Documentation

- `README.md` with installation, keybindings, supported-terminal matrix,
  roadmap.
- `CONTRIBUTING.md` with PR checklist and commit conventions.
- `docs/architecture.md` describing the layer dependency graph.
- `docs/developer-setup.md` for first-time contributors.
- `examples/config.toml` reference configuration.
