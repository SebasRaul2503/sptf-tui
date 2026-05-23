# Developer setup

## Prerequisites

| Tool        | Why                                  | Version     |
| ----------- | ------------------------------------ | ----------- |
| `rustup`    | toolchain manager                    | recent      |
| Rust stable | compiler                             | `>= 1.80`   |
| `pkg-config`, `libdbus-1-dev` / `dbus` | required by `zbus` build deps on some distros | recent |
| A modern terminal | kitty / wezterm / foot recommended | — |

On Arch:

```bash
sudo pacman -S rustup base-devel dbus
rustup default stable
```

On Debian/Ubuntu:

```bash
sudo apt install build-essential libdbus-1-dev pkg-config
rustup default stable
```

## Building

```bash
git clone https://github.com/SebasRaul2503/sptf-tui
cd sptf-tui
cargo build
cargo run
```

The pinned toolchain is declared in `rust-toolchain.toml`, so `rustup show`
will fetch the correct version on first invocation.

## Running tests

```bash
cargo test                                  # all unit + integration tests
cargo test --lib                            # library tests only
cargo test -- --nocapture                   # see println!/tracing output
```

## Linting

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
```

The repo treats warnings as errors in CI.

## Logging during development

By default logs are written to the XDG state directory and stderr is silent
(so the TUI is not corrupted). Use `-v` / `-vv` / `-vvv` to also mirror to
stderr:

```bash
cargo run -- -v
```

You can also set `SPTF_LOG`:

```bash
SPTF_LOG=sptf_tui=debug,zbus=warn cargo run
```

## Project layout

```
src/
├── app/             Composition root + event loop + AppEvent channel
├── cli.rs           clap CLI definition
├── config/          XDG-aware layered configuration (file + env)
├── core/            Errors + logging (cross-cutting)
├── domain/          Pure value types (Track, PlaybackState, …)
├── infrastructure/  Adapters to external systems (currently empty;
│                    HTTP album-art fetch lives in rendering/)
├── input/           Key events → typed Actions, modifier-aware parser
├── mpris/           zbus proxies, discovery, parser, signal-driven service
├── rendering/       ArtCache (LRU) + async loader + ratatui-image picker
├── services/        Trait definitions (PlayerService) + MockPlayerService
├── state/           AppState — the single source of truth for the UI
├── tui/             setup_terminal + tui::view::draw + theme registry
├── utils/           Small helpers shared across modules
└── widgets/         help_bar, now_playing, big_title, album_art,
                     controls, volume, banner

docs/                Project documentation
examples/            Reference config.toml
tests/               Integration tests
```

## Adding a new feature

1. **Domain first.** If the feature introduces a new value type, model it in
   `domain/` with `#[cfg(test)]` tests.
2. **Define the boundary.** If it talks to an external system, define a trait
   in `services/` and implement it in `infrastructure/` (or `mpris/`).
3. **Wire it in `app/`.** Resist the temptation to call services directly
   from the UI.
4. **Render it in `tui/`.** Add a widget if it is reusable.
5. **Document it.** Update `README.md` (user-facing) and
   `docs/architecture.md` (contributor-facing) if you introduced a new
   concept.

## Common pitfalls

- **Don't log to stderr in the render path.** It will tear the alt-screen.
  Use `tracing` so logs go to the file appender.
- **Don't block in the event loop.** Spawn tokio tasks and send results
  back through an `AppEvent` variant — see `app/events.rs` for the
  channel design.
- **Don't put DBus types in `domain/`.** That layer is pure; convert at the
  service boundary.
