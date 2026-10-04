# Replay investigation workflow

This document contains the specialized agent workflow for replay disagreements,
convention bugs, fixture changes, and positions requiring human review. Read it
only for those tasks; the repository-wide instructions remain in
[`AGENTS.md`](../AGENTS.md).

Whenever reporting a replay disagreement, convention bug, or position requiring
human review, use the following instructions.

### Scope and replay checkpoints

Before analyzing a replay turn, run
`python3 scripts/replay_reviews.py SEED --turn N` and read the matching records
in `docs/replay-reviews/`. Search that directory for related principles, cards,
and hypothetical lines too: turn numbers alone are not stable across fixture
edits. Follow the recording and staleness rules in
`docs/replay-reviews/README.md`.

Record new user explanations there in the same change as their implementation or
fixture update. Include the user's actual reasoning, perspective, conditions,
and source excerpt, not just the preferred move. Link focused regression tests
when available; explicitly mark missing coverage. Never promote an assistant's
inference to a user ruling. Before asking the user a convention question, check
these records and accessible conversation history; explain any genuinely new
condition instead of asking them to repeat an established explanation.

Also read `docs/replay-reviews/principles.md` for cross-position rulings. An
implementation guard is not itself a convention rule: before treating a changed
actor, turn, or preceding action as a new exception, identify documentation or a
user ruling that makes that distinction relevant. If the guard merely omits a
case of an already-reviewed principle, fix the omission. For repeated mistakes,
add a principle-level record and a table-driven regression across the reviewed
positions, testing the inference and its downstream evaluation rather than only
the final move. Include negative controls so the fix does not assume hidden
identities or apply without its evidentiary conditions.

After fixture edits, inspect affected review anchors. Do not silently refresh
hashes or apply stale notes as current authority. Preserve historical reasoning,
mark superseded decisions, and re-anchor only after verifying that the same
reasoning applies. A missing record is not evidence that a turn was never
discussed. This ledger supplements, rather than replaces, executable regression
tests and the expert manifest's review boundary.

For a request to fix a specific bug, verify that position and related behavior,
then report the fix. Do not automatically search for the next disagreement or
expand into other replays. Fixture edits likewise do not authorize changing
subsequent moves. A full replay scan is a separate task or checkpoint.

When the user explicitly asks to continue through a replay, re-run that replay
from the beginning and proceed until a non-obvious convention/strategy question
or genuine blocker. Do not silently switch replays. Full replay scans belong in
this workflow, not every small fix or explanation.

Within the user's authorized bug-fixing scope, investigate and fix obvious bugs
without waiting for another prompt. Stop for a non-obvious convention or
strategy question and provide the review context below. Do not change fixtures
or expected moves merely to obtain agreement.

An unexplained disagreement is an investigation task, not automatically a
request for user input. Do not stop merely because a candidate has a higher
heuristic score, its projection ends at an unknown identity, or the next bug is
in a different helper. Inspect scoring terms, actual projected actions, and
uncertainty handling, then fix clear implementation errors and continue. Unknown
cards must remain unknown; an unfinished forecast is neither proof that an
action is good nor grounds to reject it automatically.

When an explicitly requested replay investigation ends with an unresolved
disagreement, state the specific convention/strategy question that requires the
user's judgment (and why the documentation and already-reviewed expectations do
not answer it), or the concrete external blocker. If no such question or blocker
exists, continue working rather than asking the user to authorize the same
investigation again.

For that replay investigation, report the next unresolved disagreement (with
link, seed, turn, candidate clues, and reasoning), agreement for the tested
replay/range, or the concrete blocker. For localized fixes, report the fix and
focused validation; do not claim untested replay-wide agreement.
Documentation/workflow-only requests do not authorize additional engine fixes.

### Replay Link

Always include a generated clickable Hanab Live replay link, the seed, the
one-based turn, and the competing actions or concrete issue. This applies even
when reporting status rather than explicitly asking a question.

The expert replay comparison prints a link on disagreement. For a ready-to-paste
Markdown link, generate it with:

```bash
scripts/generate-hanab-live-link.sh path/to/game.json --turn 1
```

Use Hanab Live's **one-based** turn number.

Copy the generator's complete Markdown link verbatim into the response. Never
retype, reconstruct, shorten, or manually edit the compressed URL, including its
turn fragment; rerun the generator with the desired turn instead. The generator
checks that its payload decodes to the original player count, deck, actions,
variant, and seed before printing. That check protects generation, not later
transcription: compare the final response link against the tool output before
sending. If using the comparison tool's URL directly, copy that URL verbatim.

### Candidate Clues

Do not enumerate all valid or considered clues when reporting a disagreement.
Focus on the fixture move and the engine's preferred move; mention another
candidate only when it is needed to explain the disagreement.

### Move Reasoning

Whenever the engine disagrees with a fixture, include the projected lines for
both the fixture move and the engine's preferred move. Identify the players and
actions in each line, and clearly mark conditional assumptions and where the
projection stops. If a move was rejected or has no available projection, explain
that limitation rather than inventing a line.

Explain why the engine prefers its move over the fixture move, using the actual
comparison evidence and both projected lines.

Use the built-in explanation before adding temporary diagnostics:

```bash
cargo run --release -p hanabi-cli --bin hanabi-engine -- \
  analyze path/to/game.json --live-turn 15 --convention h-group --h-group-level max \
  --objective perfect-score --explain --lines 2
```

Here `--live-turn 15` means Hanab Live turn 15 (`--turn 14` is equivalent). Use
`--format json` for structured evidence. Reports include the fixture action when
evaluated, all admission/rejection decisions, and recorded pairwise comparisons.
See [decision explanations](explanations.md). Do not treat display order as
a strict ranking or a forecast discard as necessarily forced.

Use repeatable `--candidate purple:Donald` / `--candidate 4:Alice` selectors to
include the specific competing actions, or `--lines all`. Inspect
`projectedDecisions` for a future player's actual alternatives and
`comparisons[].evidence.actualBasis` for the operands and horizon actually used.
Do not substitute raw endpoint values for normalized comparison values. Generic
exclusion classifications and unrecorded exact principal variations are
explicitly marked; never present missing evidence as a proven explanation.

