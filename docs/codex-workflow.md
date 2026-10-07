# Codex workflow

This document defines how we use coding-agent sessions efficiently. It is
orchestrator guidance: the repository rules in `AGENTS.md` remain authoritative
for the agent itself.

The optimization target is not maximum agent autonomy. It is the shortest path
from an already-understood design to a reviewed commit while preserving semantic
correctness, hidden-information safety, validation quality, and a clean Git
history.

## Split responsibilities

The orchestrator owns research and judgment:

- understand the relevant architecture before assigning implementation
- resolve convention semantics from reviewed sources
- compare upstream/reference behavior when useful
- choose the exact implementation boundary
- choose focused tests and decide whether a failing regression deserves a base
  comparison
- review the actual diff and CI after push
- handle trivial CI-only cleanup directly or with a tiny mechanical task

Codex owns execution of the decided change:

- edit the named functions and direct dependencies
- keep the implementation compile-safe
- add only the requested focused coverage
- run the requested narrow validation
- commit and push the batch
- report blockers instead of reopening design work

Do not pay twice for the same research. If the task packet says a semantic choice
is decided, Codex should implement it rather than independently rediscovering it.

## Delivery unit versus session unit

A vertical capability still normally ships as one branch and one pull request.
That does **not** imply one Codex chat.

Prefer several short sessions on the same branch:

```text
vertical branch / one PR
  -> batch A / commit
  -> batch B / commit
  -> batch C / commit
  -> CI cleanup if needed
  -> review / merge
```

A fresh chat is cheap; a nearly exhausted context window is not. Continuing the
same branch from a fresh chat is expected.

A normal implementation batch should usually be one of:

- 1-3 related semantic functions
- one API boundary plus its direct callers
- one recognizer family with its existing helper path
- one mechanical migration already fully specified

Do not combine implementation, semantic archaeology, broad regression
investigation, CI cleanup, and unrelated documentation into one session.

## Default budgets

Unless a task explicitly says otherwise:

- target 10-20 minutes of active agent work
- at about 20 minutes, prefer checkpointing over broadening the investigation
- if visible context usage approaches 60-65%, finish only an already-nearby
  compile/commit boundary; otherwise checkpoint and start a fresh chat
- allow at most one targeted scope expansion caused by a concrete compile error
  or focused test failure
- do not spend the remainder of a session on "while here" cleanup

Long compiler or planner runtime is not itself a reason to consume more context.
Wait for the requested command, record its result, and avoid starting parallel
investigations.

## Required task packet

Implementation prompts should be short but concrete. Include:

1. base SHA and branch
2. exact files and functions to edit
3. ordered edits, in compile-safe sequence
4. existing helpers/APIs to use
5. semantic invariants that must not change
6. explicit out-of-scope files/families
7. exact focused tests/checks
8. stop conditions
9. required commit/push behavior

Prefer instructions such as:

> In `foo::bar`, replace the legacy mask with
> `identity_mask_in(&view.variant)`; iterate through active IDs with
> `iter_in(...)`. Then update the two named callers.

Avoid instructions such as:

> Genericize this subsystem and update directly required consumers.

"Directly required consumers", "as applicable", "remaining relevant paths", and
similar phrases delegate scope discovery back to the agent and should be used
only when that discovery is intentionally part of the task.

## Reading and search discipline

The task packet should point at exact symbols so Codex can start there.

During execution:

- read the named functions, not whole large files unless necessary
- use exact-symbol search for callers rather than broad conceptual search
- prefer a narrow line range around the symbol over printing the whole file
- inspect `git diff --stat` first, then per-file diffs; do not repeatedly dump
  the whole branch diff into context
- redirect verbose compiler/test output to a log and inspect the first useful
  error or a short targeted tail
- avoid `--nocapture`, replay traces, and profiler output unless the requested
  diagnostic needs them
- do not read architecture audits, replay reviews, roadmap history, or upstream
  sources "just in case"
- do not re-run repository inventories that the orchestrator already performed
- compilation may reveal an omitted caller; fix it only when it is a direct
  consequence of the specified API change
- if fixing the caller requires a new convention decision or another subsystem,
  stop and report it

Broad regex/code-generation rewrites are inappropriate for semantic Rust
migrations unless the task explicitly calls for them.

## Compile and validation cadence

Compile at logical boundaries, not after every call-site edit.

For a normal Rust batch:

1. make the coherent edit
2. run the exact focused test if one is assigned
3. run `bash scripts/check-scoped.sh <crate> ...` once when the batch is ready
4. run `git diff --check`
5. commit and push

