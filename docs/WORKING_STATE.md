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

### M4-V2 - Generic connection engine

Complete for historical and prospective basic Prompt/Finesse paths:

- snapshot Play domains, Prompt/Finesse discovery, speculative and committed
  scheduling, loaded planning, and recipient Prompt/completion inference map
  retained `Card` promises through the active `VariantSpec` identity universe
- connection clue matching uses physical variant touches; predecessor traversal
  and dynamic stack simulation use the variant's stack and rank order
- active and queued owner-facing promises, Prompt candidate eligibility, and
  projected stacks before the owner's turn also use active identity IDs
- historical connection planning reads public clue-time dynamic stacks and
  restricted historical identities; prospective delayed-line scoring uses the
  same physical predecessor semantics
- `ConnectionManager`, PromiseId, provenance, Prompt-before-Finesse precedence,
  deferred/ambiguous transfers, Self-Finesse eligibility, Reverse search, and
  Fix lifecycle remain in place
- focused contract tests cover No Variant/sixth-suit parity, Rainbow/Pink touches,
  six-player ordering, nonconsecutive ranks and reordered identity IDs, loaded
  progression, hidden information, and lifecycle/provenance retention; existing
  reviewed replay assertions remain the No Variant semantic authority

### M4-V3 - Focus/chop/tempo/stall/discard families

Complete for the requested historical and prospective base semantic slice:

- turn contexts carry active dynamic stack heights before and after each event;
  the legacy reducer arrays remain available to V4/V5 paths
- current/eventual/accounted trash, endgame coverage, and prior Play narrowing
  use active identity IDs, physical copy counts, and variant stack/rank order
- Trash/5/Order Chop Moves, Tempo/TCCM, Emergency Discards, Positional, and
  Stalling recognition consume active clue facts and dynamic playability
- Gentleman/Baton/Sarcastic transfer recognition uses historical visibility and
  dynamic layer progression; layered Gentleman remains legal and layered Baton
  remains illegal
- prospective Tempo/TCCM/TCM/5CM/5 Stall/Fill-In/Burn facts, first-5-stall chop
  distance, Gentleman candidates, and ordinary convention-trash discards use
  active physical identities; Burn completion scheduling uses variant progression
- focused physical/epistemic contracts cover precedence, sixth suits, custom
  identity and rank order, six-player targeting, hidden-truth independence, and
  prior Play narrowing; reviewed replay tests remain the semantic authority

Tempo still precedes TCM, 5 Stall still precedes 5CM, and Sarcastic transfers
still precede Gentleman/Baton. Chop movement remains protection without an
identity claim or Prompt; Fill-In/Burn/Stall create no ordinary Play promise.
Rank-1/rank-2/rank-5 convention meanings are unchanged.

Local V3 validation: all 10 `v3_focus_context` contracts, scoped formatting/crate
check, and `git diff --check` pass. Five of the eight requested reviewed replay
regressions pass. The other three also fail at the exact starting revision
`3335caa443bb1dbd016f42bd0ad77d55b6c19daa`:

- `second_replay_order_chop_move_protects_alices_red_two`: OCM protection passes;
  the later choice assertion selects 2s to Bob instead of purple to Cathy
- `first_seed_generation_discard_projects_reviewed_direct_green_line`: empty
  projection at the reviewed-line assertion (5s root on V3, purple root on base)
- `fifth_replay_burn_preserves_the_final_playing_clock`: the budget-exhausted
  planner selects discard #35 instead of a clue

Assertions and fixtures remain unchanged. This is a migrated V3 boundary, not
a fully green replay-validation claim. No full suite or fast validation ran.

## Current M4 boundaries

Important remaining boundaries:

- H-Group production policy is still intentionally No Variant-only
- basic connection promises intentionally retain `Card` fields, with active
  identity lookup at the migrated boundaries; broader advanced interpretation,
  knowledge materialization, and convention claims still have legacy domains
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

M4-V5 - High-level refinements and remaining standard-only semantic paths.

M4-V4 migrates Bluff and Double Bluff primitives, historical recognition and
demonstration, the advanced Finesse graph (layered, hidden, clandestine, queued,
ambiguous), Ejection/Discharge, and Priority/Load. These paths use active identity
IDs, physical clue touching, variant predecessor/successor traversal, and dynamic
historical stack heights. Connection lifecycle identities remain Card-valued and
are mapped through the V2 identity helpers.

M4-V5 is partially implemented. The 2026-10-23 batches migrated production
recognizers and direct consumers for Context, Duplication, Elimination, Five
Tech, Out-of-Order, Ignition, and Phantom. Both historical and prospective
execution retain the shared `rule_engine::apply_rule` dispatch and existing
`POST_EVENT_RULES` precedence. These are bounded production-code migrations,
not closure of complete variant replay or M4-V5 acceptance.

