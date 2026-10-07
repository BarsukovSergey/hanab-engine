# Repository Instructions

## Windows Subsystem for Linux (WSL) file ownership

If this repository is cloned inside WSL, ensure that newly created files are
owned by the same user and group as the repository parent. Windows-hosted
editing tools can accidentally create files owned by `root:root`.

## Validation

Use the smallest validation scope that answers the risk in the current change.
Preserve tests and their assertions; change when they run, not their strength.
Detailed command behavior, platform notes, CI scope, timing, and
failure-baseline mechanics live in [docs/validation.md](docs/validation.md).

| Change                                        | Local validation before responding                                           |
| --------------------------------------------- | ---------------------------------------------------------------------------- |
| Discussion / diagnosis                        | Relevant read-only inspection; no full suite                                 |
| Documentation                                 | Formatting and any affected examples/scripts                                 |
| Fixture edit                                  | Parse and replay legality; inspect/test the affected position                |
| Scoped Rust / localized convention change     | `bash scripts/check-scoped.sh <crate> [focused cargo-test args...]`          |
| Shared inference/replay/planner refactor      | Focused checks + `check-scoped.sh`; broad gates stay in CI unless requested    |
| Milestone or explicit full-validation request | Full `scripts/check.sh`                                                      |

Do not widen a scoped task into strict Clippy, `--all-targets`, broad replay
regressions, `scripts/check.sh --fast`, or full validation without a concrete
risk that the extra check addresses. Pull-request CI owns the routine broad
build/lint/docs/MSRV gates. Do not wait for CI inside an implementation session
unless the task explicitly requires it. Never describe a focused/local result
or an unobserved CI run as a full pass.

Do not overlap full validation runs or CPU-heavy benchmarks. If a Windows/Git
Bash Rust check fails before meaningful compilation because of host linker or
runtime setup, make at most one obvious attempt to use the repository's existing
configuration; do not turn a scoped task into host-toolchain repair.

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
When an implementation task names an explicit file/function boundary, a clear bug
outside that boundary is reported rather than fixed unless it directly blocks the
assigned change or the task explicitly authorizes the expansion.

## Version control

After every completed user prompt that changes this repository:

1. Run the relevant checks according to the validation matrix above.
2. Commit all coherent in-scope changes with a descriptive commit message.
3. Push the commit to the current branch's configured upstream.

A budget/stop-condition handoff is not task completion. If a coherent checkpoint
exists, commit and push it. If the current edit is knowingly incomplete or
unbuildable, do not keep working merely to satisfy this rule and do not pretend it
is complete. Preserve only the agent's own in-scope work, report the dirty files
and exact blocker, and let the next session continue from that checkpoint.

Do not create empty commits for prompts that make no repository changes. Do not
include unrelated pre-existing worktree changes in the commit. If validation,
the commit, or the push fails, report the failure instead of claiming the task
is complete. Do not open a pull request merely to obtain routine CI while a
multi-session implementation is still in progress; open it after source
implementation is complete unless early CI/review is explicitly useful.

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

## Context budget

Start scoped tasks from the files named by the task and their direct
compile/test dependencies. Use targeted search to answer a concrete question; do
not inventory the repository or read broad audit/refactor/history documents just
in case. In particular, do not read replay-review material for unrelated work.
Expand context only when a compile error, failing focused test, or an unclear
API boundary gives a concrete reason.

Keep tool output proportional to the question. Prefer exact-symbol search and a
small line range around the target over printing a whole large source file.
Inspect `git diff --stat` before broad diffs and request per-file patches when
possible. Redirect verbose compiler/test output to a log and inspect the first
useful failure or a targeted tail instead of repeatedly dumping full logs.
Do not use `--nocapture`, full replay traces, or profiler output unless the task
needs those details.

## Coding-agent execution budget

A branch or pull request is a delivery unit; an agent chat is not. One branch
may intentionally be completed by several short agent sessions. Each session
should implement one compile-safe batch, commit it, and push it before handing
off.

Treat semantic decisions, exact functions, helpers, tests, and exclusions named
by the task as settled. Do not repeat repository inventory, architecture
research, upstream comparison, or convention research unless the task delegates
that work or a concrete contradiction appears.

Default implementation-session budget is about 20 minutes and at most roughly
65% of the available context window when that usage is visible. If the task is
not nearly complete by either threshold, stop expanding the investigation.
Commit and push a coherent checkpoint when safe. If no coherent checkpoint
exists, leave only the agent's own in-scope edits uncommitted and report them
rather than spending another long session manufacturing a checkpoint. A task may
explicitly grant a larger budget when the expected compile/test runtime justifies
it.

Read the named functions first and expand only to direct compile/test
dependencies. An unexpected caller may justify one targeted search. Needing a
new semantic decision, a broad refactor, a forbidden file, or repeated
cross-subsystem exploration is a stop condition rather than permission to widen
scope.

Do not investigate a failing historical regression on the base revision unless
the task explicitly asks for baseline comparison. Report the exact failure and
let the orchestrator decide whether baseline archaeology is worth a separate
task. Routine CI-only formatting/Clippy fallout should be fixed as a separate
small mechanical step rather than reopening an exhausted semantic session.

Human/orchestrator guidance for decomposing Codex work lives in
[docs/codex-workflow.md](docs/codex-workflow.md). Coding agents do not need to
read that longer document unless the task explicitly asks them to.

## Replay positions and human review

For replay disagreements, convention-semantic investigations, fixture edits, or
human-review questions, read and follow
[`docs/replay-agent-workflow.md`](docs/replay-agent-workflow.md) before doing
the replay analysis. Do not load that workflow for unrelated core, protocol,
documentation, or infrastructure tasks.
