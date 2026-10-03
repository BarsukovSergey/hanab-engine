# M1 Acceptance

Status: **candidate ready for live smoke**, not final live-validated.

## Automated acceptance

- [x] Pinned upstream baseline is recorded with provenance.
- [x] Existing hanab.live bridge is adopted instead of reimplemented.
- [x] Persistent bot configuration is versioned and atomically stored.
- [x] H-Group level is sticky across future games and process restarts.
- [x] Each game freezes an immutable configuration snapshot.
- [x] Config commands during Playing are ignored and do not mutate persistent or active config.
- [x] Outbound `chat` and `chatPM` are blocked at the adapter while Playing.
- [x] Stale decisions are rejected using `DecisionKey(table, turn, history_hash)`.
- [x] Game and decision journals are generation-scoped and replayable.
- [x] Deterministic parity verification replays each recorded decision in a fresh engine session.
- [x] TurnTrace schema v1 normalizes existing knowledge, convention and planner diagnostics.
- [x] Deferred M7/M8 fields remain explicit rather than fabricated.
- [x] Focused Python live checks, MSRV and H-Group profile validation run in CI.

## Manual live acceptance

These require a real dedicated hanab.live bot account and are intentionally not marked complete from CI:

- [ ] Log in to hanab.live through the ordinary user endpoint.
- [ ] Join a No Variant table through a private pregame command.
- [ ] Complete a full live game without duplicate/stale actions.
- [ ] Confirm zero outbound bot chat from `tableStart` through game completion.
- [ ] Change H-Group level pregame, play a game, and confirm the next game inherits it.
- [ ] Restart the bot and confirm persistent configuration survives.
- [ ] Run `hanabi_live_verify.py` against the captured live trace and get zero mismatches/errors.

## Full inherited Rust suite

The imported upstream full replay suite is not an M1 release gate until its pre-existing timeout behavior is repaired. Routine PR CI uses the fast quality path plus MSRV, exhaustive H-Group profile checks and dedicated live-bridge tests.
