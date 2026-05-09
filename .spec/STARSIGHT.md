# starsight roadmap

> Single source of truth for what starsight is building, in what order. README
> is for users; this is for the project. Categories mirror the README "Status"
> matrices from 0.1.0 onward. Pre-0.1.0 uses bootstrap-specific buckets. The
> 42-example showcase appendix lives at the bottom with explicit
> `<a id="example-N"></a>` anchors.

> **Companion files** — [`AGENTS.md`](../AGENTS.md) (architecture invariants),
> [`CLAUDE.md`](../CLAUDE.md) (agent guardrails), [`.claude/rules/`](../.claude/rules)
> (module-level conventions). This file does not duplicate them; it links out.

## How to use this file

- **Top** (sections 1–3) — what starsight is, the 7-layer architecture, design
  axioms (invariants that survive across milestones).
- **Middle** (section 4) — milestone roadmap, one section per release,
  category-grouped checkboxes, plus per-milestone Infrastructure catch-all for
  items that don't map to README categories.
- **Bottom** (sections 5–8) — empirical defaults, antipattern warnings, common
  rules, and the chart-by-chart math inputs (consumed verbatim by
  `examples/*.rs`).

---

## 1. What starsight is — and is not

### Is

- A grammar-of-graphics scientific plotting library for Rust, layered as seven
  crates that compose: backends → scales → marks → layout → figure → (planned)
  interactivity → (planned) animation/export.
- Multi-backend: SkiaBackend (CPU raster) and SvgBackend (vector) ship today;
  WgpuBackend (GPU, 0.6), KrillaBackend (PDF/A-3u, 0.10), WasmBackend (browser,
  0.10), and the terminal cascade (Kitty / Sixel / iTerm2 / half-block /
  Braille / Unicode 16 octant, 0.8) follow.
- Scientifically opinionated by default: prismatica perceptually-uniform
  colormaps (BATLOW / VIK / ROMA_O), Wilkinson Extended ticks, deterministic
  output (sorted SVG attributes, fixed precision, seeded jitter), WCAG AA
  contrast, and antipattern warnings at plot-construction time.
- Committed (post-0.5) to Arrow at API edges, dimensional types via `uom`,
  automatic provenance metadata, locale-aware text via `cosmic-text` + `icu4x`,
  and a serde-friendly spec format that round-trips to JSON for LLM workflows.

### Is not

- A drop-in matplotlib replacement — the API is grammar-of-graphics, not
  imperative-pyplot. Use the [Coming from another language](../README.md)
  cheatsheet for migration.
- A medical imaging library — out of scope; integrate with `dicom-rs` or ITK.
- A graph visualization framework — `starsight-graph` is the future home for
  force / hierarchical / edge-bundling layouts; the core is a plotting library,
  not a graph layout engine.
- A scrollytelling framework — the spec format supports multi-state encoding;
  the JS/HTML scaffolding is downstream.
- An interactive dashboard server — the reactive layer (Layer 6, 0.6) is
  optional and engine-agnostic; full dashboards belong in higher-level tools.

---

## 2. Architecture

### Layered crate split (current, load-bearing)

The seven-layer architecture is the public-facing contract. **Layer-N depends
only on layers 1..N−1.** See [`AGENTS.md`](../AGENTS.md) for the long form and
[`.claude/rules/`](../.claude/rules) for module-level conventions.

| Crate | Role |
|---|---|
| `starsight-layer-1` | `DrawBackend` trait; SkiaBackend + SvgBackend; primitives (`Vec2`, `Point`, `Rect`, `Color`, `Transform`); `StarsightError`/`Result` |
| `starsight-layer-2` | Scales (`LinearScale`, `LogScale`, `SqrtScale`, `CategoricalScale`), Wilkinson Extended tick algorithm, coordinate systems (`CartesianCoord`, `PolarCoord`) |
| `starsight-layer-3` | Marks, aesthetics, statistics (KDE, BoxPlot summary, HistogramMark binning) |
| `starsight-layer-4` | Layout, legend, colorbar, `MultiPanelFigure` |
| `starsight-layer-5` | `Figure` builder, `plot!` macro, themes, Polars data-acceptance |
| `starsight-layer-6` | Interactivity (planned, 0.6) |
| `starsight-layer-7` | Animation / export (planned, 0.10) |
| `starsight` (facade) | Public re-exports + opinionated default |

### 1.0+ vision (informative, not committed)

A longer-term direction is the 15-crate split sketched in
`STARSIGHT_DESIGN_REFERENCE.md` §13 — `starsight-spec`, `-core`, `-stats`,
`-render`, `-text`, `-cairo`, `-wgpu`, `-term`, `-svg`, `-anywidget`, `-bayes`,
`-graph`, `-geo`, `-provenance`, plus the existing `chromata` and `prismatica`
sister crates and the facade. The 7-layer split evolves toward it incrementally
as new domains land (e.g., `starsight-bayes` lifts ArviZ-equivalent plots out
of `starsight-stats`; `starsight-geo` lifts cartographic projections out of
`starsight-layer-2`). The 1.0 milestone decides whether to commit; until then,
the 7-layer contract is what consumers depend on.

<a id="which-crate-do-i-edit"></a>

### Which crate do I edit?

| Task | Crate |
|---|---|
| New mark type | `starsight-layer-3` |
| New scale type | `starsight-layer-2` |
| New backend | `starsight-layer-1` |
| Layout / legend / colorbar | `starsight-layer-4` |
| Figure builder, `plot!` macro, themes, Polars adapter | `starsight-layer-5` |
| Interactivity (planned) | `starsight-layer-6` |
| Animation / export (planned) | `starsight-layer-7` |
| Public API surface | `starsight` (facade) |
| Compile-time editor / terminal themes | `chromata` (sister crate) |
| Compile-time perceptually-uniform colormaps | `prismatica` (sister crate) |
| Chrome / live-asset generation | `xtask` |

(The anchor `#which-crate-do-i-edit` above is the canonical link target from
[`CONTRIBUTING.md`](../CONTRIBUTING.md).)

### Layered dependency invariant

```
starsight (facade)
   │
   └── layer-7 ─── layer-6 ─── layer-5 ─── layer-4 ─── layer-3 ─── layer-2 ─── layer-1
                                  │            │            │            │            │
                                chromata   (planned)   prismatica   (none)      tiny-skia / svg
                                                                                cosmic-text
```

`@layer-boundary-check` is the subagent that verifies this invariant on
refactors.

---

## 3. Design axioms

Six commitments that survive across milestones. The "active in" line states
when each axiom transitions from intent to enforced API.

### Axiom 1 — Arrow at API edges where feasible

Prefer `&dyn arrow::array::Array` for new public mark/stat APIs. Provide a
zero-cost `From<Vec<f64>>` blanket adapter so casual callers stay ergonomic but
the canonical type is Arrow. **Why**: zero-copy interop with Polars, DataFusion,
and DuckDB; aligns with the Mosaic/VegaFusion architecture for >10⁴ aggregation
paths. **Active in**: 0.5+ for new APIs. Existing `Vec<f64>` paths stay
unchanged (no wholesale rewrite).

### Axiom 2 — Dimensional types in 0.5+ scales

`ContinuousScale<Quantity<D, U, V>>` using the `uom` crate's phantom-type
algebra. A `ContinuousScale<Length>` and a `ContinuousScale<Time>` are
different types; mixed-unit axes are rejected at compile time; axis labels
auto-format the unit. **Why**: zero-cost correctness at the type level; no
other plotting library does this. **Active in**: 0.5. Existing `LinearScale`
stays as the unit-less default for casual use.

### Axiom 3 — Determinism by default

The SVG backend emits sorted attributes, fixed decimal precision (4 places),
seeded jitter, and (under `--reproducible` / `Theme::reproducible()`) embedded
fonts. Every `cargo run --example X` produces byte-identical output across
machines for the same toolchain. **Why**: scientific reproducibility, Git-diff
sanity, archival fidelity. **Active in**: 0.4 for SVG, 0.10 for PDF (via
`SOURCE_DATE_EPOCH` + deterministic font subsetting).

### Axiom 4 — Provenance metadata available

`Figure::with_provenance(BTreeMap<String, Value>)` accepts user-extensible
metadata; `fig.save("plot.svg")` also writes `plot.svg.prov.json` with input
data hashes (xxhash on Arrow buffers), library version, host OS, timestamp,
and the user metadata. PDF/A-3u embeds this directly. **Why**: scientific
reproducibility years later; archival journals (PLOS, Nature) increasingly
require it. **Active in**: opt-in 0.5; PDF/A-3u archival default 0.10.

### Axiom 5 — Reactive optional, never required

Layer 6 interactivity (Observable-style signals; selections-as-predicates per
Vega-Lite) ships behind a feature flag. The static rendering path is always
available without the reactive runtime. **Why**: Mosaic/VegaFusion data show
static plots with server-side aggregation outperform interactive browser
rendering by orders of magnitude on >10⁶ points; static-first is the right
default. **Active in**: 0.6 (feature flag = `interactive`).

### Axiom 6 — Spec format as durable artifact

`Figure::to_spec() -> serde_json::Value` and `Figure::from_spec(...)` round-trip
losslessly via serde. JSON-Schema-validated, deterministically serialized,
content-hashable. **Why**: enables LIDA-style LLM authoring, cross-language
bindings, Quarto/Jupyter integration, meaningful version-control diffs.
**Active in**: 0.5. Targets the `starsight-spec` crate envisioned in the 1.0+
split.

---

## 4. Roadmap

Each milestone is bucketed by README category (Marks / Scales / Backends /
Statistics / Layout / Output / Themes), plus a per-milestone **Infrastructure**
catch-all and **Tests** / **Snapshots** subsections. Pre-0.1.0 uses
bootstrap-specific buckets (Repo & licensing / GitHub config / Workspace /
Stubs) since none of those items map to README categories.

Items use `[x]` for shipped, `[ ]` for unstarted. Implementation-level
sub-tasks (e.g., "add `length()` to `Vec2`") are not tracked here — they belong
in `bd` issues. This file tracks **features**.

### Pre-0.1.0 — Workspace bootstrap [shipped]

