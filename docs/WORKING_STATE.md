# Working State

Updated: 2026-10-04

## Current work

- Milestone: M3 - physical variant mechanics complete for the current target catalog
- Branch: `main`
- Open PRs: none
- Main: `73b6cee8612b3aa9918c1369715eb1deeb25a821`
- Upstream baseline: `Zamiell/hanabi-engine@be00e7852571258af7847af35646c3571f620c64`

## Completed milestones

### M1 - working hanab.live baseline

The original live/replay baseline, traceability, and policy plumbing are merged. Manual live-smoke/tagging remains an operational release step rather than an architecture blocker for the later generic-core work.

### M2 - generic core + 6-player boundary

Complete for the intended milestone boundary.

Implemented:
- 2-6 player core support with the standard hand-size schedule, including 6-player 3-card hands
- `VariantSpec` / `IdentityUniverse` and generic physical IDs
- `IdentitySet(u64)` with active-universe semantics for generic search paths
- dynamic stacks, card counts, world construction, and information-set storage
- variant metadata propagated through `FullState` and `PlayerView`
- No Variant compatibility retained for legacy H-Group/planner APIs

Relevant merged work includes PRs #9, #10, and #12.

### M3 - physical variant mechanics

Complete for the current target catalog:
- No Variant
- Black (6 Suits)
- Rainbow (6 Suits)
- White (6 Suits)
- Pink (6 Suits)
- Brown (6 Suits)
- Prism (6 Suits)
- Omni (6 Suits)
- Null (6 Suits)

Implemented:
- pinned declarative physical variant catalog
- deck composition and multiplicities
- clue color/rank domains and physical touching
- stack/playability/successor behavior and score ceilings
- generic `PhysicalAction` / `GenericClue`
- one authoritative generic physical history with legacy No Variant projection
- variant-aware offline Hanabi Live replay parsing
- variant-aware live snapshot and incremental-session reconstruction
- physical action command encoding
- explicit typed rejection at the legacy convention/policy boundary for non-No-Variant games

Relevant merged work includes PRs #11, #13, #14, and #15.

The physical path is now:

`Hanabi Live variant -> VariantSpec -> physical cards/clues/actions -> FullState / PlayerView<GenericClue> -> replay or live session`

## Intentional standard-only boundary

H-Group convention interpretation and planner strategy remain No Variant-only.

Non-No-Variant games can be parsed and reconstructed physically, but the legacy convention/policy API rejects them explicitly instead of silently applying No Variant semantics.

This is the boundary between M3 and M4.

## Next

Begin M4: variant-aware H-Group semantics for 3-6 players.

Start by introducing generic convention identity/clue vocabulary and explicit variant policy traits/boundaries. Migrate convention rules incrementally; do not perform one repository-wide H-Group rewrite.
