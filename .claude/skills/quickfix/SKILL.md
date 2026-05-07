---
description: Structured bug-fix loop matching the recent fix() commit pattern. Use when the user reports a visual regression, a crash, or any narrow bug. Reproduces in a test, fixes, runs gates, suggests a commit.
argument-hint: "<bug description or beads issue id>"
---

# /quickfix — bug-fix loop

The recent commits set the cadence: `fix(scope): subject` — short, specific, what changed (`fb984d8`, `6c26aed`, `14fef6c`, `b38698e`, `ee96b22` are templates).

## The loop

1. **File or claim a beads issue.** If `$ARGUMENTS` looks like an ID (e.g. `starsight-abc`), `bd update $ARGUMENTS --claim`. Otherwise `bd create --title="<short subject>" --description="<bug as reported>" --type=bug --priority=2` and claim the new id.
2. **Reproduce.** Pick the smallest surface that exhibits the bug:
   - Visual regression in a mark? → write or modify a snapshot test in `starsight-layer-5/tests/snapshot.rs`.
   - Crash / panic / wrong number? → write or modify a unit test in the relevant layer.
   - Edge-case backend behavior? → start from `starsight-layer-1/tests/` or layer-3 mark tests.
   Confirm the test fails before touching production code.
3. **Fix.** Implement the smallest change that makes the test pass. Don't refactor surrounding code.
4. **Re-run gates.** `/check`. If clippy or fmt fails on touched files, fix and re-run.
5. **Snapshots.** If the fix affects rendering, run `/snap` (review mode). For an intentional visual change, `/snap --update`. Inspect each diff before accepting.
6. **Commit.** One commit, matching the `fix(scope): subject` convention. Body is optional; only add when the *why* needs explaining.
7. **Close the issue.** `bd close <id>` — or `bd close <id> --reason="…"` if there's a specific resolution worth recording.

## Commit message templates

- `fix(backend): auto-detect axis-aligned paths and disable AA on them`
- `fix(pie): suppress axes for pie-only figures + auto-contrast slice labels`
- `fix(candlestick): leftmost/rightmost bars no longer half-clip the plot edge`
- `fix(legend): contrasting border around the legend rect`

Pick the scope that matches the touched layer/mark. Subject is a single imperative clause. No trailing period.

## Anti-patterns

- Don't add code comments explaining the fix — the commit message is the right place.
- Don't bundle the fix with unrelated cleanup. Two commits, not one.
- Don't claim the bug is fixed without a passing test that previously failed.