- Repo & licensing
  - [x] Create `resonant-jovian/starsight` GitHub repository (GPL-3.0-only)
  - [x] CONTRIBUTING.md, CODE_OF_CONDUCT.md, CHANGELOG.md, SECURITY.md
- GitHub config
  - [x] Issue templates (bug_report, feature_request) + config.yml
  - [x] PULL_REQUEST_TEMPLATE.md, FUNDING.yml
  - [x] CI workflows: ci.yml (fmt / clippy / check / test / deny), release.yml
        (publish + git-cliff), coverage.yml (cargo-llvm-cov + Codecov),
        snapshots.yml (insta + artifact upload), gallery.yml (xtask gallery)
  - [x] README.md skeleton with badges, feature table, roadmap
- Workspace
  - [x] Workspace `Cargo.toml`: resolver 3, edition 2024, all 8 members
        (`starsight`, `starsight-layer-1` through `starsight-layer-7`, `xtask`)
  - [x] All per-crate `Cargo.toml` files with workspace inheritance
  - [x] Feature flags defined in `starsight/Cargo.toml`
  - [x] Workspace lints: `unsafe_code = forbid`, `clippy::pedantic = warn`,
        `clippy::module_name_repetitions = warn` with `allow-exact-repetitions
        = false`
  - [x] `.rustfmt.toml`, `.clippy.toml`, `deny.toml`
  - [x] Profiles: release LTO + codegen-units 1; dev opt-level 1
- Stubs
  - [x] `StarsightError` enum (7 variants) + `Result` alias
  - [x] `DrawBackend` trait scaffolded
  - [x] Primitives: `Color`, `Point`, `Rect`, `Size`
  - [x] Stub modules for every backend (skia/, svg/, pdf/, wgpu/, terminal/)
  - [x] Stub `lib.rs` for layers 2–7
  - [x] `cargo check --workspace` and `cargo test --workspace` pass

### 0.1.0 — Foundation [shipped]

- Marks
  - [x] `LineMark` — basic, multi-series, NaN-gap handling
  - [x] `PointMark` — circle / square / triangle / cross marker shapes
- Scales
  - [x] `LinearScale` with domain inference from data
  - [x] Wilkinson Extended tick algorithm (Talbot-Lin-Hanrahan 2010); no other
        Rust crate implements this
  - [x] `CartesianCoord` with `data_to_pixel` mapping
- Backends
  - [x] `SkiaBackend` (CPU raster, .png and .jpeg output)
  - [x] `SvgBackend` (vector .svg output)
- Layout
  - [x] `Figure` builder
- Output
  - [x] PNG export via `tiny-skia::Pixmap::encode_png`
  - [x] SVG export via the `svg` crate
- Themes
  - [x] Themes scaffolding via `chromata` facade (light + dark)
- Infrastructure
  - [x] `Vec2` / `Point` / `Rect` / `Color` / `Transform` arithmetic with
        type-level safety: `Point - Point = Vec2`, `Point + Vec2 = Point`,
        `Point + Point` does not compile
  - [x] `StarsightError` + `Result` plumbed end-to-end with `?`-ergonomic
        propagation
  - [x] `plot!` macro (Layer 5)
  - [x] Facade re-exports + `prelude`
- Tests
  - [x] Unit: Vec2/Point arithmetic, scale invariants, Wilkinson algorithm
  - [x] Property tests for scale monotonicity
  - [x] Integration: CSV → `plot!` → `.save("chart.png")` round-trip
- Snapshots
  - [x] `line_basic`, `line_nan_gap`, `line_multi_series`, `point_marker_shapes`,
        `scatter_basic`, `scatter_size_varying`, `svg_basic`, `axes_only`,
        `wilkinson_ticks_simple`, …

### 0.2.0 — Core chart types part 1 [shipped]

- Marks
  - [x] `BarMark` (vertical, horizontal, grouped, stacked)
  - [x] `AreaMark` (basic; stacked variants polished in 0.3)
  - [x] `StepMark`
  - [x] `HistogramMark`
  - [x] `HeatmapMark`
- Statistics
  - [x] HistogramMark binning: Count, Width, FreedmanDiaconis variants
- Themes
  - [x] `prismatica` colormap integration with `HeatmapMark`
- Tests
  - [x] Unit: bar group/stack offsets, histogram bin assignment
- Snapshots
  - [x] `bar_basic`, `bar_horizontal`, `bar_grouped`, `bar_stacked`,
        `area_basic`, `step_basic`, `histogram_basic`, `histogram_freedman`,
        `heatmap_basic`, `heatmap_diverging`, …

### 0.3.0 — Core chart types part 2 [shipped]

- Marks
  - [x] `BoxPlotMark` with five-number summary
  - [x] `ViolinMark` (split / grouped / box-overlay variants)
  - [x] `PieMark` / `DonutMark` (via `ArcMark`)
  - [x] `CandlestickMark`
  - [x] `ContourMark` (filled + isolines, marching squares)
  - [x] Polar suite: `ArcMark`, `PolarBarMark`, `PolarRectMark`, `RadarMark`
  - [x] `ErrorBarMark`
  - [x] `RugMark`
- Scales
  - [x] `LogScale`, `SqrtScale`
  - [x] `CategoricalScale` (initial form for polar angular bins, etc.)
  - [x] `PolarCoord` (continuous and categorical angular axis)
- Statistics
  - [x] BoxPlot summary stats with Tukey fences (1.5·IQR outlier rule)
  - [x] KDE 1D with bandwidth + kernel choice (Gaussian default)
- Layout
  - [x] `LegendStripComponent`
  - [x] `LegendPosition::Inside` (default + least-overlap fallback)
  - [x] `LegendPosition::Outside` (right / top / bottom / left)
  - [x] `LegendPosition::Auto` — `Mark::prefers_outside_legend` trait method
        with overrides for the five disk-fill marks (Pie, Donut, Nightingale,
        Radar, WindRose)
  - [x] Auto-attached `Colorbar` for `HeatmapMark` / `ContourMark`
  - [x] `MultiPanelFigure` (pulled forward from 0.4 when Epic L delivered the
        `LayoutBuilder` infrastructure)
- Themes
  - [x] Per-mark `color_by` aesthetic
  - [x] User-defined `chromata::Theme` overrides
  - [x] Dark-mode env helpers
- Infrastructure
  - [x] Polars data-acceptance layer (Layer 5, behind `polars` feature flag);
        accepts eager `DataFrame` and `LazyFrame`; auto-detects column types
  - [x] `BarMark` band-center math via `f64` (eliminates 1–2 px drift on
        Bollinger overlays at high category counts)
  - [x] cargo deny licenses (BSL-1.0 / CDLA-Permissive-2.0 / NCSA / GPL-3.0)
        and RUSTSEC-2024-0436 paste ignore
- Tests
  - [x] Property tests for box-plot summaries, KDE invariants
- Snapshots
  - [x] `boxplot_basic`, `boxplot_grouped`, `boxplot_outliers`, `violin_basic`,
        `violin_split`, `pie_basic`, `donut_basic`, `candlestick_basic`,
        `contour_filled`, `contour_isolines`, `polar_arc`, `polar_bar`,
        `radar_spider`, `errorbar_basic`, `rug_basic`,
        `legend_dodge_diagonal`, `legend_dodge_full_range`,
        `legend_dodge_outside_right`, `pie_auto_outside_legend`,
        `polar_legend_dodge`, …

### 0.4.0 — Layout, composition, and theme foundations [in progress]

- Marks
  - [ ] `ContourMark` filled bands (between isolines)
  - [ ] `BarMark` mandatory zero-baseline default; `without_zero_baseline()`
        opt-out [VIZ §1.4 — Correll-Bertini-Franconeri 2020 CHI]
- Layout
  - [ ] `GridLayout` (configurable rows × cols, weighted spans)
  - [ ] `FacetWrap` (1D, wraps to grid)
  - [ ] `FacetGrid` (2D, row × col)
  - [ ] Shared axes across panels (free / fixed per axis)
  - [ ] Polar-aware legend placement (extends `Auto` policy to polar plots)
  - [ ] `JointPlot` shorthand (main + marginals)
  - [ ] `PairPlot` shorthand (n × n scatter matrix)
- Themes
  - [ ] LaTeX-style default theme (Epic J / `starsight-5hq`)
  - [ ] Backend-asymmetric background defaults +
        `Figure::background(BackgroundMode)` (`starsight-3y8`)
  - [ ] Default categorical palette: Okabe-Ito (Wong) [VIZ §1.2 — Wong 2011
        *Nature Methods*]; replaces the placeholder Tableau-10 default
- Infrastructure
  - [ ] Theme-aware `Mark::render(coord, backend, theme)` — widen the trait
        signature so all marks become theme-aware (`starsight-max`)
  - [ ] Antipattern warning infrastructure: emit `tracing` WARN at
        plot-construction time for the cases listed in section 6 [VIZ §5]
- Tests
  - [ ] Unit: facet split correctness, grid weight math, legend placement
  - [ ] Antipattern warning emission on each detection rule
- Snapshots
  - [ ] `grid_2x2`, `grid_row`, `grid_unequal`, `facet_wrap_basic`,
        `facet_grid_2d`, `joint_plot_basic`, `pair_plot_basic`,
        `theme_latex_default`, `bar_zero_baseline_warn`, …

### 0.5.0 — Scale infrastructure

- Scales
  - [ ] `BandScale` (categorical with padding)
  - [ ] `SymLogScale` (signed-symmetric log)
  - [ ] `DateTimeScale` (chrono / time interop)
  - [ ] `TimedeltaScale`
  - [ ] `ContinuousScale<Quantity<D, U, V>>` via `uom` [Axiom 2]
- Statistics
  - [ ] `ColorScale` full implementation (sequential / diverging / cyclic)
  - [ ] Diverging / cyclic colormap helpers (VIK / BERLIN diverging,
        ROMA_O cyclic from prismatica)
  - [ ] Regression / smoothing (OLS, LOESS)
  - [ ] Improved Sheather-Jones KDE bandwidth default [VIZ §1.10]
  - [ ] `BoxPlotMark` auto-promote to letter-value above n ≈ 1000 [VIZ §3.8 —
        Hofmann-Kafadar-Wickham 2017 JCGS]
  - [ ] DKW-banded ECDF (Massart C = 2) [VIZ §1.10]
  - [ ] Banking-to-45° default for `LineMark` (median-absolute-slope) [VIZ §1.5
        — Cleveland 1988 / Heer-Agrawala 2006]
