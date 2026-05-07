---
description: Run the full quality-gate sequence (fmt → clippy → check → test) in order, stopping on first failure. Use after any code change before claiming work is done.
argument-hint: "[-p <crate>]"
---

# /check — quality gates

Run in order, stop on first failure, report which step failed and why.

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features
cargo test --workspace
```

When `$ARGUMENTS` contains `-p <crate>` (e.g. `-p starsight-layer-3`), narrow `clippy`, `check`, and `test` to that crate; `fmt --check` always runs workspace-wide because it's cheap.

## Failure handling

- **fmt fails** → run `cargo fmt --all` to fix, then re-run `/check`. Don't proceed past fmt.
- **clippy fails** → fix the lint, don't suppress unless the lint is genuinely wrong (and then add a code comment explaining why).
- **check fails** → real type error; fix, don't paper over with `#[allow]`.
- **test fails** → if the failure is a snapshot diff and intentional, run `/snap --update`; otherwise fix the regression.

## Reporting

If something fails, show the first ~30 lines of the offending command's stderr. Don't dump the whole output.

If everything passes, one line: `All four gates passed (fmt / clippy / check / test).`

## Why this order

`fmt` is fastest and cheapest. `clippy` catches what `check` doesn't. `check` is faster than `test` for catching compile errors. `test` is last because it's the slowest. Stopping on first failure keeps the iteration loop tight.
