# Repository Instructions

## Windows Subsystem for Linux (WSL) file ownership

If this repository is cloned inside WSL, ensure that newly created files are
owned by the same user and group as the repository parent. Windows-hosted
editing tools can accidentally create files owned by `root:root`.

## Validation

Optimize for short, evidence-backed interactive turns. Full validation is a
checkpoint, not an automatic tax on every prompt. Preserve all tests and their
assertions; change when they run, not their strength.

| Change                                        | Local validation before responding                                                     |
| --------------------------------------------- | -------------------------------------------------------------------------------------- |
| Discussion / diagnosis                        | Relevant read-only inspection; no full suite                                           |
| Documentation                                 | Formatting and any affected examples/scripts                                           |
| Fixture edit                                  | Parse and replay legality; inspect/test the affected position                          |
| Localized convention fix                      | Reproduce the failure; focused regression and related tests; `scripts/check.sh --fast` |
| Shared inference/planner refactor             | Focused checks first; one full `scripts/check.sh` after implementation settles         |
| Milestone or explicit full-validation request | Full `scripts/check.sh`                                                                |

`--fast` retains build, lint, documentation, Python, and dead-code checks but
omits the expensive ordinary Rust suite. Run relevant Rust tests separately. Run
inexpensive formatting/Clippy before launching broad tests. A failing focused
test is a reason to keep debugging, not to start full validation.

PR CI is a targeted gate: formatting/build/Clippy/docs, MSRV, core/protocol
tests, a small green hanabi-search contract suite, Python live-bridge checks, and
dead-public-code checks. Pushes to `main` additionally run the exhaustive
H-Group profile matrix. Neither CI path is the complete ordinary hanabi-search
suite. `scripts/check.sh` remains the full local validation checkpoint.

Do not wait for CI unless requested or necessary for the task. Never describe
focused/local success or an unobserved CI run as a full pass. Report what ran,
what was deferred, and any known failures.

Do not overlap full validation runs or CPU-heavy benchmarks. For performance
comparisons, use the same precompiled workload and configuration, run before and
after sequentially, and distinguish reduced test setup from engine speed.
`check.sh` completes all stages even when tests fail. Do not restart the entire
suite for a formatting-only fix or test-only assertion adjustment: rerun the
affected stage/tests. Rerun the full suite only if subsequent implementation
changes invalidate its coverage. Do not run full validation repeatedly to
rediscover unchanged pre-existing failures.

### Timing and failure records

Use `scripts/workflow.py` when timing, performance comparison, full-validation
failure comparison, or a long replay investigation matters. Small scoped edits
and ordinary focused fixes do not need a task timer. When a timer is used,
finish stale sessions before starting another and close it before the final
response.

`check.sh` automatically records commands, output logs, stage times, exit codes,
and Git/worktree identity under ignored `target/workflow/`. Wrap other
substantive commands with
`python3 scripts/workflow.py run --label LABEL -- COMMAND ...`. Use `wait-start`
/ `wait-end` only for intervals spent waiting, not intervals used to investigate
or edit while checks run. Unrecorded waits are unknown. `report` shows task
durations, command execution, explicitly recorded waiting, and full-run counts.
Do not add overlapping command durations to task duration.

Use an explicit `baseline LOG --revision REVISION` from a completed full run to
distinguish new, persistent, and no-longer-reported test failures. Never
automatically accept new failures into the baseline. Baselines are diagnostic:
all failures retain nonzero exit status, including in CI. Stage failures and
incomplete runs must also be reported; missing results are not proof of fixes.

## Test provenance

Do not add invented game histories as authorities for convention meanings or
optimal moves. Use a human-reviewed replay position, identify its fixture and
turn, and explain the reviewed expectation. Hypothetical alternatives may branch
from that position, but must not silently replace its recorded moves.

Self-play recordings are bug reproductions, not validated strategy. Keep their
assertions limited to established rules, legality, consistency, or a specific
human-reviewed interpretation; do not freeze an old engine choice as the best
move. Artificial inputs remain appropriate for ordinary game-rule, codec,
algorithm, and data-structure unit tests and invariant-only smoke tests. See
`docs/testing.md` for the categories and known coverage gaps.

## Clear bugs discovered during examination

When examining code, explaining an engine decision, auditing behavior, or
investigating a replay, autonomously fix clear implementation bugs discovered
within that task. This is standing authorization to make those scoped fixes,
even when the immediate question asks why something happens. Do not stop after
identifying the bug or ask for another prompt merely to implement it. Reproduce
the issue, add or update appropriate regression coverage, fix it, run the
applicable implementation checks above, and resume the original examination.

A clear bug has an established expected behavior supported by the convention
documentation, a reviewed user interpretation, or an ordinary software
invariant. An unexplained score difference or uncertain convention is not
enough. Ask for judgment when the intended behavior is genuinely ambiguous; do
not invent convention rules, one-off exceptions, or fixture changes to force
parity. Respect an explicit request for read-only analysis or no edits. This
standing authorization does not expand a localized task into unrelated fixes or
a full replay scan, and documentation-only requests remain documentation-only.

## Version control

After every user prompt that changes this repository:

1. Run the relevant checks according to the validation matrix above.
2. Commit all in-scope changes with a descriptive commit message.
3. Push the commit to the current branch's configured upstream.

Do not create empty commits for prompts that make no repository changes. Do not
include unrelated pre-existing worktree changes in the commit. If validation,
the commit, or the push fails, report the failure instead of claiming the task
is complete.

## Other Repositories

- The "[hanabi-live](https://github.com/Hanabi-Live/hanabi-live)" repository
  contains the source code for the website where everybody plays the game. This
  engine has to integrate with it in various ways. The repository should be
  checked-out next to this one. You can reference the source code when you need
  to confirm a specific game mechanic or server data structure.
- The ["hanabi.github.io"](https://github.com/hanabi/hanabi.github.io/)
  repository contains the source code for the website that documents every
  H-Group convention. The engine uses these conventions when playing in H-Group
  mode. The repository should be checked-out next to this one. You can reference
  the source code when you need to confirm how a specific convention should
  work.

## Replay positions and human review

For replay disagreements, convention-semantic investigations, fixture edits, or
human-review questions, read and follow
[`docs/replay-agent-workflow.md`](docs/replay-agent-workflow.md) before doing
the replay analysis. Do not load that workflow for unrelated core, protocol,
documentation, or infrastructure tasks.
