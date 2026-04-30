---
description: Manage insta snapshot tests in starsight-layer-5. Use after rendering changes to inspect or accept new snapshots. Default behavior is review (read-only); pass --update to write.
argument-hint: "[--update | --check]"
---

# /snap — snapshot management

Every layer has a `tests/snapshot.rs`. Layer-1 and layer-5 are populated today; the rest are placeholders that fill out as the layer stabilizes. SVG is the default; PNG (`assert_binary_snapshot!`) only for backend-pure tests (layer-1 has one of each).

## Modes

- **No args** (default): run snapshot tests; if any are pending, list them and stop. Don't write.
- **`--check`**: CI-style invariant. Runs `cargo insta test --workspace --check --unreferenced reject`. Fails on any pending or orphan `.snap`.
- **`--update`**: run with `INSTA_UPDATE=always` so any pending snapshot is written in place. Use after a deliberate visual change.

## Flow

```bash
# Default — see what changed (all layers)
cargo test --workspace --test snapshot

# Or one layer
cargo test -p starsight-layer-1 --test snapshot
cargo test -p starsight-layer-5 --test snapshot

# CI invariant
cargo insta test --workspace --check --unreferenced reject

# Update locally after an intentional change
INSTA_UPDATE=always cargo test --workspace --test snapshot
```

## After running

If snapshots are pending and you ran the default mode (no `--update`), list each changed snapshot file and a one-line summary of *what* changed (read the diff yourself, don't just dump it). Ask the user to confirm before re-running with `--update` — visual regressions are harder to catch than type errors.

If you ran `--update`, run `cargo insta test --workspace --check --unreferenced reject` immediately after to confirm no orphans were left.

## Note

`cargo xtask snapshots` is a stub and shouldn't be used. The xtask source is `xtask/src/snapshots.rs` if you want to fill it in, but until that's done, use `cargo insta` directly.
