# M1: working baseline on hanab.live

M1 reuses the substantial live-bot implementation already present in the pinned Zamiell baseline. It does not reimplement WebSocket transport or the standard-game reducer from scratch.

## M1-A: baseline and provenance

- import exact upstream tree for `be00e7852571258af7847af35646c3571f620c64`;
- preserve MIT notice;
- record provenance and upstream sync policy;
- run/record baseline validation before semantic edits.

## M1-B: adopt the existing live boundary

Keep the existing flow:

```text
Hanabi Live WebSocket
  -> Python live bridge
  -> persistent Rust live-session
  -> player-safe state/view
  -> planner
  -> action
```

Do not create a second parallel engine-session abstraction without an actual boundary defect.

## M1-C: persistent configuration and immutable snapshots

Replace temporary per-table `/level` behavior:

- configuration persists across games and process restarts;
- any seated player may change pregame configuration; last write wins;
- game start resolves an immutable `GameConfigSnapshot`;
- commands received while Playing mutate neither the active snapshot nor persistent settings.

M1 only needs the configuration skeleton and H-Group level surface.

## M1-D: hard no-table-talk

Add one adapter-level outbound chat gate. While a game is Playing, every chat/PM/reply path is blocked. Local logs and persisted traces remain allowed.

## M1-E: reproducibility journal and parity

Persist enough stable data to reproduce decisions offline:

- engine/build revision;
- bot seat;
- immutable config snapshot;
- normalized public event history;
- decision turn/history fingerprint;
- recorded chosen action.

Add a verifier that replays the same history/config and checks that every bot decision recomputes identically.

## M1-F: normalize diagnostics

Adapt existing planner diagnostics into a stable project trace schema. Do not build a parallel explanation engine and never invent reasons unavailable from the engine.

## M1-G: acceptance

A real No Variant game can be played silently by the bot, replayed offline with decision parity, and a later game inherits persisted configuration.
