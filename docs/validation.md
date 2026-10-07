# Validation workflow

This document is the detailed reference for local and CI validation. Keep
`AGENTS.md` concise: it selects the validation scope; this file explains what
the helpers do and how to interpret their results.

## Choosing the scope

Prefer the smallest check that covers the changed behavior.

For a scoped Rust change, use:

```sh
bash scripts/check-scoped.sh <crate> [--test-only|--check-only] [focused cargo-test args...]
```

The helper reuses the primary worktree Cargo target cache and prints per-stage
timings. Default mode optionally runs one focused
`cargo test -p <crate> --locked ...` invocation, then repository Rust formatting
and a normal check of the affected crate. `--test-only` runs the focused test
and formatting but skips the final production-config check; use it only when an
equivalent `cargo check` already passed at the current API boundary.
`--check-only` runs formatting plus the crate check and accepts no test args.
Run extra focused tests separately when one invocation is not enough.

Do not routinely add strict Clippy, `--all-targets`, the CI search-contract
matrix, broad H-Group regressions, or `scripts/check.sh --fast` to a scoped
change. Those are broader gates unless the change directly targets them or
focused compilation exposes cross-cutting fallout.

For an ordinary coding-agent batch inside a shared inference/replay/planner
refactor, still stop at focused tests plus `check-scoped.sh` by default.
Pull-request CI owns the routine broad build, strict Clippy, documentation,
MSRV, Python, ownership, and dead-public-code gates.

Use:

```sh
scripts/check.sh --fast
```

only when the task explicitly requests a local broad checkpoint or when a
specific cross-cutting risk is not covered by the focused checks and CI. Fast
mode skips the ordinary Rust test suite, does not replace focused Rust tests,
and is not a full replay-validation claim.

The full local checkpoint is:

```sh
scripts/check.sh
```

Use it for milestones, broad changes, or explicit full-validation requests.

## Full-check behavior

Run full validations sequentially. `check.sh` uses a repository-local
cross-platform lock to reject another simultaneous `check.sh` invocation. Avoid
running standalone CPU-heavy Cargo suites or benchmarks concurrently; the lock
does not govern arbitrary commands.

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
smoke checks, Rust build/Clippy/docs, Rust 1.85 MSRV, core/protocol tests, the
generic search boundary tests, fast deterministic H-Group tests whose names use
the `contract_` prefix, Python live-bridge checks, rustdoc tests, and
dead-public-code checks.

Pushes to `main` additionally run the exhaustive H-Group profile matrix. Neither
PR CI nor push CI is the complete ordinary hanabi-search test suite.
`scripts/check.sh` remains the full local checkpoint.

Do not wait for CI unless requested or necessary for the task. Never describe an
unobserved CI run as passing, and never describe focused/local success as a full
pass. Report what actually ran, what was deferred, and any known failures.

## Check records and failure baselines

Both check modes automatically record command output, per-stage timings, exit
status, commit, and tracked-diff fingerprint under ignored `target/workflow/`.
The records survive prompts but are removed by `cargo clean`.

For an ad hoc command where a durable timing/log record is useful, invoke the
recorder through the portable Python launcher:

```sh
bash scripts/run-python.sh scripts/workflow.py run --label focused -- \
  cargo test -p hanabi-search fourth_replay_red_one_precedes --lib
```

Recording is deliberately command-scoped. There is no task-session or waiting
state to maintain between commands.

After inspecting a completed full-check log, record a diagnostic failure
baseline explicitly:

```sh
bash scripts/run-python.sh scripts/workflow.py baseline /tmp/reviewed-check.log \
  --revision COMMIT
```

Subsequent completed full runs report new, still-failing, and no-longer-reported
nextest failures. A baseline never changes exit status or accepts failures. A
focused or incomplete run cannot establish that omitted tests were fixed;
inspect test counts, stage exit codes, and logs before drawing that conclusion.