`check-scoped.sh` supports `--test-only` and `--check-only`. Use
`--test-only` when a production-config `cargo check` already passed at the
same coherent API boundary and the remaining risk is the focused lib test. Use
`--check-only` when no focused test is relevant. Keep the default combined
mode when both production-config compilation and the focused test still matter.

A plain `cargo check -p <crate> --locked` is useful between genuinely distinct
API boundaries, but repeated identical checks after tiny edits are wasted time.

Do not run `scripts/check.sh --fast`, strict workspace Clippy, all-targets,
MSRV, Python, Hawk, or broad replay suites by default. Pull-request CI owns those
routine broad gates. Run a broader local command only for a named risk or an
explicit request.

If CI later exposes a mechanical lint/format issue, fix that separately. Do not
reopen a large semantic Codex session merely to satisfy a trivial lint.

## Tests

Prefer existing reviewed regressions and existing test helpers.

For one implementation batch, normally add zero to two focused tests. For a
whole vertical slice, prefer a small set of semantic contracts rather than a new
large synthetic harness.

Fast deterministic H-Group migration contracts use a `contract_` test-name
prefix. PR CI runs that prefix directly. New migration contracts should be
deterministic, avoid planner wall-clock budgets, and assert semantic state rather
than end-to-end move timing.

A new test should justify its scaffolding cost. If the test setup is becoming
larger than the production change or starts recreating a mini framework, stop
and reuse existing helpers or ask for a smaller assertion surface.

Convention semantics still follow the provenance rules in `docs/testing.md`.
Artificial inputs are good for physical/algorithmic invariants, but they do not
become authority for an invented convention meaning.

Run only the exact replay regressions named in the task. Deadline-sensitive
planner/action-parity tests are diagnostics, not default migration gates; prefer
a deterministic `contract_` assertion for the semantic fact being changed. If
a named replay regression fails, capture the first useful failure and report it.
Do not automatically:

- run alternate replay suites
- modify fixtures/assertions
- compare against base
- profile the planner
- search neighboring failures

Those are separate diagnostic tasks when needed.

## Stop conditions

Stop implementation and report instead of widening scope when any of these
occurs:

- the worktree is unexpectedly dirty
- the branch/base is not what the task expects
- a required change falls outside the named boundary and is not a trivial direct
  caller
- a new semantic or hidden-information decision is required
- the obvious fix would alter a reviewed fixture/assertion
- the task starts requiring a broad architecture refactor
- repeated compile/test fallout crosses subsystems
- the session reaches its time/context budget without being close to a coherent
  commit

A stop is a successful handoff when it prevents speculative work. Prefer a
coherent compile-safe commit. If the budget expires mid-edit and no such boundary
exists, do not burn another long session forcing one: leave only the agent's own
in-scope edits visible, report the dirty files and blocker, and let a fresh
session continue them. Never mix unrelated pre-existing work into such a handoff.

## Branch and commit discipline

Multiple Codex chats may continue the same feature branch. Each completed batch
should end with a descriptive commit and push.

Do not reset, clean, or discard unrelated local state. A fresh session should
verify the expected branch/head and a clean worktree before editing.

Open one pull request for the vertical capability, not one PR per Codex batch.
By default, open the pull request only after all planned source-implementation
batches and focused local validation are complete. This avoids restarting the
whole PR CI matrix after every short-session push. Open an early draft only when
Linux CI or human review is itself needed to unblock the next batch.

## Model choice

Use the expensive semantic model only where judgment is actually needed.

- GPT-6.1 Sol, normally Medium: convention semantics, replay interpretation,
  hidden-information reasoning, API/lifecycle design, ambiguous regressions
- GPT-6 Luna: mechanical migrations, known call-site updates, formatting,
  Clippy, documentation, straightforward tests

If the orchestrator has already resolved the semantic question, default back to
the mechanical model.

## Handoff format

Keep the completion report compact:

- commit SHA
- exact changed files
- exact commands run and results
- any failed command with the first useful error
- any scope item deliberately deferred
- clean/dirty worktree state

Do not spend context producing a retrospective essay. The orchestrator can
inspect the actual diff, PR, and CI.

## Rule ownership

To prevent workflow drift:

- `AGENTS.md` owns mandatory coding-agent behavior and stop conditions
- `docs/validation.md` owns validation command semantics and CI coverage
- `docs/testing.md` owns test provenance and expensive-test policy
- this document owns orchestrator/session decomposition
- milestone plans such as `M4_VERTICAL_EXECUTION_PLAN.md` own milestone scope
  and acceptance, not duplicate command-level workflow rules

When these documents overlap, update the owning document and link to it rather
than copying a second detailed procedure.
