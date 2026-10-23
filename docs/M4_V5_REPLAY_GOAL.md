# Codex Goal: M4-V5 replay foundations (R1)

Start SHA: `17c059f7183c82446eb3a83a5d90e2c11e124493`  
Branch: `m4-v5-replay-foundations`  
Research map: [M4_V5_REMAINING_RESEARCH.md](M4_V5_REMAINING_RESEARCH.md)  
Prepared: 2026-10-23

Read this packet **once**; source links and the dependency analysis are already
in the research map. Do not redo repository inventory, general H-Group
research or PR #49 archaeology. Follow `AGENTS.md` and
`docs/codex-workflow.md` for worktree hygiene and narrow validation.

## Deliverable

Close the **shared historical physical bookkeeping boundary** that currently
prevents real sixth-suit discard histories from passing into H-Group replay,
and make H-Group phase arithmetic use the active physical variant. Deliver
a coherent, tested implementation; a PR with actual green CI; and
**autonomous squash merge into main** if the merge safety criteria below hold.

The deliverable is **not** complete V5, arbitrary variant H-Group clue
semantics, or a newly enabled live/CLI policy. Code that silently skips
nonstandard cards or creates a standard-only alias does not count.

## Permitted source scope

- `crates/hanabi-search/src/h_group/history_reducer.rs`
- `crates/hanabi-search/src/h_group/facts.rs`
- `crates/hanabi-search/src/h_group/interpretation/knowledge.rs`
- `crates/hanabi-search/src/h_group.rs`
- `crates/hanabi-search/src/h_group/rule_engine.rs`
- Focused `crates/hanabi-search/src/h_group/tests/` coverage, especially
  `v4_advanced_connections.rs`; direct in-module tests if necessary
- `docs/WORKING_STATE.md` and the durable research map

No whole-file rewrites, no global changes to `Clue`/`Action`/card model,
no changes to production/CLI/live variant gate, no M4-V6 convention packs.
Do not change previously reviewed fixtures or weaken existing assertions.
Don't edit `perspective`, `inverse_planning`, `public_layers`,
`extras`, `late_game`, `prospective`, or `decision` tonight merely
because the research map notes their remaining blockers. These are R2/R3.

## Work protocol and budget

- Preflight: check branch, ancestry and unexpected dirty files. Existing
  research packet/docs commits on this branch are intended changes.
- Work in **up to five coherent compile-safe implementation checkpoints**,
  in the order below. One implementation checkpoint normally fits in
  10–20 minutes of active agent work. Use focused reads and one targeted
  scope expansion at most per concrete failure.
- Preserve roughly the last 20% of the available Goal budget for final
  review, PR, CI, and one or two mechanical CI fixes. Stop new feature
  work on entering that reserve. Honor the actual configured Goal budget;
  text instructions cannot impose a hard token limit.
- At most two *new* `contract_v5_replay_` tests per checkpoint. Use
  existing builders and assert semantic/physical state, not unreviewed
  optimal moves. No invented game histories as convention authorities.
- A checkpoint is finished only with relevant focused tests, formatting,
  production-config crate check if relevant, reviewed diff, clean
  `git diff --check`, commit and push. If repeated attempts fail,
  preserve the last coherent commit and mark BLOCKED/PARTIAL; do not
  manufacture a green result.
- Record concise progress in ignored
  `target/workflow/m4-v5-replay-goal.md`: stage, SHA, checks, unresolved
  risk, next stage. Do not copy huge terminal logs into context.

## Implementation checkpoints

### 1. Card-instance facts beyond index 49

`history_reducer.rs::replay_h_group_inner_uncached` allocates only 50
`ClueFacts` entries. The actual physical deck size is available at
`view.variant.identities().deck_size()`. Allocate safely for that
physical deck, not `view.deck_size` (remaining undrawn cards).

`facts.rs::ConventionFacts` independently keeps **five arrays of length
50** and directly indexes them with `CardId::index()`. Replace that
card-instance storage with a coherent dynamic/lazily growing form,
preserving existing `Default` ergonomics and exact unknown/empty
fallback semantics for non-existent card IDs. Ensure every writer,
retraction/rebuild, and reader handles card 50+ without panic or
inventing facts. Keep identity-claim chronology, OneOf relations,
conflict handling and equality stable.

Focused contracts: card 50+ becomes known, excluded, retracted, or
demonstrated without panic; untouched/out-of-range identities remain
unknown. No Variant parity.

### 2. Event-time removed counts keyed by active identity IDs

In `history_reducer.rs`, replace `public_removed: [u8; 25]` with a
vector sized by `view.variant.identities().identities().len()`.
Map each discarded or unsuccessful-play `Card` to active
`IdentityId` via `view.variant.standard_identity_id(...)`, then
increment at that ID. Preserve exactly which events increment;
successful plays and hidden/unseen copies do not become removed.
Avoid counting the same reveal twice.

Update the immediate `basic_discarded` / Good Touch consumers to use
counts in active-ID order. Keep the snapshot at the event's historical
position: no future discard may affect an earlier clue. Do not
replace this with the current full `view.discard_pile`.

Contract: No Variant parity and a sixth-suit identity with active
index >= 25, ideally in a reordered/sparse universe. Assert
event-time copy counts / missing-predecessor behavior.

### 3. Base Save consumer of those counts

`interpretation/knowledge.rs::snapshot_save_identities` currently
accepts `[u8; 5]`/`[u8; 25]`, enumerates all 25 standard cards and
`unwrap()`s variant IDs. Its `identities` input from `reduce_clue`
is already an active-universe `IdentitySet`.

