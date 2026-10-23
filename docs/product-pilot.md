# No Variant and Rainbow product pilot

Measured on 2026-10-23, Windows native Rust, release profile. This is a small
offline product experiment, not an H-Group strength gate or an M4 migration.

## Revisions and scope

- No Variant: unmodified `0e9a3864cd9fb9ab84cba90379866fc362660e81`, descended
  from main `015731bb7872980e93b670a2ace910816727f251`.
- Rainbow runner and all three measured games:
  `33544a45f056b20da8b025e2c1ebeadaf477978a`, clean tracked worktree at
  execution.
- No production policy, convention, simulator, expert fixture, baseline or CI
  behavior changed. The ordinary new test checks invariants, not optimal moves.

## No Variant: bounded diagnostic, no completed result

Four players, H-Group Max, `plan_move` with the existing perfect-score
objective, 4,096 worlds and 50,000 exact nodes, one worker, fixed Hanabi Live
seed `p4v0s1`. Run from the repository root with Git Bash (also valid in bash on
Linux):

```sh
HANABI_SELF_PLAY_START=1 HANABI_SELF_PLAY_GAMES=1 \
HANABI_SELF_PLAY_WORKERS=1 \
HANABI_SELF_PLAY_REPORT=target/product-pilot/no-variant-seed1.json \
bash scripts/check-self-play.sh
```

The initial release build took 7m 51s. Gameplay was explicitly bounded to about
two minutes after compilation; the process was stopped at approximately 123s.
This resource boundary was announced before observing the partial result.

| Seed   | Completed                      | Applied actions in snapshot | Official score | Perfect     | Strikes / strikeout | Engine error                                              |
| ------ | ------------------------------ | --------------------------- | -------------- | ----------- | ------------------- | --------------------------------------------------------- |
| p4v0s1 | No: interrupted at time budget | 11                          | Unavailable    | Unavailable | Unavailable         | None observed before interruption; game not fully checked |

The snapshot is the position before decision 12 (zero-based turn 11). No
terminal result or final score was produced; this is neither a zero-score
completed game nor evidence of a convention bug. The unmodified harness does not
record per-move times, so total/max decision latency is unavailable. Do not
infer its strength, eventual score, or future errors from this prefix. Seeds 2–4
were not run. Completed No Variant games: **0 of 1 attempted**.

Preserved local evidence under ignored `target/product-pilot/`:

- `no-variant-seed1.log`: compile/game output, then forced termination.
- `no-variant-seed1.exit`: actual wrapper exit **127**. Cargo recorded child
  exit `0xffffffff` after Windows `Stop-Process`; this is an interruption, not a
  naturally returned engine error.
- `no-variant-seed1.timeout.txt`: resource-boundary record.
- `no-variant-seed1.active/p4v0s1.json`: seed, players and all 11 applied
  actions.
- `no-variant-seed1.ndjson`: empty; no completed game to flush.
- `no-variant-seed1.json`: absent, because final aggregation never ran.

The full 200-game clean baseline remains unestablished, as described in
[self-play.md](self-play.md). No baseline was created or updated.

## Rainbow: genuine physical lifecycle, poor play

Three players, catalog `Rainbow6Suits`, all using the existing
`ConventionAgnosticPolicy`, **not H-Group**. The complete 60-card deck comes
from each catalog identity's exact copy count. `FullState::new_variant` deals
whole hands in player order. These seeds are local shuffle seeds, not Hanabi
Live `p4v0s*` seeds.

Shuffle `splitmix64-fisher-yates-modulo-v1`: start the wrapping u64 RNG at the
seed; descend indices 59 through 1; add `0x9e3779b97f4a7c15`, mix with shifts
30/27/31 and multipliers `0xbf58476d1ce4e5b9` / `0x94d049bb133111eb`, then swap
index i with `output % (i + 1)`. The small modulo bias is intentional and
versioned. Reports retain the exact ordered deck with card instance IDs.

First run seed 1; only after its prompt completion run seeds 2 and 3, same SHA:

```sh
HANABI_PRODUCT_PILOT_SEEDS=1 \
cargo test --release --locked -p hanabi-search --test product_pilot \
  rainbow_physical_pilot -- --ignored --exact

HANABI_PRODUCT_PILOT_SEEDS=2,3 \
HANABI_PRODUCT_PILOT_REPORT=target/product-pilot/rainbow-extra.json \
cargo test --release --locked -p hanabi-search --test product_pilot \
  rainbow_physical_pilot -- --ignored --exact
```

| Seed | Terminal reason | Official score | Stack score | Strikes | Turns | Play / Discard / Clue | Engine / replay errors |
| ---- | --------------- | -------------- | ----------- | ------- | ----- | --------------------- | ---------------------- |
| 1    | TooManyStrikes  | 0              | 0           | 3       | 3     | 3 / 0 / 0             | None                   |
| 2    | TooManyStrikes  | 0              | 2           | 3       | 5     | 5 / 0 / 0             | None                   |
| 3    | TooManyStrikes  | 0              | 0           | 3       | 3     | 3 / 0 / 0             | None                   |

