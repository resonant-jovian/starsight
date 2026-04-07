# Contributing to starsight

Thanks for your help improving **starsight**! Contributions at every level are welcome — whether you're new to Rust or a seasoned reviewer. **No contribution is too small.**

> [!NOTE]
> Formal contribution rules will land at the first stable release (`1.0.0`). Until then this document captures the lightweight conventions in use.

## Code of Conduct

The **starsight** project adheres to the [Rust Code of Conduct][coc]. This describes the _minimum_ behaviour expected from all contributors. Report violations to [albin@sjoegren.se](mailto:albin@sjoegren.se).

[coc]: https://github.com/rust-lang/rust/blob/master/CODE_OF_CONDUCT.md

## Where to put what

The workspace is organized into seven layers + a facade. The "[which crate do I edit?](.spec/STARSIGHT.md#which-crate-do-i-edit)" table in the spec maps every task type to the right file. Quick summary:

| Task | Crate | File |
|---|---|---|
| Primitive types (`Color`, `Point`, ...) | `starsight-layer-1` | `src/primitives.rs` |
| Drawing primitives (`Path`, `PathStyle`) | `starsight-layer-1` | `src/paths.rs` |
| New rendering backend | `starsight-layer-1` | `src/backends/<category>.rs` |
| Scales / ticks / axes / coordinates | `starsight-layer-2` | `src/{scales,ticks,axes,coords}.rs` |
| New mark type | `starsight-layer-3` | `src/marks.rs` (single file with section dividers) |
| Statistical transform | `starsight-layer-3` | `src/statistics.rs` |
| Layout / faceting / legends | `starsight-layer-4` | `src/{layouts,facets,legends,colorbars}.rs` |
| `Figure` builder & `plot!` macro | `starsight-layer-5` | `src/figures.rs` and `starsight/src/lib.rs` |
| Interactivity (hover, zoom, pan) | `starsight-layer-6` | `src/{hovers,zooms,pans,selections}.rs` |
| Animation / export | `starsight-layer-7` | `src/{animations,exports,prints,gifs,terminals,webs}.rs` |
| Example program | root workspace | `examples/<name>.rs` |
| Build automation | `xtask` | `xtask/src/<command>.rs` |

The dependency rule: each layer depends only on layers below it, enforced by `Cargo.toml`. Don't try to import upward.

## Naming convention

To satisfy `clippy::module_name_repetitions` (with `allow-exact-repetitions = false` in `.clippy.toml`), module names are pluralized so type names don't collide with the parent module. Use `marks::Mark`, not `mark::Mark`. See `.spec/STARSIGHT.md` for the full naming rule. New files should follow the pattern.

## Local development

```bash
# Compile everything
cargo check --workspace

# Run tests + snapshots
cargo test --workspace

# Lint (pedantic, deny-all)
cargo clippy --workspace --all-targets -- -D warnings

# Format
cargo fmt --all

# Doc build (with missing-docs as error)
RUSTDOCFLAGS="-D missing-docs" cargo doc --workspace --no-deps

# License audit
cargo deny check
```

If a snapshot test fails, review the diff with `cargo insta review` and accept it if intentional.

## Code conventions

- **Doc comments on every public item.** `cargo doc` is built with `-D missing-docs`. Trait methods that return `Result` need a `# Errors` section.
- **Section dividers in long files.** Match `starsight-layer-3/src/marks.rs`: `// ── ItemName ─────...` (~100 columns wide).
- **No `unsafe` in layers 3–7.** Layer 1 may use `unsafe` for FFI; nothing else should.
- **No `println!`/`eprintln!` in library code.** Use the `log` crate.
- **No `async` in the public API.**
- **No nightly-only features.**

The full set of hard rules lives in [`README.md`](README.md#hard-rules).

## Filing issues

A useful issue contains:
- **What you tried** — the actual code, not a description.
- **What you expected** — paste from docs if relevant.
- **What happened** — actual output, panic message, or rendered image.
- **Versions** — `cargo --version`, `starsight = "..."` line, OS.

For bug reports: a minimal `cargo new` reproducer is gold. For feature requests: cite a real use case, not "it would be nice if...".

## Need help?

Reach out to [albin@sjoegren.se](mailto:albin@sjoegren.se) for anything not covered here.

## LTS guarantees

After `1.0.0`: no LTS branch for now. Patch versions are bug-fix only and minor versions are additive. See _Versioning Policy_ below.

## Minimum Supported Rust Version (MSRV)

Rust **1.89** (edition 2024). Enforced by `rust-version = "1.89"` in `[workspace.package]`, so `cargo` refuses to build with an older toolchain instead of failing deep inside a dependency.

The current floor is set by `cosmic-text 0.18` (which declares `rust-version = 1.89`); any bump in cosmic-text or another core dependency that raises its MSRV will pull ours up too. The long-term policy is _latest stable minus two_, consistent with `wgpu` and `ratatui`. Bumping the MSRV is a minor version change post-`1.0.0`.

## Versioning Policy

With **starsight** ≥ `1.0.0`:

- **Patch** (`1.x.y`) releases contain bug fixes or documentation changes only. They should not change runtime behaviour.
- **Minor** (`1.x.0`) releases may add functionality, raise the MSRV, perform minor dependency updates, deprecate APIs, or refactor internals.
- **Major** (`x.0.0`) releases break the public API.

Per [Semantic Versioning 2.0](https://semver.org/).

Until `1.0.0` the API is unstable: **pin an exact version** in your `Cargo.toml`.
