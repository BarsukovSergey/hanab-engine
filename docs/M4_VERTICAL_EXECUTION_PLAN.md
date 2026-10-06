# M4 Vertical Execution Plan

This document is the current execution plan for M4: variant-aware H-Group for
3-6 players. It complements `ROADMAP.md`, which remains the durable M1-M10
milestone sequence.

## Why this plan exists

M4 is an incremental semantic migration, but incremental does not mean one
helper, kernel, or convention per pull request. After PR #38, M4 work is grouped
by production capability.

A substantive M4 pull request should normally finish a vertical slice:

```text
production path
-> semantic result
-> focused regressions
-> No Variant parity
-> representative variant coverage
```

One pull request may contain several implementation commits. Do not split kernel,
composition, wiring, cleanup, and documentation into separate pull requests when
they are all required for the same capability.

## Baseline after PR #38

Already merged:

- generic `LogicalDeductions` / `InformationSet` boundary
- variant-aware convention-free baseline assessment and `PhysicalAction`
  selection
- `GenericClue` as the stored H-Group replay clue vocabulary
- variant-aware immediate/direct Level 1 Play kernel
- variant-aware base 5 / 2 / Critical Save kernel
- base Play + Save `direct_level_one` composition
- base Save classification wired into historical replay and prospective Save
  scoring

Important current boundaries:

- H-Group production policy is still intentionally No Variant-only
- `direct_level_one` is not yet the production authority for primary clue
  interpretation
- Good Touch is still implemented through standard-card assumptions
- connection semantics still depend heavily on standard `Card`, `Clue`,
  `Rank::ALL`, and fixed five-stack state
- `ReplayReducer` still contains standard-only storage such as
  `[u8; 5]` stack heights and `[u8; 25]` removed-card counts
- `standard_h_group_clue` remains on several active convention paths
- `RuleExecutionContext` still carries a raw H-Group profile rather than a
  resolved convention capability/ruleset boundary

These are migration boundaries, not invitations to perform a repo-wide generic
rewrite.

## Execution rules

### Vertical capability, not micro-PR

Each M4 pull request should answer:

> What production capability does the engine have after this merge that it did
> not have before?

"Added a helper", "added a struct", "composed two kernels", and "prepared future
wiring" are not sufficient outcomes by themselves.

### No dead architecture

Do not add a semantic kernel, ruleset abstraction, adapter, or storage type that
is deliberately left unwired when the same pull request can connect it to the
production path.

If an abstraction is needed, introduce the smallest form that the current
vertical slice immediately uses.

### Migrate storage with the path that needs it

Do not create a standalone migration whose only goal is replacing every
`[u8; 5]`, `[u8; 25]`, `Rank::ALL`, or legacy `Card`.

Replace fixed storage and standard-only helpers when they block the production
path being migrated. By the end of M4, no supported non-No-Variant H-Group path
may depend on those standard-only assumptions.

### Preserve semantic layers

Keep these boundaries explicit:

```text
VariantSpec / physical rules
!=
H-Group convention meaning
!=
strategic policy
```

M4 changes convention meaning and the plumbing required to execute it. It must
not duplicate physical variant mechanics from M3 or prematurely add M5-M8
policy.

### Preserve No Variant behavior

Every slice must retain reviewed No Variant behavior while adding variant-aware
semantics. Do not replace broad legacy play/connection semantics with a smaller
physical kernel and accidentally erase Prompt, Finesse, loaded, contextual, or
other existing branches.

## M4-V1 - Basic clue semantics end-to-end

### Outcome

Basic H-Group clue interpretation works end-to-end on the generic variant
boundary instead of having separate physical kernels and standard-only knowledge
logic.

### Scope

- migrate Good Touch / base Level 1 knowledge semantics to active
  `VariantSpec` / `IdentityUniverse` / `IdentitySet`
- make base Play / Save / primary meaning share one production semantic
  authority where behavior is genuinely the same
- remove duplicated base classification made obsolete by that authority
- introduce only the minimum resolved capability/rules boundary needed by this
  production path
- migrate fixed stack/count representations only where this path requires it
- keep higher-level contextual suppressions explicit rather than folding them
  into physical kernels
- update `WORKING_STATE.md` in the same pull request

### Done when

- historical replay uses the migrated base knowledge semantics
- prospective/basic clue assessment does not maintain a conflicting duplicate
  implementation
- Good Touch uses active variant identity/copy/playability semantics
- No Variant reviewed regressions remain unchanged
- representative variants cover different touching/copy behavior
- no new helper or semantic kernel is left intentionally unwired

## M4-V2 - Generic connection engine

### Outcome

The basic Prompt/Finesse connection family uses generic identities and variant
successor/predecessor semantics in both replay and prospective reasoning.

### Scope

- Prompt and Finesse connection discovery
- Reverse/Self and directly dependent special-finesse behavior when required by
  the shared connection machinery