- Output
  - [ ] WebP-lossless raster output (~25% smaller than PNG)
- Infrastructure
  - [ ] `Figure::to_spec() -> serde_json::Value` + `from_spec()` round-trip
        [Axiom 6]
  - [ ] Provenance writer: `with_provenance(BTreeMap<String, Value>)` +
        sidecar `.prov.json` on `save()` [Axiom 4]
  - [ ] Pixel-aware aggregation infrastructure (Mosaic-style above-N path);
        Mark trait exposes a render strategy the pipeline can introspect
        [VIZ §1.7, §3.1]
- Tests
  - [ ] Round-trip: `Figure → to_spec → from_spec → Figure` byte-identical
        across renders
  - [ ] Property: scale dimensional algebra rejects mixed-unit construction
- Snapshots
  - [ ] `band_scale_basic`, `log_scale_decades`, `symlog_basic`,
        `datetime_axis`, `regression_loess`, `boxplot_letter_value_n5000`,
        `ecdf_dkw_band`, `line_banking_default`, `colormap_diverging_vik`, …

### 0.6.0 — GPU + interactivity

- Backends
  - [ ] `WgpuBackend` (GPU instanced quads + MSDF text + zero-copy Arrow upload)
- Infrastructure
  - [ ] Layer 6 interactivity behind `interactive` feature flag [Axiom 5]
  - [ ] Yi et al. seven user-intent primitives (IEEE TVCG 2007): `Select`,
        `Explore`, `Reconfigure`, `Encode`, `Abstract/Elaborate`, `Filter`,
        `Connect`
  - [ ] Hover, brush, link, zoom, pan
  - [ ] Streaming data with rolling window
  - [ ] Reactive layer (Observable-style signals; selections-as-predicates per
        Vega-Lite)
- Tests + Snapshots
  - [ ] Visual regression on WgpuBackend output (SSIM ≥ 0.99 vs Skia for
        simple cases)
  - [ ] `wgpu_scatter_1m`, `wgpu_streaming_window`, `interactive_brush`, …

### 0.7.0 — 3D + cartographic projections

- Marks
  - [ ] `Surface3DMark`, `Scatter3DMark`, `Line3DMark`, `Wireframe3DMark`,
        `Arrow3DMark`, `IsosurfaceMark` (Chernyaev-corrected marching cubes,
        not vanilla Lorensen-Cline)
  - [ ] Volume rendering
- Backends
  - [ ] `Cartesian3D` coord
- Scales (cartography)
  - [ ] Equal-area projections: Mollweide, Albers, Eckert IV [VIZ §3.7]
  - [ ] Mercator with continuous-data warning (forbidden for thematic)
        [VIZ §3.7, §5]
  - [ ] Winkel Tripel (compromise reference world)
- Infrastructure
  - [ ] Orbit camera (Layer 7)
- Snapshots
  - [ ] `surface3d_mobius`, `scatter3d_basic`, `wireframe3d_floor`,
        `isosurface_orbital`, `isosurface_gyroid`, `geo_mollweide`,
        `geo_albers_us`, `mercator_warning`, …

### 0.8.0 — Terminal cascade

- Backends
  - [ ] `TerminalBackend` with auto-detect protocol cascade:
        Kitty graphics → iTerm2 inline → Sixel → half-block → Braille (2×4
        dots/cell) → Unicode 16 octant
  - [ ] Forced lowest-common-denominator output for CI logs
- Infrastructure
  - [ ] `ratatui` Widget integration
- Snapshots
  - [ ] Terminal output is text; snapshot via golden-string comparison rather
        than image SSIM
  - [ ] `tty_kitty_scatter`, `tty_sixel_heatmap`, `tty_braille_line`,
        `tty_octant_dense`, …

### 0.9.0 — Long-tail mark taxonomy

- Marks
  - [ ] `SankeyMark`
  - [ ] `TreemapMark` (squarified, Bruls-Huizing-van Wijk 2000)
  - [ ] `DendrogramMark` (Ward / single / complete / average linkage)
  - [ ] `NetworkMark` (force-directed + edge bundling)
  - [ ] `StreamgraphMark` (Byron-Wattenberg 2008 baseline + wiggle ordering)
  - [ ] `RidgelineMark` (custom y-offset layout)
  - [ ] Slope graphs, Bullet, Horizon graph (Heer-Kong-Agrawala 2009 layering
        thresholds)
  - [ ] `ParallelCoordMark`
  - [ ] Connected scatterplot (with explanatory-only doc warning;
        Haroz-Kosara-Franconeri 2016)
  - [ ] `GeoMark` (polygon fill, line, point with Natural Earth data adapter)
  - [ ] `TernaryCoord` (long-tail coordinate system)
  - [ ] `HexbinMark`, `EllipseMark`, `ArrowMark`, `RuleMark`, `TextMark`
  - [ ] Glyph viz (Chernoff faces) marked as discouraged in docs (Raciborski
        2009; Kosara 2007); `RadarMark` already shipped at 0.3 — also marked
        discouraged for accuracy beyond expert audiences
- Output
  - [ ] Geographic raster tiles (MVT / PMTiles read path) for `GeoMark` basemap
- Snapshots
  - [ ] `sankey_energy_flow`, `treemap_market_cap`, `dendrogram_clustermap`,
        `network_force`, `streamgraph_genres`, `ridgeline_temps`,
        `parallel_coords`, `geo_choropleth`, `ternary_soil`, …

### 0.10.0 — Export, animation, WASM

- Backends
  - [ ] `KrillaBackend` (.pdf, **PDF/A-3u** with embedded provenance JSON +
        parquet of input data) [Axiom 4]
  - [ ] `WasmBackend` (browser embed, .wasm)
- Output
  - [ ] .gif via Heer-Robertson 2007 staged transitions (~1 s default, cubic
        ease-in-out, identity-preserving — no linear interpolation)
        [VIZ §2.4]
  - [ ] .html interactive (WebGPU)
  - [ ] PGFPlots / TikZ emitter for direct LaTeX inclusion
- Infrastructure
  - [ ] `ndarray` + `arrow::RecordBatch` data-acceptance (Layer 5)

### 0.11.0 — Polish + linting

- Output
  - [ ] Additional dataframe accept paths (DuckDB result, plain CSV via
        `polars-csv`)
- Infrastructure
  - [ ] `chartability_check()` — Chartability heuristic suite
        (Elavsky-Bennett-Moritz 2022 EuroVis) [VIZ §1.8]
  - [ ] `simulate_cvd()` — protan / deutan / tritan colorblindness simulation
        for theme/colormap audit [VIZ §1.2]
  - [ ] Bundle Atkinson Hyperlegible Next under `bundled-fonts` feature for
        guaranteed reproducibility + accessibility (SIL OFL-licensed)
  - [ ] Recipe proc-macro for ecosystem extension (Makie `@recipe` analog)
  - [ ] API audit: deprecation pass before 1.0
- Snapshots
  - [ ] `chartability_pass`, `chartability_fail_lowcontrast`, `cvd_protan`, …

### 0.12.0 — Documentation

- Documentation
  - [ ] 25-step tutorial ladder (wiki: Hello starsight → custom Mark / Backend)
  - [ ] 14-library comparison (wiki: matplotlib / seaborn / ggplot2 / plotly /
        Vega-Lite / Plot / D3 / Bokeh / gnuplot / PGFPlots / Datashader /
        Makie / Plotters / ECharts)
  - [ ] Primary-research citations in API rustdoc (Cleveland-McGill 1984;
        Crameri 2020; Talbot-Lin-Hanrahan 2010; Lundgard-Satyanarayan 2022;
        Heer-Bostock 2010; Munzner 2014; Wickham 2010)
  - [ ] Design rationale section explaining each default
- Output
  - [ ] Gallery renders all 42 examples (`xtask gallery`)

### 1.0.0 — Stable

- API
  - [ ] Stable API surface declared
  - [ ] Version-bump policy commitment (semver; major bumps require RFC)
  - [ ] Crate-split decision: continue 7-layer or migrate toward the 15-crate
        vision (informed by post-0.5 experience with `starsight-spec`,
        `starsight-bayes`, `starsight-geo`)
  - [ ] NumFOCUS Affiliated Project application (paths to CZI ESS grants)

---

## 5. Empirical defaults

Triangulated across `STARSIGHT_DESIGN_REFERENCE.md` and
`STARSIGHT_VIZ_RESEARCH.md`. Citations are 1-line; the underlying primary
sources are catalogued in `STARSIGHT_VIZ_RESEARCH.md` Appendix A.

| Decision | Default | Citation | Active in |
|---|---|---|---|
| Sequential colormap | `prismatica::crameri::BATLOW` | Crameri 2020, *Nature Communications* | 0.2 (HeatmapMark) |
| Diverging colormap | `VIK` (or `BERLIN`) | Crameri 2020 | 0.3 (ContourMark) |
| Cyclic colormap | `ROMA_O` | Crameri 2020 | 0.3 (polar) |
| Categorical (≤8) | Okabe-Ito (Wong) | Wong 2011, *Nature Methods* | 0.4 |
| Many-category (>8) | Glasbey / Polychrome max-distance | VIZ §1.2 | 0.5 |
| Histogram bins | Freedman-Diaconis | VIZ §1.10 | 0.2 |
| KDE bandwidth | Improved Sheather-Jones | VIZ §1.10 | 0.5 |
| Quantile method | Hyndman-Fan type 7 (numpy / R compat) | VIZ §1.10 | 0.5 |
| Box plot | Tukey at n ≤ 1000; auto-promote to letter-value above | Hofmann-Kafadar-Wickham 2017, JCGS | 0.5 |
| ECDF bands | DKW with Massart C = 2 | VIZ §1.10 | 0.5 |
| Tick algorithm | Talbot-Lin-Hanrahan extended Wilkinson | Talbot et al. 2010, IEEE InfoVis | 0.1 |
| Bar y-axis | Includes zero by default; opt-out with warning | Correll-Bertini-Franconeri 2020, CHI | 0.4 |
| Banking | 45° via median-absolute-slope | Cleveland 1988 / Heer-Agrawala 2006 | 0.5 |
| Color interpolation | OKLab / OKLCh, never sRGB | All 3 research passes | 0.4 |
| WCAG | AA (4.5:1 text, 3:1 marks vs background) | WCAG 2.1 | 0.4 (Theme audit) |
| Reproducibility | Sorted SVG attrs, 4-decimal precision, seeded jitter | VIZ §1.9 | 0.4 |
| Default raster DPI | 300 (matplotlib default is 100) | DESIGN §16 | 0.4 |
| Default web raster | WebP-lossless (~25% smaller than PNG) | DESIGN §16 | 0.5 |
| Default PDF | PDF/A-3u with embedded source data | DESIGN §16 | 0.10 |
| Histogram overflow/underflow bins | Always shown (ROOT convention) | DESIGN §16 | 0.5 |
| Confidence/credible intervals | Shown when input has uncertainty metadata | DESIGN §16 | 0.5 |
| Locale formatting | Auto-detect from `LANG` / `LC_ALL`; respect decimal separator + digit grouping | DESIGN §10 | 0.6 |
| Atkinson Hyperlegible Next | Bundled under `bundled-fonts` feature | VIZ §3.9 | 0.11 |

