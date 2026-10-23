# Product pilot Goal: measure the existing bot, play real Rainbow games

Prepared: 2026-10-23.
Base `main`: `015731bb7872980e93b670a2ace910816727f251`
(merged PR #51). Branch: `product-pilot-no-variant-rainbow`.

**This replaces, rather than continues, the unstarted
`m4-v5-r2b-historical-plays` task.** Do not work on R2B or migrate more V5
recognizers here.

Read `AGENTS.md`, the relevant parts of `docs/testing.md` and
`docs/self-play.md`, then the named code below. The source and test map
is already provided. Do not repeat M4 architecture research, upstream
convention research, R2A/R2B archaeology, or broad repository inventory.

## The question this Goal must answer

1. **How well does the existing No Variant H-Group Max bot actually
   perform in complete self-play games?** Measure, never infer strength
   from passing CI or isolated convention contracts.
2. **Can the existing physical-variant, convention-free policy complete a
   genuine Rainbow6Suits game from initial deal to terminal state?**
   Verify every move is selected from the acting player's legal visible
   view and applied through the real physical simulator. Report the actual
   scores, strikes, actions, costs and failures.
3. **What is the smallest specific missing capability that separates this
   physical end-to-end proof from a useful variant-playing bot?**
   Name it and its exact API seam, with evidence.

**Don't disguise one outcome as another.** A game that terminates after
three strikes is a completed simulator lifecycle but **not a useful-bot
success**; its official final score is zero. A legal convention-free game
cannot prove H-Group semantics, and the four-bot self-play team may share
a convention mistake. Do not call a single seed a strength benchmark.

## Verified entry points / expected current limitations

- Existing No Variant benchmark:
  `crates/hanabi-search/tests/self_play.rs`,
  `scripts/check-self-play.sh` and `docs/self-play.md`. It plays
  H-Group Max as four standard players using `plan_move`,
  `InformationSet`, the official perfect-score objective and
  production planner budgets. The opt-in test accepts
  `HANABI_SELF_PLAY_START`, `HANABI_SELF_PLAY_GAMES`,
  `HANABI_SELF_PLAY_WORKERS`, `HANABI_SELF_PLAY_REPORT`.
  For fewer than 200 seeds, it does **not** require a stored baseline.
  A full clean 200-game baseline **does not yet exist**; do not create
  or update one in this Goal.
- Generic physical variant boundary:
  `hanabi_core::VariantSpec::from_catalog(VariantId::Rainbow6Suits)`;
  `VariantSpec::identities().identities()` (each entry has `suit`,
  `rank`, `copies`);
  `FullState::new_variant(3, variant, complete_ordered_deck)`;
  `state.physical_view_for(state.current_player())`;
  `LogicalDeductions::new(view)`;
  `ConventionAgnosticPolicy.select_physical_action(&deductions)`;
  `state.apply_physical(action)`. See
  `crates/hanabi-search/tests/generic_information_set.rs`
  for a complete valid deck builder and several physical variants.
- **Crucial product limitation:** `baseline.rs::ConventionAgnosticPolicy`
  deliberately never emits a clue. It plays certainly playable,
  discards certainly useless or oldest when legal, and blind-plays
  newest at eight clue tokens. This is an *inference/physical-policy
  baseline*, not an effective cooperative clue-giving strategy.
  Expect that it might finish by strikeout. Measure this honestly:
  no scripted clues, scripted correct plays, special simulator
  knowledge or secret identity fed to the policy.
- **Do not use** the standard-only `plan_move` /
  `analyze_position` (accepts legacy `InformationSet` and
  `Action`) as if it already supported arbitrary
  `PlayerView<GenericClue>` and `PhysicalAction`. The genuine
  physical baseline is the named `ConventionAgnosticPolicy`.
  The No Variant H-Group production gate remains untouched.
- `FullState::new_variant` consumes cards as listed; the deck must
  match *every* identity's physical copy count. Reordering/shuffling
  is permitted but the identity multiset may not change. It deals
  whole hands in player order; don't pretend generic pilot seeds
  are Hanabi Live `p4v0s*` seeds.

## Work budget

This is a **measurement and small opt-in harness task**, not an engine
rewrite. Use at most **four coherent checkpoints** and reserve the
final 20–25% of the Goal budget for reviewing evidence, an independent
diff review, one PR, CI and at most two targeted CI fixes.

Do not spend 200-game wall-clock time or exhaust the Goal budget
waiting for slow games. Use one No Variant seed as the initial
diagnostic; expand to at most four fixed seeds **only if time allows**.
Use one Rainbow seed first; expand to at most three total fixed seeds
only if the runner is completing reasonably. No broad profiler, full
self-play benchmark, exhaustive matrix or routine full `check.sh`.
Stop expensive experiments at the agreed resource boundary and
record **partial**, rather than extrapolating missing scores.

Use ignored paths under `target/product-pilot/` for full reports,
resumable per-seed snapshots and detailed action histories. A concise
checkpoint log may live in `target/workflow/product-pilot.md`.
Preserve error reports even if a diagnostic test exits nonzero;
do not claim a diagnostic's nonzero exit code means that every
preceding game failed.

### Checkpoint 1 — Observe actual No Variant strength (NO edits)

Preflight check expected branch, main ancestry and clean worktree.
Run the **existing unmodified** H-Group Max release-mode self-play
using one fixed seed, one worker, separate report path:

```sh
HANABI_SELF_PLAY_START=1 HANABI_SELF_PLAY_GAMES=1 \
HANABI_SELF_PLAY_WORKERS=1 \
HANABI_SELF_PLAY_REPORT=target/product-pilot/no-variant-seed1.json \
scripts/check-self-play.sh
```

Inspect `target/product-pilot/no-variant-seed1.json` and its
`.ndjson` / `.active` data if the run fails, aborts or stalls.
Record actual score, perfect/nonperfect status, turns, strikeouts,
engine errors, elapsed time, first failure, and actual exit status.
Do not change or overwrite expert fixtures or invent success.

Only if it fits the remaining wall-clock budget, extend to a
separate 4-game fixed-seed batch (seeds 1–4) with 1–2 workers,
separate report, same production config. Never change the
200-game baseline; never claim statistical strength from these
few seeds.

### Checkpoint 2 — Minimal offline Rainbow physical-game runner

Implement a **small, reusable, explicitly opt-in** integration test
in `crates/hanabi-search/tests/product_pilot.rs` (or a similarly
sized dedicated test). Prefer a single integration-test target,
not an application framework or CLI/live-server feature.

- Build complete Rainbow6Suits physical deck from catalog
  identities and exact copy counts; apply a **documented,
  deterministic seed+shuffle** with no production dependencies.
  A tiny test-only shuffle is fine. Record seed, method/version
  and ordered deck in the *offline* diagnostic report so one
  run can be reconstructed exactly; the shuffled deck is
  held exclusively by the simulator, **never by the policy**.
- Use **three players**, one acting player view per turn:
  `physical_view_for(current_player)` ->
  `LogicalDeductions::new` ->
  `ConventionAgnosticPolicy.select_physical_action` ->
  validate action against `view.legal_physical_actions()` ->
  `FullState::apply_physical`.
  All three players use that same **honestly labelled**
  non-H-Group policy. Don't manually inject clues to make the
  game's score look good.
- Run a **complete physical game** until `state.is_terminal()`
  or an explicit safety cap (e.g. 200 turns). A safety-cap
  exit is *not completion*. Capture panics and structured
  errors per game; do not silently skip positions.
- Persist `target/product-pilot/rainbow.json` and a flushed
  NDJSON action trace/checkpoint before each decision (so a
  failed or interrupted game can be reproduced). Record
  the exact physical action label (including any
  `GenericClue` values) and card instance IDs, not a
  lossy guessed Hanabi Live legacy action JSON. Include
  seed, variant, player count, policy, code SHA, full physical
  deck ordering (diagnostics only), actual terminal reason,
  official final score, stack score, strikes, turns,
  counts of Play/Discard/Clue, max and total move elapsed
  time, error details. Record a full simulator replay or
  independent reconstruction check against the same
  physical action sequence.
- Validate redaction on **every** decision view: the
  current player's own unshown identities remain hidden.
  Only the acting `PlayerView<GenericClue>` is passed into
  logical deductions and policy selection. Never let a
  future draw, physical hidden identity or discarded
  future card leak into candidate choice.
- Add a quick ordinary `#[test]` for deterministic
  deck completeness/reconstruction and at least one
  explicit `#[ignore]` full-game pilot test (bounded and
  run by exact test filter). Do not run full-game pilots
  as ordinary PR CI contracts. The opt-in test is allowed
  to produce a report of poor/strikeout play; the
  **report**, not merely the Rust test's exit code,
  determines whether the policy is viable.

### Checkpoint 3 — Run and interpret rather than paper over failures

Run the Rainbow pilot for **one fixed seed** with a focused
release test invocation. If it completes promptly, add at
most two more seeds using the same fixed code SHA and
config. Separate genuine `GameStatus::terminal` caused by
3 strikes from win/end-of-deck completion. Report **actual
measured values** for every run and the count of truly
non-strikeout endings.

If an illegal action, panic, invalid information set or
nondeterministic reconstruction occurs:
- reproduce the earliest failure from recorded seed/action
  prefix;
- fix **at most one small, provable invariant bug** in the
  named physical runner or its immediately blocking
  `hanabi-core` /`baseline.rs` boundary, with a focused
  regression and no source convention meaning changes;
- if it requires planner work, new clue strategy, a
  broad policy redesign, or altered H-Group convention
  meaning: **stop the implementation**, report exact
  public input / failing symbol, preserve the working
  diagnostic harness.
Never fake a complete success by swapping the policy,
scripting hints, choosing cards from `FullState`,
relaxing legality, removing history or increasing
the score after the fact.

If the whole-game run completes but repeatedly strikes
out because it never clues, this is a **product finding**,
not a physical-engine bug. Report that a minimal explicit
cooperative physical-clue strategy is the next
product feature; do not sneak it into this Goal as an
unreviewed H-Group shortcut.

### Checkpoint 4 — Durable product report, narrow QA, ship

Create `docs/product-pilot.md` (durable output):
- exact analyzed SHA, variant(s), players, profile/policy,
  seed+shuffle method/version and exact command/config;
- table of **actual completed** No Variant and Rainbow
  pilot results; incomplete/timeout/engine-error cases
  plainly marked and never counted as completed;
- clear split between (a) simulated end-to-end pipeline,
  (b) non-strikeout/score viability, (c) convention correctness,
  and (d) statistical strength (not proven by this run);
- direct links/paths to reproducible output under ignored
  `target/product-pilot/` and the smallest blockers;
- concrete next action **based on evidence**, not an
  invented urgency to resume R2B/R2C.
  Mention whether the first next step is adding a very
  small physical clue-giving policy or integrating real
  H-Group Level1, and why. Do not enable either now.

Preserve `docs/self-play.md`'s description of the
unestablished 200-game clean baseline. Do not create a
baseline fixture from partial runs. Do not modify existing
expert fixtures, player actions or PR CI to get green.

Checks: run the exact new ordinary test and exact opt-in
pilot test separately (release-mode for gameplay),
`bash scripts/check-scoped.sh hanabi-search --check-only`,
and `git diff --check`. Review data/report for
unearned score claims and hidden-information leakage.
If a direct core fix is needed, run its focused test
and affected crate check as well.

Commit+push one or two coherent source/report checkpoints,
then run one **independent read-only review** of all changes.
Reserve PR integration budget. Remove this temporary
`docs/PRODUCT_PILOT_GOAL.md` file from the **final PR diff**
(the task is retained in branch Git history), retain
`docs/product-pilot.md` and small reusable runner.

Open **one PR** to `main` only for coherent,
compiling, reviewed pilot tooling and an honestly measured
report. Observe required PR CI; at most two narrow
CI-fix pushes. If required CI green and no unresolved
code correctness issue, **autonomously squash-merge**,
then check main CI; do not block merge solely because
the measured bot plays poorly. Poor play is not the
same as a software failure, but do not claim a product
success from it. If source/tooling is broken or serious
safety issue unresolved, leave a draft PR and exact
blocker. Respect branch protections. No production
policy gates, deployment or live actions.

## Final response contract

Include:
1. merged PR URL / squash SHA if merged, actual PR/main CI;
2. the exact No Variant game count and scores (or
   incomplete/engine-error reasons);
3. the Rainbow seed count, actual terminal reasons,
   **official** scores, strikes, action counts and
   whether any game finished other than by strikeout;
4. evidence that each policy received only an
   observer-safe physical view; exact report/replay paths;
5. the next **smallest product capability**, separated
   from leftover M4 architecture migrations.

Do not claim a `plan_move`-quality variant planner,
H-Group correctness for Rainbow, a clean 200-game
baseline, or nonzero score unless directly measured.
