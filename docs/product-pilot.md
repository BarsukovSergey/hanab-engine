# No Variant, Rainbow and White product pilots

## White prospective draw checkpoint: INCOMPLETE

Clean source `4d0ba162c1dabeae2b69e5c75e7aa3085b96105d`, measured 2026-10-23,
same seed/deck/profile budgets as the initial checkpoint below. The earlier
owner-domain widening was caused by `ProspectiveTransition::card_action`
allocating draws with `50 - deck_size`: White's initial 45-card draw pile
produced card ID 5, duplicating a live card, instead of the actual ID 15. It now
uses the catalog deck size and the variant score ceiling (30 for White, 25 for
No Variant). The original knowledge-validation assertion is unchanged; temporary
diagnostics were removed. No convention interpretation changed.

The new deterministic physical invariant test compares the entire projected
successful-play view against `FullState` for both No Variant and White,
including unique live IDs and a blank own draw. It passes, as does the existing
No Variant transition-parity test through the final round. Commands:

```sh
cargo test --locked -p hanabi-search --lib contract_prospective_card_action_keeps_variant_draw_ids_unique
cargo test --locked -p hanabi-search --lib prospective_transitions_match_the_simulator_through_the_final_round
bash scripts/check-scoped.sh hanabi-search --check-only
git diff --check
```

The source was committed before rerunning the already-built debug integration
binary with `--include-ignored --test-threads=1`. The physical-label/deck
contract passes; both opt-in planner tests still exit 101:

| Profile       | Completed | Play / Discard / Clue | Official final score | Stack score / strikes | Failed decision |
| ------------- | --------- | --------------------- | -------------------- | --------------------- | --------------- |
| Level1 smoke  | No        | 0 / 0 / 0             | Unavailable          | 0 / 0                 | 0.4820 s        |
| Level4 seed 1 | No        | 0 / 0 / 0             | Unavailable          | 0 / 0                 | 0.1823 s        |

Level4 elapsed time is 0.1825 s; reconstruction matches. No planner result,
symbolic/exact phase or budget-fallback status is returned before the panic. The
current first blocker is `history_reducer::reduce_discard` at the
`known_playable` check: it calls `identity::is_playable_at` with five historical
stack heights and a sixth-suit identity. The exact backtrace is retained in
`target/white-draw-fix-backtrace.log`. The next minimal capability is using
active, event-time stack state in that historical discard check.

Latest evidence: `target/product-pilot/white-hgroup-first.json`,
`white-hgroup.json` and `white-hgroup.ndjson`, all labeled with source SHA
`4d0ba16`. Command results: `target/white-draw-fix-checkpoint.log`,
`white-draw-contract.log`, `white-transition-parity.log`,
`white-draw-scoped.log`. The original Level4 failure report/trace is archived as
`target/product-pilot/white-hgroup-owner-widening.json` and `.ndjson`.
Acceptance remains unmet; White 3/4 Loaded semantics remain absent and all
production gates remain unchanged. PR #53 stays draft.

## Initial White H-Group checkpoint: INCOMPLETE (historical)

Measured 2026-10-23 on Windows native Rust at clean source commit
`269949174753b50bbf5d6cf14cfece371d9efe14`. This is an offline diagnostic, not
enabled production/live support. The existing H-Group inference, candidate
generation and `plan_move` are used without action substitution.

Three players, official catalog `White6Suits`, all 60 cards with catalog copy
counts, seed 1, `splitmix64-fisher-yates-modulo-v1` as described below. Only the
acting observer's `physical_view_for` crosses the decision boundary; own-hand
and own-draw identities are asserted hidden before the checked `try_standard`
conversion, retaining the White variant. Config: ExpectedScore, 16 exact worlds,
256 exact nodes, five-second move limit. These limits apply only to this opt-in
runner. Production defaults and No Variant gates are intact.

```sh
cargo test --locked -p hanabi-search --test white_hgroup_pilot \
  -- --include-ignored --test-threads=1
```

This invocation exits **101**, not success: the physical-label/deck contract
passes, but both opt-in planner tests fail with debug assertions enabled.

