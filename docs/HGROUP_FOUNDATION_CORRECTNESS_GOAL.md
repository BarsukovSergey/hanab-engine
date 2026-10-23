# Goal: H-Group Foundation Correctness Gate (Levels 1–4)

Status: OPEN. Created 2026-10-23. Base: main 51f1784132d367ca2f5a3aeff06bd91821a9a74c.
Execution branch: quality-hgroup-foundation-l1-l4.

## Product outcome

Make the existing H-Group implementation **trustworthy for foundational Hanabi and cumulative H-Group Levels 1–4**, not merely capable of completing a game. The deliverable is a **reviewable, source-grounded correctness audit with executable regression coverage and repairs of confirmed bugs**. No new bot, second strategy, convention bypass, or blanket fallback policy.

A successful White 24/30 game proves that one offline game completed, **not** that basic H-Group works correctly. The saved White game is a **bug reproducer**, never an oracle for the optimal move or convention meaning.

This is a deliberately comprehensive, multi-checkpoint **Goal**, not a mandate to implement 48 unrelated fixes in one coding-agent session. Follow AGENTS.md and docs/codex-workflow.md. Use successive small committed, pushed batches on this branch; maintain the checklist across sessions. Do not claim complete validation while any required contract remains unaudited.

## Authority and boundaries

1. **Physical rules:** hanabi-core and hanab.live official rules/implementation for No Variant and physical semantics, not the search planner.
2. **Convention meanings:** official H-Group documentation pinned by docs/h-group-coverage.md (or verify a documented upstream revision change) plus independent **human-reviewed replay positions** as evidence. An existing engine output, a synthetic self-play choice, or a test that mirrors current code is not independent authority.
3. Scope: the No Variant physical game (2–6 players), active H-Group Levels **1, 2, 3, 4** with their cumulative enablement, and their actual connections to inference, candidate admission, decision selection, historical replay, and plan_move. White6Suits is a **cross-variant regression witness** for shared rules, not a request to implement full White-specific conventions.
4. Out of scope: implementing Levels 5–25/Max, global variant migration, switching production policy to White, rewriting the search framework, score tuning, or trying to force 30/30 on a seed. Higher-level regressions that share the modified code must still run.
5. Preserve No Variant production behavior, variant gates, observer redaction, exact game rules, and H-Group hard constraints. A changed preferred move is permitted only with a documented source-grounded reason, not because its score improved.

## Mandatory audit matrix

Create docs/HGROUP_FOUNDATION_AUDIT.md with one row for **every** Level 1–4 source section (19 + 11 + 9 + 9 = 48 inventory headings in docs/h-group-coverage.md). For each: official source URL/section, rule or principle, enabled level, production path, a positive example, a negative or counterexample where applicable, owner-perspective observation boundary, test name(s), current status PASS / CONFIRMED BUG / UNSUPPORTED / NOT YET TESTED, and the evidence. Documentation headings may share tests; do not equate the presence of a rule name with correctness. A section that is explanatory instead of independently executable still needs explicit justification and a linked invariant. No unreviewed semantic expectation may be invented to fill a row.

Check these families explicitly, independently of the general source inventory:

### A. Physical core (No Variant, 2–6 players)

- Accurate full deck, copy multiplicities, hand sizes, deal order, draw, turn order, no cards appearing/disappearing, and legal play/discard/clue actions.
- Clue touching and negative information; clue tokens and gains/spends; legal/illegal actions; successful play, duplicate/out-of-order play and strike accounting; discard legality; terminal conditions, final round and official score.
- FullState -> redacted player observation -> InformationSet -> decision -> physical action -> replay consistency. Own cards/draws are never revealed to a deciding observer. Historical visibility uses **event-time** facts, not current/future truth.

### B. H-Group Level 1: core meanings

- Focus rules; Play versus Save versus stall; clue meaning, Good Touch/Minimum Clue Value, known/playable/trash/critical identities, 2/5/Critical Saves.
- Direct Play Clue, basic Prompt and Finesse, Finesse Position and ordering, prompt-before-finesse precedence, inter-player blind response, ordinary clue response and the effect of declining/deferring.
- A valid connection may be unknown in the actor's **own hand**, but cannot require a **different player's visible card** to be an identity visibly impossible to that observer at clue time. Test observer-by-observer; do not let hidden game truth leak into the acting player's inference.

