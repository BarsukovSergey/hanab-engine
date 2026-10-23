# M4-V5 overnight execution packet

Base: `bd960555c5b0df35865d8c42617b016a1094fe49`
Branch: `m4-v5-high-level-refinements`
Prepared: 2026-10-23

This is a temporary, branch-local **bounded implementation queue**; remove
this night-only packet from the eventual V5 PR once its useful outcomes are
recorded in `WORKING_STATE.md`. It is not permission to complete all of M4-V5
or invent H-Group conventions overnight. The actual slice definition is
in [M4_VERTICAL_EXECUTION_PLAN.md](M4_VERTICAL_EXECUTION_PLAN.md). Follow
`AGENTS.md` and [codex-workflow.md](codex-workflow.md). One V5 branch/PR may
contain many short batches; this overnight run must not open or merge the PR.

## Outcome and budget

Implement, verify, and checkpoint as many of the **four named migrations below,
in order**, as are possible without changing their established convention
meaning. A phase is complete only if its narrow production path is migrated,
the relevant focused tests pass, the change is reviewed for the invariants
below, and it is committed.

Hard scope limit: **four phases**, no extra families, no "while here" fixes,
at most **two new focused contract tests per phase**, and **one final read-only
review pass**. Target approximately 10–20 minutes of active agent work per
phase; at approximately 60–65% thread context usage or an exhausted Goal
budget, checkpoint and stop rather than continuing the loop. Do not start
another phase if a prior one has an unbuildable/dirty partial edit. It is
correct to finish early.

Treat one implementation phase as one Goal continuation turn where practical;
do not reload the whole repository or repeat architectural research between
phases. Write one-line progress entries to ignored
`target/workflow/m4-v5-night.md`: phase, commit SHA, files, check result,
review findings, blocker. Existing Git commits are the durable checkpoint.

## Preflight

1. Confirm the branch and expected starting commit (later phases may have new
   in-branch commits). Stop on unexpected dirty state or branch mismatch.
2. Read only this packet, `AGENTS.md`, the named functions, and any direct
   call sites. Existing source comments cite the relevant H-Group rules;
   only investigate outside them if a *specific* semantic contradiction arises.
3. No automatic installation, toolchain repair, wide baseline archaeology,
   network research, broad replay scans, or whole-repository refactors.

## Ordered phases

### A — Duplication (first)

- Production function:
  `crates/hanabi-search/src/h_group/recognition/advanced_bluffs.rs::apply_duplication_effects`
  (currently near line 642).
- Convert the two legacy `identity_mask()` / `IdentitySet::from_mask`
  reads in its duplication proof into **active-universe domains** using
  `ClueFacts::identity_mask_in(&view.variant)` and appropriate active-ID
  operations. Do not combine active bits with legacy `Card::index()` bits.
- Replace `is_trash_at` calls in that function with physical trash checks
  using `connection_trash(view, heights, card)`. Use
  `context.before.dynamic_stack_heights` for the before-event state and
  `connection_heights(view, Some(origin.turn))` for an older originating
  clue. Preserve the distinction between the two historical times.
- Preserve the distinctions between Duplicitous Value, Blind-Play, Tempo,
  Assisted Trash Chop Move and Time Travel Chop Move, and the
  `previously_gotten` requirement. Do not touch the adjacent Context or
  Bluff family.
- Validate with one narrow `contract_v5_duplication_` test if a suitable
  existing test helper permits an invariant without fabricating H-Group
  convention authority. Check No Variant behavior and a sixth-suit/domain
  case; prefer existing reviewed facts.

### B — Elimination

- Functions in
  `crates/hanabi-search/src/h_group/recognition/advanced.rs`:
  `apply_elimination_effects`, `another_copy_survives_discard`,
  `apply_elimination_resolution_effects`,
  `remaining_elimination_candidates`,
  `compatible_elimination_candidates`, and
  `known_copies_outside_hand`.
  Adjust only the direct caller in `rule_engine.rs` when needed.
- Replace legacy five-stack playability/trash, `identity.rank.copies()`,
  and standard `ClueFacts::allows` with active physical
  `VariantSpec` copies, `connection_playable` /
  `connection_trash`, and `connection_allows`.
  Use the already available dynamic before/after stacks, not fixed reducer
  arrays. Keep `Card` as a promise identity at the boundary; map to active
  `IdentityId` when performing identity-set or physical operations.
