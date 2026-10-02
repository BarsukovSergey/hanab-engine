# Repository strategy

Hanab Engine is maintained as an independent repository rather than a GitHub fork.

## Why

The project is expected to diverge materially from the current standard-only upstream in:

- generic variant mechanics;
- six-player support;
- dedicated two-player semantics;
- sticky configuration and immutable game snapshots;
- deterministic live/replay parity and structured explainability;
- a reusable API for Hanab Mobile.

A permanent GitHub fork relationship would not provide much value once those boundaries diverge.

## What remains from upstream

We keep exact provenance by pinning reviewed upstream commit and tree SHAs in `UPSTREAM.md`. Imported baseline code retains its MIT license notice.

Upstream is still actively audited for useful fixes and improvements; changes are deliberately ported rather than blindly merged.
