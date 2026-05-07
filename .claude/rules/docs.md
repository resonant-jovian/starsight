---
paths:
  - "CHANGELOG.md"
  - "**/Cargo.toml"
---

# CHANGELOG & version bumps

## CHANGELOG entries

One bullet per public-facing change. Group under the conventional sections:

- **Added** — new public API
- **Changed** — behavior changes to existing public API
- **Fixed** — bug fixes (the bulk of recent commits)
- **Docs** — README / rustdoc / examples
- **Internal** — refactors, build, CI (only if user-visible, otherwise skip)

Match the tense of existing entries (recent CHANGELOG.md is the source of truth). Reference the relevant commit hash inline where the bullet is non-obvious: `Auto-detect axis-aligned paths and disable AA (fb984d8).`

## Version bumps

Workspace version is at `[workspace.package].version` in the root `Cargo.toml`. All member crates inherit via `version.workspace = true`. **Bump in one place only.**

- **Patch** (0.3.x → 0.3.x+1): bug fixes only, no API changes
- **Minor** (0.3.x → 0.4.0): new features, backward-compatible API additions
- **Major** (pre-1.0): breaking API changes are allowed in any minor bump per the README's pre-release notice

## MSRV

`rust-version = "1.89"` is in `[workspace.package]`. Bumping it is a breaking change pre-1.0 (downstream pins). Keep it in sync with what the code actually requires; verify with `cargo +1.89 check --workspace`.

## docs.rs

`[package.metadata.docs.rs]` in member crates controls how docs.rs builds. The repo uses `all-features = true` so the polars feature surface ships in the rendered docs. Don't add features that aren't safe to enable in a docs build.