- Preserve the established rank-2 H-Group exception, count evidence at its
  historical turn, claim retraction, Self Chop Move, Riding, and Blind Play
  precedence. Do not add future hidden reveals to historical deductions.
- Add at most two focused `contract_v5_elimination_` tests for physical
  copies/domain/variant behavior and preserved No Variant behavior.

### C — Five Tech (only after A and B are coherent)

- Function:
  `crates/hanabi-search/src/h_group/recognition/advanced.rs::apply_five_tech_effects`;
  only adjust its direct caller in `rule_engine.rs` if required.
- Migrate the legacy five-stack lookup, `Rank::ALL` connector
  construction, `identity.index()` mask, and ordinary clue-fact matching
  to active identity/rank/stack traversal through existing
  `identity.rs` connection adapters and `VariantSpec`.
- Preserve **5 Stall > 5 Tech**, Save precedence, 5 Pull, 5NE, 5ND,
  and 2-Save interaction. Rank 5 remains a *convention meaning*, not an
  arbitrary `VariantSpec` rank. If physically touched fives remain
  ambiguous, do not assert a false exact identity.
- At most two narrow `contract_v5_five_tech_` tests, plus a scoped check.
  If the semantic meaning cannot be preserved without new rulings,
  **stop the whole Goal**; do not improvise.

### D — Out-of-Order Play (only if time/context remain)

- Function:
  `crates/hanabi-search/src/h_group/recognition/advanced.rs::apply_out_of_order_effects`;
  adjust direct caller in `rule_engine.rs` only if needed.
- Migrate only standard physical assumptions in active identity/stack/rank
  traversal. Preserve historical action/connection order and the distinction
  between a genuine Out-of-Order repair and an ordinary Finesse.
- At most two `contract_v5_out_of_order_` tests. A reviewed No Variant
  check already exists:
  `reviewed_yellow_out_of_order_fix_uses_bobs_connector`; do not rewrite
  its fixture/expectation or run broader replays by default.

Context, Ignition, Phantom Playable, Charms, Unnecessary Moves, Extras,
public-layer and inverse-planning migrations are **not authorized tonight**.
They remain V5 work for an independently prepared patch map. V6 convention
packs, production enablement, and the global No Variant gate are forbidden.

## Check, review, checkpoint: once per phase

1. Inspect `git diff --stat` and only changed-file diffs. Check historical
   knowledge vs hidden simulator truth, dated vs current stacks, active
   identity masks, physical touching/copies, priority, and No Variant parity.
2. Run `cargo fmt --all` once if needed. Run the new phase's exact focused
   `contract_v5_...` test if one was added, then
   `bash scripts/check-scoped.sh hanabi-search --check-only` once; avoid a
   duplicate `cargo test` via the helper. Run `git diff --check`.
   If no suitable new test is justified, reuse a named existing focused
   contract instead of generating a large synthetic fixture. Record exactly
   what was *not* tested.
3. One targeted repair attempt for a concrete compile/focused-test failure.
   If the fix requires an unplanned subsystem, semantic choice, old-fixture
   rewrite, or a second broad exploration, **stop**, report it, and preserve
   the coherent checkpoint (or report exact dirty in-scope edits).
4. Commit each coherent passing phase with a descriptive message. Push to
   the existing feature branch if the configured credentials/sandbox permit
   it without an approval prompt; otherwise preserve local commits and
   report that the remote is behind. Never force push or merge.
5. After the final completed phase, do **one** read-only review of the
   night diff against the starting commit. Prefer a single independent
   reviewer/subagent if available; otherwise label it a self-review. Review
   hidden-information leakage, semantic drift, legacy active-ID mixing,
   historical evidence, and missing critical tests. Fix only concrete
   in-scope high-severity findings and re-run the affected focused check
   once. Do not keep spawning reviewers or chasing style suggestions.

## Final report and stop

Report: completed phases and SHAs, branch/head/dirty status, exact checks,
review findings and repairs, skipped phases, blockers, and unverified
semantics. **Do not claim full V5 completion, CI green, or independently
verified convention correctness** from local focused checks. Leave the V5
branch for human/orchestrator review in the morning. No PR, merge, release,
or live gate change.
