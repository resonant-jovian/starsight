# AGENTS.md — starsight

Universal brief for any agent working in this repo.

## Build & Test

```bash
cargo build --workspace
cargo test --workspace

# Single layer
cargo test -p starsight-layer-3

# Snapshots (insta, layer-5 only)
cargo test --workspace --test snapshot                              # run all
cargo test -p starsight-layer-5 --test snapshot                      # run one layer
INSTA_UPDATE=always cargo test --workspace --test snapshot           # update locally
cargo insta test --workspace --check --unreferenced reject           # CI mode
cargo insta accept                                                    # accept pending after a normal run

# Lint order: fmt → clippy → check → test
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features
cargo test --workspace

# Docs (CI gate)
RUSTDOCFLAGS="-D missing-docs -D rustdoc::broken-intra-doc-links" \
  cargo doc --workspace --no-deps --all-features

# Examples
cargo xtask gallery    # render every example into target/gallery/
cargo xtask showcase   # symlink example PNGs into showcase/
```

> `cargo xtask snapshots` is a **stub** in the current xtask binary. Use the `cargo insta` commands above directly.

## Workspace structure

- **starsight** (facade): re-exports layers 1-7 via prelude, by category, and by layer alias.
- **starsight-layer-1** (`background`): primitives, backends (Skia, Svg), errors, paths, colormaps, theme.
- **starsight-layer-2** (`modifiers`): scales (Linear, Band), coords, axes, ticks (Wilkinson Extended).
- **starsight-layer-3** (`components`): marks (Line, Point, Bar, Area, Heatmap, Histogram, Step, BoxPlot, Violin, Pie, Candlestick), statistics (Kde, BoxPlotStats), aesthetics, positions.
- **starsight-layer-4** (`composition`): layout, legend dispatch.
- **starsight-layer-5** (`common`): Figure, `plot!` macro, snapshot tests.
- **starsight-layer-6** (`interactivity`): winit (planned 0.6.0).
- **starsight-layer-7** (`export`): PDF, GIF, HTML, WASM (planned).

Layer-N may only depend on layer-1..N-1.

## Module paths

- Errors: `crate::errors::{Result, StarsightError}` (not `crate::error`)
- Backends: `crate::backends::DrawBackend` (not `crate::backend`)
- Skia / SVG: `starsight_layer_1::backends::{skia::SkiaBackend, svg::SvgBackend}`
- Coords: `starsight_layer_2::coords::CartesianCoord`
- Paths: `crate::paths::{Path, PathCommand, PathStyle}`

## Key conventions

- **No global state**: `plot!(x, y)` returns a `Figure`; no `plt.show()`.
- **Builder pattern**: `Figure::new(800, 600).title("…").add(mark)`.
- **NaN = gap**: `LineMark` and `AreaMark` treat `NaN` as breaks.
- **`Result<T>` everywhere**: handle or propagate; no `unwrap()` in library code.
- **MSRV 1.89**, **Edition 2024** (workspace-pinned in `Cargo.toml`).
- **Snapshots are insta-based**, every layer has a `tests/snapshot.rs`. Layer-1 and layer-5 are populated today; the rest are placeholders that fill out as those surfaces stabilize. SVG is the default snapshot format (byte-exact across OS/fonts); PNG (`assert_binary_snapshot!`) only for backend-pure tests with no text.

## What works now (0.3.x)

