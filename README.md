# Hanab Engine

Reusable Hanabi engine and hanab.live bot foundation focused on deterministic H-Group play, reproducible analysis, variants, strong two-player semantics, and future reuse by Hanab Mobile.

## Baseline

This repository is an independent derivative of the MIT-licensed `Zamiell/hanabi-engine`.

Pinned upstream baseline:

`be00e7852571258af7847af35646c3571f620c64`

The imported baseline tree is byte-for-byte/tree-identical to that upstream commit. See `UPSTREAM.md` and `SOURCES.md` for provenance and sync policy.

## Repository boundaries

- `hanab-engine`: reusable engine, conventions, planner, replay/recompute core, and initial hanab.live bot runtime.
- `hanab-mobile`: separate mobile client/consumer.
- `hanabi-live`: protocol/mechanics reference, not the home of this bot implementation.

Current implementation plan: `docs/M1_BASELINE.md`.
