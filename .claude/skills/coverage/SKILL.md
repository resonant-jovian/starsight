---
description: Run line-coverage on the workspace via cargo-llvm-cov, mirroring the live-assets.yml CI command. Use when the user asks about coverage, missing tests, or what's covered. Never use cargo tarpaulin — it OOMs locally and is banned.
argument-hint: "[--html | --json | --report] [-p <crate>]"
---

# /coverage — workspace line coverage

Source of truth: `.github/workflows/live-assets.yml` `coverage` job. Local runs must match so % matches `assets/status/coverage.json`.

## Canonical command

```bash
cargo llvm-cov --workspace --all-features --locked --exclude xtask --lcov --output-path lcov.info
```

Then surface the % via:

```bash
cargo llvm-cov report --json --output-path coverage-summary.json
jq '.data[0].totals.lines.percent' coverage-summary.json
```

`xtask` is excluded — it's build tooling, not library surface; including it slows the run without changing what users see.

## Argument forms

- *(no args)* — run the canonical lcov command, then print the line %.
- `--report` — `cargo llvm-cov report` against the existing `lcov.info` (no rebuild).
- `--html` — `cargo llvm-cov --workspace --all-features --locked --exclude xtask --html` and print the `target/llvm-cov/html/index.html` path.
- `--json` — emit `coverage-summary.json` for diffing.
- `-p <crate>` — narrow to one crate (skip `--workspace --exclude xtask`); useful when iterating on a single layer.

## Install gate

If `cargo llvm-cov --version` fails, install with `cargo install cargo-llvm-cov` (or `cargo binstall cargo-llvm-cov`). Don't fall back to tarpaulin under any circumstance — the user banned it after repeated OOM crashes under concurrent load.

## Reporting

If coverage drops vs. the committed `assets/status/coverage.json`, name the regression:

```
coverage: 96.42% (was 96.71% — regression of 0.29pp)
```

If it improves, lead with the delta. Don't dump the full lcov; show top-3 lowest-covered files via `cargo llvm-cov report` if the user asks where the gaps are.

## Why llvm-cov, not tarpaulin

- Tarpaulin's instrumented release builds OOM on this workspace under concurrent load — user-banned.
- `live-assets.yml` already uses `cargo-llvm-cov`; matching it locally means the % you see matches the badge on the README.
- `cargo-llvm-cov` uses the same source-based coverage as `rustc -C instrument-coverage`, so results are stable across runs.