### C. H-Group Level 2: basic moves

- Repeated and multi-step connections, Self/Reverse Finesse, legal sequencing, 5 Stall and the other official Level 2 cases. Check correctly timed obligations, deferral, disambiguation, and aftermath when a promised step fails.
- Level 2 must not accidentally require Level 5 advanced Ambiguous/Layered/Queued Finesse semantics; maintain explicit rule gates.

### D. H-Group Level 3: basic strategy

- Fix and promise cancellation, repeated rank-1 treatment, Sarcastic Discard, clue-efficiency constraints, choice and precedence of known plays, useful Play Clues and Saves. Separate convention correctness from subjective score optimization.

### E. H-Group Level 4: chop movement

- Trash Chop Move, Five Chop Move, Order Chop Move and documented precedence, protected/gotten/chop state across turns, and no invented card-identity claims from protection.
- Moving a card which is **actually trash** is not ipso facto an error if the actor could not know it. Conversely, knowingly spending protection on proven trash or violating the official move's prerequisites must be detected.

### F. End-to-end cross-perspective contract

For every named clue/connection under test, compare giver, all relevant intermediate actors, recipient, and historical reconstruction **at the correct decision times**:
- known visible cards and private hidden cards are interpreted distinctly;
- interpretations may differ lawfully across observers but must not contradict each observer's visible facts, the documented convention precedence, or established promises;
- a genuinely due, playable blind response must surface in the executor's convention action set with correct priority; a non-due or unsafe response must not be fabricated;
- candidate admission, prospective safety checks, recipient replay, connection lifecycle, and executed action must agree about **what is proven, conditional, or unresolved**. A generic responseSafety=Pass or GeneratorProof alone is not proof that every necessary intermediate reaction is safe;
- planner cannot discard a hard mandatory play or create a play that H-Group has not admitted, regardless of search budget/fallback.

## P0 live-game reproducer: White seed 1

Use the existing offline runner and the saved White game if available under target/product-pilot/. The original game source SHA was 5f7585da1da6a2b45d6a6551c84eafea304a834f; its JSON was white-hgroup-5f7585d.json. If files are not present, reproduce the *recorded prefix* from its original actions; do not assume current planning recreates the same later choices after code changes. Keep a minimal source-data-only fixture in tests when recovered.

- Engine turn 44 / human replay turn 45: Cathy clues Red to Bob's Red 4 (cardId 32). The true Alice hand contains Red 3 (cardId 39). Bob has cardId 40, physically White 2.
- Alice on engine turn 45 / human replay turn 46 did not gain a due own finesse; her history analysis proposed Red 3 from Bob/cardId 40 even though Bob's White 2 is **visible to Alice**, and her planner discarded cardId 2.
- Bob later treated cardId 40 as Red 3, played it on engine turn 49 and struck.
- Reported clue diagnostics included OtherClue/Unrecognized or generatorProof and responseSafety: Pass in relevant captures. **Confirm each label at its own stage**; do not conflate a recorded candidate's purpose with its final selected action.
- Suspect h_group.rs::schedule_connection (the non-SpecialFinesses branch that takes the first unclued card), historical-event visibility and/or admission/safety chain in h_group/interpretation/candidate_validation.rs, candidate_pipeline.rs, principle_validation.rs. This is a **suspected call path, not an authorized one-line fix**. Verify by a red test and call-path evidence before patching.
- Fix the first demonstrably false inference; then assess whether the clue should have been rejected, reinterpreted, or safely responded to under official Level 1–4 rules. Do not require a specific improved game score or force Alice to play if official semantics do not justify it.
- Add an equivalent **No Variant** contract that proves the general visibility rule and a counterexample where the card really is hidden and a blind Finesse is valid. Ensure it fails before and passes after the repair.

## Evidence and test methodology

