# M4-V5 R2B: historical successful physical Play vertical

Prepared 2026-10-23.
Base: `015731bb7872980e93b670a2ace910816727f251` (merged PR #51).
Branch: `m4-v5-r2b-historical-plays`.

Read `AGENTS.md`, `docs/codex-workflow.md`, the R2 entry in
`docs/M4_V5_REMAINING_RESEARCH.md`, and the end of
`docs/WORKING_STATE.md` (PRs #49–51) **once**. This file already
contains the source map and integration scenario. Do not redo repository-wide
archaeology or spend budget reviewing completed R1/R2A commits.

This is a temporary branch-local run packet. Remove it from the final
PR diff after transferring the concrete result and next blockers to
`docs/WORKING_STATE.md`; the instructions remain in Git history.

## Production capability to close

A *real, legally generated* three-player Rainbow6Suits game can make a
**successful Play of the sixth suit's first rank**, then give an ordinary
representable **rank clue** and make a legal **discard**, with complete
public history, through Level1 `replay_h_group` for all three observers.

The sixth stack progresses at the correct event, not at an arbitrary current
snapshot; no panics or invalid physical domains; the pre-Play/clue-time
inferences do not learn identities from future public reveals.

At present, `ReplayReducer::reduce_play` writes a successful played
card to `self.stack_heights[identity.suit.index()]` with
`self.stack_heights: [u8; 5]`. R2A fixed the real historical clue/discard
producer but not this historical Play path.

**Completion of this contract is not support for every variant label,
advanced Max semantics or production H-Group policy.** That is separate
R2C/V5/V6 work. An arbitrary six-suit color clue is not representable
in `PlayerView<Clue>`/`try_standard`; do not cast/alias it to one of five
colors or drop it from history to appear supported.

## Pre-reviewed dependency map — investigate only the direct seam

- `crates/hanabi-search/src/h_group/history_reducer.rs`:
  `ReplayReducer::stack_heights: [u8; 5]`; initializer in
  `replay_h_group_inner_uncached`; successful write in
  `reduce_play`; played/discarded/clued checks still call
  `is_playable_at(self.stack_heights, ...)`; when a clue is recorded,
  `HGroupClueInterpretation.stack_heights` stores this array.
  `reduce_clue` already uses `basic_heights =
  connection_heights(view, Some(entry.turn))` for important physical
  decisions and must keep the R2A fix.
- `h_group/turn_context.rs`:
  `HGroupTurnSnapshot` and `HGroupTurnView` retain legacy arrays
  **and** independent dated `dynamic_stack_heights: Vec<usize>`.
  Many V3–V5 families already read the dynamic fields. Do not
  introduce an inconsistent second source of physical truth.
- `h_group/identity.rs`: authoritative adapters already exist:
  `connection_heights(view, Some(turn))`,
  `connection_advance`, `connection_playable`,
  `connection_trash`, `connection_predecessors`,
  `connection_allows`, `connection_domain`.
  Active `IdentityId` order may not match `Card::index()`;
  rank progression may not be a numeric +1.
- `h_group.rs`: `record_declined_direct_plays` reads old
  `clue.stack_heights` and `is_playable_at` on Card positions.
  Its direct historical decision and chronology must remain sound;
  recorded `HGroupClueInterpretation` currently contains
  `stack_heights: [u8; 5]`.
- `h_group/public_layers.rs::demonstrated_hidden_layer`:
  a Level-Max/SpecialFinesses play path with an unrelated
  `standard_h_group_clue` and historical fixed stacks. A Level1
  run need not invoke it, but document this **separate advanced
  next blocker** if encountered. A physical rank clue must not
  become a colour-layer demonstration.
- `recognition/late_game.rs::apply_charm_effects`,
  `recognition/extras.rs`, `recognition/trash.rs`,
  `recognition.rs::unassigned_finesse_ranks`,
  `action_schedule.rs`, `projection_requirements.rs` and
  `strategic_value.rs` retain some fixed-stack/legacy-fact
  consumers. Inventory *only* dependencies actually reached by
  this vertical. Don't perform an unsized global rewrite because
  these names appear in a search.
- `crates/hanabi-search/src/h_group/tests/clue_vocabulary.rs`:
  `contract_v5_r2_legal_rainbow_clue_discard` is passing R2A
  end-to-end evidence. Reuse its physically valid catalog deck,
  observer views and chronology checks; do not erase old assertions.
- `docs/self-play.md` and `docs/testing.md`: generated legal
  histories prove physical/replay invariants, NOT independent
  correctness of a non-No-Variant H-Group strategy.
  Do not invent/extrapolate variant convention meaning.

## Autonomous execution budget

Up to **six** coherent implementation checkpoints (usually 10–20 minutes
of focused agent work per batch), one final independent read-only
review, then **one PR/CI/squash merge**. This queue is a maximum,
not a target to fill regardless of result.

Reserve about 20% of the **actual Goal budget** for review,
integration and at most two focused CI-fix pushes. Avoid repeated
repository reading and repeated compilation of the same unchanged
slice. Limit to two new focused `contract_v5_r2b_` tests per
checkpoint, normally one direct legal integration test and a few
narrow negative/parity contracts total.

After a successful compiler/focused test pass and diff check,
commit/push a coherent checkpoint. Keep a short ignored log in
`target/workflow/m4-v5-r2b.md` (SHA, changed production
capability, exact checks/results, real next offender). On a failing
test, one targeted repair loop; on a semantic ambiguity, do not
invent a convention ruling. Stop at the last compiling coherent
checkpoint or skip only genuinely independent work.

No full suite, 200-game self-play, exhaustive 1–25+Max or broad
planner parity runs per implementation batch. PR CI is the
routine broad gate. Keep `docs/WORKING_STATE.md` factual.

## Checkpoint 1 — Make the Play path fail honestly, then fix it

Create a deterministic `contract_v5_r2b_` integration test with
**real FullState physical actions** and full history.

- Create a legal, complete Rainbow6Suits deck using every
  `VariantSpec` identity and its exact copy count. Reorder the
  deck (preserving the identity multiset) so an initial player
  genuinely holds the sixth suit's *first physically playable*
  rank. `FullState::new_variant(3, ..., deck)` validates it.
  Do not assume that the final suit or card ID is at a fixed
  numeric index without checking actual `VariantSpec` order.
- On the first turn, let that player **successfully play** the
  six-suit rank-one card. This is a legal physical action
  whether or not the engine would have recommended it.
  Check the simulator's stack for that physical suit.
- Next actor gives a **legal representable rank clue** (reduces
  8 clue tokens to 7), next actor performs a legal discard.
  Count every publicly observed Play, Draw, Clue, Discard and
  Draw event in the original sequence. Never clear/filter
  history, inject future identities, or handedit `PlayerView`.
- At each interesting prefix and for each of the three
  observers, take `physical_view_for`, perform the *checked*
  `try_standard` conversion, build `LogicalDeductions`, run
  `replay_h_group` at `HGroupProfile::Level(Level1)`.
  Assert public stack before/after heights including the
  sixth suit, variant preservation, journal/hand consistency,
  observer's own hidden identities, and causal/public turn
  ordering.
- The first test run should exhibit the known failing
  five-stack write or a more specific first blocker.
  Verify that failure before changing source. Do not
  convert the test to a permanent should_panic, ignored
  case or catch-unwind-only gate.

### Checkpoint 2 — One authoritative physical stack timeline

Repair the producer **and its directly reached consumers**
so the sixth suit's successful Play is recorded and read at
the correct event turn.

Options may include migrating the reducer's hot storage and
direct compatibility snapshots to dynamic
`Vec<usize>` or consulting the already authoritative dated
`connection_heights(view, Some(turn))`. Choose the *smallest
coherent* implementation that does not add two contradictory
physical states. If removal of the old fixed array causes
a broad compile fallout into unrelated advanced rule
families, preserve their explicit No Variant-only legacy
boundary *without* treating it as up-to-date variant
state. An unsupported advanced path is a documented blocker,
never a silent successful fallback.

**Forbidden shortcuts**: `get_mut`/bounds guard that simply
drops sixth-suit progress; sorting/relabeling suits as the
first five; `identity.suit.index() % 5`; suppressing the
successful Play or rewriting it into a discard; current
stack heights used to justify an earlier action.

An already played rank-one should be recognized as spent
at later turns; the sixth suit's successor should be
physically playable there; copies/public status must be
updated only when an event actually reveals them.

### Checkpoint 3 — Repair direct historical meaning consumers

Follow concrete next failures within this timeline:

- `reduce_play` and `reduce_discard` known-playable,
  declined-play and trash checks should use
  `connection_playable/trash` with *dated* heights;
- `reduce_clue` already has R2A physical fact domains and
  Level1 resolution. The previous Play must change physical
  Good Touch/Save/Play domains at *clue time* without
  retroactively changing facts from before the Play;
- If the retained `clue.stack_heights` is consumed in a
  direct low-level proof, use reconstructed historical
  heights from `clue.turn` and the active variant, or
  migrate the smallest producer+consumer representation.
  Do not casually change public equality of
  `HGroupClueInterpretation` or No Variant lifecycle.
- If `public_layers`/Charms/SpecialFinesses is encountered
  only in Max, do not rewrite that entire advanced family
  to claim Level1 Play coverage; identify its first direct
  migration seam for R2C.

Existing active-id counting and card-instance storage from
R1 must continue to work, including identity IDs >=25 and
card IDs >=50. `CardId`, `Card` and `IdentityId`
represent different domains; never mix their indices.

### Checkpoint 4 — Integrated physical evidence, not green by accident

Pass the full genuine Play → Clue → Discard contract
on all three observers. Include a lower-rank/second-play
variation only if it confirms physical successor ordering
without inventing conventional significance. Test a
future-reveal negative case: a later public Play or
discard must not change an earlier clue's interpretation.

Retain the R2A `contract_v5_r2_legal_rainbow_clue_discard`,
the six `contract_v5_replay_` contracts,
`clue_vocabulary` adapter rejection tests, and exact
No Variant parity contracts. New tests should assert
state/journal/domains, not a specific unreviewed recommended
move. No full replay self-play gate.

### Checkpoint 5 — No Variant and other physical variant parity

Run a corresponding legal standard No Variant Play → Clue
→ Discard prefix; assert no drift in ordinary timeline,
Save-vs-Play and `HGroupClueInterpretation` semantics.
If practical, run one reordered/sparse variant
`VariantSpec` case that distinguishes physical rank
order from the historical fixed-five assumptions.
Limit tests and prevent second-order broad fixtures.

### Checkpoint 6 — Next highest-risk production boundary map

Only after the Level1 vertical compiles and is committed,
probe **one** remaining direct advanced Level-Max
successful-Play trace (prefer a representable historical
rank clue followed by Play). If a small direct
`public_layers`/rank-clue mismatch can be repaired without
inventing Rainbow convention meaning, include it with
No Variant test parity. Otherwise record exact first
`public_layers`, Charms, Extras or full-vocabulary
blocker with minimal repro and hand it to R2C.

**Do not** start an unbounded full
`PlayerView<GenericClue>` rewrite tonight. Full physical
clue vocabulary, including unrepresentable sixth-color
labels, is *still* a separate task even if the present
representable-label path passes. Do not silently accept
a sixth-color clue or drop the action.

## Validation, independent review, PR and autonomous merge

For each meaningful compile-safe checkpoint run at most
one appropriate
`bash scripts/check-scoped.sh hanabi-search contract_v5_r2b_ --lib`
(or exact test-name filter), then `git diff --check`.
If no tests match an early code-only phase, use
`bash scripts/check-scoped.sh hanabi-search --check-only`.
At the final source boundary run a bounded selection of
R2A/R2B, `clue_vocabulary` and relevant No Variant
`contract_` tests. CI handles broad checks. A failing
preexisting long replay parity test must be recorded as
such, not rewritten.

Run **one independent read-only review** of the full
diff against `015731b` (or self-review explicitly
labeled if no independent agent). Review dated versus
current state, sixth-suit index, active vs standard
identity masks, public observer information, correct
card-instance ownership, failed-play bookkeeping,
No Variant parity and rule-family precedence. Fix
serious in-scope findings once and recheck.

After stable, focused green changes: push and open
**one PR** `m4-v5-r2b-historical-plays` → `main`;
do not open it mid-batch. Remove this temporary
`docs/M4_V5_R2B_HISTORICAL_PLAYS_GOAL.md` file from
final diff, update `docs/WORKING_STATE.md` with the
verified implementation and remaining barriers,
and preserve the progress log locally.

Check actual PR CI. At most **two targeted CI-fix
pushes**; never weaken tests or bypass semantic
validation. If required CI is green, changes are
coherent and no serious reviewer finding remains,
**autonomously squash-merge** into `main` and
observe post-merge CI when possible. Do not wait
for human approval merely because V5 is partial.
Respect branch protection, never force merge,
never deploy/enable variant H-Group production gate.

If a serious unresolved semantic question,
broken compilation or red CI remains, leave
coherent checkpoint commits and an accurately
marked draft PR with exact blocker; don't merge.

End with merged PR URL/SHA (if any), actual
PR/main CI status, exact new tests and outcomes,
documented level of physical integration
(R2B Level1 vs experimental Max), first remaining
full-clue-label / advanced consumer blockers
and the next R2C task.
