# Goal: existing H-Group bot plays its first White (6 Suits) game

Base main `90b5b1c3bdbe482cf4acb50db31d98e5a9c4b0e4` (PR #52).
Branch `product-white-hgroup-first-game`. Prepared 2026-10-23.

**The only product goal:** connect the existing H-Group decision system and
existing `plan_move` to one real non-No-Variant game. Do NOT implement a
new clue-giving policy, continue the unstarted Rainbow-cooperative task,
or perform another general M4-V5 migration.

## Why White, not Rainbow

In the checked-in catalog `White6Suits` has the same five physical
color clues and five rank clues as No Variant. Colors do NOT touch
white cards; each rank clue touches matching ranks including white.
All physical clue labels are therefore losslessly representable as
legacy `Clue`, and their literal `Clue::matches` semantics match
this specific physical variant. This is NOT true for Rainbow color
touching or variants with nonstandard clue labels.

A narrow **offline-only, checked boundary** is available:

```
FullState::new_variant(3, VariantSpec::from_catalog(White6Suits), deck)
  -> physical_view_for(current_player)  // player-safe
  -> try_standard()                      // only checked labels; variant preserved
  -> InformationSet::new(&view)
  -> plan_move(&information, SupportedConvention::HGroup(profile), config)
  -> PhysicalAction::from(result.best_action)
  -> check physical_view.legal_physical_actions()
  -> FullState::apply_physical()
```

This MUST use the real H-Group inference, candidate generation and planner.
Never call `ConventionAgnosticPolicy`, add a replacement strategy, coerce a
physical clue label, use hidden deck/order to choose an action, or fabricate
a safe-play fallback. Keep global No Variant production/CLI/live gates intact.

Source starting points:
- `crates/hanabi-search/src/information_set.rs::InformationSet`
  already supports physical identity universes;
- `crates/hanabi-core/src/view.rs::PlayerView<GenericClue>::try_standard`,
  `PlayerView::legal_actions`, `physical_view_for`;
- `crates/hanabi-search/src/planner.rs::plan_move` /
  `PlannerConfig` / `score_ceiling` (legacy `[[u8;5];5]`);
- `crates/hanabi-search/src/h_group/history_reducer.rs::reduce_play`
  (retains five-stack historical write), plus only direct failing consumers;
- `crates/hanabi-search/tests/product_pilot.rs`: reusable
  versioned SplitMix64 deterministic physical full-deck generation,
  report/reconstruction/redaction examples. Do not change its Rainbow
  policy or measurements.

Official source:
https://hanabi.github.io/learning-path recommends level **4** for White;
https://hanabi.github.io/variant-specific/white/ documents the
white-specific Loaded Play Clue Assumption for rank-3/4 Saves.
Treat Level1 as a plumbing/first-action smoke test; use Level4
when evaluating whether the actual bot can play a coherent White game.
**A completed game is NOT proof of official White H-Group semantics**
until variant-specific meanings have been checked against those sources.

## Read-only audit clarifications (2026-10-23)

The read-only audit of main `90b5b1c3` confirms the proposed
`physical_view_for -> try_standard -> InformationSet::new ->
SupportedConvention::HGroup(profile) -> plan_move` uses the existing
H-Group inference, action generation, and planner. This is a technically
plausible *offline* White-only boundary, not a proven running game.
Implement checkpoint 1 first and report the actual first failing call.
No replacement policy, scripted action, or silent fallback.

Specific directly relevant known hazards, **not blanket refactor requests**:

- `crates/hanabi-search/src/h_group/history_reducer.rs` has
  `stack_heights: [u8; 5]` and `reduce_play` writes
  `self.stack_heights[identity.suit.index()]` after successful plays.
  A White sixth-suit successful play can index out of bounds. Repair
  event-time historical bookkeeping and direct reached consumers when
  exercised; do not silently skip the sixth-suit update.
- `crates/hanabi-search/src/h_group.rs::standard_h_group_clue`
  explicitly asserts No Variant. `inverse_planning.rs`,
  `public_layers.rs`, and `recognition/late_game.rs` contain callers.
  Confirm which paths the selected Level1/Level4 replay actually reaches.
  For White-compatible labels, any narrower adapter must prove literal
  touching parity and semantic validity at its caller, while preserving
  unsupported-variant rejection elsewhere. Do **not** simply remove the
  global assertion for every variant.
- `crates/hanabi-search/src/planner.rs::solve_partitioned` calls
  `FullState::view_for` on simulated worlds. That method intentionally
  returns None for all non-No-Variant states. If exact search is reached,
  use a checked White-compatible observer projection rather than
  turning White into No Variant, accepting hidden truth, or silently
  misreporting exact-search success.
- `planner.rs::score_ceiling` uses five suit/rank arrays; additional
  perfect-score/terminal proofs use fixed 25/24 thresholds although
  White (6 Suits) has a 30-point ceiling. For this White offline pilot,
  explicitly set `PlanningObjective::ExpectedScore`, and fix directly
  reached wrong-variant score paths or fail clearly; do not claim that
  reducing search limits alone makes them correct. Keep No Variant
  perfect-score parity unchanged.

Level1 establishes first-action plumbing, not White H-Group compatibility.
Evaluate Level4 only against supported, sourced semantics. The official
White 3/4 Save Loaded Play Clue Assumption must be independently checked
and its implementation status reported, not silently inferred from the
physical clue match. Log whether each evaluated move used symbolic or
exact planning, any budget fallback, and which documented semantic
assumptions the move required. A timer stop with no completed game
remains INCOMPLETE.

## Do the shortest vertical, not comprehensive framework work

### Checkpoint 1 — one real H-Group choice

Create one small new *opt-in* integration test/runner, e.g.
`crates/hanabi-search/tests/white_hgroup_pilot.rs`. Construct a
complete official `White6Suits` deck using every active identity's
copy count; deterministic shuffled seed(s). No fabricated 50-card
deck or unsupported conversion of 6 suits to 5. Pass only acting
observer's redacted view to the engine.

Start with one initial decision at H-Group Level1. Verify:
actual `plan_move` returns (or records exact first failure),
selected action is in `physical_view.legal_physical_actions()`,
and real FullState accepts it. Assert checked `try_standard`
retains White `VariantSpec` and clue-touching parity for
all legal physical labels, including negative white-color evidence.
Never weaken the existing No Variant semantic rejection tests.

If the first failure is the fixed-five `score_ceiling`, historical
Play array, planning prospective state or another directly reached
5-suit assumption, repair THAT specific producer and its direct
consumers via `VariantSpec` / active identities / dynamic stacks.
Keep No Variant behavior and parity. A `get_mut` guard silently
discarding sixth-suit state is forbidden.

### Checkpoint 2 — actual game loop, no alternative bot

Reuse the real H-Group `plan_move` decision for **every** turn
as above. Try official Level4 for complete-game evaluation;
the early Level1 smoke is not itself a White convention release.
Record any rule whose meaning remains White-specific and unimplemented.
Use one stable deck/shuffle seed first, then optionally two
more if one full game finishes and budget permits.

Persist an exact reproducible report and flushed action trace
under ignored `target/product-pilot/white-hgroup*`: SHA, deck,
profile, planning budgets, seed, turn, actor, physical clue/card ID,
official score, stack score, strikes, terminal reason, elapsed and
per-turn decision time, warnings/errors, and complete replay.
Independently reconstruct FullState from same deck/actions.
Own hand/draw identity must remain redacted on every decision.
No action substitution or secret-aware fallback, ever.

A test may bound runtime/turn count; an interrupted game is
**INCOMPLETE**, not a finished game or a score of zero.
For the pilot only, choose documented bounded
`PlannerConfig` (e.g. 5-s move time limit and reduced exact
world/node limits) instead of changing the production default.
Do not mask belief conflicts, ignored convention conclusions
or illegal actions as timeout fallbacks.

### Checkpoint 3 — real evidence and integration

Product acceptance for merging this capability:
- at least **one** complete White game using H-Group's actual
  action proposals and planner throughout (no scripted actions),
  with a **positive official final score** and no legality,
  replay, hidden-information or planner errors;
- at least one genuine H-Group-generated clue and one
  recipient play justified by supported interpretation /
  literal knowledge, without fabricated White-specific rules;
- report explicitly whether the official White rank-3/4
  Loaded Play Clue Assumption is implemented or still absent.
  Do NOT claim comprehensive variant H-Group support if absent;
- No Variant representative reviewed tests, focused new tests,
  physical legality and deterministic reconstruction stay green.
- main production No Variant gate unchanged.

If a broad `plan_move` type refactor, many advanced rule families
or a new unreviewed convention meaning become necessary, **stop**,
keep a precise first blocker and a compiling coherent checkpoint.
Never turn a fixed-pipeline failure into a new simplistic bot.
Do not remove No Variant gates/tests to get green.

Run narrow targeted tests, then
`bash scripts/check-scoped.sh hanabi-search --check-only`
and `git diff --check`. Avoid full test matrix and
200-game No Variant benchmark. One independent read-only review
of the final diff (if supported), including clue touching parity,
real versus simulated identity knowledge, sixth stack, active ID
order, No Variant parity and accurate product claims.
Update `docs/product-pilot.md` with exact measured evidence.
Remove this temporary Goal document from the final PR diff.

If acceptance is met, open **one PR**, observe required CI,
allow at most two targeted CI-fix pushes; on green CI and clean
review, autonomously squash-merge and verify main CI.
If acceptance is NOT met, leave a **draft PR**, coherent commits
and the exact failing action/symbol, **do not auto-merge a
nonplaying White bot**. No production or live policy enablement.

Final report: complete/incomplete per game, actions/clues/plays,
score, errors and latency; source changed; PR/SHA/CI (if any);
and ONE minimal blocker/next product capability, not a broad
list of M4 migrations.