| Boundary | Implemented behavior | Implementation checkpoint |
| --- | --- | --- |
| Context | Active physical clue/fact domains, pre-event dynamic stacks and predecessors, historical identities; Fix and existing connection suppression retained | `c61960f` |
| Context owner consumer | Exact reinterpretation claims reach active owner knowledge; literal exclusions still win | `4119cba` |
| Duplication | Active duplicate proof, pre-event trash tests and reconstructed origin-turn stacks; previous secured-copy proof and connection cancellation retained | `5745da8` |
| Elimination | Active copies, dated visibility, before/after playability, literal candidate exclusion/retraction; direct Elimination Finesse and OneOf constraint consumers | `5b3db1d` |
| Five Tech | Active saved-2 masks, dated facts, predecessor distance and connector scheduling; 5 Stall/Save precedence and rank-5 meaning retained | `8eb2e6d` |
| Out-of-Order | Active unplayed predecessors and historical identities; existing connector and exact Fix actor/target retained | `de5e47a` |
| Ignition | Dated dynamic playability/trash and active known-playable actor domains; actor selection and deck-empty guard retained | `994ff61` |
| Phantom | Active safe-action domains, dated predecessors and variant copy counts; visible missing-connector negative proof and owner-belief guard retained | `6ce94f9` |

**PARTIAL / concrete remaining barriers:** sixth-suit historical discards still
reach `history_reducer::ReplayReducer::apply_discard` and its retained
`public_removed[identity.index()]` 25-element array (observed index 25 panic).
`h_group_phase` still reads exactly five stacks (observed index 2 panic for a
two-stack physical test variant). The Elimination consumer contract therefore
seeds replay without historical variant discards and supplies empty inferred
inputs directly to the constraint graph; it does not establish complete variant
replay. These unchanged boundaries require a separate sized migration before
variant enablement. All migrated families remain PARTIAL at the complete
variant-capability level until the shared barriers and remaining consumers close.

Charms, Unnecessary Moves, remaining Extras/Max special effects, Special Finesse
discard evidence, and the public-layer/inverse-planning historical clue adapters
are untouched. The optional adapter inventory was not started: its prerequisite
of coherent earlier queues without outstanding blockers is not met. No unknown
H-Group ruling was invented and no reviewed fixture/assertion was changed.

Each implementation checkpoint passed its focused
`bash scripts/check-scoped.sh hanabi-search <filter> --lib` invocation, Rust
formatting, production-config crate check, and `git diff --check`. Filters:
`contract_v5_context` (3 contracts after consumer closure),
`contract_v5_duplication` (2), `contract_v5_elimination` (2),
`contract_v5_five_tech` (1), `contract_v5_out_of_order` (1),
`contract_v5_ignition` (2), and `contract_v5_phantom` (2). The physical/epistemic
contracts cover No Variant plus reordered sixth-suit domains/ranks and negative
evidence; they are not new convention or optimal-move authorities. The named
planner-dependent Out-of-Order replay and full local suites were not run.
The final combined `check-scoped.sh hanabi-search --test-only contract_v5_ --lib`
run passed all 13 contracts and formatting; the equivalent production-config
check passed at the final Phantom boundary. One independent read-only review
of `bd96055..6ce94f9` found no actionable material regression or critical missing
regression preventing integration. Integration is tracked in [PR #49](https://github.com/BarsukovSergey/hanab-engine/pull/49).
Its first CI run found `clippy::nonminimal_bool` in the Out-of-Order empty-stack
guard; the follow-up uses the equivalent `is_none_or` form. Live PR checks
record the final integration result; focused local success alone is not a
full-validation claim. Detailed local
commands/logs and checkpoint notes are in ignored `target/workflow/m4-v5-*`.

Legacy reducer stack/count arrays, V5 `Rank::ALL`/`Suit::ALL` loops, remaining
legacy identity claim masks, and `standard_h_group_clue` remain outside the
migrated boundaries. The reducer arrays are retained; migrated families consume
dynamic snapshots or reconstruct dated public stacks. Special Finesse discard
evidence remains a V5 boundary.
The global No Variant policy gate stays until the remaining M4 slices are ready.
Pink Promise, Black clue-choice rules, and other variant convention packs remain
M4-V6 work, along with production enablement and resolution of the retained
legacy action/clue-label boundary. No Brown/Omni/Null/Pink-specific convention
meanings are introduced by V4. M4-V6 still owns variant-specific convention packs
and production enablement; the No Variant production gate is unchanged.
