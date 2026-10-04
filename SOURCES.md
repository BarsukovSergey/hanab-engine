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

Physical variant support (M3):

- Pinned revision: [`67517fcce0a441bf75313d92dd6078d34495971b`](https://github.com/Hanabi-Live/hanabi-live/tree/67517fcce0a441bf75313d92dd6078d34495971b).
- Scope: No Variant and Black, Rainbow, White, Pink, Brown, Prism, Omni, and Null in their **6 Suits** editions. Other suit counts and combinations are outside this catalog.
- Definitions: `packages/game/src/json/variants.json` (ordered suit membership), `packages/game/src/json/suits.json` (physical suit properties), `packages/game/src/suitsInit.ts` and `packages/game/src/variantsInit.ts` (default clue domains, ranks, stack size, and maximum score).
- Mechanics: `packages/game/src/rules/deck.ts` (copy counts), `packages/game/src/rules/clues.ts` (touching, including Prism's rank-dependent color), and `packages/game/src/rules/playStacks.ts` (ascending stacks). `packages/game/src/rules/card.ts` was also inspected to locate the deck and stack helpers.
- These physical facts are independently represented by checked-in declarative specs in `hanabi-core`; no source implementation or GPL bot code is imported, and no catalog is fetched at runtime.
- Engine keys (`no-variant`, `black-6-suits`, etc.) are independent of Hanabi Live's numeric or string protocol IDs. Suits and clue colors use spec-local IDs: standard suits/colors occupy 0 through 4; the added suit occupies 5, with a clue color 5 only in Black, Pink, and Brown.
- No Variant preserves its existing identity order and 50-card deck. The added variants have 30 identities, ascending ranks 1 through 5, and a maximum score of 30. Black has a 55-card deck (one of each black rank); the others have 60 cards.
- Offline Hanabi Live replay parsing resolves the supported exact wire variant names to the engine catalog, decodes explicit decks and clue labels through the active `VariantSpec`, and replays `PhysicalAction` values deterministically.
- Live Hanabi Live snapshot/session reconstruction is also variant-aware for the same catalog: deck bounds, identity decoding, clue history, stack count, score ceiling, and physical action command encoding derive from the active `VariantSpec`.
- Observer privacy remains enforced in the live adapter: scrubbed own-hand identities are not reconstructed from simulator truth or public clue touch lists.
- Convention/policy support is intentionally separate from physical support. Non-No-Variant physical views are supported, while the legacy H-Group/policy API returns an explicit unsupported-policy error until M4 supplies variant-aware convention semantics.
- Coverage lives in the core variant/physical-action tests and protocol replay/live-variant tests. Existing No Variant compatibility regressions are retained.

### will-hanabi-bot/hanabi-bot

Behavioral/reference oracle for overlapping H-Group behavior. License observed during audit: GPL-3.0. Compare behavior and independently implement; do not copy substantial code, tests, comments, or literal implementation translations.

## Convention sources

H-Group documentation, reviewed rulings, Jeff score-hunting material, historical two-player material, and any other semantic source must be pinned here by revision/license status before implementation derived from it is committed.

A fixture, replay, or existing bot behavior is evidence, not automatically semantic authority.
