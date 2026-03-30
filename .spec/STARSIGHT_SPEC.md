# starsight

> A unified scientific visualization crate for Rust — from zero-config one-liners to GPU-accelerated interactive 3D.

---

## For everyone

starsight is a comprehensive data visualization library that closes Rust's most painful ecosystem gap: the inability to go from data to picture without leaving the language. It replaces a fragmented landscape of 20+ incomplete crates with a single, layered library spanning terminal sparklines, publication-quality PDF vector plots, interactive browser charts, and GPU-rendered 3D scientific visualization — all from one `use starsight::prelude::*;` import.

starsight belongs to the [resonant-jovian](https://github.com/resonant-jovian) ecosystem alongside `prismatica` (perceptually uniform colormaps), `chromata` (editor/terminal color themes), `ratatui-plt` (terminal-native plotting widgets), `caustic` (Vlasov–Poisson solver), and `phasma` (caustic TUI).

---

## For users

### The problem starsight solves

Rust has world-class data processing (Polars), world-class numerics (ndarray, nalgebra, faer), and world-class terminal UI (ratatui) — but no way to type the equivalent of `plt.scatter(x, y); plt.show()` and see a chart. The current landscape forces developers to choose between:

- **plotters** — powerful but verbose (~15 lines for a scatter plot), stale maintenance, no statistical charts, no interactivity, broken 3D axis labels
- **plotly-rs** / **charming** — rich features but delegate all rendering to bundled JavaScript engines (~3.5 MB), not pure Rust
- **egui_plot** — GPU-interactive but locked to the egui framework, limited chart types, no static export
- **textplots** / **ratatui Chart** — terminal-only, minimal chart types

The result: researchers export CSV and plot in Python. Dashboard builders reach for Plotly. Scientists needing 3D use ParaView. Nobody stays in Rust for the full pipeline.

starsight eliminates this with a single crate that provides:

- A **zero-config high-level API** where simple things are simple
- A **grammar-of-graphics declarative API** where complex things are composable
- **60+ chart types** spanning statistical, scientific, financial, geospatial, hierarchical, and network domains
- **Multiple rendering backends** — wgpu (GPU native), tiny-skia (CPU headless), SVG, PDF, terminal (Kitty/Sixel/Braille)
- **Native Polars DataFrame integration** — reference columns by name
- **Full interactivity** — hover, zoom, pan, selection, linked views, streaming data
- **2D and 3D** in the same API with the same backends

### Highlights

- One import, sixty chart types, five rendering backends
- `plot!()` macro for zero-config visualization in under one line
- Grammar-of-graphics `Figure` builder for full compositional control
- Native Polars `DataFrame` and `Series` acceptance — columns by string name
- GPU-accelerated via wgpu — millions of points at 60fps
- CPU fallback via tiny-skia — headless rendering for CI/servers with zero GPU requirement
- Terminal output with automatic protocol detection — Kitty → Sixel → iTerm2 → half-block → Braille
- Static export to PNG, SVG, PDF with publication-quality text (LaTeX math planned)
- Interactive native windows via winit+wgpu and browser via WASM+WebGPU
- Perceptually uniform colormaps via sister crate prismatica (260+ maps)
- Theme system with presets (minimal, dark, publication, seaborn, ggplot) and element-level customization

---

## For developers

### Positioning in the Rust ecosystem

starsight does not compete with plotters — it sits above it architecturally. Where plotters is a drawing library (analogous to matplotlib's backend layer), starsight is a visualization framework (analogous to matplotlib+seaborn+mpl's pyplot combined). It may use plotters as one optional backend, but its primary rendering paths are wgpu and tiny-skia.

starsight's relationship to the resonant-jovian ecosystem:

- **prismatica** provides starsight's colormap system — 260+ perceptually uniform colormaps as compile-time lookup tables, used directly for color scales
- **chromata** provides theme color constants — terminal and editor themes used for starsight's built-in theme presets
- **ratatui-plt** is the terminal-native widget library — starsight's terminal backend renders through ratatui-plt widgets when available, falling back to ratatui-image protocol rendering for rasterized output

### License

GPL-3.0-or-later

---

## Architecture

starsight is organized as a layered architecture with seven distinct layers. Each layer depends only on layers below it. Users enter at the layer matching their needs — most at Layer 5 (high-level API), power users at Layer 3 (grammar of graphics), and backend authors at Layer 1 (rendering abstraction).

```
┌─────────────────────────────────────────────────────────┐
│  Layer 7 — Animation and export                         │
│  Frame recording, transitions, GIF/MP4/PNG/SVG/PDF      │
├─────────────────────────────────────────────────────────┤
│  Layer 6 — Interactivity and real-time                  │
│  Hover, zoom, pan, selection, linked views, streaming   │
├─────────────────────────────────────────────────────────┤
│  Layer 5 — High-level API                               │
│  plot!() macro, DataFrame acceptance, auto-inference    │
├─────────────────────────────────────────────────────────┤
│  Layer 4 — Layout and composition                       │
│  GridLayout, faceting, legends, colorbars, insets       │
├─────────────────────────────────────────────────────────┤
│  Layer 3 — Mark/geom system (grammar of graphics)       │
│  Composable marks, aesthetic mappings, stat transforms  │
├─────────────────────────────────────────────────────────┤
│  Layer 2 — Scale, axis, and coordinate system           │
│  Linear/log/symlog/time/categorical, tick generation    │
├─────────────────────────────────────────────────────────┤
│  Layer 1 — Rendering abstraction                        │
│  Backend trait over wgpu, tiny-skia, SVG, PDF, terminal │
└─────────────────────────────────────────────────────────┘
```

### Layer 1 — Rendering abstraction

A `DrawBackend` trait abstracting over five rendering targets. Every chart is described as a backend-agnostic scene graph of primitives (paths, text, images, groups with transforms). Backend selection happens at render time, not chart construction time.

| Backend       | Crate dependency       | Use case                              |
|---------------|------------------------|---------------------------------------|
| wgpu          | `wgpu`                 | GPU native windows, WebGPU WASM       |
| tiny-skia     | `tiny-skia`            | CPU raster, headless CI, PNG export   |
| SVG           | `svg` crate            | Scalable vector output                |
| PDF           | `krilla` / `pdf-writer`| Publication vector export             |
| Terminal      | `ratatui` + `ratatui-image` | TUI rendering with protocol detection |

### Layer 2 — Scale, axis, and coordinate system

Type-safe `Scale<Domain, Range>` implementations:

- **Continuous**: Linear, Log, Symlog, Logit, Sqrt, Power, Reverse
- **Temporal**: DateTime (with auto tick: year/month/day/hour/minute/second)
- **Discrete**: Categorical, Band, Point, Binned
- **Color**: Sequential, Diverging, Qualitative (backed by prismatica colormaps)

Axis generation from scales with automatic tick computation via pluggable `TickLocator` / `TickFormatter` traits. Coordinate systems: Cartesian, Polar, Geographic (via proj), Ternary, Parallel.

### Layer 3 — Mark/geom system

Composable marks inspired by ggplot2 geoms and Observable Plot marks. Each mark maps data to visual properties through aesthetic encodings.

**Core marks**: Point, Line, Area, Bar, Rect, Arc, Text, Rule, Tick, Image, Arrow, Polygon, Contour, Surface, Volume.

**Statistical transforms** (Stat layer): Bin, KDE, Aggregate, Regression (linear/poly/LOESS), ECDF, Boxplot summary, Density2D, Hexbin.

**Position adjustments**: Identity, Dodge, Stack, Fill, Jitter, Nudge.

Extensible via a `#[starsight::recipe]` attribute macro for domain-specific chart types.

### Layer 4 — Layout and composition

GridLayout with variable-size cells, faceting (wrap and grid by categorical variables with free/fixed scales), layer composition (overlaying marks on shared axes), concatenation (horizontal/vertical), inset axes, colorbars, and legends with automatic or manual placement.

### Layer 5 — High-level API

The `plot!()` macro and `Figure` builder. Accepts Polars DataFrames, ndarray arrays, Arrow RecordBatches, and raw `Vec`/slice data. Auto-infers chart type, auto-scales axes, auto-generates legends, picks default colormaps.

### Layer 6 — Interactivity and real-time

For native (winit+wgpu) and web (WASM+WebGPU) targets: hover tooltips, box/wheel zoom, pan, box/lasso selection with callbacks, linked views, legend click-to-toggle, range sliders, streaming data append with rolling windows.

### Layer 7 — Animation and export

Frame-based animation recording to GIF and MP4. Transition animations between chart states. Static export: PNG, JPEG, WebP (via tiny-skia or wgpu readback), SVG, PDF. Interactive HTML export (optional, for web sharing). Terminal output with automatic protocol detection.

---

## Chart type taxonomy

### 2D charts (49 types)

**Relational**: Line, Scatter, Bubble, Area, StackedArea, Bar, GroupedBar, StackedBar, Stem, Step, Lollipop, Slope, Bump, Dot

**Statistical**: Histogram, Histogram2D, BoxPlot, Violin, Strip, Swarm, Boxen, KDEPlot, ECDF, Rug, RidgePlot, RainCloud, QQPlot, ErrorBar, PointEstimate, RegressionPlot

**Matrix/grid**: Heatmap, AnnotatedHeatmap, ClusterMap, ImageDisplay

**2D fields**: Contour, FilledContour, Hexbin, Streamline, Quiver, PseudocolorMesh

**Financial**: Candlestick, OHLC, Waterfall, Funnel

**Part-of-whole**: Pie, Donut, Sunburst, Treemap, Waffle

**Network/flow**: ForceGraph, Sankey, Chord, ArcDiagram

**Specialized**: Polar, Radar, ParallelCoordinates, Gantt, Gauge, CalendarHeatmap, Sparkline

### 3D charts (12 types)

Scatter3D, Line3D, Surface3D, Wireframe3D, Bar3D, Mesh3D, Cone, Streamtube, Isosurface, VolumeRender, Voxel, TriSurf

### Geographic charts (5 types)

Choropleth, ScatterMap, BubbleMap, LineMap, DensityMap

### Layout and interaction infrastructure (12 items)

GridLayout, FacetWrap, FacetGrid, PairPlot, JointPlot, MosaicLayout, InsetAxes, TwinAxes, Colorbar, Legend, RangeSlider, DataZoom

---

## Dependency stack

### Required (core)

| Crate           | Role                                    | Feature gate |
|-----------------|-----------------------------------------|--------------|
| `tiny-skia`     | CPU rasterization, headless rendering   | default      |
| `palette`       | Color space operations (sRGB, Oklab)    | default      |
| `prismatica`    | Perceptually uniform colormaps          | default      |
| `image`         | PNG/JPEG/WebP I/O                       | default      |
| `svg`           | SVG document generation                 | default      |
| `cosmic-text`   | Text shaping and layout                 | default      |
| `ab_glyph`      | Font rasterization                      | default      |

### Optional (feature-gated)

| Crate             | Role                                    | Feature gate     |
|-------------------|-----------------------------------------|------------------|
| `wgpu`            | GPU rendering (native + WebGPU)         | `gpu`            |
| `vello`           | GPU 2D compute rendering                | `gpu`            |
| `winit`           | Native window creation                  | `interactive`    |
| `egui`            | GUI controls for interactive charts     | `interactive`    |
| `wasm-bindgen`    | WebAssembly browser target              | `wasm`           |
| `web-sys`         | Browser DOM/Canvas access               | `wasm`           |
| `ratatui`         | Terminal UI framework                   | `terminal`       |
| `crossterm`       | Terminal I/O backend                    | `terminal`       |
| `ratatui-image`   | Kitty/Sixel/iTerm2 protocol rendering   | `terminal`       |
| `polars`          | DataFrame integration                   | `polars`         |
| `ndarray`         | N-dimensional array acceptance          | `ndarray`        |
| `arrow`           | Apache Arrow RecordBatch acceptance     | `arrow`          |
| `nalgebra`        | Linear algebra for 3D transforms        | `3d`             |
| `krilla`          | PDF vector export                       | `pdf`            |
| `pdf-writer`      | Low-level PDF generation                | `pdf`            |
| `statrs`          | Statistical distributions (KDE, etc.)   | `stats`          |
| `contour`         | Isoline/isoband generation              | `contour`        |
| `delaunator`      | Delaunay triangulation                  | `geo`            |
| `geo`             | Geospatial primitives                   | `geo`            |
| `proj`            | Coordinate projections                  | `geo`            |
| `geojson`         | GeoJSON I/O                             | `geo`            |
| `lyon`            | Path tessellation for GPU pipeline      | `gpu`            |
| `colorgrad`       | Gradient/colormap construction          | default          |
| `colorous`        | D3-scale-chromatic colormap ports       | default          |
| `resvg`           | SVG-to-PNG rasterization                | `resvg`          |

### Feature presets

| Preset          | Includes                                                      |
|-----------------|---------------------------------------------------------------|
| `default`       | tiny-skia CPU rendering, SVG, PNG export, basic chart types   |
| `full`          | All features enabled                                          |
| `minimal`       | Core types only, no rendering (for downstream crates)         |
| `science`       | `stats` + `contour` + `3d` + `pdf`                           |
| `dashboard`     | `interactive` + `gpu` + `polars`                              |
| `terminal`      | `terminal` feature only — TUI rendering                       |
| `web`           | `wasm` + `gpu` — browser deployment                           |

---

## Cross-ecosystem comparison

### What starsight matches or exceeds

| Capability                       | matplotlib | seaborn | Plotly | ggplot2 | Makie.jl | starsight target |
|----------------------------------|:----------:|:-------:|:------:|:-------:|:--------:|:----------------:|
| Basic 2D charts (line/bar/scatter)| Yes       | Yes     | Yes    | Yes     | Yes      | Yes              |
| Statistical charts (violin/KDE)  | Yes        | Yes     | Yes    | Yes     | Yes      | Yes              |
| 3D surface/scatter/volume        | Limited    | No      | Yes    | No      | Yes      | Yes              |
| GPU-accelerated rendering        | No         | No      | No     | No      | Yes      | Yes              |
| Interactive hover/zoom/pan       | Limited    | No      | Yes    | No      | Yes      | Yes              |
| Grammar of graphics API          | No         | Partial | No     | Yes     | No       | Yes              |
| DataFrame-native column refs     | Yes        | Yes     | Yes    | Yes     | Yes      | Yes              |
| Terminal rendering                | No         | No      | No     | No      | No       | Yes              |
| Publication PDF/SVG export       | Yes        | Yes     | Partial| Yes     | Yes      | Yes              |
| WASM/browser deployment          | No         | No      | Yes    | No      | Yes      | Yes              |
| Perceptually uniform colormaps   | Yes        | Yes     | Yes    | Yes     | Yes      | Yes (prismatica) |
| Zero-config one-liner API        | Yes        | Yes     | Yes    | No      | Partial  | Yes              |
| Streaming real-time data         | Partial    | No      | Yes    | No      | Yes      | Yes              |
| Geospatial (choropleth/maps)     | Partial    | No      | Yes    | Partial | Yes      | Yes              |
| Network/graph visualization      | No         | No      | Yes    | No      | Yes      | Yes              |

### starsight's unique advantages as a Rust crate

- **Zero GUI dependencies in default configuration** — tiny-skia is pure Rust, no system libraries
- **Compile-time colormap embedding** via prismatica — no runtime file loading, no dynamic allocation
- **Terminal-native rendering** — SSH-accessible visualization, CI pipeline charts, no display server needed
- **Single binary deployment** — wgpu backend compiles to one static binary with GPU support
- **Sub-millisecond render latency** for simple charts on the CPU path
- **Memory safety** — no segfaults from mismatched buffer sizes in GPU rendering
- **WASM-first web target** — compile the same chart code to WebAssembly with WebGPU rendering

---

## Community demand

The Rust Users Forum thread "Seeking Rust Alternative to Python's matplotlib for Plotting" (Aug 2025, 1,761 views) captures the pain. The top reply states plainly that feature parity with matplotlib does not exist. The Charton announcement thread (2025) notes existing options either feel like drawing on a canvas rather than analyzing data (plotters) or don't feel Rust-native (charming, plotly-rs). Plotters' own "Status of the project?" issue (Jul 2025, 9 reactions) and "Call for participations" signal the ecosystem's bandwidth problem. The pattern dates back to the earliest "What are our choices for a charting library?" thread (Nov 2017), making this an eight-year-old gap.

---

## MSRV

The minimum supported Rust version will be documented per release and will track the latest stable minus two releases, consistent with wgpu and ratatui MSRV policies.
