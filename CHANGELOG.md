# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> [!NOTE]
> No release has been published yet. Until `0.1.0` ships, every entry below describes the initial state of the codebase. `Changed` and `Removed` sections will start tracking diffs from `0.1.0` onward.

## [Unreleased]

### Added

- Layered workspace architecture: 7 layer crates (`starsight-layer-1` through `-7`) plus a `starsight` facade and a dev-only `xtask`.
- `DrawBackend` trait with object-safe method shape (no generics, no `Self` in return position) so backends can be selected at runtime.
- CPU raster backend (`SkiaBackend`) via `tiny-skia`, with PNG encoding and `cosmic-text` glyph rasterization.
- Vector backend (`SvgBackend`) producing valid SVG documents.
- Wilkinson Extended tick algorithm in `starsight-layer-2::ticks` — novel Rust implementation, property-tested with `proptest`.
- `LinearScale`, `Axis`, and `CartesianCoord` — the data → pixel pipeline.
- `LineMark` and `PointMark` with NaN-gap handling and circular cubic-Bézier point glyphs.
- `Figure` builder with chainable `.title()`, `.x_label()`, `.y_label()`, `.add()`, and `.save()`.
- `plot!` macro forwarding to `Figure::from_arrays`.
- Three facade access patterns: `starsight::prelude::*`, semantic modules (`marks`, `backends`, `scales`, ...), and Latin layer aliases (`background`, `modifiers`, `components`, ...).
- Plural module naming convention workspace-wide so type names never collide with their parent module under `clippy::module_name_repetitions` with `allow-exact-repetitions = false`.
- Consolidated `primitives.rs` containing `Color`, `ColorAlpha`, `Point`, `Vec2`, `Rect`, `Size`, `Transform` with `// ── ItemName ──...` section dividers.
- `paths.rs` with the drawing primitives (`Path`, `PathCommand`, `PathStyle`, `LineCap`, `LineJoin`) extracted from the backend trait file for clearer ownership.
- Per-category backend files: `backends/{rasters, vectors, prints, gpus, terminals}.rs`.
- Empty stub files (with `//!` module docs and `// ──` section dividers) covering the lifespan of the project: `scenes.rs`, `statistics.rs`, `aesthetics.rs`, `positions.rs`, `layouts.rs`, `facets.rs`, `legends.rs`, `colorbars.rs`, `inferences.rs`, `sources.rs`, all of layer-6 (`hovers`, `zooms`, `pans`, `selections`, `streams`, `windows`), all of layer-7 (`animations`, `exports`, `prints`, `webs`, `gifs`, `terminals`), and `xtask` sub-commands.
- Doc comments on every public item; `cargo doc -- -D missing-docs` is clean.
- `# Errors` sections on every public function returning `Result<T>`.
- Insta snapshot tests for the SkiaBackend, SvgBackend, line marks (basic / multi-series / NaN-gap), and scatter marks (basic / sized).
- `deny.toml` license allowlist populated with the SPDX identifiers actually used by the dependency tree (MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, 0BSD, Unicode-3.0, GPL-3.0, MPL-2.0, CC0-1.0, Apache-2.0 WITH LLVM-exception).
- `CITATION.cff` with author ORCID for the GitHub "Cite this repository" button.
- `README.md` in hook-first flow: badges, Quickstart, language-comparison table, code gallery, real snapshot screenshots, layered architecture diagram, features status, feature flags, ecosystem, roadmap, hard rules, footer.
- `CONTRIBUTING.md` with the where-to-put-what crate routing table, naming convention, local-development command list, code conventions, and filing-issues guide.
- Pre-release scaffolding: `LICENSE`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, GitHub issue and pull-request templates, `.github/workflows/{ci, release, coverage, gallery, snapshots}.yml`.
- `rust-version = "1.89"` in `[workspace.package]` so `cargo` refuses too-old toolchains with a clear message instead of failing inside a transitive dependency. Floor is set by `cosmic-text 0.18` (which declares `rust-version = 1.89`).
