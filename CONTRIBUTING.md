# Contributing to sptf-tui

Thanks for considering a contribution. This project aims to ship a real,
production-quality TUI — so we trade a bit of velocity for a clean, layered
architecture and a strong test suite.

## Ground rules

- The repo follows **SOLID** principles and a layered architecture
  (`domain` → `services` → `infrastructure`/`tui`). New code should fit one of
  the existing layers; if it doesn't, propose a new module in your PR
  description rather than smuggling it into an unrelated file.
- **No `unsafe`** — enforced by `unsafe_code = "forbid"` in `Cargo.toml`.
- **No global mutable state.** Use the existing `AppState` or the service
  layer.
- **Tests are not optional** for non-trivial logic — see
  [Testing requirements](#testing-requirements).
- The TUI must keep working in terminals **without** the kitty graphics
  protocol; album art fallbacks are part of the feature, not an afterthought.

## Development setup

See [`docs/developer-setup.md`](docs/developer-setup.md) for the full
walkthrough. The short version:

```bash
rustup show                        # picks up rust-toolchain.toml
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
cargo run
```

## Pull-request checklist

Before opening a PR, run all of:

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

PRs must:

1. Have a focused scope — prefer multiple small PRs over one large one.
2. Include unit tests for new pure logic.
3. Include doc comments on every public item touched.
4. Update [`docs/architecture.md`](docs/architecture.md) when introducing a new
   layer, trait, or cross-cutting concern.
5. Keep `cargo clippy --all-targets -- -D warnings` clean.

## Commit conventions

We use **Conventional Commits**:

```
feat: add MPRIS player discovery
fix(tui): handle resize when terminal is too small
docs: document keybindings in README
test: cover keymap fallthrough behavior
refactor(state): extract event reducer
chore: bump ratatui to 0.30
```

## Testing requirements

- Pure domain logic: covered by `#[cfg(test)]` modules co-located with the
  code under test.
- Services / infrastructure: covered with mock implementations of the
  relevant trait (e.g. `MockPlayerService`).
- TUI: use `ratatui::backend::TestBackend` for assertion-friendly snapshot
  tests. Avoid asserting on style codes; assert on rendered text or buffer
  symbols.

## Reporting issues

Bug reports should include:

- Terminal emulator and version
- Wayland / X11 / TTY
- `sptf --version`
- Steps to reproduce and the contents of
  `$XDG_STATE_HOME/sptf-tui/logs/sptf.log` if relevant
