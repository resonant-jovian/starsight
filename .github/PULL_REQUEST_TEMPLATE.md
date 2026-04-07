<!--
Thank you for your Pull Request. Please provide a description above and review
the requirements below.

Bug fixes and new features should include tests.

Contributors guide: https://github.com/resonant-jovian/starsight/blob/master/CONTRIBUTING.md

The contributors guide will include instructions for running rustfmt and building the
documentation, which requires special commands beyond `cargo fmt` and `cargo doc`.
-->

## Motivation

<!--
Explain the context and why you're making that change. What is the problem
you're trying to solve? In some cases there is not a problem and this can be
thought of as being the motivation for your change.
-->

## Solution

<!--
Summarize the solution and provide any necessary context needed to understand
the code change.
-->

## Pull request checklist

Every box in **required** sections must be checked before merge. **Optional** sections are recommended but not blocking — tick what you ran, strike through what you skipped.

---

### Code quality (required)

- [ ] Formatting passes
  ```bash
  cargo fmt --all --check
  ```
  Expected: no output, exit code 0.

- [ ] Clippy passes
  ```bash
  cargo clippy --workspace --all-features -- -D warnings
  ```
  Expected: no warnings, no errors, exit code 0.

- [ ] TOML formatting passes
  ```bash
  taplo check
  ```
  Expected: all files valid, exit code 0.

- [ ] No banned functions in library code
  ```bash
  find . -name '*.rs' \
    -not -path '*/tests/*' \
    -not -path '*/target/*' \
    -not -path '*/xtask/*' \
    -not -path '*/examples/*' \
    | xargs grep -n -E '\.(unwrap|expect)\(|panic!\(|todo!\(|println!\(|eprintln!\(' \
    | grep -v '#\[cfg(test)\]' \
    | grep -v 'mod tests' \
    | grep -v '// allowed:' \
    | grep -v '#\[test\]'
  ```
  Expected: zero lines of output. If genuinely needed, annotate with `// allowed: <reason>`.

- [ ] No unsafe in layers 3–7
  ```bash
  find ./starsight-layer-{3,4,5,6,7} -name '*.rs' \
    | xargs grep -n 'unsafe' \
    | grep -v '// allowed:'
  ```
  Expected: zero lines of output.

- [ ] Every new public item has a doc comment
  ```bash
  RUSTDOCFLAGS="-D missing_docs" cargo doc --workspace --no-deps 2>&1 | head -20
  ```
  Expected: builds successfully with no `missing_docs` warnings.

- [ ] Every new public type derives `Debug` and `Clone` at minimum
  Manual review — check all new `pub struct` and `pub enum` definitions in the diff.

### Compilation (required)

- [ ] Workspace compiles with all features
  ```bash
  cargo check --workspace --all-features
  ```
  Expected: `Finished` with no errors.

- [ ] Workspace compiles with no default features
  ```bash
  cargo check --workspace --no-default-features
  ```
  Expected: `Finished` with no errors.

- [ ] Each feature compiles independently
  ```bash
  cargo hack check --workspace --each-feature --no-dev-deps
  ```
  Expected: `Finished` for every feature. No compilation errors.

- [ ] Docs build with warnings as errors
  ```bash
  RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
  ```
  Expected: `Finished` with no errors.

### Tests (required)

- [ ] All tests pass
  ```bash
  cargo test --workspace
  ```
  Expected: `test result: ok` for every crate, zero failures.

- [ ] All tests pass with all features
  ```bash
  cargo test --workspace --all-features
  ```
  Expected: `test result: ok` for every crate, zero failures.

- [ ] All tests pass with no default features
  ```bash
  cargo test --workspace --no-default-features
  ```
  Expected: `test result: ok` for every crate, zero failures.

- [ ] Doc tests pass
  ```bash
  cargo test --workspace --doc
  ```
  Expected: `test result: ok`, zero failures. Needed because cargo-nextest skips doc tests.

- [ ] Snapshot tests are clean
  ```bash
  cargo insta test --check --unreferenced reject
  ```
  Expected: all snapshots match, no pending `.snap.new` files, no orphaned snapshots. Exit code 0.

- [ ] No flaky tests (run twice)
  ```bash
  cargo test --workspace && cargo test --workspace
  ```
  Expected: both runs pass with identical results.

- [ ] Every new function has at least one test
  Manual review — check that every new `pub fn` in the diff has a corresponding `#[test]`.

