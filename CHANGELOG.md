# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0]

### Added

- `BarMark` per-bar bases via `bases: Option<Vec<f64>>` and `bases(Vec<f64>)` builder
- `BarMark` per-bar colors via `colors: Option<Vec<Color>>` and `colors(Vec<Color>)` builder
- `BarMark::connectors(bool)` builder — draws thin gray (#888) 1px lines between consecutive bars at running-total level (vertical orientation only)
- `PointMark` per-point colors via `colors: Option<Vec<Color>>` and `colors(Vec<Color>)` builder
- `PointMark` per-point radii via `radii: Option<Vec<f32>>` and `radii(Vec<f32>)` builder
- `PointMark::alpha(f32)` builder — mark-wide alpha multiplier applied at draw time
- `HeatmapColorScale` enum (`Linear`, `Log`) and `HeatmapMark::color_scale()` / `log_scale()` builders — log path normalizes via `log10` with a small epsilon to handle zero/negative cells
- New showcase examples (`.spec/SHOWCASE_INPUTS.md`):
  - `examples/basics/bubble_scatter.rs` (spec #3) — wine-shaped data with per-point continuous color (RdPu) and size, alpha 0.5
  - `examples/basics/movie_heatmap.rs` (spec #16) — Rotten Tomatoes × IMDB cross-tab with log-scale viridis
  - `examples/scientific/laser_plasma.rs` (spec #7, single-panel) — stimulated Raman scattering electron phase space, log-scale viridis
- New snapshot tests: `snapshot_bar_waterfall`, `snapshot_point_per_point_color_size`, `snapshot_heatmap_log`

### Changed

- **Breaking:** `BarMark.base: Option<f64>` renamed and retyped to `bases: Option<Vec<f64>>`. The `.base(f64)` builder is kept as a single-broadcast convenience (stores `Some(vec![b])`), so callers using only the builder are unaffected. Direct struct-literal construction must update the field name.
- **Breaking:** `BarMark.color: Option<Color>` renamed and retyped to `colors: Option<Vec<Color>>`. Same broadcast convenience for `.color(Color)`.
- **Breaking:** `PointMark.color: Color` renamed and retyped to `colors: Option<Vec<Color>>`. `.color(Color)` builder kept; struct-literal construction must migrate.
- **Breaking:** `PointMark.radius: f32` renamed and retyped to `radii: Option<Vec<f32>>`. `.radius(f32)` builder kept.
- `BarMark::render` (the no-context fallback) now honors per-bar bases (previously ignored them and rendered every bar to baseline 0).
- `PointMark::render` now batches consecutive points by `(color, radius)` so per-point styling needs only one `draw_path` call per unique combination.
- `examples/composition/waterfall_bar.rs` rewritten as a single `BarMark` with per-bar `bases`/`colors` and `connectors(true)`.
- `examples/Cargo.toml`: prismatica dependency now enables the `colorbrewer` feature (RdPu used by `bubble_scatter`).

### Fixed

- `BarMark` waterfall layout (`starsight-7h9`) — multiple `BarMark` instances each with `x.len() == 1` collapsed onto a single x-position because `Figure::category_labels()` only reads the first mark's labels. The fix is structural: per-bar `bases`/`colors` let one `BarMark` carry the entire waterfall, so the category axis spans all 10 labels naturally.

## [0.2.0]

### Added

- `StepMark` with `StepPosition` enum (Pre/Mid/Post) for step charts
- `HistogramMark` with automatic binning via `BinMethod` (Sturges, Freedman-Diaconis, Scott, Count, Width)
- `BandScale` for categorical x-axis data in bar charts
- `DataSource` trait with `SliceSource` and `VecSource` implementations
- Chart type auto-inference via `infer_chart_kind()` - automatically chooses Line/Point/Bar/Histogram
- `Color::cycle_next()` using prismatica's Tableau10 palette for default color cycle
- Title rendering above plot area
- Axis labels (x_label, y_label) rendering
- SvgBackend opacity support (fill-opacity, stroke-opacity attributes)
- `AreaMark` baseline support (Zero or Fixed value)
- Facade exports: `StepMark`, `HistogramMark`, `BarMark`, `BinMethod`, `BinTransform`
- Facade re-exports: `AreaMark`, `AreaBaseline`, `Orientation`, `BarRenderContext`, `DataExtent`; new `starsight::inferences` and `starsight::renders` modules surfacing layer-5 chart-kind inference and chrome render helpers; `Orientation` and `ChartKind` added to `prelude`

### Changed

- BandScale replaces manual band calculation for categorical data
- AreaMark now properly closes path to baseline using coordinate system
- SvgBackend draws quadratic curves (QuadTo)
- Color.cycle_next() now uses prismatica::d3::TABLEAU10_PALETTE

### Fixed

- SvgBackend was ignoring PathStyle.opacity - now correctly applies opacity/fill-opacity/stroke-opacity
- AreaMark baseline rendering (was using hardcoded pixel positions)
- StepMark NaN gap handling

## [0.1.1]

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
