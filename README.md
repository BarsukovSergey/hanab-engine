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
.venv/bin/python -m pip install --requirement scripts/requirements-dev.txt
```

Use focused Rust tests while editing. `scripts/check.sh --fast` runs the broad
non-test quality checks; `scripts/check.sh` is the full local checkpoint. The
current shell validation script is intended for Linux/WSL; CI is the canonical
Linux environment.

Agent instructions are in [`AGENTS.md`](AGENTS.md). Replay-specific review
instructions live separately in
[`docs/replay-agent-workflow.md`](docs/replay-agent-workflow.md).
