# Hanab Engine

Reusable Hanabi engine and hanab.live bot foundation focused on deterministic
H-Group play, reproducible analysis, physical variants, strong two-player
semantics, and future reuse by Hanab Mobile.

## Current status

Physical rules, replay parsing, and live snapshot/session reconstruction support
the current nine-variant catalog:

- No Variant
- Black, Rainbow, White, Pink, Brown, Prism, Omni, and Null (6 Suits)

The physical path is variant-aware through `VariantSpec`,
`PlayerView<GenericClue>`, `PhysicalAction`, and logical search. H-Group
convention interpretation and planner policy are still intentionally No
Variant-only while M4 migrates convention semantics.

See [current working state](docs/WORKING_STATE.md) and the
[roadmap](docs/ROADMAP.md).

## Workspace

- `hanabi-core`: physical rules, authoritative state, variants, and player-safe
  observations.
- `hanabi-protocol`: Hanabi Live replay/live adapters.
- `hanabi-search`: logical deductions, convention inference, and planning.
- `hanabi-cli`: analysis CLI and the persistent engine process used by the
  online bridge.

Repository boundaries and provenance are documented in
[`UPSTREAM.md`](UPSTREAM.md), [`SOURCES.md`](SOURCES.md), and
[`docs/REPOSITORY_STRATEGY.md`](docs/REPOSITORY_STRATEGY.md).

## Development

The default Rust development toolchain is pinned by
[`rust-toolchain.toml`](rust-toolchain.toml); the separate CI MSRV check remains
Rust 1.85.

Install formatting and Python development dependencies when needed:

```sh
npm ci
python3 -m venv .venv
```

Then install `scripts/requirements-dev.txt` with the venv Python:
`.venv/bin/python` on POSIX or `.venv/Scripts/python.exe` on Windows.

For a small Rust change, use the scoped helper with the affected crate and,
optionally, focused Cargo test arguments:

```sh
bash scripts/check-scoped.sh hanabi-search direct_play --lib
```

The helper reuses the primary worktree Cargo target cache. `scripts/check.sh
--fast` remains the broad non-test checkpoint for shared inference/replay/planner
changes, and `scripts/check.sh` is the full local checkpoint. The shell
validation helpers support Linux/WSL, macOS, and Windows Git Bash; CI remains the
canonical Linux gate.

Agent instructions are in [`AGENTS.md`](AGENTS.md). Replay-specific review
instructions live separately in
[`docs/replay-agent-workflow.md`](docs/replay-agent-workflow.md).