- Marks: Line, Point (per-point color/radius/alpha), Bar (vertical/horizontal/grouped/per-bar bases+colors+connectors), Area (with baseline), Heatmap (Linear + Log scale), Histogram (auto-bin), Step, BoxPlot+BoxPlotGroup, Violin+ViolinGroup+ViolinScale (KDE-driven, split mode, Area/Count/Width norm), Pie+PieLabelMode (donut via `inner_radius`, percent/value labels), Candlestick+Ohlc, **Contour (marching squares, isolines + colormap, with seam-stroked filled bands)**, **Arc (polar wedges for Nightingale / Gauge / Sunburst)**, **Radar (polyline on polar)**, **PolarBar (stacked annular bars)**, **PolarRect (annular tile for spiral heatmaps)**, **ErrorBar (symmetric / asymmetric whiskers + caps)**, **Rug (1-D ticks along axis margin)**.
- Polars: `polars` feature → `FrameSource` + `plot!(df, x="col", y="col", color="col")`.
- Coords: `CartesianCoord` and **`PolarCoord`** (compass convention, `inscribed`/`with_center`/`with_radius` builders); `Mark::render` dispatches via `&dyn Coord` with `as_any()` downcast helpers.
- Scales: `LinearScale`, `LogScale`, `SqrtScale`, `CategoricalScale`, `BandScale` (categorical x); `infer_chart_kind` chart-type inference.
- Axes: `Axis::auto_from_data` + `Axis::category` + polar variants (`polar_angular`, `polar_angular_categorical`, `polar_radial`, `polar_radial_sqrt`, `polar_radial_log`).
- Ticks: Wilkinson Extended for numeric; `polar_ticks_{degrees,radians,categorical}` formatters (π-fraction labels reduced to lowest terms).
- Color: `Color::cycle_next` (Tableau10); LegendGlyph dispatch (line/point/bar/area).
- Layout: title + axis labels; LayoutFonts shared between layout + render.
- **Multi-panel**: `MultiPanelFigure(width, height, rows, cols)` + per-panel padding + per-panel independent axes; `Figure::render_within(viewport, backend)` is the parameterized dispatch point.
- **Polar Figure mode**: `Figure::polar_axes(theta, r)` builds a `PolarCoord` and renders with `render_grid_lines`'s polar branch (radial spokes + concentric rings); skips cartesian axis chrome.
- **Auto-attached `Colorbar`**: `Figure` introspects every mark via `Mark::colormap_legend()` and reserves a Right-side gradient strip slot when any mark exposes a legend (`HeatmapMark` and `ContourMark` with a colormap). Opt-out via `Figure::colorbar(false)`.
- **Adaptive x-tick label rotation**: `XTickLabelsComponent.band_width` threads the categorical band width through layout reservation; `crate::layout::x_tick_label_rotation()` is the shared decision (0°, 45°, or 90° clockwise) that both reservation and renderer use.
- Stats: `BoxPlotStats`, `Kde` (Gaussian, Silverman/Scott/Manual bandwidth), `percentile`, `std_dev`, **`Contour::compute(grid, &levels)` (average-of-corners saddle disambiguation)**.
- Backends: Skia (raster, with AA auto-detect on axis-aligned paths), SVG (with opacity).
- Figure + `plot!` macro (DataFrame arm gated on `polars`).

## Not yet implemented

- 0.4.0: `FacetWrap`, shared axes across `MultiPanelFigure` panels, polar-aware legend placement, path-effects halo for label-over-fill text, log-scale colorbar ticks (`Colorbar` shipped 0.3.0)
- 0.5.0: `SymLogScale`, `DateTimeScale` (LogScale/SqrtScale/CategoricalScale shipped 0.3.0)
- 0.6.0: wgpu, hover/zoom/pan
- 0.7.0: Animation, GIF
- 0.8.0: Terminal backends
- 0.9.0: Surface3D, Scatter3D
- 0.10.0: PDF (krilla), interactive HTML, WASM
- 0.11.0: ndarray / Arrow input (Polars landed early in 0.3.0)

## Where to find detail

- Path-scoped rules (load when matching files enter context): `.claude/rules/`
- Slash workflows: `.claude/skills/` (`/check`, `/snap`, `/release-prep`, `/quickfix`, `/scout`)
- Subagents: `.claude/agents/` (`@snapshot-reviewer`, `@layer-boundary-check`)
- Master spec: `.spec/STARSIGHT.md`; learning doc: `.spec/LEARN.md`
- Issue tracker / session protocol: handled by the `bd prime` SessionStart hook
