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
  `file://`); bounded LRU in-memory cache (32 entries). Cover image is
  centered inside its pane.
- **Big now-playing title** — in-house `BigTitle` widget renders each
  character in a uniform 4-cell column. Characters in `font8x8`'s BASIC,
  LATIN, and HIRAGANA sets (ASCII, Spanish/French/German diacritics,
  hiragana) are packed into 4×4 Unicode quadrant blocks. Everything else
  (kanji, katakana, emoji, …) is dropped into the middle of its column at
  the terminal's native size — so mixed-script titles like
  `君 feat. ARTIST` stay aligned. Falls back to a single-line bold
  rendering when the pane is too small.
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

### Changed

- `now_playing` pane: vertically centers the title/artist/album stack
  inside the available height instead of top-aligning it.
- `album_art` widget: image is centered inside its pane (was top-left)
  using `protocol.size_for(...)` to compute the painted rect.
- `MprisPlayerService::set_volume`: optimistically updates the cached
  snapshot after a successful DBus call. Spotify in particular does not
  always emit `PropertiesChanged` for Volume; without this, repeated
  `+`/`-` presses kept computing the same target from the stale cache.
- `App.dispatch`: after a successful player action, pushes
  `player.snapshot()` to `AppState` immediately so the UI updates without
  waiting for a signal that may never arrive.

### Fixed

- **Progress bar and elapsed time stayed frozen** while playback advanced.
  Root cause: zbus caches `#[zbus(property)]` values and only invalidates
  them on `PropertiesChanged`. The MPRIS spec forbids that signal for
  `Position`, so the cached value was stuck on the first read. Fix: mark
  `position` with `emits_changed_signal = "false"` to bypass the cache.
- **Next/Prev felt broken; cover and title didn't update on track change.**
  Same property-cache class of bug, slightly different shape: our
  `PropertiesChanged` subscriber stream raced zbus's internal cache
  updater, so `metadata()`/`playback_status()`/`volume()` often returned
  the *previous* track's values on the very signal announcing the new
  one. Fix: mark all four observed Player properties as
  `emits_changed_signal = "false"`. Every read is now a fresh
  `Properties.Get` call; the cost is ~4 extra DBus round-trips per
  refresh, which at signal-driven cadence is negligible.

### Removed

- `tui-big-text` dependency. The crate only consults `font8x8::BASIC_FONTS`
  (ASCII), so titles with any non-ASCII character had gaps where letters
  should be. Replaced by the in-house `widgets::big_title::BigTitle` (see
  Added above).

### Documentation

- `README.md` with installation, keybindings, supported-terminal matrix,
  roadmap, and international-title-rendering notes.
- `CONTRIBUTING.md` with PR checklist and commit conventions.
- `docs/architecture.md` describing the layer dependency graph.
- `docs/developer-setup.md` for first-time contributors.
- `examples/config.toml` reference configuration.
