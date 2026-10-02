# Working State

Updated: 2026-10-02

## Current work

- Milestone: M1 - working hanab.live baseline
- Branch: `m1/persistent-config`
- Draft PR: #1
- Base: `main`
- Upstream baseline: `Zamiell/hanabi-engine@be00e7852571258af7847af35646c3571f620c64`

## Implemented in PR #1

- versioned persistent bot configuration
- atomic JSON persistence
- immutable per-game configuration snapshot
- sticky H-Group level for future games
- config commands ignored during active play
- adapter-level suppression of outbound `chat` and `chatPM` while playing
- focused regression tests
- dedicated live-bridge CI job

## Validation

The inherited full Rust quality job is not a reliable baseline signal yet: the pinned upstream replay suite has tests that exceed the existing 120-second per-test limit and the job reaches its 20-minute timeout. The minimum-Rust and exhaustive H-Group profile jobs have passed on this baseline.

The dedicated live-bridge job is the authoritative CI signal for the Python-only M1 changes until the inherited full-suite timeout is repaired.

## Next

1. Get the focused live-bridge checks green.
2. Merge PR #1.
3. Start M1 journal/history fingerprint work on a new branch.
4. Add deterministic live-to-replay parity verification.