| Seed | Total decision time (ms) | Max decision time (ms) | Game elapsed (ms) |
| ---- | ------------------------ | ---------------------- | ----------------- |
| 1    | 0.1760                   | 0.0883                 | 0.5449            |
| 2    | 0.2319                   | 0.0662                 | 1.0508            |
| 3    | 0.1118                   | 0.0457                 | 0.3314            |

Decision timing includes view creation, redaction checks, legal action list,
deductions and policy selection; excludes trace writes and action application.
Game elapsed includes trace writes and reconstruction, excludes initial setup
and compilation. These very small timings are machine-dependent measurements,
not a benchmark guarantee. Release test target compilation took 58.27s; both
opt-in invocations exited **0**. A passing diagnostic test means software
invariants held, not that the bot played successfully.

**3/3 simulator lifecycles terminated, 0/3 ended without strikeout.** Each
official score is zero, including seed 2's two successful plays. At eight clues,
discard is illegal and this policy never clues, so it blind-plays newest cards.
These runs provide no evidence of useful cooperative play, Rainbow H-Group
convention correctness, statistical strength, or a variant `plan_move` planner.

## Observation boundary and reproducible evidence

On all 11 Rainbow decisions the chain is: `physical_view_for(current_player)` →
`decide(PlayerView<GenericClue>)` → `LogicalDeductions::new(view)` →
`ConventionAgnosticPolicy.select_physical_action` → membership in
`view.legal_physical_actions()` → `FullState::apply_physical`. `decide` accepts
no simulator, seed or deck. Every decision checks that own hand identities and
own historical draw identities are hidden. No scripted clues, secret-aware
plays, future draws or artificial knowledge were supplied.

Local artifacts (ignored, retained in this workspace; not Git-hosted):

- [Seed 1 report](../target/product-pilot/rainbow.json) and
  [flushed trace](../target/product-pilot/rainbow.ndjson).
- [Seeds 2–3 report](../target/product-pilot/rainbow-extra.json) and
  [flushed trace](../target/product-pilot/rainbow-extra.ndjson).
- `rainbow-release.log`, `rainbow-release.exit`, `rainbow-extra-release.log`,
  `rainbow-extra-release.exit` retain command output and exit status.

Each JSON report contains SHA, seed, shuffle version, complete ordered deck,
exact physical action labels/card IDs, terminal reason, official/stack scores,
counts, costs and errors. Each NDJSON starts with the full deck and flushes
before every decision, after selection and after application. An interrupted
prefix can be reconstructed with `new_variant(3, variant, ordered_deck)` and the
recorded physical actions. Each completed run independently reconstructed the
same action sequence in a second simulator and compared the entire `FullState`,
including history: all three matched. This verifies deterministic
reconstruction, not an independent implementation of the game rules.

The runner captures decision/application/reconstruction failures per game and
preserves the report before failing the test. Fixed catalog/deck setup errors
remain ordinary test failures outside that capture; the initial trace is already
flushed. The report's SHA is HEAD; run from a clean committed worktree as done
here so it identifies the measured source.

## Validation, review and next product step

Passed separately:

```sh
cargo test --locked -p hanabi-search --test product_pilot \
  deterministic_deck_and_reconstruction -- --exact
bash scripts/check-scoped.sh hanabi-search --check-only
git diff --check
```

The ordinary test verifies deterministic deck generation, every physical copy
count and full-state reconstruction for a short invariant-only prefix. The
ignored full-game test ran separately in release mode as shown above. Routine
full validation and 200-game self-play were not run. Independent read-only
review covered the runner's hidden-information boundary, legality, terminal
scoring, deck, replay and this evidence report; no blocking findings remained.
Required PR/main CI is reported in the delivery response, separately from these
focused local checks.

The smallest next capability is **an explicit cooperative physical clue-giving
policy**, at `baseline.rs::ConventionAgnosticPolicy::select_physical_action`'s
`LogicalDeductions<GenericClue>` → `PhysicalAction` seam (prefer a separately
named opt-in policy). At full tokens it should evaluate legal generic clues from
the giver's visible teammate cards and the variant's touch rules, with an
explicit recipient inference protocol; both giver and recipient must operate
only on their own views. Start with a small direct-clue/recipient-deduction
contract and the same three fixed full-game seeds; measure safe playable
knowledge, clue use, non-strikeout endings, official scores and latency. Do not
merely add arbitrary stall clues and call that useful strategy. This feature is
not implemented here. Integrating real H-Group Level1 comes after establishing
this cooperative physical-policy seam; R2B/GenericClue recognizer migrations are
separate architecture work and are not the demonstrated blocker.
