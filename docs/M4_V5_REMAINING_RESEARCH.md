# M4-V5 remaining work: post-PR #49 dependency audit

Audit date: 2026-10-23. Base `main`: `17c059f7183c82446eb3a83a5d90e2c11e124493`
(PR #49). This is a **research/roadmap** document, not evidence that the
remaining paths are already variant-safe. The authoritative milestone acceptance
remains in [M4_VERTICAL_EXECUTION_PLAN.md](M4_VERTICAL_EXECUTION_PLAN.md);
completed and partial boundaries are recorded in [WORKING_STATE.md](WORKING_STATE.md).

## R1 execution update (2026-10-23)

Checkpoints 1-4 implemented active physical card-instance storage, event-dated
active-ID removed counts and Save inputs, and dynamic/max-score phases. The
boundary table below records the original audit, not current storage after R1.

Checkpoint 5 is PARTIAL: a legal simulator Rainbow clue followed by discard
reproduces `reduce_clue` -> `standard_h_group_clue`'s explicit No Variant assert
(`h_group.rs:1162`). Starting with discard instead is illegal at eight clues;
removing the required clue would fabricate history. The existing Elimination
empty-history workaround remains unchanged, and genuine card-50+ history replay
is not certified. Initial-state two-/six-stack production phase inference is
covered. See `WORKING_STATE.md` for exact evidence and scope.

R2 must begin with the historical clue adapter and its immediate semantic
consumers, then repeat the legal clue/discard integration trace. Retained
`reduce_play` five-stack storage and the separate Five Tech threshold follow;
no further legacy path was probed in R1. The original suggested discard-only
integration order below is superseded by this demonstrated prerequisite.

## Executive decision

**Next integration unit: variant-safe replay bookkeeping and phase decisions.**
Do not jump immediately to Charms/Extras or claim whole-game variant replay
from the 13 new PR #49 contracts. Those tests explicitly avoid a real
sixth-suit discarded-card history in the Elimination consumer test. First
remove the reusable storage/accounting blockers and prove a real bounded
historical path; then move to the remaining V5 recognizers and adapters.

Keep the No Variant production gate unchanged through all V5 work.
Variant-specific convention meanings and production enablement belong to V6.

## Evidence: actual code boundaries

| Priority | Source path and symbol | Existing assumption | Impact / required direction |
| --- | --- | --- | --- |
| P0 | `h_group/history_reducer.rs::replay_h_group_inner_uncached`, `reduce_play`, `reduce_discard` | `public_removed: [u8; 25]`; discard and failed-play paths use `identity.index()` | Sixth-suit identity #25+ can index out of range. Use a count vector indexed by active `IdentityId` resolved through `VariantSpec`, with original event-time accumulation and No Variant parity. |
| P0 | `h_group/interpretation/knowledge.rs::snapshot_save_identities` | Inputs `[u8; 5]`, `[u8; 25]`; rebuilds counts by enumerating standard 25 cards and calling `id_for(...).unwrap()` | Merely replacing the reducer array is insufficient. Consume active-ID counts and dynamic public stacks directly. Focus domains are already active at the caller. Preserve Eight Clue Save and 5/2/Critical Save precedence. |
| P0 | `h_group/history_reducer.rs::replay_h_group_inner_uncached` | `facts = vec![ClueFacts::default(); 50]` | Card IDs can exceed 49 in a nonstandard physical deck. Allocate for the actual variant's full physical deck (`variant.identities().deck_size()`), not currently remaining draw pile. Do not change event chronology. |
| P0 | `h_group/facts.rs::ConventionFacts` | Five card-indexed arrays all length 50, with direct `card.index()` reads/writes | A second independent >49 panic surface. Use a small bounded dynamic representation or growth-on-demand for card ID slots while preserving default unknown/empty behavior and claim cancellation/rebuild semantics. |
| P0 | `h_group.rs::h_group_phase`, `h_group_phase_at` and `h_group/rule_engine.rs::apply_rule` | Exactly five public stacks; `remaining_plays = 25 - score`; two post-event rule gates use legacy 5-stack snapshots | Use the variant's actual public stack lengths and `VariantSpec::max_score()`, which also handles variants with absent ranks. For historical rule gates use `context.before/after.dynamic_stack_heights`, never today's view for an older turn. |
| P1 | `h_group/history_reducer.rs::reduce_play` and other event/clue paths | Separate `stack_heights: [u8; 5]`, updated by `identity.suit.index()`; remaining callers use fixed `Rank::ALL` and direct arrays | A successful play in sixth suit still encounters legacy storage. **Do not** mask this with `get_mut`/silent skip or claim complete event replay. Its wider migration needs a separate sized semantic boundary. |
| P1 | `h_group/turn_context.rs` and `h_group/model.rs::HGroupClueInterpretation` | Legacy `[u8; 5]` retained alongside dynamic turn stacks and on recorded clue meanings | Read-only audit consumers before retiring compatibility shape; direct wholesale replacement could change No Variant inference. |
| P1 | `h_group/history_reducer.rs::reduce_clue` | Calls `standard_h_group_clue(&view.variant, ...)`, which **asserts No Variant** | A variant clue history cannot become supported just by fixing discarded counts and phase; historical clue semantics/labels require another vertical migration. |
| P1 | `h_group/public_layers.rs::demonstrated_hidden_layer`, `h_group/inverse_planning.rs::deduce` | Retain `standard_h_group_clue` and/or fixed historical ranks | These must eventually use generic physical clue evidence, dated stacks and correct player-visible information; avoid a made-up variant convention meaning. |
| P1 | `h_group/perspective.rs`, `h_group/inverse_planning.rs` | Arrays/vectors of length 50 and calculations like `50 - source.deck_size` | Future independent deck-size/ID migration when subjective projection reaches nonstandard variants. |
| P2 | `h_group/recognition/late_game.rs` | Charms and Unnecessary Moves still use old physical clues, ranks and masks | Remaining V5 recognizer work. Ignition/Phantom were migrated in #49; do not redo them. |
| P2 | `h_group/recognition/extras.rs`, `special_discards.rs`, `draw_distribution.rs`, `prospective.rs`, `decision.rs` | Remaining fixed stack/rank/count assumptions, including some 25-point reasoning | Inventory by production path, size separately; no broad mechanical search-and-replace. |
| P3 | `docs/M4_VERTICAL_EXECUTION_PLAN.md` V6 | Convention packs and production gate | Only after V5 semantics and acceptance evidence. |

### Important distinctions

- **Card identity versus card instance:** `Card::index()` represents a legacy
  five-suit 25-position label; active `IdentityId::index()` indexes the
  `VariantSpec` universe; `CardId::index()` is a particular physical card
  instance (potentially >= 50). Do not interchange these indexes.
- **Sparse/custom physical variants:** `VariantSpec::max_score()` is the
  authoritative physically reachable stack score. It is **not** always
  `suit_count * rank_count`; `IdentityUniverse::deck_size()` is the sum of
  physical copies, not remaining `view.deck_size`.
- **Timeline:** discard counts and knowledge are accumulated as of the event.
  A later discard or play may not retroactively prove an earlier clue.
  `connection_heights(view, Some(turn))` and the before/after dynamic
  snapshots already encode dated public stacks.
- **H-Group semantics:** Do not conflate the existing internal
  `HGroupPhase::LowScore` branch (`score < 5`) with Level 19's separately
  defined Low Score Phase (`score < 2 * number of suits`). Official source:
  <https://hanabi.github.io/level-19/#the-low-score-phase-and-the-normal-score-phase>.
  The existing Five Tech clue check uses `2 * Suit::ALL.len()` (10 for
  No Variant); changing it to physical suit count preserves that No Variant
  threshold. Do not silently change `HGroupPhase::LowScore`'s historic
  No Variant classification in a mechanical phase refactor; flag the semantic
  distinction for a reviewed follow-up.
- **No Variant authority:** reviewed replay assertions and precedence remain
  authoritative. Artificial variant snapshots can test physical counts,
  domains, legality, history/observer isolation and lack of panics, but cannot
  establish a new convention meaning or optimal move.

## Recommended dependency-ordered roadmap

### R1 — Historical physical accounting, public facts and phases (NEXT GOAL)

One coherent PR with multiple small checkpoints. Deliver **actual historical
variant discard/counter behavior** and correct public phase arithmetic. Make
the phase and count producers **and their direct consumers** variant-aware:

1. Resize the reducer's `ClueFacts` storage and `ConventionFacts` card-ID
   indexes for physically possible cards beyond index 49.
2. Convert `public_removed` into per-active-identity event-time counts and
   update `basic_discarded`, Good Touch and direct Save consumption.
3. Fix `snapshot_save_identities` to use dynamic physical inputs, including
   its Eight Clue Save identity filter, without changing ordinary meaning.
4. Refactor `h_group_phase`/`h_group_phase_at` and both direct
   `rule_engine` historical gates to use active stacks and physical maximum.
   Keep standard phase parity; distinguish the Level 19 Low Score threshold.
5. Add targeted `contract_v5_replay_` tests: no-variant parity,
   reordered active IDs/sixth-suit discard, card instance ID >= 50,
   custom two-stack/max-score phase, event-time visibility. Where feasible,
   replace the synthetic empty-history shortcut in the existing V5 Elimination
   test with a genuine history, but only if the full path supports it.
6. Probe exactly one next unconverted path (e.g. successful sixth-suit play)
   and record the next blocking symbol without hiding its failure or
   embarking on a new whole-reducer rewrite.

**Exit:** PR CI + review green, autonomous squash merge permitted for a
coherent bounded change; main CI checked. Report that complete variant
play/clue history is still outside R1 unless specifically proven. The
implementation packet is `docs/M4_V5_REPLAY_GOAL.md` on the R1 branch.

### R2 — Historical play state, H-Group clue-event semantics

A separate sized vertical slice after R1. Migrate retained five-stack
historical storage/recorded clue stack snapshots, direct `Rank::ALL`/legacy
facts in `reduce_clue`, and the No-Variant-only clue adapter. Preserve
reviewed event/replay precedence and staged knowledge proofs, cover
sixth-suit **successful plays followed by a physical clue** with no hidden
future evidence. This is likely the larger, higher-risk migration. First
map all direct consumers of `HGroupClueInterpretation.stack_heights` before
choosing the replacement representation.

### R3 — Remaining high-level interpretations and historical adapters

Finish Charms, Unnecessary Moves, Extras/Max and Special Finesse discard
evidence using explicit physical/delayed-time helpers; then
`public_layers`, `inverse_planning`, and any reached `perspective`,
`draw_distribution`, `prospective` and `decision` paths. Prefer
two coherent families per PR when possible, not micro-PRs or an unlimited
whole-module rewrite. Every family must include producer, rule registry,
claim/connection consumer, No Variant regressions and representative
physical variants.

### R4 — M4-V5 acceptance and then V6

Inventory active legacy adapters after R1–R3; distinguish intentionally
No-Variant-only compatibility code from unexpected live dependencies.
Run the repository full milestone check and the manually dispatched
1–25+Max exhaustive matrix once at acceptance time (not per checkpoint).
Verify representative 3–6 player cases and supported variant catalog.
Only then begin V6's reviewed convention packs, resolved ruleset
capabilities and conditional production/CLI/live enablement.

## Why R1 is deliberately limited

The PR #49 Elimination test itself says it clears `seed.history` because
nonstandard discards hit `public_removed`. R1 can remove that real test
evasion and deliver genuinely better historical coverage. But a 6-suit play
still updates the reducer's separate five-stack array, and a historical
clue still asserts No Variant. Quietly suppressing either is **less safe**
than a consciously partial migration. A successful R1 gives R2 a stable
basis instead of requiring a giant overnight all-at-once rewrite.

The ordinary PR CI runs fast `contract_` tests, build/lint/docs/MSRV and
protocol checks; it does **not** run the full planner/replay suite.
Use `scripts/check-scoped.sh hanabi-search <focused-filter> --lib` at each
coherent implementation boundary, followed by a final bounded review,
one PR/CI pass and at most two targeted CI cleanup pushes.

## Re-evaluate after R1

Update `WORKING_STATE.md` using real evidence. If the next actual blocker
proves smaller than anticipated, R2 may be split into independent shippable
capabilities; if it requires changing H-Group clue meaning rather than
physical representation, stop for a reviewed semantic decision instead of
letting an overnight agent infer conventions.
