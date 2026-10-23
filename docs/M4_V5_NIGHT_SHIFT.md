# M4-V5 overnight execution goal

Base: `bd960555c5b0df35865d8c42617b016a1094fe49`  
Branch: `m4-v5-high-level-refinements`  
Prepared: 2026-10-23

This temporary, branch-local packet should be removed from the eventual V5
PR once the results are recorded in `docs/WORKING_STATE.md`. The authoritative
V5 boundary is [M4_VERTICAL_EXECUTION_PLAN.md](M4_VERTICAL_EXECUTION_PLAN.md).
Also follow `AGENTS.md` and [codex-workflow.md](codex-workflow.md).

## Actual goal: close production capabilities, not check off functions

Make substantial, safe progress toward **variant-aware H-Group high-level
interpretation for 3–6 players**. An implemented recognizer does not count as
finished if the historical/prospective producer, registered execution, its
claim/obligation consumer, or physical identity operations still require
five-suit assumptions. Conversely, do not refactor an already generic path
merely to claim work.

The main deliverable is the **high-level recognition/refinement vertical**:
Context, Duplication, Elimination, Five Tech, Out-of-Order Play, and, if ready,
Ignition / Phantom / Charms / Unnecessary Moves. Existing historical and
prospective paths both enter the shared `rule_engine.rs` registry. Migrate
the production implementation and minimal direct caller/consumer per family.

The optional stretch deliverable is **legacy evidence/adapter closure** in
`public_layers.rs`, `inverse_planning.rs`, and the narrow unconverted Extras
and special Finesse discard paths. Do this only when the main recognizer
vertical has coherent coverage and the adapter can be migrated without
inventing H-Group semantics or changing unrelated architecture.

This is one V5 implementation branch, **not** one unbounded chat and **not**
authorization to declare M4-V5 complete. Do not open or merge a PR, change
production/live/CLI variant gates, or implement M4-V6 variant convention packs.

## Autonomous work budget

The queue below is intentionally larger than one overnight run. Advance as
far as evidence permits; do **not** try to spend all available time.

- Maximum **eight coherent implementation checkpoints** plus **one independent
  final read-only review**. A checkpoint should finish a recognizer family
  including its direct production path, rather than just an isolated helper.
- Default approximately 10–20 active minutes **per continuation/batch**, not
  a 20-minute limit on the entire Goal. Do one feature at a time. At around
  60–65% context use, compact/refresh if the active Codex Goal supports a safe
  transition, using the short on-disk checkpoint as context; never pay for
  repeated repository archaeology. If the environment cannot safely refresh,
  stop cleanly.
- Never exceed the **Goal's actual configured budget**. The text instructions
  are not a hard token cap. No unlimited model/subagent loops.
- Normally at most **two new focused tests per checkpoint**. No big synthetic
  replay harness, no modifying approved fixtures. Keep test names
  `contract_v5_<family>_...` so CI runs them.
- At most one targeted expansion after a concrete compiler/focused-test
  failure. No full suites, profiler work, base archaeology, or infinite retries.
- Write terse checkpoints to ignored `target/workflow/m4-v5-night.md`;
  each entry: family, outcome, commit SHA, commands/result, reviewer findings,
  blockers, next batch. The Git commits are the durable state.
- Independent families may be **skipped**, not endlessly debugged. Log
  `BLOCKED: <specific missing semantic evidence or concrete failure>` and
  move to the next independent task only after restoring a coherent,
  compiling, clean state. If that cannot be done without losing unrelated work,
  stop the Goal.

## Shared invariants — check every meaningful diff

1. Action selection and historical interpretation see only player-safe and
   public facts at the relevant turn. Never infer an earlier convention from
   a future card reveal, opponent private card, or simulator truth.
2. Physical behavior is from `VariantSpec`, not `Card::index()`,
   `Rank::ALL`, fixed five-stack arrays, or `Rank::copies()`. Active
   domain operations use `identity_mask_in(&view.variant)`, active IDs,
   and `connection_* (view, heights, ...)` adapters in
   `h_group/identity.rs`.
3. `Card` can be a valid retained physical promise; do **not** globally
   replace it with `IdentityId`. Do not mix legacy 25-bit
   `IdentitySet::from_mask` with active-universe masks.
4. Preserve No Variant H-Group meanings and execution precedence:
   Save > false Play, old connection > speculative new one, Fix/retraction,
   Context/Elimination refinements, 5 Stall > 5 Tech, and established
   event-turn ownership.
5. Separate rank-specific **convention meanings** from physical rank
   ordering. Do not deduce Pink/Black/Brown/Omni/Null rule semantics from
   their physical clue behavior. Global No Variant gate remains until V6.
6. A direct physical `playable` check does not replace the full
   H-Group delayed Play/Finesse/Prompt domain. Keep lifecycle/provenance.