- [ ] Every new chart type has a snapshot test
  Manual review — check that any new Mark implementation has a snapshot in `tests/snapshots/`.

### Dependencies (required)

- [ ] cargo-deny passes all checks
  ```bash
  cargo deny check
  ```
  Expected: output ends with `advisories ok`, `licenses ok`, `bans ok`, `sources ok`.

- [ ] License check passes separately
  ```bash
  cargo deny check licenses
  ```
  Expected: `licenses ok`, exit code 0.

- [ ] No unused dependencies (fast check)
  ```bash
  cargo machete
  ```
  Expected: no unused dependencies found. If false positive, add to `[package.metadata.cargo-machete] ignored` in Cargo.toml.

- [ ] No new dependency without a comment
  ```bash
  git diff main -- '**/Cargo.toml' | grep '^+' | grep -E 'workspace = true|version = "' | head -20
  ```
  Expected: every new dependency line has an inline `#` comment explaining why. Manual review.

- [ ] Optional dependencies are behind a feature flag
  ```bash
  git diff main -- '**/Cargo.toml' | grep -A1 'optional'
  ```
  Expected: every new optional dependency has a corresponding feature entry. Manual review.

### Semver and API (required)

- [ ] cargo-semver-checks passes (skip if not yet published)
  ```bash
  cargo semver-checks --workspace
  ```
  Expected: `no semver violations found` or `not yet published` for each crate.

- [ ] No public items removed without deprecation
  ```bash
  cargo semver-checks --workspace 2>&1 | grep -i 'removed\|missing'
  ```
  Expected: zero lines of output.

- [ ] New public enums and config structs have `#[non_exhaustive]`
  ```bash
  git diff main -- '*.rs' | grep -B2 '^+pub enum\|^+pub struct' | head -30
  ```
  Expected: every new `pub enum` and config `pub struct` has `#[non_exhaustive]` above it. Pure math types (Point, Vec2, Rect, Color) are exempt. Manual review.

- [ ] No dependency types exposed in public API
  ```bash
  git diff main -- '*.rs' | grep '^+.*pub fn' | grep -E 'tiny_skia::|cosmic_text::|polars::|wgpu::|ratatui::'
  ```
  Expected: zero lines. Wrap external types in own types.

- [ ] Builder methods use `&mut self -> &mut Self` pattern
  Manual review — check new builder methods in the diff.

- [ ] String parameters accept `impl Into<String>`
  Manual review — check new `pub fn` signatures that take string arguments.

### Documentation (required)

- [ ] Commits follow Conventional Commits
  ```bash
  git log main..HEAD --format='%s' | grep -vE '^(feat|fix|refactor|test|docs|chore|perf|build|ci|style)(\(.+\))?!?:'
  ```
  Expected: zero lines of output — every commit matches the pattern.

- [ ] CHANGELOG.md updated (or git-cliff will generate it)
  ```bash
  git cliff --unreleased --strip header 2>/dev/null | head -20
  ```
  Expected: shows grouped entries for this PR's commits (Features, Bug Fixes, etc.).

- [ ] README.md updated if public API or features changed
  ```bash
  git diff main -- README.md | head -5
  ```
  Expected: non-empty diff if public API or features changed. Empty is fine for internal-only changes.

- [ ] `.spec/STARSIGHT.md` updated if architecture changed
  ```bash
  git diff main -- '.spec/STARSIGHT.md' | head -5
  ```
  Expected: non-empty diff if architecture, roadmap, or design decisions changed.

