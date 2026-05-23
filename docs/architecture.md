# sptf-tui Architecture

This document captures the *invariants* of the codebase — the rules new
contributions must respect. It is intentionally short. When tension arises
between this document and the code, fix the code or update this document; do
not let them drift.

## Goals

1. **Decoupled presentation, domain, and I/O.** The TUI must not call into
   DBus. The domain types must not depend on ratatui. Services define traits;
   infrastructure implements them.
2. **Testability before features.** Every layer must be exercisable without a
   real terminal or DBus connection.
3. **Async-safe by construction.** Nothing in the render path may block the
   tokio runtime. Long-running work uses spawned tasks + message passing.
4. **Graceful degradation.** No player, no DBus, no graphics protocol — the
   app keeps running with a clear status message.

## Layers

```
┌────────────────────────────────────────────────────────────────┐
│  app (composition root + event loop)                           │
├────────────────────────────────────────────────────────────────┤
│  tui · widgets · rendering · input         ←  presentation     │
├────────────────────────────────────────────────────────────────┤
│  services                                  ←  traits / orchestration │
├────────────────────────────────────────────────────────────────┤
│  state                                     ←  observable AppState │
├────────────────────────────────────────────────────────────────┤
│  domain                                    ←  pure data        │
├────────────────────────────────────────────────────────────────┤
│  infrastructure · mpris                    ←  adapters (DBus, HTTP) │
├────────────────────────────────────────────────────────────────┤
│  config · core                             ←  cross-cutting    │
└────────────────────────────────────────────────────────────────┘
```

Allowed dependency directions (top depends on bottom). Crossing a layer
horizontally is fine; depending **upward** is not.

### `domain/`

Pure value types (`Track`, `Album`, `Artist`, `PlaybackState`,
`PlayerSnapshot`). No I/O, no async, no ratatui. Tested with plain
`#[cfg(test)] mod tests`.

### `state/`

Holds `AppState` — the single source of truth that the UI reads on every
draw. State mutations happen through small intent-named methods.

### `services/`

Defines traits that describe what the rest of the app *needs* (e.g.
`PlayerService`). Implementations live in `infrastructure/` / `mpris/`. Tests
provide fakes.

### `infrastructure/` and `mpris/`

Concrete adapters to external systems (DBus, HTTP, filesystem cache).
`mpris/` gets its own top-level module because the protocol is large enough
to warrant it, but everything inside it still implements traits from
`services/`.

### `tui/`, `widgets/`, `rendering/`, `input/`

Presentation. `tui::view::draw` is the *only* function that produces a
frame; it composes a vertical layout (help bar → body → status banner) and
delegates each pane to a free function in `widgets/`. Below the
`MIN_USABLE_WIDTH × MIN_USABLE_HEIGHT` threshold it renders a
"terminal too small" placeholder instead of attempting the full layout.

The widgets currently shipped:

