# Working State

Updated: 2026-10-02

## Current work

- Milestone: M1 - working hanab.live baseline
- Current branch: `m1/turn-trace-acceptance`
- Current draft PR: #3
- Base: `main`
- Upstream baseline: `Zamiell/hanabi-engine@be00e7852571258af7847af35646c3571f620c64`

## Merged

- PR #1 / `dfe94f4`: persistent config, immutable game snapshot, pregame-only mutation, hard no-table-talk gate.
- PR #2 / `d13e2f1`: DecisionKey, semantic history fingerprints, generation-scoped journals, deterministic live/replay verifier.

## PR #3

Implemented:

- TurnTrace schema v1
- normalized knowledge / convention interpretations / planner candidates
- chosen action and provenance
- explicit empty/deferred hypotheses, connections, rejection reasons, complexity and risk
- persisted TurnTrace path in each DecisionRecord
- M1 acceptance checklist

## Next

1. Get PR #3 focused CI green and merge it.
2. M1 remains a live-smoke candidate until the manual hanab.live checklist is executed with a dedicated bot account.
3. After live smoke, tag the M1 baseline.
4. Begin M2 generic core + 6-player support.
