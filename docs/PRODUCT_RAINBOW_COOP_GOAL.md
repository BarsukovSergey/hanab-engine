# Goal: first useful Rainbow bot via literal cooperative clues

Base `main`: `90b5b1c3bdbe482cf4acb50db31d98e5a9c4b0e4`
(PR #52 merged; both PR/main CI green). Branch:
`product-rainbow-cooperative-clues`. Prepared 2026-10-23.

**We have changed product direction. This is NOT R2B, another M4-V5
migration, a complete H-Group variant bot or a generic search-planner
redesign.** R2B remains intentionally unstarted. Read `AGENTS.md`,
the relevant `docs/testing.md` and `docs/product-pilot.md`
sections, then only the named code below. Do not perform broad
architecture research or read old agent review journals.

## Why this particular feature, and not another migration

PR #52's three real `Rainbow6Suits` physical games all completed
with **TooManyStrikes**, official scores `0,0,0`, and `0` clues.
No legality/reconstruction/hidden-information errors were seen.
`ConventionAgnosticPolicy` deliberately NEVER clues and blind-plays
when it cannot discard at 8 tokens. Physical variants are playable
in the simulator; **cooperation is absent**. The separate No Variant
H-Group Max benchmark was interrupted after 11 actions / ~123 s,
so no current full-game No Variant score was measured. Do not
rerun that expensive diagnostic as part of this task or claim it
passed.

### Product acceptance test

Ship a **separately named**, fully observer-safe, cooperative
**literal-clue** `PhysicalAction` policy. On the same Rainbow seeds
1,2,3 and the **unchanged** pilot shuffle from PR #52:
- every game is played to actual terminal state, with no simulator,
  reconstruction, invalid-action, hidden-information or policy errors;
- **at least two of three** games finish without three strikes
  **and with positive official final scores**;
- at least one deterministic end-to-end prefix demonstrates
  **giver's physical clue → recipient gains certainty → recipient
  safely plays** without scripting the recipient move;
- output compares complete scores, strikes, action counts, turn
  counts and decision timings against the recorded PR #52 baseline
  (three zero-score strikeouts), and labels this a tiny pilot,
  **not** a general strength benchmark;
- the old `ConventionAgnosticPolicy`, H-Group semantics, production
  gate and existing reviewed fixtures are unchanged.

If that product gate is missed, do NOT portray a passing Rust test
or green CI as a useful bot. Leave coherent source commits and
honest diagnostics on a draft PR, explain the exact smallest
failure and recommended adjustment; don't start a broad second
strategy project or self-merge a bot that still has zero useful
game results.

## Source/API map (already verified)

- `crates/hanabi-search/src/baseline.rs`:
  `ConventionAgnosticPolicy::select_physical_action` already accepts
  `&LogicalDeductions<C: Copy + Into<GenericClue>>`,
  plays certainly playable, discards, never clues, and blind-plays
  at full clue tokens. `assess_card` and
  `AssessmentContext::new/assess` establish physically safe and
  useless direct knowledge.
- `crates/hanabi-core/src/view.rs`:
  `PlayerView<GenericClue>::legal_physical_actions()` lists
  real legal clues, plays, discards in stable order;
  `ObservedCard { id, identity: Option<Card>, clues: ClueFacts }`;
  `ClueFacts::record_physical(GenericClue, positive)`;
  `ClueFacts::identity_mask_in(&VariantSpec)`,
  `ClueFacts::is_empty`, `has_positive_physical_clue`.
- `crates/hanabi-core/src/variant.rs`:
  `VariantSpec::touches(IdentityId, GenericClue)`,
  `VariantSpec::standard_identity_id(Card)`,
  `VariantSpec::playable(&[usize], IdentityId)`,
  `VariantSpec::identities()`.
  Rainbow cards can be touched by **multiple physical color
  clues**; don't use legacy `Clue::matches` to project touches.
  Custom variants may have reordered/missing ranks and ID order.
- `crates/hanabi-search/src/information_set.rs`:
  `LogicalDeductions::possible_identities(CardId)`.
  Already computes the receiver's exact literal, public and
  visible-card logical exclusions; no new recipient convention
  rules are necessary.
- `crates/hanabi-search/src/lib.rs`: export the new separately
  named public policy without removing/changing existing exports.
- `crates/hanabi-search/tests/product_pilot.rs`:
  existing deterministic complete catalog deck, splitmix64
  shuffle, simulator, observer redaction, legal action validation,
  NDJSON, independent full-state reconstruction, official scoring.
  Reuse unchanged seeds/shuffle; extend with an opt-in policy
  selector and separate report name, **not** a second runner.
  Keep baseline policy the default for backward compatibility.
- Existing tests: `crates/hanabi-search/tests/baseline.rs`
  confirms the current no-clue behavior (do not break);
  `crates/hanabi-search/tests/generic_information_set.rs`
  has live physical-clue and sixth-suit examples.

## Minimal explicit cooperation protocol

This is **not** an invented H-Group clue meaning. Both sides use
one completely explicit protocol:

1. **Giver:** choose a physical clue because its *literal*
   touched/untouched information is useful. The giver is allowed
   to look at other players' visible card identities, but NOT
   the hidden cards of their own hand.
2. **Receiver:** derive possible identities only from public
   physical clue facts, other legitimately visible cards and
   public counts via the existing `LogicalDeductions` and
   `assess_card`. A clue **does not** assert "play the focus",
   "save the chop", or any finesse/promise. Play only when
   **all** legally possible identities are physically playable.
3. **Safety certificate for clue ranking:** for each legal
   clue to another player, calculate what positive physical
   fact each actually touched target card will gain, and
   which negative fact each untouched card will gain.
   Actual touching is determined from the *giver-visible*
   target identities with `variant.standard_identity_id` +
   `variant.touches`. Project the resulting
   `ClueFacts.identity_mask_in(&variant)` in **active ID**
   order. A card is conservatively certified playable after
   this clue only if the projected mask is nonempty and
   **every** remaining identity passes
   `variant.playable(current_public_stack_heights, id)`.
   The before-clue mask must *not* have already been
   certified playable to count as a newly certified card.
   Score additional information from genuine domain
   reduction; already repeated/no-op clues don't score.
   Never use the target's actual visible identity to
   **substitute for** the recipient's uncertain mask or
   award certainty directly. Don't swap observer IDs on
   a giver-only view to manufacture recipient knowledge.
4. **Action order (simple and deterministic):**
   - Act only for `view.current_player == view.observer`.
   - First play oldest **certainly playable** own card
     (same direct-knowledge proof as current baseline).
   - Otherwise choose a *legal* clue creating the most
     newly certified playable target cards, tie-break
     by target's turn proximity and stable label order.
   - Otherwise give a *legal, genuinely informative*
     physical clue while sufficient tokens remain;
     score domain reductions for target hand, choose
     deterministic ties. This permits multiple literal
     clues to narrow suit+rank without any implicit convention.
   - Otherwise discard a certainly useless own card when
     legal; else minimize loss of public literal information
     (prefer the oldest own card with the **largest**
     remaining possible-identity domain, instead of
     automatically destroying a well-clued card).
   - **At eight tokens, never blind-play while any legal
     clue exists**: make the best physical clue even
     if only a non-certifying clue is available, to
     release a discard token. When no safe Play, legal
     clue or legal discard exists, return an explicit
     error/diagnostic rather than fabricating a hidden
     safe action. Don't quietly fall back to blind Play.
   If an experiment suggests a small different tie-break
   is essential, document it and change only after a
   bounded focused test; do not hand-tune per seed.
5. Safety properties for a chosen Play must use **only**
   the acting player's own `LogicalDeductions`. For a
   clue candidate the giver may use exposed teammate
   identities to determine which public facts a legal clue
   would reveal, but must make its "will be safe" judgment
   using the *entire remaining domain*, not secret truth.
   Never use the simulator's future draw, deck or target's
   reconstructed hidden subjective hand.

Clues can be pointless if no receiver card is yet
certifiably playable; that is a measured limitation,
not permission to introduce unreviewed H-Group meanings.
Keep the algorithm small; no probabilities, world
enumeration, planners, hidden-card lookahead or new
serialization/API format.

## Implementation/validation queue

At most **three coherent checkpoints**, then independent
review and one PR. Reserve ~20% of agent budget for
review/CI. Prefer the minimum useful change in
`baseline.rs` + `lib.rs` + `product_pilot.rs` and
`docs/product-pilot.md`; one narrow integration-test
file is fine. No new production dependencies. Exact
choice of type name is yours, but export and document it
as opt-in **literal cooperative** physical policy.

### CP1. Focused behavioral contract + policy

Add 2–4 small `contract_physical_coop_` tests
or an exact narrow test target. Use *legal*
`FullState::new_variant` prefixes and real
`PlayerView<GenericClue>`:
- At 8 tokens, a Rainbow clue is chosen instead of
  a speculative Play (when legal clue exists).
- A Rank-1 clue on a target holding physical playable
  Rank-1 makes the receiver's **own actual deduction**
  certainly playable. At its turn the **receiver policy**
  chooses that safe Play. No scripted play, no hand
  identity leak.
- Rainbow collateral multi-color touching must
  not create a false singleton or false safe play;
  negative facts on untouched slots apply.
- Ordinary No Variant and/or active reordered ID
  case preserves correct active physical clue masks,
  playability and deterministic choice.

Run exact focused tests and
`bash scripts/check-scoped.sh hanabi-search
contract_physical_coop_ --lib` **only if lib
tests were added**; otherwise use the exact
integration-test target and
`bash scripts/check-scoped.sh hanabi-search --check-only`.
Run `git diff --check`, commit+push the coherent
implementation once it compiles. Do not use the
existing 200-game H-Group suite.

### CP2. Compare *same* three Rainbow seeds

Extend existing ignored
`rainbow_physical_pilot` with
`HANABI_PRODUCT_PILOT_POLICY=cooperative` (default
continues baseline); namespaced policy/report data,
unchanged seeds/shuffle/ordered deck. Preserve every
pre-decision trace and exact action label.
For each actual Play, record whether
`assess_card` certified it on the acting view
before action; assert direct-knowledge policy doesn't
select speculative plays. Keep official final score
and three-strike termination separate.

Run release pilot on seed 1, then seeds 2–3 if seed 1
is fast and error-free. Use separate ignored paths
`target/product-pilot/rainbow-coop.json` and
`rainbow-coop-extra.json`. Compare with PR #52:
the baseline finished three strikeouts with
official score 0 on every seed (stack 0,2,0).
Report actual new non-strikeout rate, official/stack
scores, legal clue counts, safe plays and time
per decision. Do **not** compare across changed
deck shuffle or different seeds.

If seed 1 fails, reproduce exact prefix and make
at most **one** small causal fix. If blocked by
a large missing policy/inference capability,
stop and hand off the exact failure; no broad
engine rewrite and no fake positive result.

### CP3. Product evidence and release decision

Update durable `docs/product-pilot.md` with a
new dated section: unchanged seed pairing, policy
name, code SHA, actual tables, exact commands,
results, independent review conclusions and
unresolved limitations. Explicitly distinguish:
physical legality, cooperative literal inference,
measured product score, and H-Group conventions
(still NOT supported for Rainbow).
Keep the previous failed measurements and
incomplete No Variant report; never overwrite
them with new narrative.

Run exact ordinary tests, an appropriate scoped
production-config check, `git diff --check`
and a single independent read-only diff review
(if available). Check no information leak,
no counterfeit convention meaning,
Rainbow color collateral, active ID masks,
negative clue evidence, legality, No Variant
regression and deterministic ties.

If product acceptance (>=2/3 Rainbow seeds
with non-strikeout **and positive official scores**,
plus observed clue→recipient-safe-Play), focused
checks and required PR CI all pass: open
**one PR** targeting main and autonomously
squash-merge after CI and independent review;
then inspect main CI. At most two targeted
mechanical CI fixes; don't change product
expectations just to make the report green.

If gate misses or serious semantics are
unresolved, do not merge as a successful
product feature. Commit/push a coherent
branch and leave a draft PR with exact
measurements/blocker for the next iteration.
No deployment, production policy switch,
No Variant gate change or convention packs.

Remove this temporary Goal document from
the final PR diff; retain the evidence in
`docs/product-pilot.md`, focused tests and
opt-in implementation. Don't create an
unreviewed H-Group optimal-move fixture.

## Final handoff format

- branch/PR/merge SHA and actual PR/main CI;
- per-seed original vs cooperative official
  score, strikeout, clue count and timings;
- demonstrated real giver-clue → recipient
  literal certainty → receiver Play;
- exact pre-decision legal view/no hidden
  information and reconstruction checks;
- acceptance gate PASS/FAIL, not just tests;
- one clear next *product* step, not
  an automatic R2B/V5 migration.