7. Legacy `ReplayReducer` `[u8; 5]`/`[u8; 25]` storage may remain behind
   the No Variant compatibility boundary. Change it only when a named
   production path directly requires migration.

## Preflight, then work the ordered capability queue

Preflight: verify correct branch, expected base ancestry, and no unexpected
dirty worktree. Read `AGENTS.md`, this packet, named functions and *direct*
callers. `rule_engine.rs::apply_rule` and `rules.rs::POST_EVENT_RULES`
are the shared dispatch boundary; change their semantics only when strictly
required. Existing inline source links provide H-Group semantic provenance.
No full repo inventory.

### Queue 1 — Context / Duplication interpretation (priority)

**A1 Context** — `recognition/advanced_bluffs.rs::apply_context_effects`
(currently starts around line 14).

- Migrate `IdentitySet::all()`, `clue.matches`,
  `ClueFacts::allows`, `is_playable_at` and `is_trash_at`,
  `view.play_stacks[identity.suit.index()]`, and `Rank::ALL[height]`
  at this **specific** recognizer's physical boundaries.
- Use `physical_clue_matches`, active clue domains,
  `connection_allows`, `connection_playable`/`connection_trash`,
  `connection_heights`/`connection_successor` as appropriate.
  Use **pre-event dynamic heights** for preceding-evidence conditions,
  not current stacks; don't substitute a current-knowledge identity for
  `context.historical.identity`.
- Preserve Fix suppression, Stale Ones, focus inversion vs a viable existing
  Prompt/Finesse, Selfish Clue/Finesse, and ordinary Context precedence.
- Direct producer/consumer test: at least one focused No Variant regression
  or deterministic invariant and one sixth-suit/active-order assertion when
  feasible. No invented convention meaning.

**A2 Duplication** — `recognition/advanced_bluffs.rs::apply_duplication_effects`
(currently around line 642).

- Convert two legacy fact-domain `identity_mask()` uses and fixed-stack
  `is_trash_at` uses to active domains with the **correct dated stack**
  (before event vs `origin.turn`).
- Preserve `previously_gotten`, direct vs blind focus, old duplicate
  proof, Duplicitous Value / Blind-Play / Tempo, Assisted Trash Chop Move,
  and Time Travel Chop Move.
- A1 and A2 are separate compile-safe checkpoints, but together close
  the Context/Duplication interpretation layer. Inspect direct
  `rule_engine` and resulting signal/connection consumers.

### Queue 2 — Remaining advanced refinements (priority)

**B1 Elimination** — `recognition/advanced.rs`:
`apply_elimination_effects`, `another_copy_survives_discard`,
`apply_elimination_resolution_effects`,
`remaining_elimination_candidates`,
`compatible_elimination_candidates`,
`known_copies_outside_hand`. Adapt only direct `rule_engine.rs` call.

- Replace physical five-stack, fixed copy counts,
  `ClueFacts::allows`, legacy clue mask/domain uses with active
  `VariantSpec` copies, `connection_playable`/`connection_trash`,
  `connection_allows`, and dynamic **before/after** stacks.
- Keep rank-2 convention exception, claim creation/retraction,
  Elimination Self Chop Move, Riding/Blind Play precedence, and
  historical count evidence.
- Check active sixth-suit copies and No Variant/observer-visible negative
  cases; never rely on hidden current or future identities.

**B2 Five Tech** — `recognition/advanced.rs::apply_five_tech_effects`;
only direct `rule_engine.rs` caller if required.

- Migrate standard `Rank::ALL` connector, index/mask construction,
  `stack_heights[identity.suit.index()]`, and physical identity
  assertions; use active variant stack/rank traversal.
- Preserve 5 Stall/Save precedence, 5 Pull, 5NE, 5ND, and 2-Save
  interaction. A `Rank::Five` condition remains an H-Group convention
  condition, not arbitrary numeric physical playability.

**B3 Out-of-Order** — `recognition/advanced.rs::apply_out_of_order_effects`;
only its direct `rule_engine.rs` caller if needed.

- Migrate active stack/identity/rank reasoning while preserving the
  ordering and exact cancellation/scheduling of existing connections.
- Reviewed No Variant regression:
  `reviewed_yellow_out_of_order_fix_uses_bobs_connector`
  in `h_group/tests/replay.rs`. Do not rewrite it or run it routinely
  if planner wall-clock variability makes it unsuitable as a focused
  semantic gate; add a narrow deterministic assertion where possible.

B1/B2/B3 are separate checkpoints. Keep the existing production rule
registry precedence intact.

### Queue 3 — High-level and late-game completion (stretch)

Run this queue only if earlier work is compiling/committed and the Goal
budget is still healthy. These families are independent **unless** a
specific direct dependency is discovered; skip a blocked family.

