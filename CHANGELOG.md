# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] 

### Added

- **Polar coordinate system.** `Coord` trait (object-safe via `Any` super-trait + `as_any()`) covering `CartesianCoord` and the new `PolarCoord`. `PolarCoord::inscribed(theta_axis, r_axis, plot_area)` carves an inscribed disk; `data_to_pixel(theta, r)` uses compass convention (theta = 0 up, increasing clockwise).
- `LogScale`, `SqrtScale`, `CategoricalScale` — `Box<dyn Scale>` subset that backs polar radial axes (`polar_radial_sqrt` is Florence Nightingale's value-as-area invariant) and log-scaled heatmap color bars.
- Polar `Axis` constructors: `polar_angular`, `polar_angular_categorical`, `polar_radial`, `polar_radial_sqrt`, `polar_radial_log`.
- `polar_ticks_{degrees,radians,categorical}` formatters in `starsight::ticks` — degree-suffixed labels, π-fraction labels reduced to lowest terms, categorical band-center positions matching `CategoricalScale`.
- `Scale::clone_box` + `impl Clone for Box<dyn Scale>` — enables `Axis: Clone`, which `Figure::polar_axes` uses to bundle the configured polar axes into the figure builder.
- `Figure::polar_axes(theta_axis, r_axis)` switches a figure to polar mode. The figure renders into a `PolarCoord` instead of a `CartesianCoord`, dispatching through `render_grid_lines`'s polar branch (radial spokes + concentric rings) and skipping cartesian axis chrome.
- `MultiPanelFigure` — uniform `(rows, cols)` grid of sibling `Figure`s with configurable padding. Each panel composes its own axes/title/legend independently; `Figure::render_within(viewport, backend)` is the parameterized dispatch point that translates layout output by the panel origin.
- `ContourMark` + `ContourMode { Isolines, FilledBands, FilledWithLines }` — marching-squares contours with optional colormap tinting per level. `Isolines` mode ships fully; `FilledBands` is API-stable but currently falls back to isoline rendering (polygon-tracing follow-up tracked separately).
- `Contour::compute(grid, &levels) -> Vec<Polyline>` — marching-squares extractor with average-of-corners saddle disambiguation (matplotlib default). New `Grid` (row-major scalar field) and `Polyline` (per-segment 2-point output) types in `statistics`.
- `ArcMark` — polar wedge mark for Nightingale coxcomb (#34), Gauge (#41), and Sunburst (#39 var C). Per-wedge `r_inner`/`theta_half_widths`/`start_offset` plus a default 8-color palette and per-wedge stroke.
- New showcase examples: `examples/scientific/{nightingale,gauge,contour_fields}.rs` and `examples/composition/donut_sunburst.rs` — covering `.spec/SHOWCASE_INPUTS.md` entries #22, #34, #39 var C, and #41.
- `ChartKind::Contour` variant; the enum becomes `#[non_exhaustive]` so future polar / 3D variants slot in without a breaking change.
- New snapshot tests: `snapshot_polar_grid_{linear,log,categorical}`, `snapshot_multipanel_2x2_basic`, `snapshot_contour_isolines`, `snapshot_arcmark_{full_nightingale,partial_gauge,nested_sunburst}`.
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
- `BoxPlotMark` + `BoxPlotGroup` — box-and-whisker chart with median line, whiskers, caps, and 1.5×IQR Tukey outliers. `palette` / `color` / `half_width` / `show_outliers` / `label` builders.
- `ViolinMark` + `ViolinGroup` + `ViolinScale { Area, Count, Width }` — kernel-density violins with optional inner box overlay, configurable bandwidth (Silverman / Scott / Manual), `cut` extension, and split-mode for paired comparisons.
- `PieMark` + `PieLabelMode` — solid pie or donut (`inner_radius` builder) with arc-bezier slice geometry, configurable start angle, label modes (`Percent` / `Value` / `None`), and a 6-color default palette.
- `CandlestickMark` + `Ohlc` — financial OHLC bars with up/down body color dispatch, vertical wicks, configurable `body_width` and `wick_width`.
- `BoxPlotStats::compute(&[f64])` — five-number summary with 1.5×IQR fences and outlier classification.
- `Kde { bandwidth, kernel }` — 1-D Gaussian kernel density estimator with `Bandwidth::{Silverman, Scott, Manual(f64)}` strategies. `evaluate_at` / `evaluate_grid` entry points.
- Bandwidth helpers: `silverman_bandwidth`, `scott_bandwidth`, `std_dev`. Promoted `percentile()` to `pub` and re-exported through the facade.
- `LegendGlyph { Line, Point, Bar, Area }` (`#[non_exhaustive]`) carried by `LegendEntry` — the legend now draws the right shape per mark (filled disk for `PointMark`, fill-rect for `BarMark`/`HistogramMark`/`HeatmapMark`, translucent area + top stroke for `AreaMark`, line stroke for `LineMark`/`StepMark`/fallback). `Mark::legend_glyph()` is the trait hook with `LegendGlyph::Line` as the default.
- `LayoutFonts { label, title }` shared between layout-pass reservations and render-time text drawing, removing the duplicated `12.0` / `16.0` literals between `figures.rs` and `renders.rs`.
- Polars data integration behind the `polars` feature flag (pulled forward from 0.11.0): `FrameSource` + `extract_f64` / `extract_f64_with_nulls` / `extract_strings` helpers + `From<LazyFrame> for FrameSource`. The `plot!` macro grows a DataFrame arm: `plot!(df, x = "col", y = "col", color = "col"?)` dispatches `LineMark` / `BarMark` / `PointMark` per column types and partitions by color column.
- New gallery binaries: `examples/composition/{box_plot,violin,pie,donut}.rs`, `examples/scientific/candlestick.rs`, and `examples/data/polars_integration.rs` (replaces the earlier placeholder).
- Snapshot tests for every new mark + the legend-glyph regression: `snapshot_boxplot_basic` / `_with_outliers` / `_palette`, `snapshot_violin_basic` / `_no_box` / `_split` / `_palette`, `snapshot_pie_basic`, `snapshot_donut_basic`, `snapshot_candlestick_basic` / `_custom_colors`, `snapshot_legend_glyph_dispatch`, and `snapshot_polars_line` / `_grouped_scatter` (gated on the `polars` feature).

### Changed

- **Breaking:** `Mark::render` and `Mark::render_bar` now take `coord: &dyn Coord` instead of `&CartesianCoord`. Cartesian marks call `require_cartesian(coord)?` at the top of their impl; polar marks (`ArcMark`) call `require_polar(coord)?`. Returns `StarsightError::Config` when the coord type doesn't match.
- **Breaking:** `Axis.scale: LinearScale` widened to `Box<dyn Scale>` so a single `Axis` type carries linear / log / sqrt / categorical mappings — required by polar radial axes and log color bars. Constructor builders unaffected; struct-literal callers must wrap in `Box::new`.
- **Breaking:** `render_grid_lines(coord, backend, theme)` now takes `coord: &dyn Coord` and dispatches by coord type. Cartesian path is unchanged; polar branch is additive.
- **Breaking:** `ChartKind` becomes `#[non_exhaustive]` so future `Polar` / 3D variants can land without a major bump.
- **Breaking:** `BarMark.base: Option<f64>` renamed and retyped to `bases: Option<Vec<f64>>`. The `.base(f64)` builder is kept as a single-broadcast convenience (stores `Some(vec![b])`), so callers using only the builder are unaffected. Direct struct-literal construction must update the field name.
- **Breaking:** `BarMark.color: Option<Color>` renamed and retyped to `colors: Option<Vec<Color>>`. Same broadcast convenience for `.color(Color)`.
- **Breaking:** `PointMark.color: Color` renamed and retyped to `colors: Option<Vec<Color>>`. `.color(Color)` builder kept; struct-literal construction must migrate.
- **Breaking:** `PointMark.radius: f32` renamed and retyped to `radii: Option<Vec<f32>>`. `.radius(f32)` builder kept.
- `BarMark::render` (the no-context fallback) now honors per-bar bases (previously ignored them and rendered every bar to baseline 0).
- `PointMark::render` now batches consecutive points by `(color, radius)` so per-point styling needs only one `draw_path` call per unique combination.
- `examples/composition/waterfall_bar.rs` rewritten as a single `BarMark` with per-bar `bases`/`colors` and `connectors(true)`.
- `examples/Cargo.toml`: prismatica dependency now enables the `colorbrewer` feature (RdPu used by `bubble_scatter`).
- **Breaking:** `Mark` trait gains a `legend_glyph()` method (default `LegendGlyph::Line`). External impls that don't override pick up the default.
- **Breaking:** `LegendEntry` gains a `glyph: LegendGlyph` field — struct-literal callers must add it.
- **Breaking:** `LayoutCtx` replaces `font_size: f32` and `title_font_size: f32` with `fonts: LayoutFonts`. Callers building a `LayoutCtx` directly migrate via `LayoutFonts { label: 12.0, title: 16.0 }` or `LayoutFonts::default()`.
- `render_axes` / `render_legend` / `render_title` / `render_axis_labels` now take a trailing `&LayoutFonts` argument and read sizes from there instead of redefining literals.
- `TitleComponent` breathing room bumped from `h + 12` to `h + 16` so chart titles on square heatmap canvases sit a touch further from the canvas top. All affected snapshots regenerated.
- `xtask gallery` does one upfront `cargo build --release --examples` and then exec's each binary directly, removing the per-example cargo overhead (~2s × 23 examples).
- Workspace `polars` dep moves from a bare version string to `{ default-features = false, features = ["lazy"] }` so layer-5 can inherit it cleanly while keeping the lazy-frame entry point.

### Fixed

- `BarMark` waterfall layout (`starsight-7h9`) — multiple `BarMark` instances each with `x.len() == 1` collapsed onto a single x-position because `Figure::category_labels()` only reads the first mark's labels. The fix is structural: per-bar `bases`/`colors` let one `BarMark` carry the entire waterfall, so the category axis spans all 10 labels naturally.
- `Axis::category(&[])` edge case (`starsight-262`) — debug builds now panic with a clear message; release builds keep the previous best-effort behaviour. Documented the band-edge tick-position invariant and the bar-mark scale-bypass behaviour (`starsight-o8p`).
- Square heatmap title spacing (`starsight-hko`) — `TitleComponent` reserves an extra 4px so the title doesn't graze the canvas top.
- Layout font-size duplication (`starsight-h4l`) — title / tick / axis-title font sizes are now sourced from a single `LayoutFonts` instance, eliminating drift between layout and render passes.
- Legend glyph dispatch (`starsight-f4t`) — `PointMark` legends show a filled dot, bar / area entries get the right swatch shape; line marks remain on a horizontal stroke.
- Gallery build performance (`starsight-qv7`) — `cargo xtask gallery` now builds examples once and exec's each binary, removing the cargo overhead per invocation.
- `category_axis_panics_on_empty_labels` test now gated on `cfg(debug_assertions)` so `cargo tarpaulin --release` no longer fails on a `debug_assert!` that's a no-op in release (`5642de7`).
- `cargo xtask gallery` now passes `--all-features` to its `cargo build` invocation so feature-gated examples (e.g. `polars_integration` with `required-features = ["starsight/polars"]`) build alongside the default set instead of being silently skipped (`bc66ae4`).

## [0.2.0] - 2026-04-27

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

## [0.1.1] - 2026-04-07

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
