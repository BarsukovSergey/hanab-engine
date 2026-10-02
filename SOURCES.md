# Sources and provenance

This file records external repositories and documentation used to design, implement, verify, or compare Hanab Engine.

## Primary code baseline

### Zamiell/hanabi-engine

- Repository: `https://github.com/Zamiell/hanabi-engine`
- Baseline commit: `be00e7852571258af7847af35646c3571f620c64`
- Baseline tree: `2c1c2efb5d08032cd20021eb86fb2e78307789b2`
- License: MIT
- Role: primary code baseline and architecture foundation.
- Policy: preserve the upstream MIT notice and clearly record imported revisions.

## Reference implementations

### Hanabi-Live/hanabi-live

Reference for Hanabi Live protocol, variants, physical mechanics, and table lifecycle. GPL-licensed implementation is used as a reference; substantial GPL code is not copied into this MIT codebase without a deliberate licensing decision.

### will-hanabi-bot/hanabi-bot

Behavioral/reference oracle for overlapping H-Group behavior. License observed during audit: GPL-3.0. Compare behavior and independently implement; do not copy substantial code, tests, comments, or literal implementation translations.

## Convention sources

H-Group documentation, reviewed rulings, Jeff score-hunting material, historical two-player material, and any other semantic source must be pinned here by revision/license status before implementation derived from it is committed.

A fixture, replay, or existing bot behavior is evidence, not automatically semantic authority.