---

## 6. Antipatterns + warnings

Detected at plot-construction time (not at render time) and emitted via
`tracing` at WARN level. Construction is not refused — users can override —
but the default behaviour surfaces the known empirical failure mode.

| Antipattern | Detection rule | Warned in | Suggested alternative |
|---|---|---|---|
| Bar with non-zero baseline | `BarMark` with `y_min != min(0, data_min)` and no explicit `without_zero_baseline()` | 0.4 | Add zero baseline, or use `LineMark` for trend |
| Pie > 5 slices | `PieMark` / `DonutMark` slice count > 5 | 0.4 | `BarMark` or `WaffleMark` |
| Rainbow / jet on sequential data | colormap signature matches jet/rainbow (channel-pattern test) | 0.4 | `BATLOW` or any prismatica viridis-class |
| > 8 categorical colors | distinct categorical color count > 8 | 0.4 | Faceting, shape encoding, or direct labeling |
| 3D bar / 3D pie | `BarMark3D` or `PieMark3D` constructed (planned only — not shipped) | 0.7 | 2D equivalent |
| Dual y-axis | Two `YAxis` instances on one `CartesianCoord` | 0.4 | Small multiples or normalized index |
| Single-error-bar dynamite plot | `BarMark` + `ErrorBarMark` with n ≤ 5 underlying samples | 0.4 | Strip / dot plot showing raw points |
| Mercator on thematic continuous data | `Mercator` projection + continuous data layer | 0.7 | Mollweide / Albers / Eckert IV |
| Tukey boxplot on n > 1000 | `BoxPlotMark` constructed with n > 1000 and `tukey()` not explicitly called | 0.5 | Auto-promote to letter-value |
| Color-only categorical encoding | Only `color` channel encodes class | 0.4 | Add shape or pattern channel |
| Connected scatterplot for non-explanatory use | `ConnectedScatterplotMark` without explicit `.explanatory()` flag | 0.9 | Two side-by-side plots |
| Glyph viz (Chernoff) | `ChernoffMark` constructed | 0.9 (doc warning) | Small multiples |

---

## 7. Common rules

### Architecture

See [`AGENTS.md`](../AGENTS.md) for the layered workspace invariant
(layer-N depends only on 1..N−1) and [`.claude/rules/`](../.claude/rules) for
module-level conventions. **This file does not duplicate those.**

### Naming

Module names are pluralized to satisfy `clippy::module_name_repetitions` with
`allow-exact-repetitions = false`. Use `marks::Mark`, not `mark::Mark`. Public
type names mirror this: `Mark` lives in `marks` (or as `crate::Mark` via
re-export from `lib.rs`).

### Library code never `unwrap()` or `expect()` on user input

Return `Result<T, StarsightError>`. Internal invariants that cannot be violated
without a programmer error may use `.expect("…")` with a clear message; user-
facing data must always be fallible. (Echoes [`CLAUDE.md`](../CLAUDE.md).)

### Snapshot / test conventions

- Snapshots live under `starsight-layer-5/tests/snapshots/` and use `insta`.
- Naming: `<feature>_<variant>` (e.g. `line_basic`, `bar_negatives`,
  `polar_legend_dodge`).
- Update workflow: `/snap` skill (review by default, `--snap --update` to
  accept).
- `cargo insta test --check --unreferenced=reject` rejects orphans on CI.
- For non-image outputs (terminal protocol bytes), snapshot via
  `insta::assert_snapshot!` (string-comparison) rather than image SSIM.

### Key technical facts

These survive the rewrite of `agents.md` and are referenced by the codebase:

- `tiny-skia 0.12` renamed `ClipMask` → `Mask`.
- `Color::from_rgba()` returns `Option`; `from_rgba8()` is infallible.
- `cosmic-text` R↔B channel swap applies only to `softbuffer` display, **not**
  to PNG / SVG output.
- Wilkinson Extended tick weights: simplicity 0.2, coverage 0.25, density 0.5,
  legibility 0.05; nice numbers `{1, 2, 2.5, 5} × 10ⁿ`.
- `Point + Point` does not compile (no `Add<Point> for Point`).
  `Point - Point = Vec2` (displacement).
- Premultiplied alpha: `(r·a, g·a, b·a, a)` — `tiny-skia` uses this internally;
  watch for it when manually composing pixmaps.
- WCAG contrast: `(L1 + 0.05) / (L2 + 0.05)`, minimum 4.5:1 for normal text,
  3:1 for non-text marks against the chart background.
- Edition 2024, resolver 3, MSRV 1.85, license GPL-3.0-only (workspace +
  every member crate).

### Deduplication rule

Anything documented in [`CLAUDE.md`](../CLAUDE.md), [`AGENTS.md`](../AGENTS.md),
or [`.claude/rules/`](../.claude/rules) is **linked**, not copied. This section
is capped at ~200 lines; if it grows, audit for duplication first.

---

## 8. Showcase examples

Verbatim copy of the chart-by-chart math inputs, consumed by `examples/*.rs`.
Each example heading has an explicit `<a id="example-N"></a>` anchor (since
markdown auto-slugs from heading text would be unstable across renamings).
Cross-references from `CONTRIBUTING.md`, `examples/*.rs`, and `CHANGELOG.md`
point to these anchors.

### 0.1.0 — Line + Point + LinearScale + tiny-skia backend

Only `LineMark` and `PointMark` on `Cartesian2D` with `LinearScale`. No layout,
no stats, no faceting.

<a id="example-1"></a>
#### 1  Lorenz attractor family · `0.1.0` (Line only) / `0.7.0` (3D)

**Mark:** `LineMark` (0.1.0 2D projection), `Line3DMark` (0.7.0)
**Coord:** `Cartesian2D` (0.1.0) → `Cartesian3D` (0.7.0)
**Why 0.1.0:** Only needs LineMark + LinearScale. Project to 2D manually.
**Unlocked at 0.7.0:** True 3D with orbit camera.

Classic Lorenz system, integrated with RK4:

```
dx/dt = σ (y − x)
dy/dt = x (ρ − z) − y
dz/dt = x y − β z
```

Generate 11 trajectories by sweeping ρ ∈ {13, 15, 18, 21, 24.06, 28, 35, 50, 100, 160, 250}
with σ = 10, β = 8/3.  IC: (1, 1, 1) + tiny per-trajectory jitter (0.001 · i).
Step dt = 0.005, 80 000 steps each, discard first 5 000 (transient).

For multi-Poincaré variant, additionally store plane-crossing events
at z = ρ − 1 and plot 2D return maps (xₙ₊₁ vs xₙ) as scatter insets.

Color each trajectory by its ρ value on `inferno` from prismatica.

<a id="example-6"></a>
#### 6  Kruskal–Szekeres diagram · `0.1.0`

**Mark:** `LineMark`, `TextMark`
**Coord:** `Cartesian2D`
**Why 0.1.0:** Only parametric lines. TextMark is a stretch — use title/annotation fallback or defer text to 0.9.0.

Schwarzschild → Kruskal-Szekeres coordinate transform:

```
For r > 2M:
  u = ±√(r/2M − 1) · e^(r/4M) · cosh(t/4M)
  v =  √(r/2M − 1) · e^(r/4M) · sinh(t/4M)

For r < 2M:
  u =  √(1 − r/2M) · e^(r/4M) · sinh(t/4M)
  v = ±√(1 − r/2M) · e^(r/4M) · cosh(t/4M)
```

Set M = 1.
Draw curves of constant r: r ∈ {0, 0.5, 1.0, 1.5, 1.9, 2.0, 2.5, 3.0, 5.0, ∞}.
Draw curves of constant t: t ∈ {0, ±1, ±2, ±3, ±∞}.
r = 2M (horizon) maps to u = ±v (45° lines).
r = 0 (singularity) maps to v² − u² = 1 (hyperbola).

Parametrize each constant-r curve by sweeping t ∈ [−20, 20], 500 points.
Parametrize each constant-t curve by sweeping r ∈ [0.01, 8], 500 points.

### 0.2.0 — Bar + Area + Histogram + Heatmap

Adds `BarMark`, `AreaMark`, `Bin` stat, `HeatmapMark`.

<a id="example-3"></a>
#### 3  Bubble scatter with continuous color + size · `0.2.0`

**Mark:** `PointMark` with `size` and `color` aesthetics
**Scale:** `ColorScale` sequential (RdPu from prismatica)
**Why 0.2.0:** PointMark exists at 0.1.0, but continuous color+size aesthetics need the
`ColorScale` infrastructure partially landing here (full `ColorScale` at 0.5.0 — can
use a manual color-map fallback at 0.2.0).

```
N = 178
alcohol ~ Uniform(11.0, 15.0)
color_intensity = 0.8 · alcohol − 4.0 + N(0, 1.5)
proline = 300 + 80 · alcohol + N(0, 200)
```

Size aesthetic: `sqrt(proline) * 0.5`
Color aesthetic: `color_intensity`
Alpha: 0.5