- `widgets::help_bar` — top-of-screen version + key cheatsheet.
- `widgets::album_art` — cover pane; renders the cached
  [`StatefulProtocol`](https://docs.rs/ratatui-image) centered inside its
  block, or a `loading…` / `no cover` placeholder.
- `widgets::now_playing` — title (rendered via `BigTitle`) + artist +
  album, vertically centered. Falls back to a single-line bold rendering
  when the pane is too small.
- `widgets::big_title` — Quadrant-resolution title renderer. Every
  character gets a uniform 4-cell-wide × 4-cell-tall slot:
  characters in `font8x8::{BASIC, LATIN, HIRAGANA}` are packed as 4×4
  Unicode quadrant blocks; any other character (kanji, katakana, emoji,
  …) is dropped into the middle of its slot at the terminal's native
  size. This keeps mixed-script titles like `君 feat. ARTIST` aligned.
- `widgets::controls` — status icon + identity + position/length + Gauge
  progress bar.
- `widgets::volume` — Gauge + percentage.
- `widgets::banner` — footer status (connection state or current error).

`input` turns key events into typed `Action` values via a
modifier-aware key string parser, so the config file can rebind any
action without the UI knowing the underlying key codes. `rendering`
handles album-art protocol selection (`ratatui-image::Picker` queries the
terminal once at startup) and owns the bounded-LRU `ArtCache`.

### `app/`

The composition root. The `App` struct owns the terminal, the state, and the
event loop. The loop is a single `tokio::select!` that consumes from one
unified `AppEvent` enum so future event sources (MPRIS notifications, signals)
can be added without restructuring.

### `core/` and `config/`

Cross-cutting: error categories, tracing init, XDG path resolution, layered
configuration loading.

## Event flow

```
  ┌──────────┐    ┌─────────────────┐    ┌─────────┐
  │ terminal │──▶ │ spawn_input_pump│──▶ │ App     │
  └──────────┘    │  + tick         │    │ on_event│──▶ AppState (dirty?)
                  └─────────────────┘    │         │      │
  ┌─────────────┐ ┌─────────────────┐    │         │      ▼
  │ MPRIS DBus  │▶│ realtime_watcher│──▶ │         │   tui::view::draw
  │ signals     │ │  (Properties +  │    │         │      │
  └─────────────┘ │   NameOwner +   │    │         │      ▼
                  │   position tick)│    │         │   terminal.draw()
  ┌─────────────┐ └─────────────────┘    │         │
  │ HTTP / file │ ┌─────────────────┐    │         │
  │ art URL     │▶│ spawn_art_fetch │──▶ │         │
  └─────────────┘ │  (one-shot)     │    │         │
                  └─────────────────┘    └─────────┘
```

Key properties:

- A single typed `AppEvent` enum (`Tick`, `Input`, `PlayerSnapshot`,
  `PlayerError`, `ArtLoaded`, `ArtFailed`, `InputError`) funnels every
  producer. `App` owns the only receiver; every producer task carries a
  cloned `Sender`.
- The render path is **pull-based** and **dirty-flag-gated**: after
  handling an event, `terminal.draw(...)` runs only if any setter on
  `AppState` actually changed something. Position ticks that return the
  same value don't trigger a redraw.
- Long-running work (HTTP fetches, image decoding) runs on
  `tokio::spawn`. The MPRIS watcher subscribes to two DBus signal streams
  (`PropertiesChanged` + `NameOwnerChanged`) and a low-rate position
  ticker; album-art fetches are one-shot tasks per track change.
- The realtime watcher refreshes the full snapshot on every
  `PropertiesChanged`, but bypasses zbus's property cache for `Metadata`,
  `PlaybackStatus`, `Position`, and `Volume` (see *zbus property cache*
  below).

## zbus property cache

By default, `#[zbus(property)]` getters read from a cache that's updated
when `PropertiesChanged` arrives. We mark all four observed Player
properties with `emits_changed_signal = "false"`, so each read becomes a
fresh `Properties.Get` call. Two reasons:

1. `Position` is *spec-forbidden* from emitting `PropertiesChanged`. The
   cache would otherwise stick at the first read and the progress bar
   would never advance.
2. Our `PropertiesChanged` subscriber stream races zbus's internal cache
   updater. Without bypassing the cache, `metadata()` after a track
   change might return the *previous* track — and the UI never updates.

The extra ~4 DBus round-trips per refresh are negligible at
signal-driven cadence.

## Logging

`tracing` with a file appender at `$XDG_STATE_HOME/sptf-tui/logs/sptf.log`.
Stderr is *only* used when the user passes `-v`; otherwise it would corrupt
the alt-screen. The default filter level lives in the config file.

## Error handling

- The binary uses `anyhow::Result` for ergonomics.
- Service-facing APIs return typed errors from `core::error` so the UI can
  *react* (e.g. "Spotify not running" banner vs. generic error toast).
- The app loop **never crashes** on a recoverable error: it updates a status
  banner and keeps going.
