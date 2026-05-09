# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.3] - 2026-05-09

### Fixed

- Hero and social-card PNGs lost all text on the GitHub Actions Ubuntu runner because none of the SANS family entries (`-apple-system`, `BlinkMacSystemFont`, `&quot;Segoe UI&quot;`, Roboto, Helvetica, Arial) resolved in the runner's font database — `usvg` silently dropped glyphs and the rasterized PNG shipped without the wordmark, tagline, or meta strip. Bundle DejaVu Sans + DejaVu Sans Mono in `xtask/src/chrome/fonts/` and load them into `usvg`'s `fontdb` *before* `load_system_fonts()` so chrome PNG output is byte-stable across macOS / Linux / Windows runners. The canonical SVGs are still browser-rendered with system-native faces — only the PNG path is pinned (`be46dee`).

### Docs

- Replace the inline math `<img src="assets/math/*.svg">` model in the README's "Worked Example — the Lorenz Attractor" prose with a single themed SVG card per theme (`assets/prose/lorenz-intro-{light,dark}.svg`). The previous approach put each math fragment behind a fixed pixel `height` that interacted with the surrounding markdown font's line-height differently on every viewer (crates.io vs github.com vs GitHub mobile vs IDE preview), so math glyphs floated above or oversized against the prose. Baking the whole paragraph into one fixed-pixel SVG card insulates the layout from the surrounding markdown flow exactly the way `status_panel`, `comparison_matrix`, and the other text-heavy chrome composites already do (`01be5f2`).
- New `xtask/src/chrome/prose_card.rs` module: typesets `assets/prose/<stem>.tex` LaTeX paragraphs through `latex` + `dvisvgm --no-fonts --exact-bbox`, embeds the glyph-path output inside a palette-themed rounded-rect card frame, and writes per-theme `<stem>-{light,dark}.svg`. Prose is in TeX Gyre Heros (Helvetica-equivalent sans), math is in Latin Modern Math via amsmath. Theme cascading via `<g color={p.text}>` + inner `<svg fill="currentColor">` so glyph paths inherit the right shade without rendering twice.
- New `Asset::ProseCard` variant for `cargo xtask chrome --asset prose-card`. `tools_present()` warns once and skips silently if `latex` / `dvisvgm` are absent so contributors without TeX Live can still rebuild every other chrome composite.
- `CITATION.cff` and the README BibTeX block bumped to `0.3.3`.
- New "Required tooling" + "Optional: regenerating chrome assets" tables in `CONTRIBUTING.md` covering cargo-insta, cargo-deny, cargo-llvm-cov, the `npx svgo` SVG optimization step, and the TeX Live bundles needed for prose-card regen (texlive-basic, texlive-latex, texlive-latexextra for `preview.sty`, texlive-mathscience, texlive-fontsrecommended, texlive-fontsextra for `tgheros`, texlive-bin for `dvisvgm`). Per-OS install commands for Arch, Debian/Ubuntu, and macOS. Fresh-machine setup is now a single doc lookup.

### Internal

- Retire the per-equation math infrastructure: delete `xtask/src/chrome/math.rs`, the `Asset::Math` enum variant + its no-theme dispatch carve-outs in `xtask/src/chrome/mod.rs`, and the `assets/math/` tree (12 `.tex` / `.svg` pairs + README). The latex+dvisvgm pipeline lives in `prose_card.rs` now.
- `live-assets.yml` chrome cron passes `--skip-examples` so it never re-rasterizes example `_dark.png` files. Examples aren't live data — they only change when source moves — and re-rendering them on the Ubuntu runner picked different system fonts than the macOS contributor render, flipping font metrics on every cron / local commit cycle. The hero composite embeds whatever example PNGs are already in the working tree, pinning the embedded thumbs to the committed macOS-rendered copies (`be46dee`).
- New `/coverage` skill mirroring the live-assets.yml `cargo llvm-cov --workspace --all-features --locked --exclude xtask` command for local coverage runs (`2c10ac2`). `cargo tarpaulin` remains banned for local use — OOMs under concurrent load.