- predecessor/successor traversal through `VariantSpec`
- generic stack progression required by connection simulation
- shared historical/prospective connection semantics where practical
- retain existing ambiguity, loaded-state, Fix, and provenance behavior

Do not pull advanced Bluff families into this slice unless a common connection
primitive cannot be safely separated.

### Done when

- a supported variant can form and resolve the basic connection family without
  converting its clue/identity domain to the standard five-suit model
- No Variant connection regressions stay stable
- semantic tests assert connections/obligations, not merely final selected
  actions

## M4-V3 - Focus, chop, tempo, stall, and discard families

### Outcome

Convention families driven primarily by focus/chop/history/resource context no
longer rely on standard physical assumptions.

### Scope

Group the related non-advanced-connection rules, including as applicable:

- Chop Moves
- Tempo / Clarity behavior
- Stalling
- Special Discards
- trash/endgame-adjacent focus behavior
- 6-player hand-size/count-sensitive cases

Migrate only the shared infrastructure required by those production paths.

### Done when

- these families operate on active variant physical facts
- 3-6 player behavior uses actual hand/count state rather than five-player
  assumptions
- precedence and negative cases are covered where the family can overlap basic
  Play/Save/connection readings

## M4-V4 - Advanced connections and bluffs

### Outcome

The advanced connection graph can interpret variant games without standard-only
identity traversal.

### Scope

Group connection-heavy advanced behavior such as:

- Bluffs
- intermediate and double Bluff families
- loaded and Fix interactions
- ejections/discharges
- priority-related connection behavior
- advanced connection scheduling that shares the same generic dependency graph

### Done when

- connection hypotheses, obligations, and lifecycle/provenance remain
  deterministic across supported variant identities
- No Variant advanced replay regressions remain stable
- representative variant tests cover semantic state and precedence

## M4-V5 - High-level refinements and remaining standard-only semantic paths

### Outcome

The remaining H-Group 1-25/max semantic stages are reachable without silently
falling back to standard Hanabi assumptions.

### Scope

Migrate the remaining active families, including as applicable:

- Context
- Duplication
- Elimination
- Five Tech
- Out-of-Order Play
- Ignition
- Phantom Playable
- Charms
- Unnecessary Moves
- Extras
- public-layer / inverse-planning / late-game semantic paths still using the
  standard clue adapter

This is also where remaining fixed storage on active variant H-Group paths
should disappear if earlier slices did not require it.

### Done when

- every enabled rule family has a variant-safe execution path
- supported variant execution no longer reaches `standard_h_group_clue`
- standard-only compatibility helpers may remain only behind explicitly
  No-Variant-only APIs/tests

## M4-V6 - Variant rule pack and production enablement

### Outcome

H-Group can be intentionally enabled in production for the target 3-6 player
variant catalog.

Target catalog:

```text
No Variant
Black
Rainbow
White
Pink
Brown
Prism
Omni
Null
```

### Scope

- implement the remaining variant-specific H-Group meanings from reviewed
  sources/rulings
- finish resolved convention capabilities/rules for variant semantics
- enable planner/candidate/CLI/live policy only for variants whose required M4
  semantics are complete
- cover 6-player-specific count/hand cases
- remove the global No-Variant-only H-Group gate only when the replacement
  boundary is explicit and tested
- perform final M4 documentation sync

Do not invent variant conventions from physical mechanics alone. Physical
touching/copies/playability come from `VariantSpec`; convention meanings need
H-Group/reference/review authority.

## Validation and review policy

For each vertical slice:

1. run focused semantic tests first
2. run `bash scripts/check-scoped.sh hanabi-search ...` for the affected
   focused test target
3. for shared replay/inference/planner refactors, run
   `bash scripts/check.sh --fast` after implementation settles
4. use full `bash scripts/check.sh` at the M4 checkpoint rather than after
   every ordinary slice

Tests should assert the semantic state that matters: identity domains,
Play/Save classification, hypotheses, connections, obligations, precedence,
provenance, admitted/rejected actions, or other relevant facts. A matching final
move alone is insufficient evidence for a convention migration.

## M4 acceptance

M4 is complete when:

- H-Group 1-25/max has a variant-safe production path for the target 3-6 player
  catalog
- supported non-No-Variant paths do not silently convert to standard Hanabi
- No Variant behavior remains stable against reviewed regressions
- variant-specific meanings are backed by source/review evidence rather than
  inferred from variant names
- 6-player count/hand-size cases are covered
- planner/CLI/live policy can intentionally select the supported variant
  H-Group ruleset
- the full milestone validation passes

M4 does not include the separate 2-player semantic model, replay recovery,
clarity/complexity policy, risk policy, production performance work, or mobile
integration. Those remain M5-M10 in `ROADMAP.md`.

## Pull-request sizing rule

Prefer approximately these six substantive vertical pull requests over a long
sequence of micro-PRs. Split a vertical slice only when there is an independent
shippable capability or a genuine unresolved semantic decision.

Do not split merely because one slice needs several commits or touches several
closely related files.
