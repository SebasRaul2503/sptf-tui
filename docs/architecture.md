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

Presentation. `tui::view::draw` is the *only* function that produces a frame;
widgets are reusable building blocks called from `view`. `input` turns key
events into typed `Action` values. `rendering` handles album-art protocol
selection.

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
  ┌──────────┐    ┌─────────────┐    ┌─────────┐
  │ terminal │──▶ │ EventSource │──▶ │ App     │
  └──────────┘    │ (tokio task)│    │ on_event│
  ┌──────────┐    │             │    │  ↓      │
  │  tick    │──▶ │             │    │ dispatch│──▶ AppState
  └──────────┘    │             │    │         │     ↓
  ┌──────────┐    │             │    │         │   tui::view::draw
  │ MPRIS    │──▶ │             │    │         │     ↓
  │ (future) │    └─────────────┘    └─────────┘   terminal.draw()
  └──────────┘
```

Key properties:

- A single typed `AppEvent` enum funnels every input source.
- The render path is **pull-based**: after handling an event the loop calls
  `terminal.draw(...)` once. There is no separate render task fighting for
  the screen.
- Long-running work (HTTP fetches, image decoding) runs on `tokio::spawn`
  and posts results back through an `AppEvent` variant (added in iteration 4).

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