<a id="example-7"></a>
#### 7  Laser–plasma phase space · `0.2.0`

**Marks:** `HeatmapMark` (phase space), `LineMark` + `AreaMark` (density profiles)
**Layout:** `GridLayout` 4×3 → deferred to 0.4.0, use single-panel per image at 0.2.0
**Why 0.2.0:** Needs HeatmapMark + AreaMark (both 0.2.0). Full multi-panel at 0.4.0.

Stimulated Raman scattering simulation snapshots:

```
x ∈ [159.0, 160.0] µm, 1000 points
p_ion ∈ [−100, 100] keV/c, 200 bins
p_electron ∈ [−5, 5] keV/c, 200 bins
```

Phase space density via analytic warm plasma model:

```
f_e(x, p) = n₀/√(2π T_e) · exp(−p²/(2 T_e)) · [1 + δ cos(k_L x)]
f_i(x, p) = n₀/√(2π T_i) · exp(−p²/(2 T_i)) · [1 + ε cos(k_IAW x)]
```

With T_e = 2.0 keV, T_i = 0.05 keV, k_L = 2π/0.2, k_IAW = 2π/0.35,
δ evolving over 3 snapshots: 0.02, 0.15, 0.6.
ε = δ/3.

Density profile (bottom row): integrate f over p, plot n_e(x) and n_i(x).
Electric field envelope (top row): |E| ~ √(n_e perturbation) scaled.

Color: sequential `viridis`, log scale on particle count.

<a id="example-16"></a>
#### 16  Perspective heatmap table · `0.2.0`

**Mark:** `HeatmapMark`
**Scale:** `ColorScale` sequential
**Why 0.2.0:** Pure HeatmapMark, no other dependencies.

Synthetic movie-rating cross-tabulation:

```
x_bins = 30 (Rotten Tomatoes rating buckets, 0–100 in steps of ~3.3)
y_bins = 30 (IMDB rating buckets, 1.0–10.0 in steps of 0.3)

cell_value(i, j) = max(0, 2e9 · φ(x_i; 65, 15) · φ(y_j; 7.0, 1.0)
                        − 5e8 · φ(x_i; 30, 10) · φ(y_j; 4.0, 0.8))
```

where φ is the Gaussian PDF. Produces a bright ridge along the positive-correlation
diagonal with a dimmer secondary lobe. Log-scale color mapping.

<a id="example-37"></a>
#### 37  Waterfall chart · `0.2.0`

**Mark:** `BarMark` (floating, base ≠ 0)
**Why 0.2.0:** Waterfall is just BarMark with explicit base positions. Needs BarMark (0.2.0).

```
Item               Value         Running Total    Type
Revenue           +4,200,000     4,200,000       increase
COGS              −1,800,000     2,400,000       decrease
Gross Profit      [subtotal]     2,400,000       subtotal
OpEx                −900,000     1,500,000       decrease
R&D                 −500,000     1,000,000       decrease
Marketing           −300,000       700,000       decrease
EBITDA            [subtotal]       700,000       subtotal
D&A                 −150,000       550,000       decrease
Interest             −50,000       500,000       decrease
Net Income        [total]         500,000        total
```

Each bar's base = running total after previous entries; top = running total after current.
Colors: increase `#2e7d32`, decrease `#c62828`, subtotal/total `#1565c0`.
Thin gray connector lines between bars. Currency-formatted labels outside bars.

### 0.3.0 — BoxPlot + Violin + KDE + Pie/Donut + Contour + Candlestick

Adds `BoxPlotMark`, `ViolinMark`, `KDE` stat, `PieMark`/`DonutMark` (`ArcMark`),
`ContourMark`, `CandlestickMark`. Also Polars integration.

<a id="example-19"></a>
#### 19  Violin raincloud plot · `0.3.0`

**Mark:** `ViolinMark` + `PointMark` (strip) + `BoxPlotMark` (mini)
**Stat:** `KDE` (1D, Gaussian kernel, bandwidth factor 0.5)
**Why 0.3.0:** ViolinMark + KDE stat + BoxPlotMark all land at 0.3.0.

Multimodal data:

```
Category A (bimodal):
    weights = [0.3, 0.7]
    component_1 ~ N(−2.0, 0.8),  component_2 ~ N(6.5, 1.2)
    n = 10 000

Category B (unimodal):
    N(4.0, 1.5),  n = 10 000

Category C (bimodal symmetric):
    weights = [0.5, 0.5]
    component_1 ~ N(1.5, 0.5),  component_2 ~ N(7.0, 0.8)
    n = 10 000

Category D (skewed):
    LogNormal(mean=1.2, σ=0.4),  n = 10 000
```

Raincloud layout: half-violin on one side, jittered strip (alpha 0.05, jitter width 0.15)
on the other, miniature box plot in the center. KDE `cut=0` to clip tails at data range.

<a id="example-22"></a>
#### 22  Contour plots — four scalar fields · `0.3.0`

**Mark:** `ContourMark` (filled + isolines)
**Stat:** `Contour` (marching squares)
**Why 0.3.0:** ContourMark lands at 0.3.0.

##### Panel A — Rosenbrock ("banana valley")

```
f(x, y) = (1 − x)² + 100 · (y − x²)²
x ∈ [−2, 2], y ∈ [−1, 3], grid 200×200
levels: logspace(0, 3.5, 20)
```

##### Panel B — Himmelblau (multi-modal)

```
f(x, y) = (x² + y − 11)² + (x + y² − 7)²
x, y ∈ [−5, 5], grid 300×300
levels: logspace(−0.5, 3, 25)
```

Four identical basins at f = 0: (3.0, 2.0), (−2.805, 3.131), (−3.779, −3.283), (3.584, −1.848).

##### Panel C — Rastrigin (oscillating)

```
f(x, y) = 20 + x² − 10·cos(2πx) + y² − 10·cos(2πy)
x, y ∈ [−5.12, 5.12], grid 400×400
levels: linspace(0, 80, 30)
```

##### Panel D — Gaussian mixture (topographic)

```
5 peaks:
  weights  = [1.0, 0.8, 0.6, 0.7, 0.5]
  centers  = [(0,0), (3,2), (−2,3), (−1,−2), (2,−3)]
  stds     = [(1.0,1.0), (0.5,1.5), (1.2,0.8), (0.7,0.7), (1.5,0.5)]

f(x, y) = Σ wₖ · (1/(2π σx σy)) · exp(−(x−μx)²/(2σx²) − (y−μy)²/(2σy²))
x, y ∈ [−5, 6], grid 250×250
```

Colormap: filled `plasma`, overlay black isolines at half the level count with linewidth 0.5.

<a id="example-38"></a>
#### 38  Candlestick / OHLC with Bollinger Bands · `0.3.0`

**Mark:** `CandlestickMark`, `LineMark` (SMA + Bollinger), `BarMark` (volume)
**Layout:** `GridLayout` 2×1 → deferred to 0.4.0, single-panel at 0.3.0
**Why 0.3.0:** CandlestickMark lands at 0.3.0.

Geometric Brownian Motion with GARCH(1,1) volatility clustering:

```
Price process:
    S(t+dt) = S(t) · exp((μ − σ²(t)/2) · dt + σ(t) · √dt · Z)
    Z ~ N(0, 1)

GARCH(1,1):
    σ²(t) = ω + α · ε²(t−1) + β · σ²(t−1)
    ω = 0.00001,  α = 0.10,  β = 0.85
    (persistence α + β = 0.95)

Initial: S₀ = 100, μ = 0.0002
```

Simulate at 5-minute intervals (78 per 6.5-hour day = 9,360 ticks across 120 days),
aggregate daily: Open = first, Close = last, High = max, Low = min.

Volume: `V(t) = 10⁶ · exp(50 · |r(t)|)` where r(t) = daily log return.

Overlays: SMA(20), SMA(50), Bollinger Bands SMA(20) ± 2σ₂₀.
Candle colors: up `#26a69a`, down `#ef5350`.

<a id="example-39"></a>
#### 39  Donut / sunburst — energy transition · `0.3.0`

**Mark:** `ArcMark` on `PolarCoord`
**Why 0.3.0:** PieMark/DonutMark (arc geometry) lands at 0.3.0.

**Shipped status:** Variant A → `examples/composition/donut.rs` (donut with vote-share data; same arc geometry as the spec). Variant B → `examples/composition/energy_transition.rs`. Variant C → `examples/composition/donut_sunburst.rs`.

##### Variant A — Fibonacci donut

```
Segments: [34%, 21%, 17%, 13%, 9%, 6%]
Inner radius ratio: 0.60 of outer radius
Palette: #4E79A7, #F28E2B, #E15759, #76B7B2, #59A14F, #EDC948
```

##### Variant B — Energy transition multi-ring

```
Outer ring (2025):  Solar 22%, Wind 19%, Hydro 16%, Nuclear 18%, Gas 15%, Coal 10%
Inner ring (2020):  Solar 10%, Wind 12%, Hydro 18%, Nuclear 20%, Gas 22%, Coal 18%

Outer: outerRadiusRatio = 1.0, innerRadiusRatio = 0.75
Inner: outerRadiusRatio = 0.65, innerRadiusRatio = 0.40
```

##### Variant C — Software sunburst (3-level)

```
Frontend (2500 LOC):  React 1200, CSS 600, Tests 400, Utils 300
Backend (2300 LOC):   API 900, DB 700, Auth 400, Queue 300
Shared (400 LOC):     Types 250, Config 150
DevOps (500 LOC):     CI 200, Docker 180, Monitoring 120
```

<a id="example-34"></a>
#### 34  Nightingale coxcomb / rose chart · `0.3.0`

**Mark:** `ArcMark` on `PolarCoord` (radius ∝ √value)
**Why 0.3.0:** ArcMark lands at 0.3.0. PolarCoord needed (minimal form usable here).

Florence Nightingale Crimean War mortality rates (deaths per 1000 per annum):

