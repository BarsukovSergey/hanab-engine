# Validation workflow

This document is the detailed reference for local and CI validation. Keep
`AGENTS.md` concise: it selects the validation scope; this file explains what
the helpers do and how to interpret their results.

## Choosing the scope

Prefer the smallest check that covers the changed behavior.

For a scoped Rust change, use:

```sh
bash scripts/check-scoped.sh <crate> [focused cargo-test args...]
```

The helper reuses the primary worktree Cargo target cache, optionally runs one
focused `cargo test -p <crate> --locked ...` invocation, then runs repository
Rust formatting and a normal check of the affected crate. Run extra focused
tests separately when one invocation is not enough.

Do not routinely add strict Clippy, `--all-targets`, the CI search-contract
matrix, broad H-Group regressions, or `scripts/check.sh --fast` to a scoped
change. Those are broader gates unless the change directly targets them or
focused compilation exposes cross-cutting fallout.

For a shared inference/replay/planner refactor, run focused checks first and
use:

```sh
scripts/check.sh --fast
```

after the implementation has settled. Fast mode runs the broad build, lint,
documentation, Python, ownership, and dead-public-code stages but skips the
ordinary Rust test suite. It does not replace focused Rust tests and is not a
full replay-validation claim.

The full local checkpoint is:

```sh
scripts/check.sh
```

Use it for milestones, broad changes, or explicit full-validation requests.

## Full-check behavior

Run full validations sequentially. `check.sh` uses a repository-local
cross-platform lock to reject another simultaneous `check.sh` invocation.
Avoid running standalone CPU-heavy Cargo suites or benchmarks concurrently; the
lock does not govern arbitrary commands.

The full script runs all ordinary Rust tests without fail-fast, continues to
later validation stages after stage failures, prints per-stage timings, and
returns nonzero if any stage failed. Missing prerequisites still stop the run
immediately.

Do not rerun the whole suite for a formatting-only fix or a narrow assertion
adjustment. Rerun the affected stage or focused tests unless later
implementation changes invalidate the previous full-run coverage. Do not
repeatedly rediscover unchanged pre-existing failures.

## Platform behavior

The shell validation helpers support Linux/WSL, macOS, and Windows Git Bash.
Shared environment setup lives in `scripts/shell-common.sh`.

`check.sh` skips Unix ownership validation under Windows Git Bash, accepts both
POSIX and Windows virtualenv layouts, and uses the portable validation lock. If
a Rust check fails before meaningful compilation because the Windows host
linker, runtime path, or equivalent toolchain setup is unavailable, make at most
one obvious attempt to use the repository's existing configured toolchain. Do
not install or repeatedly repair host tooling as part of a scoped code task;
report the deferred check and rely on Linux CI for the broader gate.

## CI scope

Pull-request CI is intentionally targeted. It runs formatting, shell-helper
smoke checks, Rust build/Clippy/docs, Rust 1.85 MSRV, core/protocol tests, a
small green hanabi-search contract suite, Python live-bridge checks, rustdoc
tests, and dead-public-code checks.

Pushes to `main` additionally run the exhaustive H-Group profile matrix. Neither
PR CI nor push CI is the complete ordinary hanabi-search test suite.
`scripts/check.sh` remains the full local checkpoint.

Do not wait for CI unless requested or necessary for the task. Never describe an
unobserved CI run as passing, and never describe focused/local success as a full
pass. Report what actually ran, what was deferred, and any known failures.

## Workflow timing and failure records

Workflow recording is opt-in. Use it for performance work, full-check
comparison, or long replay investigations, not for ordinary scoped edits. Invoke
it through the portable Python launcher:

```sh
bash scripts/run-python.sh scripts/workflow.py start "Fix turn 27 play priority"
bash scripts/run-python.sh scripts/workflow.py run --label focused -- \
  cargo test -p hanabi-search fourth_replay_red_one_precedes --lib
scripts/check.sh --fast
bash scripts/run-python.sh scripts/workflow.py finish
bash scripts/run-python.sh scripts/workflow.py report
```

Both check modes automatically record command output, per-stage timings, exit
status, commit, and tracked-diff fingerprint under ignored
`target/workflow/`. Extra commands only need the workflow wrapper when a
recorded timing session is intentionally active. The records survive prompts but
are removed by `cargo clean`.

Use `wait-start` and `wait-end` only for intervals actually spent waiting for
tools. Do not count editing or investigation while a command runs as waiting.
The report separates session elapsed time, the union of command intervals,
explicitly recorded waiting, and full-run count. These quantities overlap and
must not be added together. Zero recorded wait does not prove there was no wait.

After inspecting a completed full-check log, record a diagnostic failure
baseline explicitly:

```sh
bash scripts/run-python.sh scripts/workflow.py baseline /tmp/reviewed-check.log \
  --revision COMMIT
```

Subsequent completed full runs report new, still-failing, and
no-longer-reported nextest failures. A baseline never changes exit status or
accepts failures. A focused or incomplete run cannot establish that omitted
tests were fixed; inspect test counts, stage exit codes, and logs before drawing
that conclusion.