| Profile                   | Completed | Applied Play / Discard / Clue | Official final score | Stack score / strikes | Decision time |
| ------------------------- | --------- | ----------------------------- | -------------------- | --------------------- | ------------- |
| Level1 first-action smoke | No        | 0 / 0 / 0                     | Unavailable          | 0 / 0                 | 0.4741 s      |
| Level4 game, seed 1       | No        | 0 / 0 / 0                     | Unavailable          | 0 / 0                 | 0.4325 s      |

Level4 elapsed time including evidence writes/reconstruction: 0.4328 s. The
initial Level4 decision fails before returning a symbolic/exact result or budget
status; no timeout fallback or replacement action is applied. Its empty applied
prefix reconstructs identically. It is not a completed zero-score game.

**Historical blocker (resolved by the draw-ID fix above):**
`h_group/history_reducer.rs::finish` rejects an ordinary owner-knowledge domain
widening while computing prospective continuations of real turn 0:

```text
ReplaceDomain { card: CardId(5), identities: IdentitySet(1073217503), source: Clue(3) }
```

`Clue(3)` is an internal prospective history turn, not a fourth applied game
action. The invariant is preserved; its error now includes the offending effect.
The later checkpoint traced this invariant failure to duplicate prospective card
IDs, rather than a missing convention rule. No convention rule was invented.

Earlier dirty-worktree release diagnostics first reproduced a Good Touch panic:
a sixth-suit singleton was decoded with the No Variant iterator. This checkpoint
fixes that producer and direct consumers with active identity IDs and includes
an invariant-only duplicate-accounting regression. The intermediate release
build then made a legal Level1 choice and a Level4 rank-1 clue followed by the
recipient's successful play (Symbolic/WorldLimit, no budget exhaustion), before
failing at `identity::is_playable_at` on the sixth suit at real turn 2. These
exploratory logs are **not** clean-SHA acceptance evidence: release also omits
the earlier debug owner-knowledge assertion. No completed game is claimed.

The official [White rules](https://hanabi.github.io/variant-specific/white)
apply the Loaded Play Clue Assumption to 3/4 Saves, excluding 2/5; the
[learning path](https://hanabi.github.io/learning-path) recommends Level4. That
White-specific assumption is **absent from this implementation**; the existing
base Save resolver explicitly omits Loaded conventions. Physical touching parity
does not certify these semantics. No White convention release or supported
recipient-play acceptance claim is made.

Historical Level4 evidence under ignored `target/product-pilot/` is archived as
`white-hgroup-owner-widening.json` and `.ndjson`; the ordinary `white-hgroup*`
report names now contain the latest checkpoint above. The trace flushes before
each decision, on failure, and at the result. Compiler and test logs are
`target/white-checkpoint.log`, `target/white-unit-final.log`,
`target/white-no-variant-final.log`, `target/white-gates-final.log`, and
`target/white-scoped.log`. The historical release diagnostics remain in
`target/white-first.log`, `target/white-first-after.log`, and
`target/white-game-backtrace.log`.

Independent read-only review found no blocking issue in the local Good Touch fix
or redacted action path. It required rerunning from a clean commit (done) and
noted that the report's redaction-check flag records enabled assertions, not
proof that an arbitrary failing decision passed them. The current failure occurs
after those assertions. Acceptance is unmet; keep the PR draft.

Focused validation (all exit 0 unless explicitly described above):

```sh
cargo test --locked -p hanabi-search --lib contract_admission_good_touch_preserves_white_identity
cargo test --locked -p hanabi-search --lib third_replay_two_new_duplicate_fours_still_violate_good_touch
cargo test --locked -p hanabi-search --lib standard_semantic_adapter_rejects
bash scripts/check-scoped.sh hanabi-search --check-only
git diff --check
```

These cover one new invariant, one reviewed No Variant Good Touch regression,
two unchanged semantic rejection tests, formatting and production crate
compilation. The physical-label/deck integration contract also passes. No full
suite, 200-game baseline or CI pass is claimed.

## Earlier No Variant and Rainbow measurements

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
