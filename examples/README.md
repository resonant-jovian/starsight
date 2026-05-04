# starsight examples

Thirty-four runnable examples grouped by what they teach. Each `.rs` file lives
next to its rendered `.png`, so you can browse the gallery on GitHub without
running anything.

## Running an example

```bash
cargo run --release --example <name> --manifest-path examples/Cargo.toml
```

The names below are the same ones registered as `[[example]]` entries in
[`Cargo.toml`](Cargo.toml). Each program writes its PNG to the path shown
in the table — the gallery xtask (`cargo xtask gallery`) collects all of
them into `target/gallery/examples/`.

## Groups

### [`basics/`](basics) — start here

The smallest possible programs that exercise one mark type at a time. If
you've never used starsight before, read these in order.

| Example | What it shows |
|---|---|
| [`quickstart`](basics/quickstart.rs) — [PNG](basics/quickstart.png) | The `plot!` macro: three values, one save call, no builders |
| [`line_chart`](basics/line_chart.rs) — [PNG](basics/line_chart.png) | A single `LineMark` with title and axis labels |
| [`scatter`](basics/scatter.rs) — [PNG](basics/scatter.png) | Two `PointMark` series, color-coded with auto-derived legend |
| [`bar_chart`](basics/bar_chart.rs) — [PNG](basics/bar_chart.png) | Grouped `BarMark`s — quarters × product lines |
| [`heatmap`](basics/heatmap.rs) — [PNG](basics/heatmap.png) | A 30×30 `HeatmapMark` of a synthetic 2D field |
| [`histogram`](basics/histogram.rs) — [PNG](basics/histogram.png) | `HistogramMark` over 5 000 deterministic Gaussian samples |
| [`bubble_scatter`](basics/bubble_scatter.rs) — [PNG](basics/bubble_scatter.png) | Per-point continuous color (RdPu colormap) and per-point radius on `PointMark`, alpha 0.5 — wine-shaped synthetic data (spec #3) |
| [`movie_heatmap`](basics/movie_heatmap.rs) — [PNG](basics/movie_heatmap.png) | `HeatmapMark` with `log_scale()` — synthetic Rotten Tomatoes × IMDB cross-tab, log lift on the dim secondary lobe (spec #16) |

### [`theming/`](theming) — recolour without touching the data

Same shape, different paint. Demonstrates the two main ways to restyle a
figure: swap a `Theme`, or swap a colormap.

| Example | What it shows |
|---|---|
| [`custom_theme`](theming/custom_theme.rs) — [PNG](theming/custom_theme.png) | `chromata::popular::gruvbox::DARK_HARD.into()` — any of chromata's 1 104 editor themes drops into `Figure::theme(...)` |
| [`custom_colormap`](theming/custom_colormap.rs) — [PNG](theming/custom_colormap.png) | Same heatmap data as `basics/heatmap`, rendered with `INFERNO` from the prismatica colormap library |

### [`composition/`](composition) — multiple marks, one figure

Layered overlays, polished references, and per-bar customisation. The
auto-derived legend lives here.

| Example | What it shows |
|---|---|
| [`statistical`](composition/statistical.rs) — [PNG](composition/statistical.png) | Noisy daily readings + a 7-day rolling-mean overlay |
| [`recipe`](composition/recipe.rs) — [PNG](composition/recipe.png) | The reference for "what a good starsight chart looks like" — three series, custom palette, `DEFAULT_LIGHT` theme, full chrome |
| [`gallery`](composition/gallery.rs) — [PNG](composition/gallery.png) | A `LineMark` model fit over `PointMark` observations |
| [`waterfall_bar`](composition/waterfall_bar.rs) — [PNG](composition/waterfall_bar.png) | A P&L-walk waterfall — single `BarMark` with per-bar `bases` + `colors` and `connectors(true)` linking running totals (spec #37) |
| [`box_plot`](composition/box_plot.rs) — [PNG](composition/box_plot.png) | `BoxPlotMark` over four synthetic groups with five-number summary and Tukey-fence outliers |
| [`violin`](composition/violin.rs) — [PNG](composition/violin.png) | `ViolinMark` (KDE-driven) with optional inner-box overlay |
| [`pie`](composition/pie.rs) — [PNG](composition/pie.png) | `PieMark` with `show_percent()` and a custom 6-color palette |
| [`donut`](composition/donut.rs) — [PNG](composition/donut.png) | `PieMark` with `inner_radius(0.5)` and raw value labels |
| [`donut_sunburst`](composition/donut_sunburst.rs) — [PNG](composition/donut_sunburst.png) | Three-level nested `ArcMark` sunburst on `Figure::polar_axes` (spec #39 var C) |

### [`scientific/`](scientific) — real numerical experiments

These exist to prove the library handles real data, not just toy series.
Both run their own ODE/transform integration before plotting.

| Example | What it shows |
|---|---|
| [`lorenz_line`](scientific/lorenz_line.rs) — [PNG](scientific/lorenz_line.png) | Lorenz attractor (RK4, 80 000 steps) projected onto the x–z plane, accent from `prismatica::matplotlib::INFERNO` |
| [`kruskal_szekeres_line`](scientific/kruskal_szekeres_line.rs) — [PNG](scientific/kruskal_szekeres_line.png) | Kruskal–Szekeres extension of Schwarzschild — constant-r hyperbolas, constant-t rays, horizons, singularities |
| [`laser_plasma`](scientific/laser_plasma.rs) — [PNG](scientific/laser_plasma.png) | Stimulated Raman scattering — electron phase-space density on a 200×200 grid, log-scale viridis (spec #7, single-panel; multi-panel deferred to 0.4.0) |
| [`candlestick`](scientific/candlestick.rs) — [PNG](scientific/candlestick.png) | OHLC candlesticks with wicks and up/down body color dispatch — 30 trading days |
| [`nightingale`](scientific/nightingale.rs) — [PNG](scientific/nightingale.png) | Florence Nightingale coxcomb — `ArcMark` on `Figure::polar_axes`, 12 monthly stacks of 3 mortality categories on `polar_radial_sqrt` (value-as-area invariant, spec #34) |
| [`gauge`](scientific/gauge.rs) — [PNG](scientific/gauge.png) | Single-value 270° gauge — partial-sweep `ArcMark` with foreground value arc + muted background arc (spec #41) |
| [`contour_fields`](scientific/contour_fields.rs) — [PNG](scientific/contour_fields.png) | 2×2 `MultiPanelFigure` of Rosenbrock + Himmelblau + Rastrigin + Gaussian-mixture contour plots (spec #22) |
| [`bollinger_candlestick`](scientific/bollinger_candlestick.rs) — [PNG](scientific/bollinger_candlestick.png) | Two-panel `MultiPanelFigure`: candles + 20d SMA + Bollinger bands on top, daily volume `BarMark` below (spec #38) |
| [`radar_spider`](scientific/radar_spider.rs) — [PNG](scientific/radar_spider.png) | Three-player competence radar across 8 dimensions — `RadarMark` overlays on `Figure::polar_axes` with translucent fill (spec #31) |

### [`data/`](data) — DataFrame integrations

| Example | What it shows |
|---|---|
| [`polars_integration`](data/polars_integration.rs) — [PNG](data/polars_integration.png) | `plot!(df, x = "...", y = "...", color = "...")` against a Polars LazyFrame, with auto-derived legend |

### [`planned/`](planned) — placeholders for 0.4.0+ features

Each of these writes a static PNG announcing a deferred feature so
`cargo xtask gallery` always emits a uniform set of outputs. The real
implementations land per the [project roadmap](../README.md#roadmap).

| Example | Lands in | What it'll show |
|---|---|---|
| [`terminal`](planned/terminal.rs) — [PNG](planned/terminal.png) | 0.8.0 (ratatui backend) | Inline terminal rendering |
| [`interactive`](planned/interactive.rs) — [PNG](planned/interactive.png) | 0.6.0 (winit + wgpu backend) | Windowed chart with hover and zoom |
| [`surface3d`](planned/surface3d.rs) — [PNG](planned/surface3d.png) | 0.9.0 (vello/wgpu backend) | A 3D surface |
| [`streaming`](planned/streaming.rs) — [PNG](planned/streaming.png) | 0.7.0 (windowed data source) | Live data plotted over a sliding window |
| [`faceting`](planned/faceting.rs) — [PNG](planned/faceting.png) | 0.4.0 (`FacetWrap`) | A small-multiples grid via the planned `FacetWrap` composer (`MultiPanelFigure` shipped 0.3.0) |

## Adding a new example

1. Drop a `.rs` file into the group folder that fits best.
2. Have it `.save("examples/<group>/<name>.png")` so the PNG ends up next
   to the source.
3. Register it in [`Cargo.toml`](Cargo.toml) under the matching `# ──`
   section header — `name` and `path` are the only fields needed.
4. Run it once (`cargo run --release --example <name> --manifest-path
   examples/Cargo.toml`) so the PNG is committed alongside the source.
   `cargo xtask gallery` runs every registered example and aggregates the
   PNGs into `target/gallery/examples/` for CI artifact upload.