```
Month       Disease   Wounds   Other
Apr 1854       1.4      0.7     7.0
May 1854       6.3      0.0    11.4
Jun 1854      11.4      0.0     9.7
Jul 1854      36.6      3.6    11.5
Aug 1854     104.0      2.9    12.6
Sep 1854     114.3      3.2     9.2
Oct 1854     136.2      4.1    10.4
Nov 1854     361.1      5.1    10.9
Dec 1854     361.7      4.9    14.8
Jan 1855     423.6     27.4    14.7
Feb 1855     401.4     31.8    12.2
Mar 1855     313.6     36.1    13.3
```

12 angular bins (months), 3 stacked radial layers. r ∝ √value.
Colors: blue (disease), red (wounds), black (other).

<a id="example-41"></a>
#### 41  Gauge / radial progress arc · `0.3.0`

**Mark:** `ArcMark` (partial annular ring)
**Coord:** `PolarCoord`
**Why 0.3.0:** ArcMark lands at 0.3.0.

```
startAngle = 225°, endAngle = −45°, total sweep = 270°
angle(v) = 225 − (v/100) · 270

Three color zones:
    [0–30%]:   #67e0e3 (teal)
    [30–70%]:  #37a2da (blue)
    [70–100%]: #fd666d (coral)

Current value: 67%  →  angle = 44.1°
Major ticks: every 10 units (27° apart)
Minor ticks: every 2 units (5.4° apart)
Arc width: 10 px
```

Multi-ring variant: 3 concentric arcs for CPU / Memory / Disk.

### 0.4.0 — Layout + Faceting + Legend + PairPlot + JointPlot

Adds `GridLayout`, `FacetWrap`, `FacetGrid`, `Legend`, `Colorbar`, `PairPlot`, `JointPlot`.

<a id="example-2"></a>
#### 2  Multi-panel distribution dashboard · `0.3.0` (shipped)

**Marks:** `BarMark` + `Bin` stat (0.2.0), `AreaMark` + `KDE` stat (0.3.0), `BoxPlotMark` (0.3.0), `PointMark` (0.1.0)
**Layout:** `MultiPanelFigure` 2×2 (0.3.0)
**Status:** Shipped at `examples/composition/distribution_dashboard.rs`. Originally tagged 0.4.0 because GridLayout was deferred; pulled forward when `MultiPanelFigure` covered the layout need during Epic L.

```
overall ~ Beta(α=5, β=3) scaled to [45, 92]
potential = overall + Exp(λ=0.08) · (92 − overall) + N(0, 2)
country = Categorical(weights) among 10 labels
N = 18 000
```

Panel A: histogram, 20 bins. Panel B: KDE. Panel C: box plots by country. Panel D: scatter.

<a id="example-4"></a>
#### 4  Neuroscience multi-panel · `0.4.0`

**Marks:** `LineMark` (0.1.0), `PointMark` (0.1.0), `BarMark` (0.2.0), `ErrorBarMark`, `EllipseMark`
**Layout:** complex `GridLayout` with spanning (0.4.0)
**Why 0.4.0:** GridLayout with spanning needed. ErrorBar and Ellipse are annotation primitives
that could land at 0.2.0 or 0.3.0 but the layout gates the full example.

```
distance ∈ [0, 4] µm, 50 points
dendrite_weight ~ N(−0.3, 0.15)
axon_weight     ~ N(0.1, 0.2)
```

Panel H/I: scatter + error bars, 3 series. Panel D: paired dot plot (AUC 0.5–1.0).
Panel J: PCA scatter, 2 clusters, 95% confidence ellipses via
`x(t) = μ + √(χ²₀.₉₅,₂) · (λ₁½ cos t · v₁ + λ₂½ sin t · v₂)`.

<a id="example-5"></a>
#### 5  Microbiome clustering panels · `0.4.0`

**Marks:** `PointMark` (0.1.0), `BarMark` stacked + horizontal (0.2.0)
**Layout:** `GridLayout` with spanning (0.4.0)
**Why 0.4.0:** GridLayout with spanning.

PCoA scatter: 3 Gaussian clusters:

```
cluster_1: N([−0.5, 0.8], Σ₁),  n₁ = 600
cluster_2: N([0.5, −0.4], Σ₂),  n₂ = 800
cluster_3: N([−0.2, −0.6], Σ₃), n₃ = 400
Σ_k = [[0.04, ρ·0.02], [ρ·0.02, 0.03]]
```

Stacked bar: 11 species, 3 states, Dirichlet proportions.
Horizontal bars: top-10 driver species per state.

<a id="example-9"></a>
#### 9  Hexbin joint plot · `0.4.0`

**Marks:** `HexbinMark`, `BarMark` (marginals)
**Layout:** `JointPlot` shorthand (0.4.0)
**Why 0.4.0:** JointPlot layout lands at 0.4.0. HexbinMark can be added at 0.2.0 or 0.3.0.

```
N = 5000
x ~ Gamma(shape=2.0, scale=1.0)
y = −0.4 · x + N(0, 0.8)
```

Hexbin: flat-top, 25 hexes across. Color: `BuGn`, log(count+1).
Marginals: 40-bin histograms. Kendall τ annotation as `TextMark`.

<a id="example-10"></a>
#### 10  PCA / ICA decomposition · `0.4.0`

**Mark:** `PointMark` with density-based color, `ArrowMark`
**Layout:** `GridLayout` 2×2 (0.4.0)
**Why 0.4.0:** GridLayout.

```
s₁ ~ Laplace(0, 1),  s₂ ~ Laplace(0, 1),  N = 50 000
A = [[1, 1], [0, 2]]
[a, b]ᵀ = A · [s₁, s₂]ᵀ
```

4 panels: source, mixed + PCA/ICA axes, PCA-whitened, ICA-recovered.
Point color: 2D histogram density bin → `jet`-like colormap.

<a id="example-11"></a>
#### 11  Multi-model regression comparison · `0.4.0`

**Marks:** `PointMark`, `LineMark` (LOESS), `RuleMark`
**Layout:** `FacetGrid` 4×2 (0.4.0)
**Why 0.4.0:** FacetGrid.

```
N = 300, x_actual = Uniform(0.5, 10.0)
y_true = 4.8 · (1 − exp(−0.5 · x_actual))
```

7 models with bias functions:
```
bias_boosting(x)  = 0.3 · sin(x)
bias_elastic(x)   = −0.15 · x + 0.8
bias_nn(x)        = 0.2 · (x − 5)²/25
bias_rf(x)        = 0.1 · x
```

Top: scatter + LOESS + 1:1 rule. Bottom: residuals.

<a id="example-13"></a>
#### 13  Density scatter facet grid · `0.4.0`

**Marks:** `PointMark` with KDE-2D color, `LineMark` (regression)
**Layout:** `FacetWrap` 4×3 (0.4.0)
**Why 0.4.0:** FacetWrap.

12 organism panels. Per organism (ρ, intercept, σ):

```
E. coli: (−0.47, 1.0, 0.6)  ...  H. sapiens: (−0.25, 0.5, 0.8)
log_expression ~ N(3.0, 1.5)
log_evol_rate = ρ · log_expression + intercept + N(0, σ)
```

2D KDE for point color → `YlOrRd`. OLS regression line.

<a id="example-14"></a>
#### 14  SHAP beeswarm + heatmap · `0.4.0`

**Marks:** `PointMark` (beeswarm), `HeatmapMark`
**Layout:** `GridLayout` 3×3 (0.4.0)
**Why 0.4.0:** GridLayout. Beeswarm is a PointMark position adjustment.

```
n_samples = 2000
shap_j = β_j · (feature_value_j − 0.5) + N(0, 0.005)
β = [0.025, 0.018, 0.012, 0.010, 0.006, 0.003]
```

Beeswarm: y = feature index (density-jittered), x = SHAP, color = feature value.

<a id="example-15"></a>
#### 15  Palmer penguins multi-view · `0.4.0`

**Marks:** `HexbinMark`, `PointMark`, `LineMark`, `AreaMark` (KDE)
**Layout:** `PairPlot` (0.4.0)
**Why 0.4.0:** PairPlot shorthand.

```
Adelie:    body_mass ~ N(3700, 460),  culmen_length ~ N(38.8, 2.7), ...
Chinstrap: body_mass ~ N(3733, 384),  culmen_length ~ N(48.8, 3.3), ...
Gentoo:    body_mass ~ N(5076, 504),  culmen_length ~ N(47.5, 3.1), ...
N = 344
```

Upper tri: hexbin. Diagonal: KDE per species. Lower tri: scatter + OLS.

<a id="example-17"></a>
#### 17  Reciprocal-space scattering maps · `0.3.0` (shipped)

**Marks:** `HeatmapMark` (0.2.0), `PointMark` + `ErrorBarMark` (0.3.0)
**Layout:** `MultiPanelFigure` 2×3 (heatmap row + h-cut row) (0.3.0)
**Status:** Shipped at `examples/scientific/reciprocal_space.rs`. Originally tagged 0.4.0; pulled forward when `MultiPanelFigure` + `ErrorBarMark` both landed in 0.3.0 (Epic L Phase I).

```
S(h, k) = Σ_Q  A_Q / ((h − Q_h)² + (k − Q_k)² + Γ²)
```

Bragg peaks at Q ∈ {(1,0), (0,1), (2,1), (1,2), (½,½), (3/2,1/2)}, Γ = 0.05.
Three temperatures: A_Q scaled by [1.0, 0.7, 0.3].
Bottom row: 1D cuts with error bars √(I).

<a id="example-18"></a>
#### 18  Gene expression temporal profiles · `0.4.0`

**Marks:** `PointMark`, `LineMark` (spline), `BoxPlotMark`
**Layout:** `FacetGrid` 6×3 + side bar (0.4.0)
**Why 0.4.0:** FacetGrid.

```
time_points = [4, 5, 6, 7, 8]
y(t) = A_g · sin(ω_g · t + φ_{g,k}) + baseline_g + N(0, 0.15)
```

Six categories: Magnitude, Timing, Rate, Shape, Inversion, Additive.
3 genes per category, 30 replicates per time point per genotype.

### 0.5.0 — Scale infrastructure

Adds `LogScale`, `SymlogScale`, `DateTimeScale`, `BandScale`, `CategoricalScale`,
`ColorScale` (prismatica-backed), `TickLocator`/`TickFormatter` traits.

<a id="example-8"></a>
#### 8  Spiral heatmap / polar calendar · `0.5.0`

