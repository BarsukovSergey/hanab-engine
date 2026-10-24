# No Variant MVP readiness

Delivery: [Goal #55](https://github.com/BarsukovSergey/hanab-engine/issues/55),
branch `delivery/no-variant-mvp`, single
[Draft PR #64](https://github.com/BarsukovSergey/hanab-engine/pull/64).
**Release sign-off is PENDING. No merge or live verification is claimed.**

## Delivery evidence

| Issue      | Result                             | Evidence and limits                                                                                                                                                                                                                                                                     |
| ---------- | ---------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| #57        | PASS                               | `5a8e6c1`: event-time visible contradictions cannot form basic Finesse connections; own hidden blind response preserved. Negative contract red before / green after; two reviewed No Variant Self-Finesse regressions passed.                                                           |
| #58        | Harness/report PASS; gameplay FAIL | `b259b5c`: [exact game evidence](no-variant-smoke-result.json). Level1 / 3p completed 23 points, no strikes. Level4 / 4p hit the explicit 2-second deadline before turn 15; legal-prefix parity passed. Max gameplay was not run after the first error.                                 |
| #59        | SKIPPED                            | Its explicit branching rule applies: the small-budget deadline did not establish a source-backed semantic or legality violation. No speculative repair or preferred-action rewrite.                                                                                                     |
| #60        | PASS within opening scope          | `34f19bd`: [78 checks](no-variant-profile-result.json), all 26 profiles on three reviewed 4p opening positions. Action legality, no strike from required actions, equal complete analysis across equal views with distinct hidden assignments. Not all-rule or full-game certification. |
| CI cleanup | Focused PASS                       | `e62226f`: restore the Python return annotation and extract a test helper, retaining all assertions. Focused contract, Python bridge checks and affected crate Clippy passed.                                                                                                           |
| #61        | Offline checks; live PENDING       | Recorded 3p opening, session/rebuild parity, player-safe trace replay, legal action encoding and bridge guards; commands below. Actual private game remains external.                                                                                                                   |

The support target is No Variant H-Group Levels 1–25/Max, 3–5 players. Measured
gameplay is narrower: one completed 3p Level1 game, an aborted 4p Level4 game
and no new 5p game. No claim of reliable full-game coverage across that target
follows from these checks. 2p and White remain on hold.

The known main test `second_replay_order_chop_move_protects_alices_red_two`
remains unresolved. Its preferred action has not been rewritten, and this
delivery does not claim a source-backed semantic resolution. Review must assess
this known limitation alongside the deadline failure before release.

## Offline protocol checks

The stored `no-variant-live-opening.json` is an exact extraction of the existing
CLI `traced_opening_snapshot`, table 39, three players, observer 0, initial
draws only. It is recorded input, not a new live game or a convention oracle.
The original CLI action assertions are preserved. A transport turn-0 update is
used only to check append/rebuild equality; it invents no strategic move.

```sh
bash scripts/check-scoped.sh hanabi-protocol -- --test no_variant_live
cargo test -p hanabi-cli --locked --test analyze deterministic_planner_selects_the_rank_two_opening -- --exact
python -W error::ResourceWarning -m unittest discover -s scripts/tests -p 'test_hanabi_live_*.py'
python scripts/check-no-variant-live.py --engine target/debug/hanabi-engine.exe
```

The last command requires a current prebuilt engine
(`cargo build --locked -p hanabi-cli`; use the extensionless binary path on
Unix). It uses the real `live-session`, `live-action`, `TraceRecorder` and
`hanabi_live_verify.py`. It compares incremental and one-shot responses, records
two offline decisions and verifies each in a fresh Rust process. Reports and
scrubbed traces go to `target/no-variant-live/`. A request uses the engine's
standard 120-second move budget and a 130-second process timeout; there are no
retries or invented fallback actions. It never connects to hanab.live or sends
the recorded action.

The [recorded offline result](no-variant-live-result.json) checked two decisions
in fresh engine sessions: two matches, zero mismatches and zero errors, in
102.70 seconds. Incremental and one-shot complete responses matched, with
`budgetExhausted: false`. A prior 10-second diagnostic failed full-response
parity with `budgetExhausted: true`; that failure is retained in the report. The
successful check uses the ordinary engine budget and does not establish
deterministic full-response parity for interrupted analyses or fix the #58
2-second gameplay failure.

The Rust contract checks own-hand masking and exact session/rebuild view
equality, including failed-update isolation. Existing focused Python tests cover
history fingerprints, rejection of duplicate/stale submissions (also when public
history changes at the same turn), trace masking, frozen game configuration and
chat blocking while playing. These offline assertions are not evidence that a
real server game completed successfully.

## Remaining release gates

- Required PR CI must be green on the final source revision. A historical
  success or a local focused check is not a current full CI pass.
- Bounded independent review of the semantic integration is still required; no
  approval review has been observed on #64. Do not merge automatically.
- Live sign-off is **PENDING**: the current environment has neither configured
  `HANABI_USERNAME`/`HANABI_PASSWORD` nor a captured live trace. No login was
  attempted and no credentials were printed.

The external operational step is to complete the existing
[manual live acceptance](M1_ACCEPTANCE.md#manual-live-acceptance) with a
securely configured dedicated bot account: use an approved release build and the
Python requirements, invite the bot only to a private No Variant table, complete
a game, check duplicate/stale-action and chat silence records, verify pregame
level persistence across a new game and restart, then run:

```sh
python scripts/hanabi_live_verify.py <captured-run-directory> --engine <approved-engine-binary>
```

Require nonzero checked decisions, zero mismatches and zero errors. Attach only
player-safe verification results to #61. Offline traces must not be submitted as
proof of this manual step. Do not start #62/#63 as part of sign-off.