- [ ] New feature flags documented in README and Cargo.toml
  ```bash
  diff <(grep -oP '^\w+ = ' starsight/Cargo.toml | sort) \
       <(grep -oP '\| `\w+`' README.md | sed 's/| `//;s/`//' | sort)
  ```
  Expected: no differences — every feature in Cargo.toml appears in the README table.

### Git hygiene (required)

- [ ] Commit scopes use layer names
  ```bash
  git log main..HEAD --format='%s' | grep -oP '\(.+?\)' | sort -u
  ```
  Expected: scopes like `(layer-1)`, `(layer-2)`, `(primitives)`, `(scale)`, `(ci)`, etc.

- [ ] No merge commits
  ```bash
  git log main..HEAD --merges --oneline
  ```
  Expected: zero lines of output.

- [ ] Branch is up to date with main
  ```bash
  git fetch origin main && git merge-base --is-ancestor origin/main HEAD && echo "up to date"
  ```
  Expected: `up to date`.

### Release-specific (required only for version bumps)

- [ ] Version bumped in workspace Cargo.toml
  ```bash
  grep '^version' Cargo.toml
  ```
  Expected: shows the new version, e.g. `version = "0.2.0"`.

- [ ] cargo-release dry-run passes
  ```bash
  cargo release --workspace --no-publish --no-tag --no-push
  ```
  Expected: completes without error, lists all crates in dependency order.

- [ ] git-cliff generates correct changelog
  ```bash
  git cliff --bump --unreleased
  ```
  Expected: shows new version header with grouped commits.

- [ ] All snapshot baselines reviewed and accepted
  ```bash
  cargo insta test && cargo insta pending-snapshots
  ```
  Expected: zero pending snapshots.

- [ ] Tag format is correct
  ```bash
  echo "v$(grep '^version' Cargo.toml | head -1 | sed 's/.*= "//;s/"//')"
  ```
  Expected: prints `v0.x.y`. Use this as the git tag.

- [ ] MSRV still holds
  ```bash
  cargo +1.85.0 check --workspace --locked
  ```
  Expected: `Finished` with no errors on the declared MSRV toolchain.

---

### Optional — run when relevant, skip with ~~strikethrough~~

- [ ] **cargo-nextest** — faster parallel test runner (use instead of `cargo test` above if installed)
  ```bash
  cargo nextest run --workspace --all-features
  ```
  Expected: all tests pass. Note: does not run doc tests — still need `cargo test --doc` above.

- [ ] **cargo-mutants** — mutation testing on changed code (slow, run on math/scale/coordinate changes)
  ```bash
  git diff origin/main...HEAD > /tmp/pr.diff
  cargo mutants --in-diff /tmp/pr.diff --timeout 60
  ```
  Expected: zero **missed** mutants on changed code. **Caught** = good, **unviable** = fine, **timeout** = fine. Full workspace run (very slow, hours): `cargo mutants --workspace`.

- [ ] **cargo-llvm-cov** — code coverage (run on significant changes, always on releases)
  ```bash
  cargo llvm-cov --workspace --all-features --fail-under-lines 80
  ```
  Expected: coverage at or above 80%. HTML report: `cargo llvm-cov --workspace --all-features --html --open`.

- [ ] **cargo-udeps** — thorough unused dependency detection (needs nightly, slower than machete)
  ```bash
  cargo +nightly udeps --workspace --all-targets
  ```
  Expected: no unused dependencies. Run weekly or when dependency list changed significantly.

- [ ] **cargo-hack feature powerset** — deeper feature combination testing (slow, run before releases)
  ```bash
  cargo hack check --workspace --feature-powerset --depth 2 --no-dev-deps
  ```
  Expected: all 2-feature combinations compile. Full powerset is exponential — depth 2 is practical.

- [ ] **cargo-audit** — security advisory check with fix suggestions
  ```bash
  cargo audit
  ```
  Expected: no vulnerabilities found. Overlaps with `cargo deny check advisories` but also offers `cargo audit fix`.

- [ ] **cargo-expand** — debug macro expansion (use when plot! macro or derive macros misbehave)
  ```bash
  cargo expand -p starsight-layer-5 macros 2>/dev/null | head -50
  ```
  Expected: shows the expanded macro output. Requires nightly installed (not as default).

- [ ] **cargo-flamegraph** — profile rendering performance (use when optimizing hot paths)
  ```bash
  RUSTFLAGS="-C force-frame-pointers=yes" \
    cargo flamegraph --example profile_render -o flamegraph.svg -F 1997
  ```
  Expected: generates `flamegraph.svg`. Open in browser. Wide bars at the top = optimize these.

- [ ] **criterion benchmarks** — run when changing render pipeline, scale math, or layout
  ```bash
  cargo bench --workspace
  ```
  Expected: no regressions reported. Compare against baseline with `cargo bench -- --save-baseline pr`.

- [ ] **cargo-msrv verify** — explicit MSRV check (alternative to testing with pinned toolchain)
  ```bash
  cargo msrv verify
  ```
  Expected: MSRV verified as 1.85. Covered by `cargo +1.85.0 check` in the release section — this is the standalone tool alternative.