**Mark:** `RectMark` on `PolarCoord`
**Scale:** `DivergingColorScale` (0.5.0)
**Why 0.5.0:** DivergingColorScale + proper PolarCoord with categorical angular binning.

```
years = 1997..=2017  (21 rings)
weeks = 1..=52       (angular bins, 2π/52 each)

temperature(year, week) =
    10.0 · sin(2π · (week − 5)/52) + 0.05 · (year − 1997) + N(0, 3)
```

Map θ = 2π · (week − 1)/52. Map r = year − 1996.
Tile width: Δθ = 2π/52, Δr = 0.9.

<a id="example-33"></a>
#### 33  Polar bar / wind rose · `0.5.0`

**Mark:** `BarMark` on `PolarCoord` (stacked radial)
**Scale:** `CategoricalScale` (angular, 0.5.0) + `BandScale`
**Why 0.5.0:** CategoricalScale for 16 compass bins.

```
16 directions × 4 speed bins [0–5, 5–10, 10–15, 15+ m/s]

SSW = 16.8%, SW = 19.1% vs NE = 4.1%, ENE = 2.7%  (7:1 asymmetry)
```

Full data table:

```
Direction:  N     NNE   NE    ENE   E     ESE   SE    SSE   S     SSW   SW    WSW   W     WNW   NW    NNW
Bin 0–5:   1.2   0.9   0.8   0.5   0.6   0.8   1.0   1.5   2.0   2.5   3.0   2.2   1.8   1.5   1.2   1.0
Bin 5–10:  1.8   1.2   1.5   0.8   1.0   1.2   2.0   2.8   3.5   4.5   5.5   4.0   3.5   2.8   2.0   1.5
Bin 10–15: 1.0   0.6   1.0   0.6   0.7   0.8   1.5   2.5   2.8   4.0   5.0   3.8   3.0   2.2   1.5   1.0
Bin 15+:   0.5   0.3   0.8   0.8   0.5   0.6   1.0   1.5   2.0   5.8   5.6   4.0   3.5   2.0   1.2   0.7
```

Stack with `#c6dbef, #6baed6, #2171b5, #08306b`.

<a id="example-31"></a>
#### 31  Radar / spider chart · `0.5.0`

**Mark:** `RadarMark` (polygon + gridlines)
**Coord:** `PolarCoord` (categorical angular axis)
**Scale:** `CategoricalScale` (0.5.0)
**Why 0.5.0:** Categorical angular axis.

```
                Perf  Ecosystem  LearnCurve  TypeSafety  Concurrency  Expressiveness  Community  Tooling
Rust              95        55          25          95           90              80         50       85
Python            30        95          90          25           35              85         95       70
Go                80        60          75          70           92              45         55       80
```

Fill opacity 0.15. Colors: Rust `#DE5028`, Python `#3776AB`, Go `#00ADD8`.

<a id="example-42"></a>
#### 42  Parallel coordinates · `0.5.0`

**Mark:** `ParallelCoordMark` (polylines across vertical axes)
**Scale:** needs multiple independent `LinearScale` instances (0.1.0) + potentially `CategoricalScale` (0.5.0) for axis labels
**Why 0.5.0:** Multiple synchronized scales, proper tick formatting.

```
Cluster A (descending): means = [8.0, 6.5, 3.0, 1.5, 7.0], stds = [0.5, 0.8, 0.6, 0.4, 0.7]
Cluster B (ascending):  means = [2.0, 4.0, 7.0, 8.5, 3.0], stds = [0.6, 0.7, 0.5, 0.3, 0.8]
Cluster C (middle):     means = [5.0, 5.0, 5.0, 5.0, 5.0], stds = [1.2, 1.0, 0.9, 1.1, 0.6]
n = 100 each
```

X-crossing between axes 2 and 3. Alpha = 0.15, Tableau-10 first 3 colors.

<a id="example-40"></a>
#### 40  Funnel chart · `0.5.0`

**Mark:** `BarMark` (centered, decreasing width) / `TrapezoidMark`
**Scale:** `CategoricalScale` (0.5.0) for stage labels
**Why 0.5.0:** Categorical y-axis.

```
Website Visits:     100,000
Product Page Views:  35,000  (35.0%)
Add to Cart:         12,000  (12.0%)
Checkout Started:     6,500   (6.5%)
Payment Completed:    4,200   (4.2%)
Order Fulfilled:      4,000   (4.0%)
```

25:1 ratio top-to-bottom. `minSize = 5%`. Cool→warm gradient.

### 0.7.0 — 3D visualization

Adds `Scatter3D`, `Surface3D`, `Wireframe3D`, `Line3D`, `Isosurface`, `VolumeRender`.
Camera orbit/pan with nalgebra.

<a id="example-12"></a>
#### 12  3D vector field + wireframe surface · `0.7.0`

**Marks:** `Wireframe3DMark`, `Arrow3DMark` (quiver)
**Coord:** `Cartesian3D`
**Why 0.7.0:** 3D marks + camera transforms.

Surface:

```
z(x, y) = −cos(πx/90) · cos(πy/90) · 2.0
x ∈ [−90, 90], y ∈ [−90, 90], 40×40 grid
```

Gradient floor at z = −4:

```
∂z/∂x = (π/90) sin(πx/90) cos(πy/90) · 2
∂z/∂y = (π/90) cos(πx/90) sin(πy/90) · 2
```

Quiver box variant: `F(x,y,z) = (−y, x, sin(2πz))` on [−1.5, 1.5]³, 12×12×8.

<a id="example-23"></a>
#### 23  Surface3D — five parametric surfaces · `0.7.0`

**Mark:** `Surface3DMark`
**Why 0.7.0:** Surface3DMark.

##### A — Möbius strip

```
x(u, v) = (1 + v/2 · cos(u/2)) · cos(u)
y(u, v) = (1 + v/2 · cos(u/2)) · sin(u)
z(u, v) = v/2 · sin(u/2)
u ∈ [0, 2π], v ∈ [−0.5, 0.5], grid 100×20
Color by u → `twilight` (cyclic)
```

##### B — Enneper minimal surface

```
x = u − u³/3 + u·v²,  y = v − v³/3 + v·u²,  z = u² − v²
u, v ∈ [−2, 2], grid 100×100
Color by z → `coolwarm`
```

##### C — Monkey saddle

```
z = x³ − 3·x·y²
x, y ∈ [−2, 2], grid 100×100
Color → `RdBu_r` centered at z=0
```

##### D — Klein bottle (figure-8 immersion)

```
x = (a + cos(u/2)·sin(v) − sin(u/2)·sin(2v)) · cos(u)
y = same · sin(u)
z = sin(u/2)·sin(v) + cos(u/2)·sin(2v)
a = 2, u, v ∈ [0, 2π], grid 150×80
```

##### E — Gaussian mixture landscape

5-peak formula from example 22D rendered as z = f(x, y). Grid 200². Contour lines on z = 0 floor.

<a id="example-24"></a>
#### 24  Isosurface — hydrogen orbital · `0.7.0`

**Mark:** `IsosurfaceMark` (marching cubes)
**Why 0.7.0:** Isosurface lands at 0.7.0.

```
ψ₂₁₀(x, y, z) = (1/(4√(2π))) · z · e^(−√(x²+y²+z²)/2)

Grid: x, y, z ∈ [−15, 15] Bohr radii, 100³
Isosurfaces at |ψ|² = ±0.005 in red/blue
```

<a id="example-25"></a>
#### 25  Isosurface — gyroid minimal surface · `0.7.0`

**Mark:** `IsosurfaceMark`
**Why 0.7.0:** Isosurface.

```
f(x, y, z) = sin(x)·cos(y) + sin(y)·cos(z) + sin(z)·cos(x)

Domain: [−2π, 2π] or [−4π, 4π], grid 100³ per period
Iso-value: 0
```

Variants: Schwarz P `cos(x)+cos(y)+cos(z)=0`, Neovius `3(cos x+cos y+cos z)+4·cos x·cos y·cos z=0`.

<a id="example-26"></a>
#### 26  Isosurface — Barth sextic · `0.7.0`

**Mark:** `IsosurfaceMark`
**Why 0.7.0:** Isosurface.

```
φ = (1 + √5)/2 ≈ 1.618

f = 4·(φ²x² − y²)·(φ²y² − z²)·(φ²z² − x²) − (1 + 2φ)·(x² + y² + z² − 1)²

Domain: [−1.5, 1.5]³, grid 100³, iso-value: 0
```

65 double points, icosahedral symmetry. Color by distance from origin → `plasma`.

### 0.9.0 — All remaining mark types (40+ from taxonomy)

Adds everything not yet covered: `SankeyMark`, `TreemapMark`, `DendrogramMark`,
`NetworkMark`, `StreamgraphMark`, `RidgelineMark`, `GeoMark`, `TernaryCoord`, etc.
This is the "long tail" milestone.

<a id="example-20"></a>
#### 20  Ridgeline / JoyPlot — monthly temperatures · `0.9.0`

**Mark:** `AreaMark` (overlapping, ridgeline layout)
**Layout:** custom y-offset per row
**Why 0.9.0:** Ridgeline is a specialized layout/position adjustment not in earlier milestones.

```
For each month i ∈ 0..12:
    primary_mean  = 55.0 + 23.0 · sin(2π(i − 3)/12)
    primary_std   = 6.5 − 1.5 · sin(2π(i − 3)/12)
    secondary_mean = primary_mean + 12.0
    secondary_std  = primary_std · 0.6

    sample = 0.7 · N(primary_mean, primary_std)
           + 0.3 · N(secondary_mean, secondary_std)
    n = 500 per month
```

Overlap 0.85, fill by x-value → `viridis` option C.

<a id="example-21"></a>
#### 21  Ridgeline — Joy Division pulsar · `0.9.0`

**Mark:** `AreaMark` (white fill, black background, occluding)
**Why 0.9.0:** Same ridgeline layout.

```
80 rows × 300 columns
Per row:
    baseline: N(0, 0.02) per column
    pulse: amplitude ~ U(0.3, 1.0), center ~ 0.5 + N(0, 0.02), width ~ U(0.03, 0.08)
    sub_pulse (50%): offset ±0.15, amp = primary · U(0.1, 0.4), width · 0.7
```

