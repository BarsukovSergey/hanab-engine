# H-Group foundation audit: No Variant and cumulative Levels 1–4

Status: **PAUSED at the user's request** (2026-10-24), checkpoint 1 incomplete.
No automatic continuation is authorized. Full 48-section certification is no
longer the immediate objective; future work requires separately bounded tasks.
This is a coverage map and baseline, not certification. See
[the Goal](HGROUP_FOUNDATION_CORRECTNESS_GOAL.md).

## Revisions and authority

- Main baseline: `51f1784132d367ca2f5a3aeff06bd91821a9a74c`.
- Execution baseline: `50ffd99267939b48d8d6961a8e52dcea02248090`, branch
  `quality-hgroup-foundation-l1-l4`, initially clean. Baseline commands below
  ran at this revision before changes. No comparison with older revisions was
  run.
- Convention authority: official `hanabi/hanabi.github.io`, pinned revision
  `dd55b51aa622f709059a17e0d6afc2adb8402408`, as required by
  [h-group-coverage.md](h-group-coverage.md). Read the pinned
  [L1](https://github.com/hanabi/hanabi.github.io/blob/dd55b51aa622f709059a17e0d6afc2adb8402408/docs/level-1.mdx),
  [L2](https://github.com/hanabi/hanabi.github.io/blob/dd55b51aa622f709059a17e0d6afc2adb8402408/docs/level-2.mdx),
  [L3](https://github.com/hanabi/hanabi.github.io/blob/dd55b51aa622f709059a17e0d6afc2adb8402408/docs/level-3.mdx),
  [L4](https://github.com/hanabi/hanabi.github.io/blob/dd55b51aa622f709059a17e0d6afc2adb8402408/docs/level-4.mdx),
  and L1's linked beginner Good Touch, Save, and Minimum Clue Value sections
  from the sibling checkout. Website URLs in the matrix preserve the exact
  pinned inventory; the web fetcher returned 404 for all four level landing
  pages, so the pinned official source files supply authority. No upstream
  semantic revision is adopted.
- Physical authority: official Hanabi Live
  [rules](https://github.com/Hanabi-Live/hanabi-live/blob/b0792ded044937776b36fc797dc25f060499e4f5/docs/rules.md),
  read from the clean sibling checkout at
  `b0792ded044937776b36fc797dc25f060499e4f5`. Server/game implementation
  comparison is still pending, especially deal order and terminal timing.
- Reviewed positions: [testing.md](testing.md), the expert manifest, and
  [cross-position rulings](replay-reviews/principles.md). The manifest marks
  p4v0s415 fully reviewed, whereas testing.md limits its active continuation to
  36 reviewed actions. This provenance discrepancy is unresolved; use the
  conservative boundary until reconciled, without silently changing either.

## Meaning of coverage

The 48 inventory rows below are **0 PASS, 0 CONFIRMED BUG, 0 UNSUPPORTED, 48 NOT
YET TESTED**. Existing tests run successfully do not by themselves discharge a
row's positive, negative, perspective, and downstream requirements. No rule has
yet been demonstrated unsupported. A failing preferred-action assertion is
recorded separately and is not automatically a confirmed semantic bug. Status
counts refer to whole source rows, not individual assertions.

Each row's examples are source-derived contract targets, **not newly invented
game fixtures or assertions that the current engine is correct**. They must be
anchored to the exact official example or a reviewed replay before becoming
convention regressions. The production column identifies the audit entry point,
not a completed call-path proof. Test IDs refer to existing evidence below;
“missing” requires a new independently justified contract. All levels are
cumulative: `Lk+` means enabled at levels k through 4 within this audit, with
applicable higher-level compatibility retained.

Paths in the production column are relative to
`crates/hanabi-search/src/h_group/`, except `h_group.rs` in its parent.
Perspective boundaries:

- **E**: event-time public stacks, clues, hand membership and visible
  identities; never future reveals or future draws.
- **O**: the deciding owner's hand remains hidden. Own literal/established
  convention knowledge may constrain it; simulator truth may not.
- **V**: another player's event-time visible card cannot be assigned a
  contradicting identity. Giver, intermediates and recipient may lawfully have
  different hypotheses.
- **P**: protection/gotten state is public convention state, not a physical
  identity or permission to play.

## Source matrix

| ID    | Official source section                                                                                                                            | Rule / linked invariant                                                                                         | Enabled                     | Production entry point                                                                       | Positive contract target                                                                     | Negative / limitation target                                                                                                                        | Owner observation boundary                                       | Existing test IDs / gap                                   | Status         | Evidence disposition                                                                              |
| ----- | -------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- | --------------------------- | -------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | --------------------------------------------------------- | -------------- | ------------------------------------------------------------------------------------------------- |
| L1-01 | [The Chop](https://hanabi.github.io/level-1/#the-chop)                                                                                             | Rightmost unclued discard position; known-trash discard does not redefine chop.                                 | L1+                         | `recognition/chop_moves.rs`; `action_schedule.rs`                                            | Retain chop while discarding a clued useless card.                                           | Gotten/protected card must not become an ordinary chop.                                                                                             | E/O/P; distinguish safe discard from chop.                       | T-FC; missing ordinary chop pair                          | NOT YET TESTED | Definition needs executable chop and discard-state invariant.                                     |
| L1-02 | [The Definition of Playable](https://hanabi.github.io/level-1/#the-definition-of-playable)                                                         | Convention-playable includes a delayed chain whose predecessors are already accounted for.                      | L1+                         | `primary.rs::PrimaryClueResolution`; `h_group.rs::schedule_connection`                       | Delayed focus with all predecessors secured.                                                 | Missing predecessor cannot become a promise merely because it exists in the deck.                                                                   | E/O/V at clue time.                                              | T-PRIMARY, T-CONN; missing admission cross-check          | NOT YET TESTED | Physical immediate playability and convention playability require distinct checks.                |
| L1-03 | [The Definition of Trash](https://hanabi.github.io/level-1/#the-definition-of-trash)                                                               | Unplayable forever or duplicated clued identity; convention trash can overlap playable.                         | L1+                         | `interpretation/knowledge.rs::snapshot_good_touch_identities`; `interpretation/knowledge.rs` | Played identity or blocked upper card classified useless.                                    | Unknown duplicate identity cannot be assumed from hidden truth.                                                                                     | E/O/V; identity versus copy.                                     | T-GT; missing overlap case                                | NOT YET TESTED | Must distinguish duplicate convention claims from stack impossibility.                            |
| L1-04 | [Save Clues](https://hanabi.github.io/level-1/#save-clues)                                                                                         | Chop-only 2/5/critical Save; unique playable protection remains a Play Clue.                                    | L1+                         | `primary.rs::PrimaryClueResolution::direct_save`                                             | Critical chop protected by color or rank.                                                    | Ordinary non-chop card is not a Save; arbitrary 3/4 is not saveable.                                                                                | E/O/V; historical discard counts.                                | T-PRIMARY; missing full giver/recipient pair              | NOT YET TESTED | Number saves and playable overlap need exact precedence verification.                             |
| L1-05 | [Clue Focus](https://hanabi.github.io/level-1/#clue-focus)                                                                                         | Retouch-only leftmost; otherwise chop; otherwise leftmost new.                                                  | L1+                         | `recognition.rs`; `history_reducer.rs`                                                       | New touched chop overrides another new card.                                                 | Retouch-only cannot use ordinary new-card focus; account for gotten cards.                                                                          | E/P; same historical hand ordering.                              | T-FC; missing three-way No Variant contract               | NOT YET TESTED | Existing context tests do not certify every focus branch.                                         |
| L1-06 | [Good Touch Principle](https://hanabi.github.io/level-1/#good-touch-principle)                                                                     | Touch useful eventual plays; exclude already played and duplicate secured copies, subject to source exceptions. | L1+                         | `interpretation/knowledge.rs::snapshot_good_touch_identities`; `principle_validation.rs`     | Ancillary useful future card retains non-immediate possibilities.                            | Reject bad touch except justified Save/special-move exception.                                                                                      | E/O/V; giver's known own duplicates count, unknown faces do not. | T-GT; missing Save exception chain                        | NOT YET TESTED | Linked beginner GTP read; admission, replay and claims must agree.                                |
| L1-07 | [Save Principle](https://hanabi.github.io/level-1/#save-principle)                                                                                 | Protect 5, unique 2, critical and unique playable; can override GTP to prevent loss.                            | L1+                         | `chop_safety.rs`; `principle_validation.rs`                                                  | Historical funded protection opportunity constrains discard risk.                            | Occupied recipient or absent opportunity cannot prove safe chop.                                                                                    | E/O/V; preserve old evidence after token exhaustion.             | T-SAVE; missing L1–4 gate and GTP override pair           | NOT YET TESTED | Reviewed principles exist; not rerun at checkpoint 0.                                             |
| L1-08 | [Minimum Clue Value Principle](https://hanabi.github.io/level-1/#minimum-clue-value-principle)                                                     | Obtain at least one previously ungotten useful card; account for documented exceptions.                         | L1+                         | `principle_validation.rs`; `candidate_pipeline.rs`                                           | New indirect connection contributes real acquisition.                                        | Re-identifying secured work alone is not new value; preserve forced stalls/Fixes.                                                                   | E/O/V/P; count knowledge before clue.                            | T-MCV; missing low-level exception table                  | NOT YET TESTED | Reviewed admission ruling forbids repair by score penalty.                                        |
| L1-09 | [The Early Game](https://hanabi.github.io/level-1/#the-early-game)                                                                                 | Before first chop discard, exhaust available Play/Save Clues, excluding ordinary Tempo.                         | L1+                         | `history_reducer.rs`; `decision.rs`                                                          | Available valid Play Clue prevents premature chop discard.                                   | Do not count illegal/unfunded clues or require L9-only stall rules.                                                                                 | E/O; actual action and chop at that time.                        | T-OCM red; missing focused phase pair                     | NOT YET TESTED | Baseline failure compares two clues, not clue versus discard; do not overinterpret it.            |
| L1-10 | [The 5 Save](https://hanabi.github.io/level-1/#the-5-save)                                                                                         | First rank-5 touch on chop is Save.                                                                             | L1+                         | `primary.rs::PrimaryClueResolution::direct_save`                                             | Rank-5 on untouched chop protects it.                                                        | Color touching a 5 is not classified 5 Save.                                                                                                        | E/O/V; pre-clue chop.                                            | T-PRIMARY; missing historical boundary                    | NOT YET TESTED | Separate critical/color and later stall interpretations.                                          |
| L1-11 | [The 2 Save](https://hanabi.github.io/level-1/#the-2-save)                                                                                         | Rank-2 chop Save obeys visible-copy rule and double-chop exception.                                             | L1+                         | `primary.rs`; `turn_context.rs`                                                              | Either duplicate on double chops may be saved.                                               | Visible other copy off chop forbids ordinary 2 Save; color only critical Save.                                                                      | E/O/V; visibility is relative to historical giver.               | T-PRIMARY, T-HIGH; missing double-chop/early obligation   | NOT YET TESTED | Four-player and own-hidden-copy controls required.                                                |
| L1-12 | [The Prompt](https://hanabi.github.io/level-1/#the-prompt)                                                                                         | Previously clued unknown connector; leftmost eligible; not an already due play.                                 | L1+                         | `h_group.rs::schedule_connection`; `action_schedule.rs`                                      | Unknown compatible clued card becomes due connector.                                         | Already-playing card is not a Prompt; impossible visible card cannot connect.                                                                       | E/O/V across all actors.                                         | T-CONN, T-P0, T-PROMPT; full pipeline pending             | NOT YET TESTED | Ordering, prior knowledge and successful alternatives covered; complete pipeline remains open.    |
| L1-13 | [The Finesse](https://hanabi.github.io/level-1/#the-finesse)                                                                                       | Blind connecting play demonstrates the promise promptly.                                                        | L1+                         | `h_group.rs::schedule_connection`; `action_schedule.rs`                                      | Own hidden finesse position may carry the required predecessor.                              | Different player's visibly wrong identity cannot be the predecessor.                                                                                | E/O/V; indistinguishable hidden worlds agree.                    | T-CONN, T-HIGH, T-P0; complete actor matrix pending       | NOT YET TESTED | C1 red/green repair, own-hidden control and one conditional due response; full rule remains open. |
| L1-14 | [Finesse Position](https://hanabi.github.io/level-1/#finesse-position)                                                                             | Leftmost unclued position at clue time.                                                                         | L1+                         | `h_group.rs::schedule_connection`; `history_reducer.rs`                                      | Original connector follows its card ID after drawing shifts slots.                           | New draw cannot replace old promised card; L5 queued rules stay gated.                                                                              | E/O/P; original hand membership.                                 | T-CONN; missing L1–4 versus L5 controls                   | NOT YET TESTED | Position and due-time tests needed beyond identity matching.                                      |
| L1-15 | [Finessed Cards](https://hanabi.github.io/level-1/#finessed-cards)                                                                                 | Finessed cards are already gotten for later focus.                                                              | L1+                         | `facts.rs`; `recognition.rs`                                                                 | Touching gotten finesse and a new card focuses the new card.                                 | Invisible clue does not become literal physical color/rank knowledge.                                                                               | E/O/P; promise lifetime.                                         | T-CONN, T-FC; missing focus lifecycle pair                | NOT YET TESTED | Protection, literal facts and connection promises must remain distinct.                           |
| L1-16 | [Prompts > Finesses](https://hanabi.github.io/level-1/#prompts--finesses)                                                                          | Compatible Prompt precedes blind Finesse.                                                                       | L1+                         | `h_group.rs::schedule_connection`; `action_schedule.rs`                                      | Later-seat valid Prompt precedes earlier-seat Finesse.                                       | Chop-moved-only card is not promptable; visibly incompatible Prompt cannot win.                                                                     | E/O/V/P; precedence uses actor knowledge.                        | T-CONN, T-CM-PROMPT                                       | NOT YET TESTED | Need all actors and candidate admission, not only selected connector.                             |
| L1-17 | [Card Notes](https://hanabi.github.io/level-1/#card-notes)                                                                                         | Explanatory UI section; linked invariant is persistent causal card knowledge.                                   | L1+                         | `facts.rs`; `history_reducer.rs`                                                             | Promise/protection survives replay by card ID.                                               | Presentation note must not reveal a hidden face or create a new fact.                                                                               | E/O/P.                                                           | T-CONN; missing notes-to-invariant certification          | NOT YET TESTED | UI shortcuts are not an engine transition; persistence still needs evidence.                      |
| L1-18 | [Rewind](https://hanabi.github.io/level-1/#rewind)                                                                                                 | Explanatory UI; linked invariant is correct historical reconstruction.                                          | L1+                         | `history_reducer.rs`; `turn_context.rs`                                                      | Reconstruct prefix using then-visible facts.                                                 | Later reveal/draw cannot rewrite earlier knowledge.                                                                                                 | E/O/V.                                                           | T-DATED, T-CONN; missing full replay equivalence          | NOT YET TESTED | One dated lookup test passed; full inference boundary remains unaudited.                          |
| L1-19 | [Empathy](https://hanabi.github.io/level-1/#empathy)                                                                                               | Explanatory UI; linked invariant is observer-correct redaction and reasoning.                                   | L1+                         | `perspective.rs`; `prospective.rs`                                                           | Re-rooted recipient view hides recipient's cards.                                            | Giver-visible truth is not recipient knowledge.                                                                                                     | E/O/V; future draws hidden in projections.                       | T-BOUNDARY, T-OBS; missing all-seat same-history contract | NOT YET TESTED | Existing architecture oracle is relevant but not rerun here.                                      |
| L2-01 | [The 5 Stall (Cluing Off Chop 5's)](https://hanabi.github.io/level-2/#the-5-stall-cluing-off-chop-5s)                                              | Early/forced off-chop rank-5 stall; precedes 5CM.                                                               | L2+                         | `recognition.rs`; `candidate_pipeline.rs`                                                    | Early off-chop 5 is protected without a play promise.                                        | Ordinary mid-game off-chop 5 is not automatically a stall; preserve useful clues.                                                                   | E/O/P; historical early/forced state.                            | T-FC; missing L1-disabled and forced cases                | NOT YET TESTED | Do not import L9 severity rules wholesale.                                                        |
| L2-02 | [The Double Prompt / Triple Prompt / Quadruple Prompt](https://hanabi.github.io/level-2/#the-double-prompt--triple-prompt--quadruple-prompt)       | Multiple clued predecessors, left-to-right and in stack order.                                                  | L2+                         | `h_group.rs::schedule_connection`; `history_reducer.rs`                                      | Official r4 chain after r1 prompts r2 then r3.                                               | Second step not due before first resolves; failed first cannot persist blindly.                                                                     | E/O/V; each actor's successive turns.                            | T-CONN; missing triple/quadruple and failed-step cases    | NOT YET TESTED | Ordering helper coverage is not lifecycle certification.                                          |
| L2-03 | [The Double Finesse / Triple Finesse / Quadruple Finesse](https://hanabi.github.io/level-2/#the-double-finesse--triple-finesse--quadruple-finesse) | Multiple blind connectors across players or turns; recipient waits for visible continuation.                    | L2+                         | `h_group.rs::schedule_connection`; `action_schedule.rs`                                      | Four-seat r2/r3 chain into r4, and repeated actor variant.                                   | Do not assume deferred second step played; no L5 ambiguity requirement.                                                                             | E/O/V; 3-/4-player owners and recipient.                         | T-CONN, T-HIGH; missing full L2 actor matrix              | NOT YET TESTED | Deferral/disambiguation need dated action evidence.                                               |
| L2-04 | [The Prompt + Finesse](https://hanabi.github.io/level-2/#the-prompt--finesse)                                                                      | Clued predecessor first, original finesse position next turn.                                                   | L2+                         | `h_group.rs::schedule_connection`; `history_reducer.rs`                                      | r2 Prompt then r3 blind response into r4.                                                    | New draw is not the original finesse position; recipient must allow continuation.                                                                   | E/O/V; original card ID.                                         | T-CONN; missing complete official-example branch          | NOT YET TESTED | Source explicitly requires event-time visibility of the second connector.                         |
| L2-05 | [The Reverse Finesse](https://hanabi.github.io/level-2/#the-reverse-finesse)                                                                       | Recipient defers to later visible connector; nonresponse allows direct interpretation.                          | L2+                         | `h_group.rs::schedule_connection`; `history_reducer.rs`                                      | Bob waits for Cathy's r1 before playing clued r2.                                            | If Cathy does unrelated action, Bob may instead hold r1; do not fabricate due play.                                                                 | E/O/V at clue and response turns.                                | T-CONN; missing decline-to-direct contract                | NOT YET TESTED | Non-legacy identity parity is partial evidence only.                                              |
| L2-06 | [The Self-Finesse](https://hanabi.github.io/level-2/#the-self-finesse)                                                                             | Recipient supplies hidden connector only after ordinary alternatives excluded.                                  | L2+                         | `h_group.rs::schedule_connection`; `interpretation.rs::infer_clue_to_self`                   | No direct 3 or visible predecessor after intermediary declines.                              | Direct Play, Prompt and Reverse Finesse alternatives precede Self-Finesse.                                                                          | E/O/V; own truth ignored.                                        | T-CONN; missing all alternative/decline timings           | NOT YET TESTED | Existing hidden-truth negative is relevant; no complete row claim.                                |
| L2-07 | [Trash](https://hanabi.github.io/level-2/#trash)                                                                                                   | Includes higher ranks behind exhausted lower copies.                                                            | L2+                         | `interpretation/knowledge.rs::snapshot_good_touch_identities`; `interpretation/knowledge.rs` | Both r3 discarded makes r4/r5 useless.                                                       | One remaining unseen r3 copy does not make suit dead.                                                                                               | E/O; discard counts public, deck faces hidden.                   | T-GT; missing L1/L2 contextual distinction                | NOT YET TESTED | Shared definition with L1-03 still needs exact positive/negative mapping.                         |
| L2-08 | [One-Away-From-Playable Cards](https://hanabi.github.io/level-2/#one-away-from-playable-cards)                                                     | One missing new clue/connection, not one numerical rank; insufficient for direct Play/Save absent exception.    | L2+                         | `primary.rs`; `interpretation.rs`                                                            | Saved r2 and unaccounted r1 make r3 one-away.                                                | Do not admit direct play promise merely because only one predecessor is absent.                                                                     | E/O/V; already secured versus ungotten.                          | T-PRIMARY; missing source-example admission pair          | NOT YET TESTED | Category feeds L4 protection; cannot be inferred from rank distance alone.                        |
| L2-09 | [Stomping on a Finesse](https://hanabi.github.io/level-2/#stomping-on-a-finesse)                                                                   | Directly cluing an already blind-playing card wastes a clue.                                                    | L2+                         | `interpretation.rs::stomps_unresolved_visible_prefix`; `principle_validation.rs`             | Pending finesse remains executable without redundant clue.                                   | Distinguish safety Fix and new useful acquisition from waste.                                                                                       | E/O/P; established promise before candidate.                     | T-MCV; missing focused L2 stomp pair                      | NOT YET TESTED | Definition links to MCVP and lifecycle, not a blanket retouch ban.                                |
| L2-10 | [What to Do After a Strike](https://hanabi.github.io/level-2/#what-to-do-after-a-strike)                                                           | Retract inference since mistaken promise; retain explicit clue facts.                                           | L2+; source says all levels | `history_reducer.rs`; `facts.rs`                                                             | Failed connector cancels unsafe downstream promised play.                                    | Do not erase literal clues, earlier independent facts or every unrelated chain.                                                                     | E/O/V; public failure and causal origin.                         | T-CONN; missing strike-reset scope contract               | NOT YET TESTED | Verify all-level behavior rather than assuming L2 gate.                                           |
| L2-11 | [The Wrong Prompt](https://hanabi.github.io/level-2/#the-wrong-prompt)                                                                             | Specific failed-Prompt exception can preserve original Finesse intent.                                          | L2+                         | `history_reducer.rs`; `h_group.rs::schedule_connection`                                      | Official clued r4 misplays as r2; original finesse r2 remains intended.                      | Generic strike does not authorize a blind next play; shifted slot follows old ID.                                                                   | E/O/V at original clue and failed Prompt.                        | Missing dedicated reviewed/source contract                | NOT YET TESTED | Must coexist with L2-10 without disabling strike reset.                                           |
| L3-01 | [Playing Multiple 1's](https://hanabi.github.io/level-3/#playing-multiple-1s)                                                                      | Starting 1s oldest first; fresh first; chop focus overrides; focus follows play order.                          | L3+                         | `action_schedule.rs`; `recognition/order_chop.rs`                                            | Official mixed group order 5,1,3,2.                                                          | Fresh rule cannot override chop focus; non-1s do not inherit this ordering.                                                                         | E/O/P; draw history and positive clues.                          | T-OCM; missing dedicated ordering/gate table              | NOT YET TESTED | OCM baseline failure does not certify this ordering.                                              |
| L3-02 | [The Fix Clue](https://hanabi.github.io/level-3/#the-fix-clue)                                                                                     | Correct violated GTP and unsafe inferred identity before misplay.                                               | L3+                         | `interpretation.rs::fix_condition_is_live`; `history_reducer.rs`                             | Fill a falsely promised p5 to literal p3 and stop play.                                      | Cancelled prior belief alone cannot make unrelated new clue a Fix.                                                                                  | E/O/V; actual live prior promise.                                | T-FIX; missing L3-only positive/negative chain            | NOT YET TESTED | Existing reviewed regression candidate, not yet rerun.                                            |
| L3-03 | [The Fix Clue (That Touches Multiple Cards)](https://hanabi.github.io/level-3/#the-fix-clue-that-touches-multiple-cards)                           | Fix does not simultaneously promise ancillary card playability.                                                 | L3+                         | `primary.rs`; `history_reducer.rs`                                                           | Fixed blue1 plus new blue card leaves new card non-immediate.                                | Ordinary focus cannot create b2 promise on ancillary new card.                                                                                      | E/O/V; keep literal touching facts.                              | T-FIX; missing ancillary-card regression                  | NOT YET TESTED | Need all recipients' domains, not just Fix label.                                                 |
| L3-04 | [The Fix Clue (That Gives No Additional Information)](https://hanabi.github.io/level-3/#the-fix-clue-that-gives-no-additional-information)         | Repeated stop clue fixes only the next intended card.                                                           | L3+                         | `interpretation.rs`; `history_reducer.rs`                                                    | Re-clue remaining two 1s: stop next one, retain other play.                                  | Do not mark every retouched card bad; respect later-level ordinary Tempo alternatives.                                                              | E/O/P; next scheduled play before clue.                          | Missing focused stop-clue pair                            | NOT YET TESTED | Literal no-op can be convention information; requires source-backed fixture.                      |
| L3-05 | [The Sarcastic Discard (SD)](https://hanabi.github.io/level-3/#the-sarcastic-discard-sd)                                                           | Knowing duplicate discard transfers identity; multiple targets remain ambiguous.                                | L3+                         | `history_reducer.rs`; `recognition.rs`                                                       | Clued useful r3 discarded identifies recipient's unique matching 3.                          | Two matching 3s do not imply leftmost; actor must know duplicate, not see own face.                                                                 | E/O/V; actor knowledge before discard.                           | T-FC; missing ambiguity/lifecycle pair                    | NOT YET TESTED | Distinguish from physical trash discard and Prompt ordering.                                      |
| L3-06 | [Misplay Cost Principle](https://hanabi.github.io/level-3/#misplay-cost-principle)                                                                 | One clue to prevent strike generally worthwhile; two clues is conditional strategy.                             | L3+                         | `decision.rs`; `principle_validation.rs`                                                     | Funded one-clue Fix prevents known strike.                                                   | Do not spend arbitrary clues under a blanket rule; third-strike loss changes risk.                                                                  | E/O/V; known danger rather than hidden face.                     | Missing evidence-linked comparison contract               | NOT YET TESTED | Strategy guidance; test accounting/legality, not invented universal optimal move.                 |
| L3-07 | [Efficiency](https://hanabi.github.io/level-3/#efficiency)                                                                                         | Count new eventual useful work; separate clarification from new acquisition.                                    | L3+                         | `information_value.rs`; `principle_validation.rs`; `decision.rs`                             | New r1 and later r5 can count two even if only r1 plays now.                                 | Already-gotten connector is not obtained again.                                                                                                     | E/O/V/P; pre-clue acquisition state.                             | T-MCV; missing L3-only downstream accounting              | NOT YET TESTED | Reviewed principle exists; subjective optimization not a semantic oracle.                         |
| L3-08 | [Tempo](https://hanabi.github.io/level-3/#tempo)                                                                                                   | Explanatory strategy; link progress accounting to actual remaining turns and efficiency.                        | L3+                         | `decision.rs`; `symbolic_line.rs`                                                            | Realized play progress retained in equal-time comparison.                                    | Undefined continuation/hidden draw cannot count as guaranteed score.                                                                                | E/O/V; terminal horizon and funded steps.                        | Missing explicit low-level accounting invariant           | NOT YET TESTED | No universal score weights or preferred move implied by the paragraph.                            |
| L3-09 | [Information Lock Principle](https://hanabi.github.io/level-3/#information-lock-principle)                                                         | Focus and in-between domains lock at clue time; direct contradiction can break lock.                            | L3+                         | `facts.rs`; `interpretation/knowledge.rs`; `history_reducer.rs`                              | r2/r3 intermediate promises remain locked through later actions.                             | Future evidence cannot silently add identities; literal contradictory clue must retract false lock.                                                 | E/O/V; provenance and causal cancellation.                       | T-CONN, T-OBS; missing lock-break cross-observer contract | NOT YET TESTED | Strong connection to P0 event-time reconstruction.                                                |
| L4-01 | [Chop Moves](https://hanabi.github.io/level-4/#chop-moves)                                                                                         | Persistent protection moves chop, without identifying card.                                                     | L4+                         | `recognition/chop_moves.rs::apply_chop_move_effects`; `facts.rs`                             | Protected old chop retained across draw and turns.                                           | Protection alone cannot make card playable/known.                                                                                                   | E/O/P; actual trash can be protected if unknown.                 | T-FC; missing No Variant persistent-state pair            | NOT YET TESTED | Actor-known trash and simulator-only trash must be distinguished.                                 |
| L4-02 | [The Trash Chop Move (TCM)](https://hanabi.github.io/level-4/#the-trash-chop-move-tcm)                                                             | Known-trash clue moves cards right of rightmost trash; direct Save preferred except documented necessity.       | L4+                         | `recognition/chop_moves.rs`; `interpretation.rs`                                             | Multi-trash clue protects only cards after rightmost touched trash.                          | No protection left of focus; avoid knowingly protecting trash or replacing available direct Save without exception.                                 | E/O/V/P; prove trash from recipient knowledge.                   | T-FC; missing direct-Save exception/admission pair        | NOT YET TESTED | Existing custom-variant invariant cannot certify all No Variant prerequisites.                    |
| L4-03 | [The 5's Chop Move (5CM)](https://hanabi.github.io/level-4/#the-5s-chop-move-5cm)                                                                  | Off-chop rank5 exactly one unclued card away protects one card; Stall precedes.                                 | L4+                         | `recognition/chop_moves.rs`; `primary.rs`                                                    | Rightmost touched 5 one-away after early game moves chop.                                    | Two-away is not 5CM; ignore already-clued cards when counting; no early ordinary 5CM.                                                               | E/O/P; historical chop and stall context.                        | T-FC, T-PRIMARY; missing counting/full replay controls    | NOT YET TESTED | Need useful Play versus Stall versus 5CM precedence audit.                                        |
| L4-04 | [The Order Chop Move (OCM)](https://hanabi.github.io/level-4/#the-order-chop-move-ocm)                                                             | Skipped ordinary unknown-1 order selects seat-distance protection; reset normal order next turn.                | L4+                         | `recognition/order_chop.rs::apply_order_chop_move_effects`                                   | Skip two 1s moves player two seats away.                                                     | Known identity, unequal positive clues, prior finesse or known-bad skipped 1 excludes OCM; non-1 never qualifies.                                   | E/O/P; negative clue differences alone ignored.                  | T-OCM red; missing exception/reset/gate matrix            | NOT YET TESTED | Initial protection assertions pass before unrelated action-choice failure.                        |
| L4-05 | [Chop Moves as a Category](https://hanabi.github.io/level-4/#chop-moves-as-a-category)                                                             | Explanatory classification; protection refers to another card, not Save on touched focus.                       | L4+                         | `primary.rs`; `model.rs`; `candidate_pipeline.rs`                                            | TCM effect targets untouched protected card.                                                 | Classification cannot claim touched trash was saved/playable.                                                                                       | E/O/P.                                                           | T-FC; missing category/effect accounting invariant        | NOT YET TESTED | Linked to target/protection and acquisition invariants, not standalone strategy.                  |
| L4-06 | [Chop Moves & New Clues](https://hanabi.github.io/level-4/#chop-moves--new-clues)                                                                  | Protected counts old; first literal touch on only-old cards has leftmost protected exception.                   | L4+                         | `recognition.rs`; `history_reducer.rs`                                                       | New plus protected focus selects new; only-old selects first touched protected as specified. | Ordinary leftmost retouch must not override documented exception.                                                                                   | E/O/P; distinguish physical clue from protection.                | T-FC; missing exact No Variant exception pair             | NOT YET TESTED | Source's rainbow example must be adapted only while preserving prerequisites.                     |
| L4-07 | [Chop Moves & Prompts](https://hanabi.github.io/level-4/#chop-moves--prompts)                                                                      | Protection-only cards are not Prompt targets.                                                                   | L4+                         | `h_group.rs::schedule_connection`; `action_schedule.rs`                                      | Blind finesse chosen over protected-only chop card.                                          | Invisible protection is not matching positive clue; genuine literal Prompt still precedes.                                                          | E/O/V/P.                                                         | T-CM-PROMPT, T-CONN                                       | NOT YET TESTED | Reviewed negative exists; baseline run pending.                                                   |
| L4-08 | [Critical Discards after a Chop Move (Mistake)](https://hanabi.github.io/level-4/#critical-discards-after-a-chop-move-mistake)                     | A critical card on the next discard after CM signals mistaken CM and undo.                                      | L4+                         | `history_reducer.rs`; `facts.rs`                                                             | The next discard is critical and retracts the implicated CM.                                 | Noncritical next discard cannot trigger undo; a later critical discard alone does not satisfy this timing condition; preserve unrelated protection. | E/O/P; criticality at relevant event.                            | Missing causal undo regression                            | NOT YET TESTED | Requires exact causal and timing proof; no blanket rollback.                                      |
| L4-09 | [Accidental Chop Moves (Mistake)](https://hanabi.github.io/level-4/#accidental-chop-moves-mistake)                                                 | Keep mistaken CM unless critical-discard exception applies; team relied on protection.                          | L4+                         | `history_reducer.rs`; `facts.rs`; `chop_safety.rs`                                           | Later realization alone keeps protection.                                                    | Do not re-expose now-critical card just because original CM was unnecessary.                                                                        | E/O/V/P; later losses affect others' assumptions.                | Missing persistence versus critical-undo pair             | NOT YET TESTED | Explanation links to persistent protection and historical Save opportunities.                     |

## Executable evidence register

Test names below are existing entry points, not new authority. B0–B4 denote
actual baseline executions; other entries remain candidates for later focused
checks. No existing test/assertion/fixture has been modified.

| ID          | Existing test(s), relative to `h_group::tests` unless qualified                                                                                                                                                                                                                                                                                                                                                                                                           | Observed baseline / limitation                                                                                                                                                                                  |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| T-GT        | `basic_semantics::contract_basic_good_touch_no_variant_matches_legacy_usefulness_and_claims`; `basic_semantics::contract_basic_good_touch_cannot_depend_on_hidden_simulator_truth`; `basic_semantics::contract_basic_good_touch_black_uses_one_copy_and_requires_each_missing_lower_identity`                                                                                                                                                                             | Passed in B1; helper invariants, not complete convention authority.                                                                                                                                             |
| T-CONN      | `generic_connections::contract_generic_connection_no_variant_and_sixth_suit_prompt_finesse_parity`; `contract_generic_connection_prompt_precedes_earlier_finesse_in_six_player_order`; `contract_generic_connection_multi_step_rank_order_preserves_lifecycle_and_provenance`; `contract_generic_connection_deferred_prompt_and_fix_keep_promise_history`; `contract_self_finesse_requires_no_direct_alternative_and_ignores_hidden_truth` (all in `generic_connections`) | Passed in B1; inspect oracle and observer scope per row before promotion.                                                                                                                                       |
| T-FC        | `generic_focus_context::contract_v3_focus_context_five_stall_precedes_five_chop_move`; `contract_v3_focus_context_custom_trash_chop_is_protection_and_hidden_truth_independent`; `contract_v3_focus_context_sarcastic_precedence_requires_actor_knowledge` (all in `generic_focus_context`)                                                                                                                                                                               | Passed in B1; variant migration contracts, not exhaustive foundation coverage.                                                                                                                                  |
| T-HIGH      | `v4_advanced_connections::contract_complete_double_finesse_precedes_apparent_three_bluff`; `contract_v5_replay_save_preserves_historical_giver_relative_visibility` (both in `v4_advanced_connections`)                                                                                                                                                                                                                                                                   | Passed in B1; higher-level compatibility witnesses.                                                                                                                                                             |
| T-PRIMARY   | `h_group::primary::tests::direct_save_no_variant_base_rules_and_precedence`; `direct_play_excludes_touched_nonplayable_and_untouched_playable_identities`; `five_chop_move_suppresses_a_hypothetical_play_reading` (all in `h_group::primary::tests`)                                                                                                                                                                                                                     | Not run in this checkpoint.                                                                                                                                                                                     |
| T-DATED     | `h_group::turn_context::tests::future_reveals_and_draws_do_not_leak_into_an_earlier_turn`; `generic_connections::contract_generic_connection_historical_dynamic_stacks_exclude_future_and_failed_plays`                                                                                                                                                                                                                                                                   | Passed in B4 and B1 respectively; does not prove full scheduler visibility.                                                                                                                                     |
| T-OBS       | `architecture::observer_epistemic_state_is_invariant_to_leaked_own_hand_truth`; `architecture::every_replay_clue_uses_the_same_hypothetical_and_recipient_interpretation`; `architecture::every_expert_replay_prefix_satisfies_state_and_knowledge_invariants`                                                                                                                                                                                                            | Not run in this checkpoint; all-prefix check is broader than current baseline.                                                                                                                                  |
| T-BOUNDARY  | Integration binaries `generic_information_set`, `information_set`, `baseline`                                                                                                                                                                                                                                                                                                                                                                                             | B2 passed; physical/logical boundary, not H-Group observer inference.                                                                                                                                           |
| T-SAVE      | `replay::save_principle_does_not_infer_safe_chop_for_an_occupied_player`; `replay::first_seed_chop_safety_survives_later_token_exhaustion`                                                                                                                                                                                                                                                                                                                                | Not run; reviewed context in principles ledger.                                                                                                                                                                 |
| T-MCV       | `replay::reviewed_green_bluff_requires_minimum_clue_value`; `h_group::strategic_value::tests::reviewed_turn_nineteen_counts_new_cards_not_old_connectors`                                                                                                                                                                                                                                                                                                                 | Not run; use recorded review boundary, not invented score oracle.                                                                                                                                               |
| T-FIX       | `reviewed_cancelled_identity_cannot_turn_a_fresh_clue_into_a_fix`                                                                                                                                                                                                                                                                                                                                                                                                         | Not run; further multi-card and stop-clue coverage missing.                                                                                                                                                     |
| T-OCM       | `second_replay_order_chop_move_protects_alices_red_two`                                                                                                                                                                                                                                                                                                                                                                                                                   | **B3 failed** at `tests/learning_rules.rs:145`, preferred-action assertion. Assertions for Alice's card0 protection, signal, rule and mutation passed before this point; later Bob assertions were not reached. |
| T-CM-PROMPT | `replay::fourth_replay_two_save_does_not_invent_a_prompt_on_a_chop_moved_card`                                                                                                                                                                                                                                                                                                                                                                                            | Not run; reviewed position candidate for L4-07.                                                                                                                                                                 |

T-PROMPT denotes the three new
`generic_connections::contract_foundation_prompt_*` contracts described in the
source-backed Prompt continuation below. They cover algorithm ordering,
exclusion, historical facts and successful alternatives; they do not yet
discharge the complete admission/executor/recipient contract.

T-P0 denotes the five `contract_foundation_` tests listed in the checkpoint1
section. C1-scoped runs all five. These are partial behavioral evidence for rows
referencing T-P0, not whole-section PASS.

## Baseline commands and results

Run each Cargo command through
`bash scripts/run-python.sh scripts/workflow.py run --label LABEL -- COMMAND`.
Git Bash was invoked explicitly as `C:/Program Files/Git/bin/bash.exe` because
`bash` is absent from this PowerShell PATH. Rust compiled/executed successfully;
no host-toolchain workaround was needed. Durable local records are under ignored
`target/workflow/`, with additional logs in `target/foundation-audit/`.

| Run | Command after recorder separator                                                                             | Result                                                                                                   | Recorder ID                        |
| --- | ------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| B0  | `cargo test -p hanabi-core --locked`                                                                         | PASS: 6 unit + 9 physical_actions + 15 standard_game + 11 variant_catalog = 41 tests; 0 doc tests; 9.35s | `cce93f9d5ed04a4eb5db4c1ff646a83a` |
| B1  | `cargo test -p hanabi-search --locked contract_ --lib`                                                       | PASS: 66, 474 filtered out; 0.32s                                                                        | `ba71900312104a8bb8d5901f23bd3531` |
| B2  | `cargo test -p hanabi-search --locked --test generic_information_set --test information_set --test baseline` | PASS: 16 + 2 + 5 = 23 tests; 13.63s                                                                      | `c5e95010957a43f6a1bd8bade2de0a82` |
| B3  | `cargo test -p hanabi-search --locked second_replay_order_chop_move_protects_alices_red_two --lib`           | FAIL: 0 passed, 1 failed, 539 filtered out; exit101, 0.38s                                               | `16df8acb6338434585819caa994414fb` |
| B4  | `cargo test -p hanabi-search --locked future_reveals_and_draws_do_not_leak_into_an_earlier_turn --lib`       | PASS: 1, 539 filtered out; 0.14s                                                                         | `2d796c1619054c9293674c03b5035938` |

B3 exact discrepancy: at engine turn4 / live turn5 of `game-p4v0s9`, actual
`Clue { target: PlayerId(1), clue: Rank(2) }`, expected
`Clue { target: PlayerId(2), clue: Suit(4) }` (Purple to Cathy). Failure text:
“Alice must extinguish the available Play Clue before ending the Early Game”.
Both compared actions are clues. No root cause or correct new preferred action
is inferred from this failure. Preserve it for a separate bounded diagnosis
after P0. No historical-base archaeology was performed.

Review lookup `scripts/replay_reviews.py p4v0s9 --turn 5` matched prefix key
`06779f0a9eef24ffc3e96e055dba65017aa41afea8a27164809900bd7b45b665` and
unresolved record `t5-five-pull-must-verify-its-response`. That record concerns
a different alternative (5s/Five Pull), and supplies no human ruling that rank2
versus Purple is optimal. Keep its owner-response admission warning without
promoting the record to approval of the current competing actions.

An initial B3 command incorrectly used the nonexistent module path
`h_group::tests::learning_rules::...` with `--exact`; it executed **zero tests**
(recorder `a6c5facf445340088bb80e5b878ac93c`) and provides no pass evidence. The
corrected command above actually exercised the test. There was also a failed ad
hoc Python metadata inspection due to PowerShell quoting; metadata was
subsequently read with `ConvertFrom-Json` and no source data was changed.

Observed main CI for the exact main baseline:
[CI run 40002512753](https://github.com/BarsukovSergey/hanab-engine/actions/runs/40002512753)
reported completed/success. This targeted CI is **not** full local validation
and does not cover the failing ordinary B3 test. Full `bash scripts/check.sh`,
full reviewed replay regressions, exhaustive profile workflow and PR CI have not
yet been run for this audit; certification requires them as specified in the
Goal. Focused records above are not a full-suite failure baseline accepted by
`workflow.py baseline`.

## Physical and cross-perspective checklist

All items remain **NOT YET TESTED as complete audit requirements**. B0/B2
establish existing executable baseline evidence only. Compare production code,
official rules and independent assertions before marking any item PASS.

| ID  | Required contract                                                                                                  | Current evidence                                                                                        | Missing work                                                                                                                                                    |
| --- | ------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A1  | 50-card No Variant deck, 3/2/2/2/1 copies per suit; 2–6-player hand sizes and exact deal order                     | B0 `validates_setup_and_deals_standard_hand_sizes`, `variant_catalog::no_variant`                       | Sizes are currently checked against `standard_hand_size`, the production helper; independently assert official 5/5/4/4/3 counts and server deal order.          |
| A2  | Draw, turn order, ID/copy conservation across legal games                                                          | B0 `many_random_legal_games_preserve_invariants`, `cloned_states_replay_deterministically`              | Existing random legal-game loop covers 2–5 players (`2 + seed % 4`); add deterministic 6-player coverage and independent copy/deal checks.                      |
| A3  | Exact legal clue touch; positive and negative information; zero-clue/self/empty-touch rejection without mutation   | B0 physical legality and B2 `generic_clues_filter_positive_and_negative_touches_without_conventions`    | Explicit official-rule mapping and 2/3/4/6-player negative table.                                                                                               |
| A4  | Token spend, capped discard/5 gains; discard prohibited at8; misplay gives no token                                | B0 token/stack/illegal-action tests                                                                     | Inspect assertions independently; include capped5 reward, zero tokens and exhausted-copy cases.                                                                 |
| A5  | Ordered/duplicate/missing-predecessor plays, strikes, third-strike score0 and no draw                              | B0 `misplay_adds_a_strike_and_publicly_reveals_the_card`, `third_misplay_ends_the_game_without_drawing` | Independent 2/3/4/6 cases and exact official final score versus stack score.                                                                                    |
| A6  | Final round includes last drawer; perfect-score termination; no terminal actions                                   | B0 `final_round_grants_every_player_one_more_action`, physical terminal tests                           | Official implementation timing comparison; last-card and exhausted-initial-deal boundaries.                                                                     |
| A7  | FullState → redacted view → InformationSet → decision → physical action → replay                                   | B0 own-hand/draw hiding; B2 boundary tests                                                              | Deterministic whole-chain equivalence for 2/3/4/6, including no hidden draw leak.                                                                               |
| F1  | All-clue-time observer visibility and indistinguishable hidden worlds                                              | B1 hidden-truth contracts; B4 dated lookup; C1 White three-observer and No Variant 3/4-player controls  | Broader event-time history and indistinguishability contracts, including draws/removals and all actors.                                                         |
| F2  | Correct due obligations, deferral/decline/failure/cancellation and active executor actions                         | B1 connection lifecycle; C1 typed forced Alice response and hypothetical Bob continuation               | General due/non-due, deferral/cancellation and planner execution at varied budgets.                                                                             |
| F3  | Giver admission, prospective validation, recipient replay and actual action agree on proven/conditional/unresolved | B1 context helpers; C1 captured proposal/admission/recipient/owner-response stages                      | Complete soundness contracts beyond captured Pass/GeneratorProof; all necessary intermediate responses.                                                         |
| F4  | Cumulative enablement L1→L4; L5+ rules do not leak; applicable advanced compatibility retained                     | B1 includes advanced contract witnesses                                                                 | Explicit low-level gate matrix, affected reviewed replays, deterministic mandatory-action planner integration at varied budgets.                                |
| F5  | Valid metamorphic invariants                                                                                       | B0 conservation and deterministic replay witnesses                                                      | No Variant suit renaming, compatible seat rotations, copy symmetry, complete replay reconstruction and observer indistinguishability, preserving prerequisites. |

## P0 recovery and next narrow checkpoint

Recovered locally: `target/product-pilot/white-hgroup-5f7585d.json`. SHA256 of
the unmodified file:
`7272f774e6e88c6fb2c809ec33e44e1ebdf6b4cfea6187d5492855460ea9079a`. Its metadata
says source SHA `5f7585da1da6a2b45d6a6551c84eafea304a834f`, 3 players, seed1,
White6Suits, H-Group Level4, 74 decisions. It is a pilot report with
`orderedDeck` and `decisions[].action`, **not** a protocol fixture with an
`actions` property. The matching NDJSON and audit capture are available beside
it. At checkpoint 0 no engine reconstruction or semantic verdict had been made.
The recovery and repair below supersede that limitation; the numbered list
remains the overall checkpoint 1 plan.

Checkpoint 1 must:

1. Convert only recorded setup/deck/action data into a minimal protocol fixture;
   retain source SHA/hash provenance, not diagnostic scores or inferred
   meanings. Parse and replay legality; do not regenerate later choices through
   self-play. Preserve the recorded prefix at least through engine49 / live50.
2. Before analysis, run `scripts/replay_reviews.py SEED --turn N` for its actual
   fixture identity and read matching notes. The script currently only reads
   `crates/hanabi-protocol/tests/fixtures/game-SEED.json`; if the recovered
   fixture goes elsewhere, record the missing lookup explicitly and use the same
   prefix-key mechanism without fabricating a review. Search the ledger by
   cards/conditions too. No White/card39/card40 review was found by the initial
   targeted search. The Goal's report is a bug hypothesis, not blanket
   endorsement of a specific next move.
3. Generate the verified live replay link for live45/46/50, then capture the
   original clue, Alice's event-time view, Bob's view, original hand membership
   and each intermediate obligation. Keep turn numbers zero/one-based explicit.
4. Establish the first contradiction with source L1-12–16, L2-03–06 and L3-09:
   Alice may hypothesize r3 in her own hidden card39; she cannot assign r3 to
   Bob's visibly White2/card40. Do not assume this mandates Alice's play: prove
   the whole clue's convention prerequisites and alternatives separately.
5. Add deterministic red contracts for the recorded observer contradiction,
   equivalent No Variant visible-card rejection and a genuinely hidden valid
   Finesse control, including 3-/4-player perspectives. Assert inference and
   due/action state before planner choice. Record exact failing output/SHA.
6. Trace `h_group.rs::schedule_connection` through dated identity lookup and
   replay, then `interpretation/candidate_validation.rs`,
   `candidate_pipeline.rs` and `principle_validation.rs`. Capture candidate
   purpose, selected action, GeneratorProof and responseSafety at their own
   stages. Fix only the first proven shared cause; assess admission
   independently. No score tuning.
7. Run focused red→green checks, `check-scoped.sh hanabi-search` with the
   relevant focused arguments, affected reviewed positions and applicable
   advanced regressions. Record any red baseline without changing its assertion.
   Commit/push the coherent repair and update only rows actually discharged.

## Checkpoint 1: first visibility contradiction repaired

The source-data-only
[White fixture](../crates/hanabi-search/src/h_group/tests/fixtures/white-seed1-foundation.json)
preserves all 60 ordered cards and exactly the first 50 recorded actions. Its
seed describes the pilot's splitmix64 deck, not Hanabi Live seed generation.
Scores, inferred meanings and generated replacement actions are not authority.

Review lookup `white-seed1-foundation --turn 46` failed because that helper only
reads protocol fixtures; this reproduction lives under search tests, outside the
expert catalog. The targeted ledger search found no White/card39/ card40 ruling.
Authority is the ordinary visibility invariant and pinned beginner Finesse
Red4/Red3 paragraph, not approval of the pilot's strategy.

**C1: confirmed visibility defect, repaired.** The non-SpecialFinesses branch of
`h_group.rs::schedule_connection` selected the first unclued position without
checking its event-time visible identity. At engine45/live46 Alice's replay of
the engine44 Red clue assigned Bob's visibly White2/card40 the Red3 connection.
No Variant reproduced the same defect with a visible Blue2 assigned Red1.

The repair checks that selected position through `HistoricalView::identity`. It
does not skip a wrong position, inspect the hidden owner's true face, tune
scores, disable a convention or use a later reveal to rewrite an old clue.
Speculative `ConnectionPlanningContext::simulate` and `commit` both call this
shared scheduler.

| Run       | Exact command                                                     | Actual result                                                                                                                                                                                                                                  |
| --------- | ----------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| C1-red    | `cargo test -p hanabi-search --locked contract_foundation_ --lib` | Pre-repair production `6189471`: 2 passed / 2 failed. White failed on Alice assigning card40 White2 to Red3; No Variant failed on actor1/card9 Blue2 assigned Red1. Recorder `f770cecd3b934f15a760063b09270d4c`, 62.43s including compilation. |
| C1-green  | Same command after the production filter                          | 4 passed. Recorder `9c937cfc7dbc42adbc7afa9b8c08a037`, 83.52s including compilation.                                                                                                                                                           |
| C1-scoped | `bash scripts/check-scoped.sh hanabi-search contract_ --lib`      | 71 passed: original 66 advanced/migration witnesses plus 5 new contracts. Rust formatting and production crate check passed. Tests36s including compilation, formatting8s, check4s.                                                            |

Three additional exact focused filters passed with
`cargo test -p hanabi-search --locked FILTER --lib`:
`clue_giver_does_not_prompt_their_own_ambiguous_rank_three` (recorder
`9ebc774065e9495999d8a9d25a02de06`),
`fourth_replay_two_save_does_not_invent_a_prompt_on_a_chop_moved_card`
(`6e700e43ae034e1ab89f844619b11024`), and
`recognizes_expert_replay_layered_reverse_finesse`
(`1c9eed4979424d61a05fa3f6e6185365`). These are focused reviewed regressions,
not replay-wide action-parity passes. An initial scoped compilation failed on a
guessed test-only `ClueFacts::has_any_clue` API; replacing it with the actual
positive-clue predicates fixed compilation without weakening the assertion.

New tests in
[foundation.rs](../crates/hanabi-search/src/h_group/tests/foundation.rs) and
[generic_connections.rs](../crates/hanabi-search/src/h_group/tests/generic_connections.rs):

- `contract_foundation_white_recorded_prefix_is_legal_and_redacted`: replay all
  50 actions; retain exact identities and hidden own card39. Even after Bob's
  strike publicly reveals card40, its earlier face stays visible to Alice and
  hidden to Bob.
- `contract_foundation_white_finesse_hypotheses_respect_observer_visible_cards`:
  all three observers' engine44 hypotheses respect event-time visible cards.
- `contract_foundation_no_variant_finesse_rejects_visible_wrong_connector`:
  3-/4-player algorithmic negative controls, not invented strategy histories.
- `contract_foundation_no_variant_finesse_allows_hidden_own_connector`:
  3-/4-player positive controls retain the own-hand blind connection.
- `contract_foundation_white_finesse_response_reaches_executor_and_recipient`:
  Alice's card39 becomes an admitted forced play at the convention executor
  boundary. In an explicitly hypothetical legal branch where she demonstrates
  Red3, Bob gains the Red4/card32 play and no White2/card40 play. The recorded
  discard/strike actions remain unchanged.

These prove subcontracts of L1-12–16, L3-09 and F1/F2, not their entire rows.
All 48 whole-section statuses remain NOT YET TESTED. Negative hypothesis
assertions alone permit no chain; positive hidden and executor/recipient
controls prevent interpreting blanket suppression as success.

Admission remains separate. The saved pilot audit at source
`1c546ae1f2b5a43e123eaaa991e8d63432f3260e` captured engine44 Red-to-Bob as a
`PlayClue` proposal, `Unrecognized` recipient primary, `generatorProof` and
`responseSafety=Pass`; do not conflate these stages with action classification.
Manual diagnostic `foundation_white_recorded_clue_admission_trace` captures the
current proposal, admission, recipient diagnostic meaning and own due responses.
Run it with
`cargo test -p hanabi-search --locked foundation_white_recorded_clue_admission_trace --lib -- --ignored --nocapture`.
Recorder `037865c8ec2042658dcb1bd81f4cae82` captured Play/PlayClue, no declared
required response, GeneratorProof, Unrecognized, MCV2 and all three principle
checks Pass after repair. This is observed output, not an oracle. The earlier
uncaptured diagnostic returned a missing optional diagnostic field, which is not
evidence that recipient primary interpretation was absent. It is explicitly
ignored because it prints evidence, not an independent expectation; no existing
failed assertion was ignored. After repair Alice has connection card39/Red3 and
due card39; Bob has no currently due connection before Alice's response. Missing
`required_response` plus generic Pass/GeneratorProof still does not certify
every intermediate response or general admission soundness.

Tool defects discovered at this boundary (subsequently repaired below):

- Built-in CLI analysis of this White fixture at live45 fails with
  `current player is invalid`: `main.rs::run_analyze` uses No-Variant-only
  `FullState::view_for`. The diagnostic uses the existing physical view and
  checked standard-label adapter instead. C3 below repairs the CLI boundary.
- `replay_link.rs` hard-codes variant0, and its round-trip validator checks that
  constant rather than the fixture variant. It cannot generate a trustworthy
  White link yet. Do not publish a No Variant URL for this fixture; repair and
  independently test the codec boundary in a separate small batch. C2 below
  supplies that repair and a verified White link.

Independent read-only review found no blocking problem in the guard or fixture,
independently checking all cards/actions against the original/hash. It limited
the initial four tests' claims to visibility. The later executor/recipient test
covers one conditional response; actual planner execution at varied budgets and
complete admission certification remain open.

Checkpoint0
[CI](https://github.com/BarsukovSergey/hanab-engine/actions/runs/40016876486)
failed formatting of the original Goal document; the other six jobs passed.
Commit `6becf63` formats that document only. Inspection confirmed whitespace and
equivalent emphasis around “recorded prefix”, with no changed requirement. The
first overly strict non-whitespace comparison rejected the emphasis-marker
change; the corrected comparison passed. Certification/full validation remain
pending.

## Checkpoint 1: trustworthy variant metadata in audit tools

**C2: replay-link variant loss, repaired.** `replay_link.rs` encoded every
replay with wire variant0, and its validator checked that same constant. The
official `hanabi-live` source at `b0792ded044937776b36fc797dc25f060499e4f5`
(`misc/variants.txt`, `packages/client/src/lobby/hypoCompress.ts`) requires the
actual decimal variant ID in the third field. Wire IDs for all nine existing
supported catalog entries are now encoded and independently checked; unsupported
names retain the existing parser gate. No new physical variant or convention was
enabled. No Variant's original URL bytes and seed wrapping are unchanged.

**C3: CLI observation boundary, repaired.** `run_analyze` used the deliberately
No-Variant-only `FullState::view_for`, rejecting a legal White actor with
`current player is invalid`. It now takes the physical redacted view and uses
the existing checked clue-label adapter. An unsupported clue vocabulary gets a
specific error, not a false invalid-player diagnosis.

**C4: report domain truncation, repaired.** The newly accessible physical report
path exposed serializers using legacy `IdentitySet::iter`, silently dropping the
sixth suit. The active `IdentityUniverse` now reaches logical and convention
knowledge, candidate meanings, root and nested projection steps, branch
continuations, alternatives, and text rendering. Board identities are rendered
directly from visible physical cards; nonstandard suit5 is explicitly `s5`, and
own faces remain hidden. This is reporting plumbing; inference, planner policy
and production variant enablement did not change.

| Run                   | Command / condition                                                                                                      | Actual evidence                                                                                                                                                                                                                                                                                          |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| C2-red                | `cargo test -p hanabi-protocol --locked catalog_replay_links_preserve_official_variant_ids --lib`                        | Failed before repair: Black wire0 instead of official2. Recorder `0a15181908564d9b8ee11165aa258d92`. Test covers all9 source-backed IDs and rejects substituted variant fields with unchanged deck/actions.                                                                                              |
| C3-red                | `cargo test -p hanabi-cli --locked --test foundation_reporting`                                                          | Failed before the CLI adapter: `current player is invalid`, recorder `e777de0c82c741dc9a4ec3d16c164a3f`.                                                                                                                                                                                                 |
| C4-red                | `cargo test -p hanabi-cli --locked --bin hanabi-engine active_identity_serializers_preserve_sixth_suit_and_nested_steps` | Replayed the original legacy serializer body inside the new parameterized API: `[]` instead of White2. This is a controlled legacy-body reproduction, not a claim of running the entire old revision. Recorder `4408c5c0d8e04eb2bd539874f78a6a51`. Restored the active iterator before green validation. |
| C3-green              | `cargo test -p hanabi-cli --locked --test foundation_reporting`                                                          | Passed; six stacks, visible White2, redacted own cards and a White variant13 URL. Recorder `de8df9768dae4fe299c6015ec42ef4d2`.                                                                                                                                                                           |
| C2-scoped             | `bash scripts/check-scoped.sh hanabi-protocol replay_link --lib`                                                         | Three codec tests passed; formatting and production check passed.                                                                                                                                                                                                                                        |
| C3/C4-scoped          | `bash scripts/check-scoped.sh hanabi-cli --bin hanabi-engine`                                                            | Four unit tests passed, including sixth-suit and nested-step serialization; formatting and production check passed.                                                                                                                                                                                      |
| Reporting regressions | `cargo test -p hanabi-cli --locked --test foundation_reporting --test replay_link --test explain`                        | 11 passed: four existing explain tests, one new physical report invariant and six link tests, including exact No Variant URL and all five expert deck/action round trips. Recorder `acc043624e7e44698ad6069a178e2145`.                                                                                   |

An intermediate compile identified one omitted text-rendering caller of the new
identity-formatting signature; threading the same universe through text
projection rendering fixed it. The test assertions were preserved.

Independent read-only review verified all9 IDs against official source and the
CLI universe propagation/redaction boundaries. One minor finding, a five-suit
text stack caption on a six-stack board, was corrected. These tests do not
certify all historical/prospective redaction or H-Group admission semantics.

The generator now emits this verified source-data replay link:
[1, turn 46](https://hanab.live/shared-replay-json/315wmuob0tux2mfrfniw-zza2ajz3sqgih1ddvxny-bkc0kkpgrh1vsalpelqp-fuc,03lbehpb1b1afkfa-oafllcfiengbetgaeefj-esfr1afmfzfpkae1fvkb-e3e2fowc1cf0xctce7tb-efeqewxclaeCe4gbfcwc-e8wbeE,13,1#46).
The fixture still contains the original discard/strike, not a regenerated game.

Native `analyze` evidence is now available under ignored
`target/foundation-audit/p0-native-admission.json` and `p0-native-owner.json`.
Commands use the recovered fixture,
`--convention h-group --h-group-level 4 --format json --lines 1 --move-time-ms 5000 --exact-world-limit 16 --exact-node-limit 256`,
with `--live-turn 45 --candidate red:Bob` and `--live-turn 46` respectively. At
live45 the selected recorded Red-to-Bob clue still has PlayClue / generatorProof
/ Unrecognized / actionCoverage0, connectionSteps1 and responseSafety Pass. At
live46 actual `plan_move` selects Play39 as `forcedRoot=true`
(budgetExhausted=false); recorded Discard2 is notAdmitted and has no retained
projection. The chosen forecast starts Alice Play39, Bob Purple-to-Cathy, Cathy
Purple-to-Bob. The latter choices are conditional forecasts, not a proof that
Bob must immediately play Red4. This is one real planner budget observation, not
certification over all budgets.

CI for the preceding inference repair `373cc57` is observed successful:
[run40018886867](https://github.com/BarsukovSergey/hanab-engine/actions/runs/40018886867).
Full local validation, new-checkpoint CI and the remaining semantic gates stay
open. Next semantic batch: source-backed P0 admission/recipient/projection
contracts and deterministic hard-obligation behavior across planner budgets,
with a No Variant reviewed-position equivalent. The 48 whole-section statuses
remain unchanged; tool correctness is supporting evidence, not row
certification.

## Checkpoint 1 continuation: projection size and mandatory-action boundaries

### C5: resolved-world observer projection indexed a 50-card array

`PerspectiveProjector::project_resolved_owned` snapshots card identities before
redacting the modeled observer's hand and draw history. Its fixed 50-element
array was indexed by physical `CardId`, although White has 60 cards. This path
is used by prospective Save validation over giver-hand worlds
(`prospective.rs::prospective_clue_marks_focus_saved`). The defect is a physical
indexing invariant, independent of convention strategy or planner scores.

The new
`perspective::tests::contract_resolved_projection_preserves_late_variant_cards_and_redaction`
builds legal No Variant and White discard/clue histories through the last draw.
These artificial histories assert no convention meaning. They resolve the source
hand only as the explicit resolved-world API requires, and compare all three
observer outputs with independent core `FullState::physical_view_for` results.
Full view equality includes each owner's hidden faces and draw history. The No
Variant control reaches card49; White reaches card59.

Before repair,
`cargo test -p hanabi-search --locked contract_resolved_projection_preserves_late_variant_cards --lib`
passed the No Variant loop then panicked in White at `perspective.rs:216`:
`len is 50 but the index is 51`. Red recorder
`7b64134380a34c238ad79299711348bd`, ignored log
`target/foundation-audit/late-projection-red.log`. The repair snapshots exactly
`view.variant.identities().deck_size()` entries. The existing early rejection of
symbolic blank hands remains intact. This is not a global variant migration or
proof of every convention on White.

### Hard obligations, expired budgets and clue-transition evidence

New contracts distinguish producing a wrong action from returning an explicit
deadline error with no action. The White Level4 obligation is Alice Play39 at
live46. The independent No Variant compatibility witness is Bob Play22 (g2) at
p4v0s415 live18, after recorded live16 rank4 and live17 discard0. The
user-explained review `t16-finesse-plays-both-greens-before-handoff` matches
position key `307b4db26fd808f70266f672577e783e1118b32a4b0780485b60a2eaa0bb797f`;
its ruling requires the Finesse to play both greens. This prefix lies within the
conservative reviewed boundary. It is tested at Max as compatibility evidence,
not certification of all Max history at Level4.

`foundation::contract_foundation_completed_obligations_never_yield_an_unrelated_deadline_action`
first asserts that ordinary inference establishes and admits each required
action, then supplies an unrelated preferred action. An expired request may
return only that forced action or `Stopped(Deadline)`. The public zero-budget
API must return the explicit stop. The tested world/node settings are 0/0, 1/1
and16/256, but an expired budget does not exercise world enumeration.

`planner::tests::contract_forced_fallback_rejects_stale_preference_and_unestablished_belief`
therefore separately exercises deterministic fallback with a directly supplied
hard constraint: stale higher-score preference, completed/unfinished
projections, and known/unknown world count. Without an established world count,
fallback must stop; a returned result must contain and select only the required
candidate. This is an algorithm invariant, not an invented convention history.

During test development, recorder `4728a9a8e28e4530aad869e026c9d7d4` exposed an
invalid new-test assumption that zero budget must return success. Independent
review corrected that oracle before acceptance. It also rejected p4v0s9
live8/Card12 as a hard-obligation authority: that existing regression only
establishes ordinary clued-play selection and has no turn-specific review. No
production behavior or pre-existing assertion was changed to make these mistaken
assumptions pass. The corrected deadline contract passed (recorder
`7f6628170b034e1690f24420fb722222`). A later focused compile exposed one new
test's `PlayerId` argument type mismatch (recorder
`95cd146df6f640e2bdc6521fc7b8cd05`); corrected to the actual `u8` API.

`foundation::contract_foundation_recorded_clue_transition_preserves_each_observers_view`
uses the explicit recorded giver and compares the clue transition against core
physical replay for all three White and four No Variant observers. No draw or
action is substituted. This checks identical observer information states; it
does not require a giver's partial model of another observer to equal that
observer's actual information.

The expanded manual admission diagnostic
`foundation_white_recorded_clue_admission_trace` reports Alice as Play in both
prospective and actual replay, with Red4 connected through Alice39/Red3.
Projected Bob is Unrecognized while actual Bob is Play with that same
connection. The giver Cathy does not see her own cards, whereas actual Bob does;
those are different information states. Thus generatorProof admission without
recipient recognition remains an unresolved proof gap, not a demonstrated unsafe
admission solely because the labels differ. The diagnostic is explicitly
ignored/manual; no failing semantic assertion was disabled. Evidence: ignored
`target/foundation-audit/admission-projection.log`.

Observed CI for `88b14a4` failed only on two `default_trait_access` warnings in
the new CLI unit test:
[run40021077015](https://github.com/BarsukovSergey/hanab-engine/actions/runs/40021077015).
Separate mechanical commit `dbd2a7b` names the two concrete default types;
`bash scripts/check-scoped.sh hanabi-cli --bin hanabi-engine` passed four tests,
formatting and production check. No assertion or semantic code changed.

CI for mechanical commit `dbd2a7b` is observed successful:
[run40022156632](https://github.com/BarsukovSergey/hanab-engine/actions/runs/40022156632).

Final focused results for the C5 repair and boundary contracts:

| Command                                                                                                     | Actual result                                                                                                                         | Recorder                           |
| ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| `bash scripts/check-scoped.sh hanabi-search perspective::tests --lib`                                       | 9 passed, including late White/No Variant full-view parity and symbolic-blank rejection; Rust formatting and production check passed. | `db4d86603883456499ee221b0ccab61b` |
| `cargo test -p hanabi-search --locked foundation::contract_ --lib`                                          | 5 passed, including recorded clue parity for seven observers and both source-backed due responses.                                    | `a86384474137455497e66b162e77b5c8` |
| `cargo test -p hanabi-search --locked contract_forced_fallback --lib`                                       | 1 passed; known/unknown count and completed/unfinished projection controls.                                                           | `6875725d6b744bf88f3105c683c9b847` |
| `cargo test -p hanabi-search --locked incomplete_world_traversal_cannot_prove_contextual_save_safety --lib` | 1 existing Save safety regression passed; incomplete traversal remains insufficient proof.                                            | `8d6a0c807f3041f48eb53744aeda253e` |

Independent read-only review checked the final Vec repair, red reproduction,
test provenance, exact reviewed No Variant prefix, observer equivalence,
deadline/fallback oracle and the stated limits. No blocking finding remained;
two wording issues were corrected. This is batch review, not checkpoint4
certification. No full repository validation or new pilot was run in this batch;
its eventual PR CI must be observed separately.

The 48 whole-section dispositions remain unchanged. Next narrow step: finish
L1-12/L1-13 Prompt/Finesse admission and historical observer contracts using
official examples/reviewed No Variant positions, including impossible
connectors, already-due plays and recipient consequences. Quantify the P0
recipient proof gap over valid giver-hand worlds before calling admission safe
or declaring a defect; differing-information labels alone are insufficient.

## Checkpoint 1 continuation: source-backed Prompt contracts

The three new `generic_connections::contract_foundation_prompt_*` tests use the
pinned official
[L1 Prompt paragraph](https://github.com/hanabi/hanabi.github.io/blob/dd55b51aa622f709059a17e0d6afc2adb8402408/docs/level-1.mdx#the-prompt),
[beginner Prompt explanation](https://github.com/hanabi/hanabi.github.io/blob/dd55b51aa622f709059a17e0d6afc2adb8402408/docs/beginner/prompt.mdx)
and its `prompt/prompt-4.yml` diagram as independent authority. They supply
partial algorithm snapshots, not invented game histories or optimal moves. They
cover 32 configurations across cumulative Levels1–4:

- **Ordering and exclusion:** in 3-/4-player inputs, unknown red-clued
  candidates preserve left-to-right order; an already-due candidate cannot
  become a new Prompt candidate.
- **Successful alternatives:** the official Blue3, Green3, Red3 sequence
  preserves one Prompt promise and its provenance through both alternative
  plays. Red4 is physically unplayable until Red3 succeeds. The source's
  Yellow/Purple stacks remain on1 while Red/Blue/Green are on2; thus a prior
  rank3-only card was not already certainly playable.
- **Event-time knowledge:** for a conditional Red4 Self-Prompt interpretation,
  the same Red clue also retouches a previously rank3-clued connector. Prior
  rank3-only knowledge permits a new Prompt; prior Red+3 knowledge already
  establishes playable Red3 and must not generate a new Prompt. Current facts
  are identical; only `facts_before` differ. Other stacks on1 preserve the prior
  rank3 uncertainty. This asserts neither the complete recipient superposition
  nor a particular best action.

Independent review caught a draft-test prerequisite error: `[2;5]` stacks made
every rank3-only candidate already playable. Restoring the exact source stack
context fixed the oracle before delivery, without changing production code. An
intermediate scoped run passed its two then-compiled tests but failed formatting
while the third draft test was being added (recorder
`32510588f5a446a6928170b024d66f43`); the final coherent version is revalidated.
The final contracts do not prove complete giver admission, actor execution or
recipient inference; L1-12 remains NOT YET TESTED at whole-section level.

The reviewed p4v0s1 live32 negative authority
`t32-known-purple-three-is-not-a-new-prompt` still matches position key
`4d46a4f32b10bf5927ab18ee5c401156a76de5ad2748a019641ed94ded99d327`. Its existing
Max regression remains the human-reviewed compatibility witness.

Inspection also found unconditional later-visible duplicate selection with L5
Ambiguous Finesse comments. Independent review distinguishes the initial
possibility of a visible connector from the advanced deduction after an earlier
player's decline. L1 does not independently specify a replacement tie-break for
this ambiguous case. No speculative gate or forced earlier play was added;
post-decline lower-level certification remains an open contract. The existential
unknown-own fallback must not be deleted: ordinary hidden Finesses need it.

CI for `3d88dd4` is observed failed only on `similar_names` for the new test's
`state`/`stale` local names:
[run40023182349](https://github.com/BarsukovSergey/hanab-engine/actions/runs/40023182349).
The mechanical correction renames the preference variable and preserves every
assertion. A crate-scoped strict Clippy run is justified here by that observed
failure; it is not a full repository-validation claim.

Final delivery validation:
`bash scripts/check-scoped.sh hanabi-search generic_connections:: --lib` passed
all 16 module tests, Rust formatting and the production crate check (recorder
`8f132e436a9c46dbb103b1e353972fc9`). The focused crate Clippy command
`cargo clippy -p hanabi-search --lib --tests --locked -- -D warnings` also
passed (recorder `e20450214027475693a42b48a212a25a`). It ran before the final
negative-oracle correction, which only removed the unsupported requirement that
excluding an already-due candidate must create a Prompt on another card. That
earlier test was already green; review corrected its authority, not its
compatibility with the implementation. The final module rerun includes this
correction. Existing test assertions were preserved.

Independent read-only review verified all three final contracts, source stack
prerequisites, `facts`/`facts_before` order and partial-evidence claims; no
blocking finding remained. No new production semantic defect was demonstrated in
this batch. The existing reviewed Max Prompt regression was not rerun in this
delivery; its provenance anchor was checked only.

Mechanical CI repair `b5e28da` is pushed and its CI is observed successful:
[run40039622412](https://github.com/BarsukovSergey/hanab-engine/actions/runs/40039622412).
This is not full local validation. All 48 whole-section dispositions and the
known red B3 baseline remain unresolved. The user's stop instruction cancels
automatic execution of the previously listed next steps; PR54 stays Draft.

## Review, delivery and remaining gates

Checkpoint 0 changes documentation only. Independent read-only review by a
separate agent checked all 48 inventory titles/URLs, B0–B4 summaries and
recorder IDs, evidence status honesty, and selected direct source references.
Three map findings were corrected: nonexistent production path, two test
namespaces, and the next-discard timing restriction in L4-08. This review does
not certify all semantic constraints or implementation behavior and does not
replace checkpoint4 review of the repairs, test authority and observer safety.
Validation of the map: Prettier on this file, exact 48-row inventory/11-column
comparison, relative Markdown link resolution, and `git diff --check`. The Goal
remains active. All 48 rows, A1–A7 and F1–F5 need their full requirements
discharged. P0's first visibility defect is repaired; admission and broader
cross-perspective certification remain open. B3 remains a recorded red baseline,
and the p4v0s415 review-boundary discrepancy needs resolution.

Before certification: complete checkpoints1–3, independent review of code and
assertions, focused and affected higher-level regressions, full
`bash scripts/check.sh`, observed PR/main CI, and one bounded White smoke with
an action log and no score target. Keep a draft PR while these remain open; do
not merge or declare success. Pilot completion/score is an observation only.
