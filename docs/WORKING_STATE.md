# Working State

Updated: 2026-10-06

## Current milestone

M4 - variant-aware H-Group semantics for 3-6 players.

The durable M1-M10 sequence remains in [`ROADMAP.md`](ROADMAP.md). The current
M4 implementation sequence is in
[`M4_VERTICAL_EXECUTION_PLAN.md`](M4_VERTICAL_EXECUTION_PLAN.md).

## Completed boundaries

### M1 - working hanab.live baseline

Complete for the intended milestone boundary.

The live/replay baseline, traceability, configuration, and policy plumbing are
merged. Manual live smoke remains an operational release step rather than an
architecture blocker.

### M2 - generic core + 6-player boundary

Complete for the intended milestone boundary.

Implemented:

- 2-6 player core support with the standard hand-size schedule
- `VariantSpec` / `IdentityUniverse` and generic physical IDs
- `IdentitySet(u64)` with active-universe semantics
- dynamic stacks, counts, world construction, and information-set storage
- variant metadata through `FullState` and `PlayerView`
- legacy No Variant compatibility for H-Group/planner APIs

Relevant merged work includes PRs #9, #10, and #12.

### M3 - physical variant mechanics

Complete for the current catalog: No Variant plus Black, Rainbow, White, Pink,
Brown, Prism, Omni, and Null in their 6-Suits editions.

Implemented:

- declarative physical variant catalog
- deck composition, clue domains/touching, stacks/playability, and score ceilings
- generic `PhysicalAction` / `GenericClue`
- authoritative generic physical history with legacy No Variant projection
- variant-aware offline Hanabi Live replay parsing
- variant-aware live snapshot/session reconstruction and action encoding

Relevant merged work includes PRs #11, #13, #14, and #15.

The physical path is:

`Hanabi Live -> VariantSpec -> physical cards/clues/actions -> FullState / PlayerView<GenericClue> -> replay/live session`

## M4 progress

Merged M4 boundaries now include:

- PR #17: generic `LogicalDeductions` / `InformationSet` clue boundary
- PR #18: explicit No Variant gate on the legacy `FullState::view_for()` API
- PR #21: variant-aware convention-free baseline assessment and
  `PhysicalAction` selection
- PR #22: `GenericClue` as the stored H-Group replay clue vocabulary
- PR #23: variant-aware immediate/direct Level 1 Play kernel
- PR #36: variant-aware base 5 / 2 / Critical Save kernel
- PR #37: base Play + Save `direct_level_one` composition
- PR #38: base Save recognition wired into historical replay and prospective
  Save scoring

The migration has therefore passed the physical/generic boundary and started
using variant-aware convention kernels in production. It is not yet valid to
enable full non-No-Variant H-Group policy.

## Current M4 boundaries

Important remaining boundaries:

- H-Group production policy is still intentionally No Variant-only
- `direct_level_one` is not yet the production authority for primary clue
  interpretation
- Good Touch still uses standard-card assumptions
- Prompt/Finesse and broader connection semantics still depend heavily on
  standard `Card`, `Clue`, `Rank::ALL`, and fixed five-stack state
- the H-Group replay reducer still contains standard-only storage such as
  `[u8; 5]` stack heights and `[u8; 25]` public removed counts
- `standard_h_group_clue` remains on active convention paths
- `RuleExecutionContext` still carries a raw H-Group profile rather than a
  resolved variant/convention capability boundary

These are migration boundaries. Do not turn them into a repo-wide generic
rewrite.

## Current execution strategy

M4 work is now organized by vertical production capability rather than one
helper, kernel, convention, or wiring step per pull request.

The planned slices are:

1. M4-V1 - Basic clue semantics end-to-end
2. M4-V2 - Generic connection engine
3. M4-V3 - Focus/chop/tempo/stall/discard families
4. M4-V4 - Advanced connections and bluffs
5. M4-V5 - High-level refinements and remaining standard-only semantic paths
6. M4-V6 - Variant rule pack and production enablement

See [`M4_VERTICAL_EXECUTION_PLAN.md`](M4_VERTICAL_EXECUTION_PLAN.md) for scope,
acceptance, validation, and pull-request sizing rules.

## Next slice

M4-V1 - Basic clue semantics end-to-end.

The target is a production result, not another unwired kernel:

- migrate Good Touch/base Level 1 knowledge semantics to active
  `VariantSpec` / `IdentityUniverse`
- make base Play/Save/primary meaning share a production semantic authority
  where behavior is genuinely common
- remove duplicate base classification made obsolete by that authority
- introduce only the minimum capability/rules boundary that this path
  immediately uses
- migrate fixed stack/count representation only where this path requires it
- preserve No Variant reviewed behavior
- add representative variant semantic regressions

Do not include M5 2-player semantics, broad advanced Prompt/Finesse migration,
clarity/risk policy, or unrelated infrastructure cleanup in M4-V1.
