---
description: Pre-release checklist for a new starsight version (0.3.x → 0.3.x+1 or 0.3.x → 0.4.0). Verifies version bump, CHANGELOG entry, all quality gates, snapshot invariant, and gallery / showcase regeneration.
argument-hint: "<new-version>"
disable-model-invocation: true
---

# /release-prep — pre-release checklist

User-invoked only (Claude does not autonomously cut releases).

## Inputs

`$ARGUMENTS` should be the target version, e.g. `0.3.1` or `0.4.0`. Bail if missing.

## Checklist

Run each, stop on the first failure, and report exactly which step failed.

1. **Workspace version is `<new-version>`** — `grep '^version' Cargo.toml` should show the target. If not, ask the user to bump (don't bump silently).
2. **CHANGELOG has an entry for `<new-version>`** — `grep -n "## \[$new_version\]" CHANGELOG.md` should hit. If not, the user owes a CHANGELOG entry.
3. **`cargo fmt --all --check`** — clean.
4. **`cargo clippy --workspace --all-targets --all-features -- -D warnings`** — clean.
5. **`cargo check --workspace --all-features`** — clean.
6. **`cargo test --workspace`** — all green.
7. **`RUSTDOCFLAGS="-D missing-docs -D rustdoc::broken-intra-doc-links" cargo doc --workspace --no-deps --all-features`** — clean.
8. **`cargo insta test --workspace --check --unreferenced reject`** — no pending or orphan snapshots.
9. **`cargo xtask gallery`** — succeeds. PNGs in `target/gallery/`.
10. **`cargo xtask showcase`** — succeeds. `showcase/` is fresh.
11. **`cargo package -p starsight --no-verify`** — packages cleanly. (Repeat for each member crate that publishes; the facade is `starsight`, layers may not all publish.)

## After the checklist passes

Don't tag, don't push, don't `cargo publish` from this skill. Print:

```
All 11 checks passed for <version>.
Suggested next: git tag v<version> && git push --tags && cargo publish -p starsight
(User must run those manually.)
```
