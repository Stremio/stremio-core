# Contributing to stremio-core

Thank you for helping improve stremio-core.

## Before you start

- Search existing issues and pull requests before starting work.
- Discuss large features or architectural changes with the maintainers first.
- Base your branch on `development` and keep each pull request focused on one change.

## Development

stremio-core requires Rust 1.77 or newer (the MSRV is checked in CI).

```sh
cargo build
cargo test
```

When changing code:

- Fix the root cause with the smallest practical change.
- Do not include unrelated refactors, formatting, or cleanup.
- Match the patterns and style of the surrounding code; group crate imports into a single `use crate::{...}` block.
- Follow the runtime discipline: new actions, internal messages, and events belong in `src/runtime/msg/`, and effects must resolve to `Internal` or `Event` messages, never to an `Action`.
- Bump `SCHEMA_VERSION` in `src/constants.rs` and add a migration when changing types persisted to storage.
- Add or update unit tests in `src/unit_tests/` for behavior changes.

## Validation

Run the checks relevant to your change before opening a pull request:

```sh
cargo fmt --all -- --check
cargo clippy --all --no-deps -- -D warnings
cargo test
git diff --check
```

For changes affecting the WASM bridge, also build it:

```sh
cd stremio-core-web && npm ci && npm run build
```

When developing core against stremio-web, run `cargo ww` in `stremio-core-web/` instead — it rebuilds the bridge on every change (dev build), so the web app picks up core changes live. Requires [`cargo-watch`](https://github.com/watchexec/cargo-watch).

## Pull requests

- Use a clear branch name without tool or agent prefixes.
- Keep commit subjects, the pull request title, and its description concise.
- Use subject-only commit messages and keep unrelated changes in separate commits or pull requests.
- Explain what changed, why it changed, and how it was verified.
- Write pull request descriptions and review replies yourself; do not paste generated responses into the review discussion.
