# Codex Goal: M4-V5 R2A — real historical Rainbow clue / discard

Prepared: 2026-10-23. Base main SHA:
`c18202c07bad54958a3e05d9e6f628cb44632e20`.

Branch: `m4-v5-r2-historical-clues`.

Read `AGENTS.md`, `docs/codex-workflow.md`,
`docs/M4_V5_REMAINING_RESEARCH.md` (R2 entry), and
`docs/WORKING_STATE.md` (PR #50 R1 status), then only the **named**
source paths below. Do not repeat whole-repo archaeology or restart V1–R1.

This is a temporary branch-local Goal handoff. Before the final PR, move
any durable findings into `WORKING_STATE.md` and remove this packet from
the final diff; the Git commit history retains it.

## Product capability to ship

Replay a **real, legal, unmodified Rainbow6Suits rank-clue → discard
history**, using the same physical variant and observer-visible records
through `FullState`, `PlayerView::try_standard`,
`LogicalDeductions` and `replay_h_group` at **HGroup Level1**,
without the old `standard_h_group_clue` panic or any subsequent hidden
No-Variant-only alias.

Preserve No Variant reviewed history behavior and source-of-knowledge
chronology. This is a **history/inference infrastructure** test, not
authority to define Rainbow-specific H-Group conventions or to enable
production policy for variants.

An independent, future task (R2B) will migrate successful sixth-suit
`reduce_play` stack writes and additional physical clue vocabularies.
R2A should leave a precise blocker list for that next task.

## Why this is more than deleting an assertion

- `h_group/history_reducer.rs::reduce_clue` converts a legacy
  `ObservedEvent<Clue>` clue to a `GenericClue`, then calls
  `standard_h_group_clue(&view.variant, public_clue)`.
- `h_group.rs::standard_h_group_clue` is a **deliberate No Variant
  semantic gate** and has two explicit panic tests in
  `h_group/tests/clue_vocabulary.rs`. Do not globally remove it,
  convert all variant meanings to standard meanings, or delete its
  tests for a green result.
- `PlayerView<GenericClue>::try_standard` already checks that each
  physical clue label in its history fits the legacy `Clue` domain;
  it keeps the original `VariantSpec` and physical touching.
  This R2A test uses a legal **rank clue in that representable subset**.
  A sixth-color label is not representable by `Clue` and is **not**
  solved merely by R2A. Do not extend `Clue` or `Action` globally.
- The producer additionally contains physical operations through
  `self.stack_heights: [u8; 5]`, `Rank::ALL`,
  `identity.index()`, legacy fact masks and `Suit::ALL`.
  Fix *reached, semantically physical* operations using active
  `VariantSpec` IDs, `connection_heights(view, Some(turn))`,
  `connection_predecessors`, `connection_playable/trash`,
  `connection_allows` etc. Some fixed rank-2/rank-5 checks are
  **convention meanings**, not requests to mechanically erase them.

## Scope and ownership

Preferred sources:
- `crates/hanabi-search/src/h_group/history_reducer.rs`
- `crates/hanabi-search/src/h_group.rs` (only the adapter/dated
  physical seam, if needed)
- `crates/hanabi-search/src/h_group/turn_context.rs` (only direct
  replay context effect)
- `crates/hanabi-search/src/h_group/model.rs` **only if** the
  existing `HGroupClueInterpretation.stack_heights` compatibility
  shape cannot be bypassed by reconstructing dated public stacks
- `crates/hanabi-search/src/h_group/tests/`, plus a narrow
  direct consumer or a dedicated integration test module
- `docs/WORKING_STATE.md`

Other files may be touched **only** if a concrete first failing path
proves a direct producer/consumer dependency; list the dependency in
the checkpoint log. Do not proactively rewrite `public_layers`,
`inverse_planning`, `recognition/late_game.rs`, `extras.rs`,
`perspective.rs`, `prospective.rs`, or H-Group decision policy. In
particular, Charms and Unnecessary Moves belong to subsequent V5
tasks, not this trace.

## Budget and checkpoints

Maximum **five coherent implementation checkpoints** plus one final
independent review and bounded PR/CI cleanup. One focused batch should
normally fit 10–20 minutes of active agent work; reserve approximately
20% of Goal budget for final review and CI. Never treat the instruction
text as a hard token budget: obey actual Goal budget. At ~60–65% thread
context, checkpoint and refresh safely if supported or stop; do not
continue on unreliable compacted state.

Keep concise logs under ignored
`target/workflow/m4-v5-r2a.md`: checkpoint, code SHA,
validation commands/status, first offender, next step. Do not run
full check.sh, ignored planner/exhaustive tests or full self-play
for ordinary implementation checks.

### Checkpoint 1 — Pin the failing legal reproduction

Write a deterministic `contract_v5_r2_` test from the **same**
legally constructed Rainbow6Suits trace documented in
`WORKING_STATE.md`:

- Three players, physical variant from catalog.
- Cards from the active universe with their actual physical copy
  counts, not a fabricated 50-card deck or reordered illegal copies.
- At eight clue tokens: player 0 gives player 1 the **rank of
  player 1's first card**, then player 1 discards its first card;
  this is legal (a discard as the first action is not).
- Build an acting/observer-safe physical view and pass it through
  `try_standard`; preserve complete history, never clear the
  events or selectively remove a clue.
- Run `LogicalDeductions` and `replay_h_group` Level1. Assert
  no unexpected panic, observer safety, actual turn-by-turn public
  clue/discard evidence, active physical identities and event-time
  counts, not a guessed optimal move.
- Verify initial test **actually reproduces** the original assertion
  before fixing it; don't burn cycles attempting to make unrelated
  expert fixtures reach it.
- Verify independently that the same physical history can be
  reconstructed, rather than hand-editing an observed event to
  bypass core legality.

Use direct test assertion for the actual absence of panic and semantic
facts once fixed; do not leave a permanent ignored/should_panic test
as the only new coverage.

### Checkpoint 2 — Remove the blocker at the narrow clue-producer edge

Make the **historical clue producer** use already validated public
physical clue evidence for the R2A path without treating all variants
as standard H-Group convention packs.

The input history is already `ObservedEvent<Clue>` after checked
`try_standard`. Distinguish **physical conversion / touching**
from **rule meaning**. Prefer a small adapter boundary rather than
removing the invariant from
`h_group.rs::standard_h_group_clue` globally.

Keep existing explicit `standard_semantic_adapter_rejects_other_variants_even_with_standard_labels`
and `standard_semantic_adapter_rejects_nonstandard_labels` tests
valid at the deliberately standard-only adapter. Keep No Variant
`contract_replay_clue_vocabulary_retains_public_labels_and_input_parity`
and existing reviewed replay assertions.

Do not let unsupported physical color labels silently fall back to
legacy colors or drop events. Record the remaining
`PlayerView<GenericClue>` full-vocabulary limitation as R2B/V6,
not completed R2A.

### Checkpoint 3 — Physical clue-time state and direct consumption

For the **actual trace** and closely related Level1 paths, replace
any reached legacy facts/heights/rank operations where they have
physical meaning:

- active identity IDs: `identity_mask_in(&view.variant)`,
  `IdentitySet::from_bits`/`iter_ids`, active
  `connection_domain/bit/card/allows`; do not mix
  `Card::index()` bits with active `IdentityId`.
- before-clue **dated dynamic** stack heights, original clue-time
  histories and public evidence, not end-of-replay stacks/future card
  reveals.
- `Rank::ALL` adjacent physical connectors should be replaced with
  active predecessor/successor logic **where reached**; literal
  rank-2/rank-5 H-Group convention meanings remain literal until V6.
- The producer's `HGroupClueInterpretation` already stores
  `GenericClue` but retains five-stack
  `stack_heights`. Prefer recomputing the historical snapshot
  from its `turn` and `view.variant` in direct consumers over
  adding a field to every public clue interpretation. If a new
  dynamic field is unavoidable, migrate direct constructors/consumers
  coherently and preserve No Variant equality/parity.
- Preserve exact focus, Good Touch, Save/Play precedence,
  historical givers, and observer-relative knowledge. Never pass
  hidden simulator truth into the decision/replay implementation.

Don't convert unrelated V5 rule families merely because they share
`history_reducer.rs`. On a truly unrelated blocker, record and stop.

### Checkpoint 4 — Prove the integration boundary

Make the real `contract_v5_r2_` legal clue/discard test pass.
Add at most two further focused tests for **one** sixth-suit active
clue-domain/observer negative case and **one** unchanged No
Variant replay parity, if existing reviewed cases don't suffice.

Confirm that the history is not blanked and the physical variant is
still Rainbow6Suits throughout. Validate the actual reducer journal
and the event-time discard behavior as meaningful outputs; don't
stop at a catch_unwind assertion that merely suppresses a panic.

If it fails in `reduce_play`, `public_layers`, `inverse_planning`,
or an unplanned semantic interpretation family, capture the precise
first offender, note whether it is required for Level1 replay, and
either make a small direct repair or stop with a coherent PARTIAL PR.
Do **not** bypass a real rule by rewriting test history.

### Checkpoint 5 — Independent bounded next-boundary probe

Only after checkpoint 4 is coherent, do one **read-only, bounded**
probe for the next actual successful sixth-suit play or nonstandard
physical clue history blocker. List precise symbol and proposed
migration boundary for future R2B. Do not start an unsized
historical-play storage rewrite or global `PlayerView<GenericClue>`
conversion as a final-hour stretch.

Update `WORKING_STATE.md` with real closure, tests, and separately
observed limitations. The final No Variant production gate remains.

## Checks and merge

Per coherent batch:
`bash scripts/check-scoped.sh hanabi-search contract_v5_r2_ --lib`
(or an exact new test filter), and `git diff --check`. Run
`--check-only` when a batch has no matching focused test. Check
correct feature-gated production compilation. Up to two focused
new tests per checkpoint; avoid synthetic rule-authority assertions.

One final independent **read-only** review (if subagent available),
on diff against `c18202c`. Examine hidden-truth leakage, dated vs
current public knowledge, active identity bits, silent physical
label loss, No Variant convention meaning/precedence, rank-2/5,
and tests that suppress rather than exercise the replay.

If coherent product capability has landed on branch and focused
tests + scoped validation pass, push once final, **open one PR**
targeting main, inspect required PR CI and jobs, and make at most
two targeted CI-fix pushes. On green required PR CI and no
unresolved material reviewer findings, **autonomously squash-merge
PR into main** and inspect main CI; do not wait for human approval
merely because V5 remains partial. Respect branch protection;
no force merges or deploys.

If tests remain red, scoped production compilation fails, or
a substantive knowledge/semantics error remains unresolved,
do not merge: preserve coherent commits and a clear draft PR
with the exact failing symbol and next task. Never claim CI green
without observing completed success. No blanket variant H-Group
policy enablement.

**Final handoff**: PR URL, squashed SHA if merged,
actual PR/main CI, passed tests, completed vs PARTIAL
capabilities, source functions changed, remaining R2B blockers,
and whether the exact previously failing Rainbow
clue→discard history **passes** without modified history.