**C1 Ignition** — `recognition/late_game.rs::apply_ignition_effects`
(line ~13). Migrate remaining legacy `identity_mask()` knowledge
read and dated stack/rank assumptions. Preserve actor eligibility,
old versus new Finesse obligations, and clue history.

**C2 Phantom** — `recognition/late_game.rs::apply_phantom_effects`
(line ~326). Replace remaining legacy fact masks and fixed successor
construction with active variant predecessors/successors and dynamic
stacks. Preserve negative proof, connection timing, and Phantom vs
ordinary Play/Finesse precedence.

**C3 Charms** — `recognition/late_game.rs::apply_charm_effects`
(line ~585). Target the `standard_h_group_clue` on an old clue,
`Rank::ALL` connectors, fixed five-stack checks, and legacy identity
set operations. Recover the original physical clue and *dated*
stack state safely. If the recorded clue adapter cannot preserve
variant-specific touching semantics, log BLOCKED instead of treating
an unmatched clue as No Variant.

**C4 Unnecessary Moves** — `recognition/late_game.rs::
apply_unnecessary_trash_push_at_clue` and
`apply_unnecessary_move_effects` (lines ~1031 and ~1176).
Migrate their active fact-domain/physical conditions while keeping
trash push and clue-context precedence.

Do **not** rewrite the whole 1,673-line `late_game.rs` file or touch
`apply_priority_effects` (already migrated in V4) merely because
they share the file.

### Queue 4 — Evidence adapters and Extras (diagnostic stretch only)

If the previous queues have no outstanding blockers, perform **one**
targeted read-only inventory of the following legacy boundaries:

- `h_group/public_layers.rs::demonstrated_hidden_layer`:
  `standard_h_group_clue`, `Rank::ALL`, fixed historical heights.
- `h_group/inverse_planning.rs::deduce`: a historical
  `standard_h_group_clue` conversion into legacy `Action`.
- `recognition/extras.rs::apply_extra_effects`,
  `apply_max_special_effects`: remaining large physical assumptions.
- `recognition/special_discards.rs`: Special Finesse discard evidence.

Record an exact file/function/blocker/required API boundary map. Do not start
an unsized whole-file or cross-subsystem rewrite. If **one** seam is already
self-contained, has a reviewed source, and is not dependent on a V6
convention pack, you may implement **one** additional tiny checkpoint,
subject to the overall eight-checkpoint limit. Otherwise hand its map to
the morning review.

## Checkpoint protocol — no fake green claims

For each completed phase:

1. Review `git diff --stat`, named file diffs, and any direct
   consumer changes for hidden-truth leakage, dated vs current facts,
   active-mask mixing, connection/priority regressions, and variant
   physical behavior. No more than one bounded repair loop.
2. Add 0–2 focused tests using existing helpers.
   Use `contract_v5_<family>_...` names for new semantic contracts.
   Run the **single** focused `bash scripts/check-scoped.sh
   hanabi-search <filter> --lib` when available; otherwise
   `bash scripts/check-scoped.sh hanabi-search --check-only`.
   Run `git diff --check`; only run a separately named replay on a
   concrete regression risk, not a broad replay suite.
3. Commit a **compile-safe, coherent production + validation checkpoint**
   and push if the current sandbox/permissions allow it without
   privilege escalation. Don't open PRs or merge. If a partial diff
   cannot be made coherent, stop rather than committing broken code or
   discarding somebody else's work.
4. Record the outcome in the ignored progress note. Do not repeat
   completed-family investigation in the next Goal continuation turn.
5. A family is DONE only if its actual production path, relevant
   signal/connection consumer, negative evidence, No Variant behavior
   and a representative physical variant case have been addressed.
   Otherwise label `PARTIAL` with exact remaining consumers.

After the final checkpoint, do **one** separate read-only review of the
night diff against the stated base (independent reviewer/subagent if
available; otherwise explicitly self-review). Review only material bugs
and missing tests. Fix an in-scope high-severity finding at most once,
rerun its scoped checks, and commit. Do not spawn recursive reviewers,
mass-review all history, or change reviewed replay expectations.

Update `docs/WORKING_STATE.md` with what is genuinely completed/partial,
remaining active standard-only consumers and actual check status. Remove
this temporary packet from the eventual V5 PR, but keep the night
checkpoint available on the branch until the morning human review.

## Exit / morning handoff

Stop on an exhausted Goal budget, inability to maintain a coherent
worktree, non-local semantic question, repeated cross-subsystem fallout,
security/permission prompt, unsafe hidden-information premise, or
completion of the permitted queue. For a localized semantic BLOCKED
item, record it and continue only with a truly independent item.

End with a compact report: completed families/SHAs, PARTIAL/BLOCKED
families, direct production paths migrated, tests/check commands and
results, review findings, branch HEAD/dirty status and the exact
next morning decision. Do not claim V5/M4 completion, PR CI green,
independent convention correctness, or variant production enablement
from focused local tests.
