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
| Shared inference/replay/planner refactor      | Focused checks first; `scripts/check.sh --fast` after implementation settles |
| Milestone or explicit full-validation request | Full `scripts/check.sh`                                                      |

Do not widen a scoped task into strict Clippy, `--all-targets`, broad replay
regressions, or full validation without a concrete risk that the extra check
addresses. Do not wait for CI unless the task requires it. Never describe a
focused/local result or an unobserved CI run as a full pass.

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

## Context budget

Start scoped tasks from the files named by the task and their direct
compile/test dependencies. Use targeted search to answer a concrete question; do
not inventory the repository or read broad audit/refactor/history documents just
in case. In particular, do not read replay-review material for unrelated work.
Expand context only when a compile error, failing focused test, or an unclear
API boundary gives a concrete reason.

## Replay positions and human review

For replay disagreements, convention-semantic investigations, fixture edits, or
human-review questions, read and follow
[`docs/replay-agent-workflow.md`](docs/replay-agent-workflow.md) before doing
the replay analysis. Do not load that workflow for unrelated core, protocol,
documentation, or infrastructure tasks.