1. **Inventory & baseline first:** record main SHA, current relevant test pass/fail baselines and existing reviewed replay expectations. Include known pre-existing failures without quietly rewriting them. List missing semantic contracts before implementation.
2. **Independent expected outcomes:** derive each new convention assertion from an exact official paragraph/example or human-reviewed replay, not from the current inferred output. Capture source and rationale in the test or audit table.
3. **Red -> green -> regression:** for each confirmed error, add a deterministic failing contract, capture the failure on the current code, fix the smallest correct shared boundary, run the focused test, affected reviewed replays, and higher-level compatibility regressions. Preserve existing tests and fixture truth.
4. **Property/metamorphic contracts:** when valid, assert suit-renaming symmetry for No Variant, compatible seat/rotation transformations, identity-copy invariants, replay reconstruction, and **indistinguishability** (two hidden worlds giving the same observer-visible history must yield the same admissible conclusions/actions). Never transform a position in a way that changes the rule's prerequisites.
5. **Risk-weighted breadth:** at least 2-, 3-, 4-, and 6-player physical cases; at least 3-/4-player multi-perspective finesse cases; edge cases for zero clues, exhausted copies, known trash, delayed/failed connections, and end-of-deck. Make the suite deterministic and fast enough for CI; use opt-in full self-play solely as supplementary smoke.
6. **Separate three failure types:** (a) wrong/no inference, (b) wrong admission or unsafe clue, (c) wrong planner selection given correct admitted actions. Never try to fix (a) or (b) with planner heuristic weights. Performance limits are diagnostics, not excuses for violating hard obligations.
7. **No bulk test laundering:** do not delete or weaken failing assertions, secretly update expected actions to the new implementation, silently mark failures as ignored, or count a green test with no independent oracle as validated semantics.

## Execution checkpoints

**Checkpoint 0 — Baseline and map.** Commit docs/HGROUP_FOUNDATION_AUDIT.md with 48 source rows, core-physics checklist, existing test evidence, and the P0 reproduction plan. Clearly mark untested entries.

**Checkpoint 1 — Red contracts and P0 fix.** Reproduce the White discrepancy and an independent No Variant equivalent; locate the first bad event-time inference and fix the smallest proven source. Also investigate clue admission. Commit focused tests + repair with before/after evidence.

**Checkpoint 2 — Levels 1–2.** Systematically test the full relevant source matrix; add meaningful negative cases, fix confirmed bugs in small coherent batches and record exact evidence for each. Preserve all applicable higher-level behaviors.

**Checkpoint 3 — Levels 3–4 and physical integration.** Repeat with Fix/strategy/chop behavior, cross-seat scenarios, physical replay and observer safety. Fix and document confirmed failures. Check no advanced rules leak into lower levels.

**Checkpoint 4 — Certification.** An independent read-only review of assertions, semantic authority, hidden-information safety, diff, and reported coverage; focused regressions; full repository validation using **bash scripts/check.sh** (explicitly authorized for this Goal); PR CI; one bounded rerun of White as a smoke, with action log and no score requirement. Update the audit matrix with actual PASS/BUG/UNSUPPORTED/NOT TESTED statuses and concrete references.

A checkpoint may span multiple **short Codex chats**, following docs/codex-workflow.md. Keep commits and the audit file current. No giant unreviewable one-shot rewrite.

## Acceptance / stopping rules

SUCCESS only if:
- All 48 Level 1–4 documentation sections have traceable, honest disposition and executable behavioral evidence where the section states executable semantics; all required physical and cross-perspective contracts above have explicit test evidence.
- Every reproduced and within-scope **confirmed** basic-rule defect found during this audit has a source-backed minimal repair and demonstrated red/green regression. No open known P0 basic-convention contradiction remains.
- Full No Variant validation, affected higher-level replays, observer safety, and main production gates pass; PR and main CI are checked; independent reviewer agrees the matrix and test authority are credible.
- Real self-play is only smoke; better average score, disappearance of one strike, and a green test count are **not acceptance criteria**.

If any required rule remains UNSUPPORTED/NOT TESTED, a true semantic ambiguity cannot be resolved from authority, a repair requires substantial architectural rewrite, or regressions remain red: **do not call the Goal complete and do not merge**. Commit coherent partial work, open/update a DRAFT PR with precise remaining rows, risk, source links, failing tests, and the next narrow step.

If acceptance is met, publish a normal PR, obtain independent review and green CI, and only then merge according to the repository workflow. No unreviewed production gate changes.

## Final report

Report: baseline SHA; 48-row coverage table completion; count of tested/untested/unsupported rules; before/after P0 evidence; each confirmed bug with root cause and reproducer; validation commands/results; non-regressions for No Variant; remaining ambiguities and limitations; PR and CI URLs. State separately what is **proven by tests**, what is **observed in pilot games**, and what is **still uncertain**.