Black background, white fill, `hspace = −0.95`. No axes.

<a id="example-27"></a>
#### 27  Sankey diagram — national energy flow · `0.9.0`

**Mark:** `SankeyMark`
**Why 0.9.0:** SankeyMark is a specialized mark type in the long-tail milestone.

18 nodes, 35 links across 4 layers (Source → Conversion → End Use → Output):

```
Coal→ElecGen: 92          Coal→Industrial: 40
Gas→ElecGen: 190           Gas→DirectHeat: 85
Gas→Residential: 60        Gas→Commercial: 45
Gas→Industrial: 75         Oil→Refining: 310
Oil→ElecGen: 15            Nuclear→ElecGen: 78
Wind→ElecGen: 120          Solar→ElecGen: 65
Hydro→ElecGen: 28          Biomass→ElecGen: 42
Biomass→DirectHeat: 20

ElecGen→Rejected: 210      ElecGen→Residential: 95
ElecGen→Commercial: 85     ElecGen→Industrial: 110
ElecGen→Transport: 8       ElecGen→Exports: 12

Refining→Transport: 250    Refining→Industrial: 30
Refining→Rejected: 30

DirectHeat→Residential: 55 DirectHeat→Commercial: 30
DirectHeat→Industrial: 20

Residential→Services: 130  Residential→Rejected: 80
Commercial→Services: 105   Commercial→Rejected: 55
Industrial→Services: 175   Industrial→Rejected: 100
Transport→Services: 65     Transport→Rejected: 193
```

Color per source hue, links at opacity 0.4.

<a id="example-28"></a>
#### 28  Treemap — tech market cap · `0.9.0`

**Mark:** `TreemapMark` (squarified tiling)
**Why 0.9.0:** TreemapMark.

28 leaves, 6 sectors. Apple $3,000B vs Pinterest $18B (170:1).

```
Consumer Electronics: Apple 3000, Samsung 450, Sony 113, LG 50
Software & Cloud:     Microsoft 2800, Salesforce 250, Adobe 220, ServiceNow 180, Oracle 350, SAP 210, Intuit 100
Semiconductors:       NVIDIA 2100, TSMC 750, Broadcom 600, AMD 200, Qualcomm 185, Intel 120, ASML 280
Internet & Social:    Alphabet 1900, Meta 1200, Tencent 380, Snap 22, Pinterest 18, Spotify 100
E-Commerce:           Amazon 1900, Shopify 120, MercadoLibre 96
Enterprise:           IBM 200, Accenture 210, Palantir 120, Snowflake 60, Datadog 40, MongoDB 30, Confluent 10, HashiCorp 8
```

Squarified tiling, inner padding 1–2 px, outer 3–5 px.

<a id="example-29"></a>
#### 29  Dendrogram with clustermap · `0.9.0`

**Mark:** `DendrogramMark` + `HeatmapMark`
**Stat:** Ward linkage
**Why 0.9.0:** DendrogramMark.

30 points in 5D, 4 clusters:

```
Cluster A: center = [2, 8, 1, 5, 3],  n=10, σ=0.5
Cluster B: center = [8, 2, 7, 1, 9],  n=8,  σ=0.6
Cluster C: center = [5, 5, 4, 8, 6],  n=7,  σ=1.0
Cluster D: center = [1, 1, 9, 9, 1],  n=5,  σ=1.5
```

Ward linkage, color threshold 7.0. Heatmap: `vlag`, z-score per column.

<a id="example-30"></a>
#### 30  Force-directed network graph · `0.9.0`

**Mark:** `NetworkMark`
**Why 0.9.0:** NetworkMark + force layout.

Stochastic Block Model: 5 communities [20, 18, 16, 14, 12], 80 nodes:

```
       C1     C2     C3     C4     C5
C1   0.25   0.020  0.015  0.010  0.025
C2   0.020  0.28   0.030  0.015  0.010
C3   0.015  0.030  0.22   0.025  0.015
C4   0.010  0.015  0.025  0.30   0.020
C5   0.025  0.010  0.015  0.020  0.26
```

~200–250 edges. Node radius: `4 + √(degree)·3`. Edge opacity 0.15. k = 0.5.

<a id="example-32"></a>
#### 32  Streamgraph — genre popularity · `0.9.0`

**Mark:** `AreaMark` (stacked, wiggle baseline)
**Why 0.9.0:** Wiggle baseline + inside-out ordering are specialized features.

8 layers × 100 steps. Bostock bump function:

```
y_i(t) = Σⱼ Aᵢⱼ · exp(−((t/n − cᵢⱼ) · zᵢⱼ)²)
```

Layer params:

```
"Rock":       [(3.2,0.15,8.5), (1.8,0.45,12), (2.5,0.70,6), (1.2,0.90,15), (0.8,0.30,5)]
"Electronic": [(2.0,0.50,10), (3.5,0.80,7), (1.5,0.20,9), (0.9,0.60,14), (2.8,0.35,6.5)]
"Hip Hop":    [(0.5,0.10,12), (1.0,0.30,8), (2.5,0.55,5.5), (4.0,0.75,9), (3.0,0.90,7.5)]
"Pop":        [(3.8,0.20,6), (2.2,0.40,11), (3.0,0.65,8), (1.5,0.85,13), (2.0,0.50,7)]
"Jazz":       [(4.0,0.05,7), (2.5,0.25,10), (1.0,0.60,15), (0.5,0.80,9), (0.3,0.95,12)]
"Classical":  [(2.8,0.10,9), (1.5,0.35,6), (0.8,0.55,14), (0.4,0.75,8), (0.2,0.90,11)]
"Country":    [(1.0,0.15,11), (2.0,0.40,7.5), (3.0,0.60,5), (1.5,0.80,10), (2.5,0.95,8)]
"R&B":        [(1.5,0.20,8), (2.5,0.45,6), (3.5,0.70,9), (2.0,0.85,12), (1.0,0.50,7)]
```

Wiggle: g₀(x) = −(1/n)·Σᵢ(n−i+0.5)·fᵢ(x). First 8 Tableau-10 colors.

<a id="example-35"></a>
#### 35  Ternary plot — Shannon entropy + soil · `0.9.0`

**Mark:** `PointMark` + `ContourMark` on `TernaryCoord`
**Why 0.9.0:** TernaryCoord is a long-tail coordinate system.

##### Panel A — Shannon entropy

```
H(p₁, p₂, p₃) = −Σ pᵢ · log(pᵢ)
Simplex grid, scale = 60
Max H = log(3) ≈ 1.099 at (⅓, ⅓, ⅓)
```

##### Panel B — Soil scatter

```
Sandy Loam: [65, 20, 15]%, n=35, Dir(50)
Clay:       [20, 25, 55]%, n=30, Dir(40)
Silt Loam:  [20, 60, 20]%, n=40, Dir(45)
Loam:       [40, 40, 20]%, n=30, Dir(35)
```

<a id="example-36"></a>
#### 36  Choropleth — temperature anomaly · `0.9.0`

**Mark:** `GeoMark` (polygon fill)
**Coord:** `GeoProjection` (Equal Earth)
**Data:** Natural Earth 110m (177 polygons, ~240 KB TopoJSON)
**Why 0.9.0:** GeoMark + projection.

```
anomaly = (latitude / 90) · 2.5 + N(0, 0.3)
```

Arctic amplification pattern: +2 to +4 °C at high latitudes, 0 to +1 °C equatorial.
Colormap: `viridis` 7-step or `RdBu` diverging centered at +1.0 °C.

### 0.12.0 — Documentation + gallery

The gallery generation milestone. These examples are _rendered_ at 0.12.0 as part of
`xtask gallery`, but each example's code is written alongside the release where its
marks first become available.

### Release → Example mapping (compact)

| Release | Examples | Count |
|---------|----------|-------|
| **0.1.0** | 1 (2D Lorenz), 6 (Kruskal–Szekeres) | 2 |
| **0.2.0** | 3 (Bubble), 7 (Plasma), 16 (Heatmap table), 37 (Waterfall) | 4 |
| **0.3.0** | 19 (Violin), 22 (Contour), 34 (Nightingale), 38 (Candlestick), 39 (Donut/sunburst), 41 (Gauge) | 6 |
| **0.4.0** | 2 (Dashboard), 4 (Neuro), 5 (Microbiome), 9 (Hexbin joint), 10 (PCA/ICA), 11 (Regression), 13 (Density facets), 14 (SHAP), 15 (Penguins), 17 (Scattering), 18 (Gene expr) | 11 |
| **0.5.0** | 8 (Spiral), 31 (Radar), 33 (Wind rose), 40 (Funnel), 42 (Parallel coords) | 5 |
| **0.7.0** | 1 (3D Lorenz), 12 (Vector field), 23 (Surface3D ×5), 24 (Orbital), 25 (Gyroid), 26 (Barth) | 6 |
| **0.9.0** | 20 (Ridgeline), 21 (Pulsar), 27 (Sankey), 28 (Treemap), 29 (Dendrogram), 30 (Network), 32 (Streamgraph), 35 (Ternary), 36 (Choropleth) | 9 |
| **0.12.0** | Gallery renders all 42 | — |
| **Total** | | **42** (+1 3D upgrade) |

### Full taxonomy coverage

**Unique marks (28):** Point, Line, Bar, Area, Hexbin, Heatmap, BoxPlot, Rect,
Arc, Violin, Contour, Surface3D, Line3D, Wireframe3D, Arrow/Arrow3D, Text,
Rule, ErrorBar, Ellipse, Sankey, Treemap, Dendrogram, Network, Radar,
Candlestick, Funnel/Trapezoid, Geo, ParallelCoord.

**Unique stats (12):** Bin, KDE-1D, KDE-2D, Boxplot, OLS regression, LOESS,
Hexbin aggregation, Contour (marching squares), Ward linkage, Force layout,
Wiggle baseline, GARCH.

**Unique layouts (9):** GridLayout, FacetWrap, FacetGrid, JointPlot, PairPlot,
Custom y-offset (ridgeline), multi-ring concentric, spring embedding, side-by-side.

**Unique coords (6):** Cartesian2D, Cartesian3D, Polar (continuous θ),
Polar (categorical θ), Ternary, GeoProjection (Equal Earth).