Replace the fixed count/height inputs with dynamic, variant-indexed
slices; pass the historical basic public stacks and removed counts
from `reduce_clue`. Use `IdentitySet::iter_ids` / active-bit
operations, including the Eight Clue Save branch. Preserve historical
giver-relative Visible Rule, 5 Save, 2 Save, Critical Save,
Eight Clue Save, Save-vs-Play precedence and existing No Variant
results. Reuse `ClueInterpretationPlan::direct_save`; don't invent
another Save implementation. Using `GenericClue` at this physical
boundary is encouraged if it does not broaden the change.

The enclosing `reduce_clue` still uses `standard_h_group_clue`;
**do not interpret this checkpoint as enabling arbitrary variant
historical clues**. If changing this adapter safely requires
generalized clue semantics, mark that as an R2 blocker, not a
reason to widen this Goal.

### 4. Active physical H-Group phase calculation

In `h_group.rs::h_group_phase`/`h_group_phase_at`, use actual
stack lengths, `&[usize]`, and `view.variant.max_score()` for
remaining physical plays. In `rule_engine.rs::apply_rule`, update
both phase gates (Priority, ChopMoves) to use **before/after dated
dynamic_stack_heights**, not retained legacy `[u8; 5]` nor
current global stacks.

Test:
- Exact No Variant phase and endgame-parity cases (25-point max);
- Custom two-suit and six-suit variants, including reordered or
  sparse ranks, without indexing a nonexistent third stack;
- Earlier/later turn phase should not leak current public stacks.

IMPORTANT: `HGroupPhase::LowScore` currently uses `score < 5`;
Level 19's different Low Score Phase is `score < 2 * suits`:
<https://hanabi.github.io/level-19/#the-low-score-phase-and-the-normal-score-phase>.
Preserve existing No Variant `HGroupPhase` classification in this
infrastructure patch; do **not** silently equate these semantics.
The separate Five Tech `2 * Suit::ALL.len()` gate can be changed to
active physical suit count only if it is a small clearly bounded
physical adjustment with No Variant parity. Otherwise record it for R2.

### 5. Real production-path integration evidence

Prefer upgrading the existing
`contract_v5_elimination_active_copy_count_resolution_and_constraints`
in `h_group/tests/v4_advanced_connections.rs`, which currently
**clears `seed.history`** before replay as a workaround for the
25-count panic. Exercise a genuine sixth-suit discard history through
`replay_h_group` and the direct Elimination consumer where the path
permits. Do not fake the history or weaken its existing semantic
assertions.

Add one narrow card-instance `CardId >= 50` history test and a
phase test that reaches the real inference/rule gate when possible.
Use legal physical event data and established invariants; artificial
history must never certify a new H-Group convention or best move.

If the path then hits **an independent, known legacy boundary** (e.g.
the retained five-stack `reduce_play` or No Variant-only
`standard_h_group_clue`), document the exact first offender and
mark that coverage PARTIAL. Do not add `get_mut`/conditional skips
to hide a failed sixth-suit play. Probe at most one such extra blocker
before ending this Goal. A bounded, sound discard-only replay is
valuable even though arbitrary variant play/clue replay remains R2.

## Checks and final review

For each coherent checkpoint, run one suitable
`bash scripts/check-scoped.sh hanabi-search contract_v5_replay_ --lib`
(or a narrower exact test filter; `--check-only` when no new tests
apply), plus `git diff --check`. Do not compile after every line
edit or run full `scripts/check.sh`, exhaustive 1–25+Max,
wall-clock planner replays, MSRV, Hawk, or strict workspace Clippy
locally without a specifically identified risk. PR CI owns those
broad routine gates.

After last implementation checkpoint, run a final focused contract
selection, then **one** read-only review of the full branch diff
(independent reviewer agent if available; otherwise call it
self-review). Check card-ID vs identity-ID confusion, sparse
variants, future-information leakage, copy accounting, No Variant
phase/Save parity, and changed match/precedence. Do at most one
bounded high-severity in-scope repair cycle. Update
`docs/WORKING_STATE.md` with actual evidence and R2/R3 blockers;
keep `docs/M4_V5_REMAINING_RESEARCH.md` as durable dependency map
and remove **this temporary Goal file** from the final PR diff once
the instructions are already loaded into Codex.

## Required end-to-end PR, CI and merge

Do not open PR during intermediate pushes. After a coherent
implementation, focused green checks and final review:

1. Push and open **one PR** from `m4-v5-replay-foundations` to
   `main`. Body must specify exactly which physical paths are
   proved and which still fail at retained legacy blockers.
2. Check actual PR CI and logs. At most **two targeted CI cleanup
   pushes** to fix proven, in-scope mechanical failures; do not
   weaken existing tests or broaden semantic scope. Don't wait
   forever on queued jobs or required permissions.
3. If all required PR checks green, no unresolved serious reviewer
   issue, code compiles, and the delivered partial capability is
   coherent, **squash-merge the PR autonomously**. Completion of
   the whole V5 is *not* a prerequisite. Respect branch protections;
   never force merge.
4. Verify the resulting `main` SHA and observe main CI when
   available. If main CI is pending, report pending, not green.
5. If the implementation is incoherent, CI red, or material semantics
   uncertain, leave a draft PR with exact blockers or a pushed
   checkpoint branch; **do not merge**. No deployments/releases.

Final report must include code commits and squash SHA (if merged),
PR URL, exact new tests and their results, actual PR/main CI,
review findings, completed and partial capabilities, and the next
concrete R2 step. Do not claim full variant H-Group support.
