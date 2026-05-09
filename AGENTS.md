# gesttalt

`gesttalt` is an open-source, Rust-native **agentic coding environment**. It hosts
multiple coding agents inside a single workspace and lets you drive them, compare
them, and orchestrate them against the same project — without leaving the editor.

## What it is

- A desktop application built in Rust on top of [GPUI], the same UI toolkit that
  powers the [Zed] editor. We deliberately follow Zed's UI patterns (workspace
  with title bar, docks, status bar, command palette) so the experience feels
  immediately familiar to anyone coming from Zed.
- A first-class home for **CLI coding agents**. We treat agents like `claude`,
  `codex`, and `opencode` as pluggable backends: each one is detected, queried
  for available models and reasoning levels, and driven over its JSON event
  stream from inside the app.
- Tightly integrated with [`fabrik`][fabrik], our fast build system, so the agent
  loop (edit → build → test → diagnose) runs against incremental, cache-aware
  builds rather than full rebuilds. Fabrik is a first-party dependency, not a
  third-party plugin.

## What it is not

- Not a fork of Zed. We borrow GPUI and the workspace patterns, but the product
  is its own thing: agent orchestration first, editing second.
- Not tied to any single agent vendor. Adding a new CLI agent is a matter of
  writing a small adapter (see `crates/agents`).

## Repo layout

```
crates/
  gesttalt/   desktop app (GPUI workspace, docks, title bar, status bar)
  agents/     adapters for CLI coding agents (claude, codex, opencode)
              + the `agents-probe` binary that exercises them end to end
```

## Working on the project

- Toolchain is pinned via `mise.toml` (Rust 1.95, minimal profile + clippy +
  rustfmt + rust-analyzer + rust-src). Run `mise install` once.
- Build: `cargo build`. Run the desktop app: `cargo run -p gesttalt`.
- Probe the locally installed coding-agent CLIs: `cargo run -p agents --bin agents-probe`.
  This will detect each of `opencode`, `codex`, `claude` on `PATH`, list the
  models they expose, list their reasoning levels, and send a tiny prompt to
  each so you can see live JSON event output.

## Conventions

- Edition 2024, workspace-level lints (see root `Cargo.toml`).
- Keep adapters in `crates/agents` free of GPUI dependencies — they must be
  reusable from headless tools (CI, scripts, the probe binary) as well as the
  desktop app.
- New CLI agent? Implement the `AgentCLI` trait in a new module under
  `crates/agents/src/` and register it in `crates/agents/src/lib.rs::all()`.

[GPUI]: https://www.gpui.rs/
[Zed]: https://zed.dev/
[fabrik]: https://github.com/tuist/fabrik
