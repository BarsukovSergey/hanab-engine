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

### M4-V1 - Basic clue semantics end-to-end

Complete for the requested base semantic slice:

- replay Good Touch filters active identity IDs using variant stack positions,
  required lower-identity copy availability, and observer-visible live claims
- clue-time exact claims map visible cards, literal clue facts, prior Play
  domains, and pending expected cards into the active identity universe
- production primary resolution accepts generic clues, identity sets, and
  dynamic stack heights, preserving contextual suppressions, separate 8CS,
  rank-2/rank-5 and non-playable Critical Save precedence, and PlayOrSave
- replay keeps broader delayed Play domains; it does not replace connection
  interpretation with immediate `direct_play`
- the unused `direct_level_one` helper is removed; its composition coverage now
  exercises the production resolver. `direct_save` remains shared by historical
  and prospective Save classification with PR #38 scoring behavior preserved
- focused tests cover No Variant parity, sixth-suit identities, Rainbow/Pink
  touching, Black copy counts, exhausted predecessors, and observer safety

## Current M4 boundaries

Important remaining boundaries:

- H-Group production policy is still intentionally No Variant-only
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

M4-V2 - Generic connection engine.

Migrate Prompt/Finesse discovery and progression to generic identities and
variant successor/predecessor semantics, preserving reviewed No Variant
connections and their provenance.

Intentional V2+ boundaries remain: `snapshot_play_identities`, ConnectionManager,
reducer connection/context storage, and advanced recognizers still use standard
cards, fixed stack/count arrays, or `standard_h_group_clue`. The reducer adapts
its No Variant snapshots to dynamic slices at the migrated base semantic calls.
The global No Variant policy gate stays until the remaining M4 slices are ready.
Pink Promise, Black clue-choice rules, and other variant convention packs remain
outside this base semantic slice.
