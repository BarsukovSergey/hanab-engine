# Working State

Updated: 2026-10-02

## Current work

- Milestone: M1 - working hanab.live baseline
- Current branch: `m1/journal-parity`
- Current draft PR: #2
- Base: `main`
- Upstream baseline: `Zamiell/hanabi-engine@be00e7852571258af7847af35646c3571f620c64`

## Merged

PR #1 (`dfe94f4`) established:

- versioned persistent bot configuration
- immutable per-game configuration snapshots
- sticky H-Group level
- pregame-only config mutation
- config/chat freeze beginning at `tableStart`
- hard adapter-level no-table-talk gate during play
- focused live-bridge CI

## PR #2

Implemented:

- canonical semantic public-history fingerprints
- `DecisionKey(table, turn, history_hash)`
- stale-result rejection when public history changes without a turn change
- generation-scoped GameRecord and DecisionRecord journals
- frozen config stored with each live game record
- semantic engine-response fingerprints that ignore transport table IDs
- deterministic replay verifier using a fresh engine session for each recorded decision
- parity CLI and regression tests

## Validation policy

Normal pull-request CI runs:

- minimum supported Rust check
- fast build/lint/docs/dead-code validation
- exhaustive H-Group profile matrix
- dedicated live-bridge Python type/compile/test/CLI checks

The inherited ordinary full Rust replay suite is currently excluded from routine PR CI because the pinned upstream baseline contains replay tests that exceed its existing per-test/job time limits. `scripts/check.sh` without `--fast` remains available for manual/full regression work while that baseline issue is repaired.

## Next

1. Merge PR #2 after current focused CI is green.
2. Normalize existing planner/trace diagnostics into the stable M1 TurnTrace skeleton.
3. Run M1 acceptance checklist and tag the baseline.
