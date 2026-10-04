# Roadmap

This is the durable milestone sequence. Current implementation details belong in
[`WORKING_STATE.md`](WORKING_STATE.md); historical architecture decisions belong
in the focused architecture/review documents.

| Milestone | Goal                                  | Status      |
| --------- | ------------------------------------- | ----------- |
| M1        | Working hanab.live baseline           | Complete    |
| M2        | Generic core and 6-player boundary    | Complete    |
| M3        | Physical variant mechanics            | Complete    |
| M4        | Variant-aware H-Group for 3-6 players | In progress |
| M5        | Proper two-player ruleset             | Planned     |
| M6        | Replay, explainability, and recovery  | Planned     |
| M7        | Complexity and clarity reduction      | Planned     |
| M8        | Risk and policy hardening             | Planned     |
| M9        | Production hardening                  | Planned     |
| M10       | Mobile/offline integration            | Planned     |

## M4 acceptance direction

M4 migrates convention semantics incrementally rather than rewriting H-Group at
once.

- generic physical observations enter logical search without lossy conversion
- convention-free assessment/action selection uses the active `VariantSpec`
- convention replay/history vocabulary becomes generic-clue aware
- H-Group semantic slices are migrated with representative variant regressions
- legacy No Variant behavior remains stable throughout
- planner/CLI/live policy boundaries never silently reinterpret a non-No-Variant
  game as standard Hanabi

Physical mechanics are already an M3 responsibility; M4 should not duplicate the
variant catalog or Hanabi Live codecs.
