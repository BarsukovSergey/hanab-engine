# Working State

Updated: 2026-10-04

## Current milestone

M4 - variant-aware H-Group semantics for 3-6 players.

## Completed boundaries

### M1 - working hanab.live baseline

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

The first M4 boundary is merged:

- `LogicalDeductions<C = Clue>` and `InformationSet<C = Clue>` accept
  `PlayerView<GenericClue>` directly without translating history labels
- legacy `FullState::view_for()` is explicitly No Variant-only; arbitrary
  variants use `physical_view_for()`

H-Group convention interpretation, convention actions, and planner policy remain
No Variant-only. That is intentional until each semantic slice is migrated.

## Next slice

Make the convention-agnostic baseline/search action boundary variant-aware before
migrating H-Group rules. In generic paths, avoid standard-only helpers such as
`Suit::ALL`, `Rank::ALL`, `Card::index()`, and `CardRank::copies()` when
the active `VariantSpec` is the source of truth.

See [`ROADMAP.md`](ROADMAP.md) for the durable milestone sequence.