## [0.3.2] - 2026-05-07

### Docs

- `CITATION.cff` and the README BibTeX block bumped to `0.3.2`.

### Internal

- CI workflows: merged the daily `chrome.yml` regen and the every-push `coverage.yml` into one `Live Assets` workflow with sequential `coverage` → `chrome` jobs (via `needs:`) so the two bot pushes can no longer race for `main`. Added a rebase-retry loop on each `git push` as defense against external pushes during the long chrome run. Closes the `not a fast-forward` failure observed on 2026-05-07 when both bots tried to push their commits onto the same parent.
- `release.yml`: added `push: branches: [main]` trigger with `dry-run` hard-forced in the `verify` step. Continuous validation of the publish chain (Cargo metadata, doc lints, `cargo deny`, `cargo publish --dry-run` per crate) without waiting for a tag; tag-driven real releases (`v*.*.*`) continue to work unchanged. Hard-force means an accidental main push can never produce a real release.
- `cargo xtask chrome --live` now also regenerates `assets/social/card-{light,dark}.{svg,png}` (was previously rebuilt only by full `cargo xtask chrome` runs). The social card embeds the same `Cargo.toml` meta strip as `hero` (version, rust, edition, license), so promoting it to the live set keeps the open-graph PNG aligned with the version on every push and on the daily cron. `live-assets.yml` `git add` widened to include `assets/social/`.
- `0.3.1` GitHub release/tag was lost in the process of trying to retrofit the new release.yml into the historical `0.3.1` commit (GitHub's Immutable Releases feature permanently reserves the name once a release has existed). `0.3.2` is the recovery release: same code surface as `0.3.1`, plus the CI improvements above.

## [0.3.1] - 2026-05-07

### Docs

- README hero, gallery, and Lorenz-card composites now ship as 2× retina PNGs alongside the existing SVGs; README `<picture>` blocks reference the PNG. Mirrors the dual-format `assets/social/card-{light,dark}.{svg,png}` pattern and removes a multi-megabyte inlined-example SVG payload from the README hot path on crates.io and GitHub.
- README image URLs pin to the `0.3.1` tag instead of `main`, so the rendered README on crates.io stays aligned with the assets that shipped with this release rather than drifting with `main`.
- `CITATION.cff` and the README BibTeX block bumped to `0.3.1`.

### Internal

- New `xtask/src/chrome/png.rs` helper — generalizes the rasterization recipe from `social_card` (parse via `usvg`, allocate `tiny_skia::Pixmap` at `tree.size() * scale`, render via `resvg`) so `hero`, `gallery`, and `lorenz_card` emit a PNG alongside their canonical SVG without each duplicating the plumbing.
- `cargo xtask chrome` parallelizes the trailing `svgo` pass via `std::thread::scope`. Worker count comes from `available_parallelism()`, clamped to `[2, 32]` (was effectively serial). Per-file resilience preserved (svgo's hard cap on `d`-attribute length still skips Lorenz cleanly).
- `cargo xtask chrome` now runs the **example pre-renders in parallel** — `ensure_example_outputs` spawns up to 32 concurrent example binaries instead of executing them serially. The old "~17 min for 68 examples" comment was the dominant wall-time term on cold caches; on a 24-thread box the 68-job sweep now completes in roughly job-count / worker-count × per-job cost.
- `cargo xtask chrome` also runs **light + dark composite regen in parallel** via `std::thread::scope` so hero / gallery / lorenz-card PNG rasterizations overlap.
- `cargo xtask gallery` parallelizes the per-example run loop with the same `std::thread::scope` pattern, capped at 32 workers.
- `cargo xtask chrome --asset <name>` scopes the trailing svgo pass to just the file(s) that asset writes, instead of re-optimizing all 36 composite SVGs.
- New `cargo xtask chrome --no-svgo` flag — skip the trailing svgo optimization pass for fast local iteration. CI continues to optimize.
- `chrome.yml` `timeout-minutes` raised 30 → 60: cold-cache regen is ~13 min build + ~17 min for 68 examples + svgo + chrome composition; 30 was just too tight (`59dfb21`).
- `chrome.yml` + `coverage.yml` checkout switched to `ssh-key: BOT_DEPLOY_KEY` so pushes can bypass branch-protection rulesets that `GITHUB_TOKEN` is blocked by; `coverage.yml` also gains an explicit `permissions: contents: write` for parity (`59dfb21`).
- Backtick snake_case identifiers in `xtask/src/chrome/mod.rs` doc-comment for `clippy::doc_markdown` (`updated_at`, `lorenz_card`, `social_card`, `coming_from`, `comparison_matrix`) (`59dfb21`).

## [0.3.0] - 2026-05-07

### Added

- `LegendPosition { Auto, Inside, Outside(Edge) }` + `Edge { Right, Left, Top, Bottom }` enums and `Figure::legend_position(LegendPosition)` builder. `Auto` (default) picks `Outside(Edge::Right)` for figures composed of disk-fill marks (`ArcMark` / `PieMark` / `DonutMark` / `RadarMark` / `PolarBarMark` / `PolarRectMark`) and `Inside` everywhere else. `Outside(edge)` reserves a strip outside `plot_area` so the legend never overlaps data (matplotlib `bbox_to_anchor=(1.05, 1)` equivalent). Re-exported via `starsight::prelude`.
- `Mark::pixel_extent(coord) -> MarkExtent { Bbox / Segments / Rects / Polygons }` trait hook; per-mark accurate footprint contributions feed the legend dodge. Replaces the bbox-based `data_pixel_rect` heuristic that false-positived on diagonal lines and full-range scatter. `MarkExtent` is re-exported via `starsight::marks` (advanced API; not in prelude).
- `Mark::prefers_outside_legend() -> bool` trait hook (default `false`); disk-fill marks (`ArcMark`, `PieMark` / `DonutMark`, `RadarMark`, `PolarBarMark`, `PolarRectMark`) override `true` so figures whose data covers the inscribed disk auto-default to outside-legend without explicit configuration.
- `Mark::wants_polar_grid() -> bool` trait hook (default `true`); `ArcMark` / `PieMark` / `DonutMark` override `false` so polar figures composed of decorative wedges suppress the spoke + ring grid.
- `LegendStripComponent` in `starsight-layer-5::layout` — reserves a strip on the chosen canvas edge for outside-the-plot legends. Sized from the actual entry count and label widths so the strip always fits the legend.
- `Mark::wants_axis_padding() -> bool` and `Mark::legend_entries() -> Vec<(Color, String, LegendGlyph)>` trait hooks. Point-shape marks (the default `true`) drive the 5% axis inset; bar / heatmap / contour return `false` so their data stays edge-aligned. `legend_entries` lets `PieMark` and `ArcMark` (when `wedge_labels` is set) emit one entry per slice/wedge so the figure auto-builds a color → category legend.
- `Figure::axis_padding(Option<f64>)` — explicit opt-in / opt-out override of the mark-mix-aware 5% padding default.
- `ArcMark::wedge_labels(Vec<String>)` builder — per-wedge labels surfaced through `legend_entries` for sunburst color→category legends.
- Polar `Axis` constructors auto-fill `tick_positions` + `tick_labels` from the existing polar tick formatters, so any figure on `Figure::polar_axes` now shows a default grid without needing to pre-build ticks.
- New showcase examples: `examples/composition/violin_raincloud.rs` (#19), `examples/composition/distribution_dashboard.rs` (#2, 2×2 panel), `examples/scientific/reciprocal_space.rs` (#17, 2×3 heatmap + cut panel). Counts 38 → 41 in `examples/README.md`.
- New showcase example: `examples/composition/energy_transition.rs` (#39 var B) — two concentric `ArcMark` rings comparing 2020 vs 2025 global energy mix on `Figure::polar_axes`. Pairs with `donut.rs` (Variant A) and `donut_sunburst.rs` (Variant C). Count 41 → 42 in `examples/README.md` (`6184ce7`).
- `Mark::wants_axes() -> bool` trait hook (default `true`); `PieMark` overrides to `false` so pie-only figures suppress numeric x/y axis chrome. The figure honours this only when *every* mark on it returns `false`; mixed charts keep the axes for the others (`6c26aed`).
- `Path::is_axis_aligned()` on `starsight-layer-1::paths::Path` — walks the command list and returns `true` iff every `LineTo` (and the implicit subpath-close segment) is purely horizontal or vertical, with any `QuadTo` / `CubicTo` an immediate `false`. Backends use this to drop antialiasing on grid / tick / axis hairlines (`fb984d8`).
- New snapshot tests: `snapshot_polar_grid_with_data`, `snapshot_violin_raincloud`.
- Facade: `AreaMark`, `BinMethod`, `HeatmapMark`, `ColormapLegend`, `Colorbar` now re-exported through `starsight::prelude` and `starsight::marks` / `starsight::statistics` so example code doesn't need deep-path imports.
- `PolarBarMark` — stacked annular bars (`r_base` for layered wind-rose stacks, optional uniform / per-bar angular widths). Backs `examples/scientific/wind_rose.rs` (#33).
- `PolarRectMark` — annular tile mark for spiral heatmaps and polar calendars. Backs `examples/scientific/polar_calendar.rs` (#8). Both polar marks share `build_arc_wedge` from `arc.rs` (promoted to `pub(crate)`) and the new `crate::marks::palette::POLAR_DEFAULT` Tableau-10 cycle.
- `ErrorBarMark` — vertical/horizontal whiskers attached to `(x, y)` points with optional perpendicular caps. Symmetric `ErrorBarMark::new(xs, ys, errors)` plus the asymmetric `.errors_pair(Vec<(low, high)>)` builder. Backs `examples/scientific/error_bars.rs`.
- `RugMark` + `AxisDir { X, Y }` — short perpendicular ticks along a chosen axis margin showing 1-D data distribution. Backs `examples/composition/rug_with_histogram.rs`.
- `Colorbar` chrome component (`starsight-layer-5/src/colorbar.rs`) — vertical gradient strip with 5 tick labels (min/25/50/75/max), 1px outline, optional rotated axis label. Auto-attached when any mark exposes a `ColormapLegend` via the new `Mark::colormap_legend()` trait hook (default `None`; `HeatmapMark` and `ContourMark` with a colormap return `Some`). Opt-out via `Figure::colorbar(false)`.
- `Mark::colormap_legend() -> Option<ColormapLegend>` trait hook + `ColormapLegend { colormap, value_min, value_max, label, log_scale }` struct in `starsight-layer-3::marks` so layer-5 can introspect marks for a colormap legend without a closed enum.
- Adaptive x-tick label rotation: `XTickLabelsComponent.band_width: Option<f32>` threads category band-width into layout reservation; `crate::layout::x_tick_label_rotation()` is the shared decision (0°, 45°, or 90° clockwise based on `max_label_width vs band_width`) used by both reservation and renderer. Bollinger volume panel hits 45°.
- New showcase examples: `examples/scientific/{wind_rose, polar_calendar, error_bars}.rs` and `examples/composition/rug_with_histogram.rs` (count 34→38 in `examples/README.md`).
- New snapshot tests: `snapshot_polar_{bar,rect}`, `snapshot_errorbar_{vertical,horizontal,asymmetric}`, `snapshot_rugmark_{x_axis,y_axis}`. Layer-5 now has 55+ snapshots.
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
- New showcase examples: `examples/scientific/{nightingale,gauge,contour_fields}.rs` and `examples/composition/donut_sunburst.rs` — covering `.spec/STARSIGHT.md` entries #22, #34, #39 var C, and #41.
- `ChartKind::Contour` variant; the enum becomes `#[non_exhaustive]` so future polar / 3D variants slot in without a breaking change.
- New snapshot tests: `snapshot_polar_grid_{linear,log,categorical}`, `snapshot_multipanel_2x2_basic`, `snapshot_contour_isolines`, `snapshot_arcmark_{full_nightingale,partial_gauge,nested_sunburst}`.
- `BarMark` per-bar bases via `bases: Option<Vec<f64>>` and `bases(Vec<f64>)` builder
- `BarMark` per-bar colors via `colors: Option<Vec<Color>>` and `colors(Vec<Color>)` builder
- `BarMark::connectors(bool)` builder — draws thin gray (#888) 1px lines between consecutive bars at running-total level (vertical orientation only)
- `PointMark` per-point colors via `colors: Option<Vec<Color>>` and `colors(Vec<Color>)` builder
- `PointMark` per-point radii via `radii: Option<Vec<f32>>` and `radii(Vec<f32>)` builder
- `PointMark::alpha(f32)` builder — mark-wide alpha multiplier applied at draw time
- `HeatmapColorScale` enum (`Linear`, `Log`) and `HeatmapMark::color_scale()` / `log_scale()` builders — log path normalizes via `log10` with a small epsilon to handle zero/negative cells
- New showcase examples (see `.spec/STARSIGHT.md` showcase appendix):
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

- Numeric axes get a 5% inset via `Axis::auto_from_data` post-Wilkinson (matplotlib `axes.margins` convention) when at least one mark on the figure is point-shaped — `Point` / `Line` / `Area` / `ErrorBar` / `BoxPlot` / `Violin` / `Candlestick` / `Step` / `Rug`. Bar / histogram / heatmap / contour figures stay edge-aligned because their data IS the axis structure. `Figure::axis_padding(Some(f))` overrides per-figure.
- Categorical y-tick labels on horizontal-bar charts use a uniform column-left x (max-label-width-aligned) instead of right-anchor, so mixed-length category strings ("Tokyo" vs "Mexico City") no longer wobble. Numeric ticks keep right-anchor for decimal alignment.
- Bordered legend tries corners in TR → TL → BR → BL order, picking the first whose rect doesn't intersect the merged data extent (cartesian only — polar legends keep TR per the documented 0.3.0 limitation).
- Mark legend entries are filtered out when their colormap is already shown by the auto-attached `Colorbar` — heatmap / contour figures no longer emit a redundant labeled-mark legend on top of the colorbar.
- Polar render path inscribes the disk with a 12px inset and a slightly enlarged title slot, so titles and chart edges no longer collide on tight 800×600 canvases (gauge / polar_calendar / wind_rose).
- `snapshot_arcmark_partial_gauge` rebuilt as a layered gauge (rim + track + value + tick labels) matching the example, so the snapshot reads as a real gauge instead of a single wedge.
- **Behavior change (auto-attached colorbar).** `HeatmapMark` and `ContourMark` figures now reserve a Right-side colorbar slot by default (16px gradient strip + 5 tick labels + 1px outline). Opt out via `Figure::colorbar(false)` to recover pre-0.3.0 layout. Tracked as `starsight-kdi`.
- `DEFAULT_LIGHT` text colors bumped for ≥7:1 WCAG contrast against white: `tick_label` `#555555`→`#333333`, `axis` `#666666`→`#444444`, `title` `#222222`→`#111111`. `DEFAULT_DARK` lifted in lockstep against the `#1E1E1E` background. Tracked as `starsight-405`. Path-effects halo for label-over-fill cases (heatmap text, contour labels) deferred to 0.4.0.
- `DEFAULT_FIGURE_PADDING_PX` bumped from 4 to 8 (matches `MultiPanelFigure::padding` default) — every chart kind now has a consistent 8px breathable margin between canvas edge and the layout slot stack. Tracked as `starsight-c6h`.
- `render_title` now centers the title-x over the plot area rather than the full title-slot rect, so the title balances over the data on charts with a y-axis label slot. Tracked as `starsight-cet` (1).
- Y-axis tick labels apply `tw.ceil()` and `.round()` to pixel-snap their right edge so labels read flush within sub-pixel precision. Tracked as `starsight-cet` (2).
- Legend inset bumped 10px → 16px around the top-right corner so axis-extreme data points have visible breathing room before the legend rect. Tracked as `starsight-bls`.
- `RadarMark` default `fill_alpha` lowered 40 → 25 so 3+ overlapping series stay distinguishable in PNG raster output. Tracked as `starsight-61l`.
- `ContourMark::render_filled_bands` now strokes each band polygon with `PathStyle::stroke(color, 1.0)` — matplotlib's standard fix that covers anti-aliased seam pixels between adjacent cells/bands. Tracked as `starsight-3h6`.
- `ArcMark` default palette extracted to a shared `crate::marks::palette::POLAR_DEFAULT` (Tableau 10 vibrant). `PolarBarMark` reuses the same default. `examples/scientific/nightingale.rs` "other causes" wedge color updated `0x8AA38E` (sage) → `0xC2A35F` (mustard) for higher visibility. Tracked as `starsight-dbh`.
- `examples/scientific/gauge.rs` enriched with three layered `ArcMark`s (outer rim + bg track + value arc), each stroked white for definition. Title now reads "Battery — 78 / 100". Tracked as `starsight-0kb`.
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

- Disk-fill chart legends (pie / donut / sunburst / nightingale / gauge / wind rose / radar / polar calendar) no longer overlap the data. Whenever any labeled mark on a figure is a disk-fill mark, the legend defaults to outside-right placement (top-aligned, in a reserved right-edge strip outside the plot area). Override with `Figure::legend_position(LegendPosition::Inside)` to recover the pre-fix in-plot dodge.
- Inside-plot legend dodge now uses per-mark `MarkExtent` instead of a coarse data bbox. Diagonal `LineMark` data and full-range scatter no longer trigger an all-corners-overlap fallback to TR; least-overlap (count → clipped area → TR > TL > BR > BL priority) picks the cleanest corner.
- Polar figures share the same legend dodge machinery as cartesian; tall multi-entry legends on gauge / nightingale / wind-rose / polar-calendar no longer extend into the inscribed disk. When the figure includes a labeled disk-fill mark, the polar render path also reserves an outside-right strip.
- `snapshot_arcmark_partial_gauge` no longer wraps a full polar grid through the empty 90° wedge — `wants_polar_grid()` opt-out drops grid spokes/rings behind ArcMark/PieMark/DonutMark figures.
- `MultiPanelFigure` accepts a theme via `.theme(...)` so dark-mode composites no longer leave a white canvas behind dark panels (`contour_fields_dark.png`).
- `BarMark` bars now align with the categorical-axis grid + tick positions; previously a 1–2 px subpixel drift was visible at high bar density (90-bar bollinger volume panel). Both vertical and horizontal orientations route through the f64-mediated band-center math like the rest of the marks.
- `BarMark` waterfall layout (`starsight-7h9`) — multiple `BarMark` instances each with `x.len() == 1` collapsed onto a single x-position because `Figure::category_labels()` only reads the first mark's labels. The fix is structural: per-bar `bases`/`colors` let one `BarMark` carry the entire waterfall, so the category axis spans all 10 labels naturally.
- `Axis::category(&[])` edge case (`starsight-262`) — debug builds now panic with a clear message; release builds keep the previous best-effort behaviour. Documented the band-edge tick-position invariant and the bar-mark scale-bypass behaviour (`starsight-o8p`).
- Square heatmap title spacing (`starsight-hko`) — `TitleComponent` reserves an extra 4px so the title doesn't graze the canvas top.
- Layout font-size duplication (`starsight-h4l`) — title / tick / axis-title font sizes are now sourced from a single `LayoutFonts` instance, eliminating drift between layout and render passes.
- Legend glyph dispatch (`starsight-f4t`) — `PointMark` legends show a filled dot, bar / area entries get the right swatch shape; line marks remain on a horizontal stroke.
- Gallery build performance (`starsight-qv7`) — `cargo xtask gallery` now builds examples once and exec's each binary, removing the cargo overhead per invocation.
- `category_axis_panics_on_empty_labels` test now gated on `cfg(debug_assertions)` so `cargo tarpaulin --release` no longer fails on a `debug_assert!` that's a no-op in release (`5642de7`).
- `cargo xtask gallery` now passes `--all-features` to its `cargo build` invocation so feature-gated examples (e.g. `polars_integration` with `required-features = ["starsight/polars"]`) build alongside the default set instead of being silently skipped (`bc66ae4`).
- SVG backend no longer compounds opacity (`starsight-2ja`). The umbrella `opacity` attribute multiplied with the per-channel `fill-opacity` / `stroke-opacity`, so `RadarMark`'s `fill_alpha=25` (style.opacity 0.098) rendered at 0.098² ≈ 0.0096 — ~1% of the requested alpha, so polygons looked nearly transparent in `radar_spider.svg` while the canonical PNG was correct. Drop the umbrella attribute and emit per-channel only — same model as the skia backend. Affects `bubble_scatter`, `radar_spider`, and `distribution_dashboard` SVGs (`488c9f9`).
- Axis-aligned grid / tick / axis paths now stay crisp at 100% zoom (`starsight-yrp.6`). Both backends previously antialiased every path uniformly, leaving 1-px hairlines fuzzed across two pixel rows. The new `Path::is_axis_aligned()` classifier hits `paint.anti_alias = false` on the skia backend and `shape-rendering="crispEdges"` on the SVG backend (rects always get the attribute since they're axis-aligned by construction); curves and diagonals still antialias normally (`fb984d8`).
- `PieMark`-only figures suppress the numeric x/y axis chrome (`starsight-yrp.2`). Pie / donut charts no longer render a stray 0..1 / 0..100 axis behind the wedges (`6c26aed`).
- `PieMark` slice labels auto-pick a contrasting color via Rec. 601 luminance (`starsight-yrp.3`) — white text when the slice fill luminance is below 0.5, black otherwise. Labels stay readable on dark blue / pink slices that the previous always-black default washed out. The explicit `.label_color(c)` builder still wins when set away from the default (`6c26aed`).
- Leftmost / rightmost candlesticks no longer half-clip the plot edge (`starsight-yrp.5`). `CandlestickMark::data_extent` now widens by half a band on each x side (median timestamp spacing × 0.5 for n ≥ 2; ±0.5 for the single-row degenerate case) so the first and last candle bodies sit fully inside the plot rect (`14fef6c`).
- Legend rect gets a 1-pixel `theme.axis` border so its right and bottom edges are visible against bright theme plot backgrounds — previously the translucent white fill blended into the data with no edge cue (`starsight-yrp.1`, `b38698e`).

### Docs

- README replaced with a multi-asset chrome layout: hero, gallery, lorenz card, social card, roadmap, status panel (now with optional coverage %), 8-stage pipeline, 7 status matrices (marks / scales / backends / stats / layout / output / themes), 15-library coming-from card stack, 14×9 comparison matrix, badges row, and 5-button action row. Every asset ships in light + dark variants, swapped via `<picture>` / `prefers-color-scheme`, and is rendered end-to-end by `cargo xtask chrome` so there's no shields.io / external service dependency at viewing time. README is now self-contained on crates.io / lib.rs / docs.rs.

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
