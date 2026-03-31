# starsight

> A unified scientific visualization crate for Rust — from zero-config one-liners to GPU-accelerated interactive 3D.
> Single definitive reference: spec, architecture, API, implementation guide, links, and roadmap.

---

## For everyone

starsight is a comprehensive data visualization library that closes Rust's most painful ecosystem gap: the inability to go from data to picture without leaving the language. It replaces a fragmented landscape of 20+ incomplete crates with a single, layered library spanning terminal sparklines, publication-quality PDF vector plots, interactive browser charts, and GPU-rendered 3D scientific visualization — all from one `use starsight::prelude::*;` import.

starsight belongs to the [resonant-jovian](https://github.com/resonant-jovian) ecosystem alongside [prismatica](https://github.com/resonant-jovian/prismatica) (perceptually uniform colormaps), [chromata](https://github.com/resonant-jovian/chromata) (editor/terminal color themes), [ratatui-plt](https://github.com/resonant-jovian/ratatui-plt) (terminal-native plotting widgets), [caustic](https://github.com/resonant-jovian/caustic) (Vlasov-Poisson solver), and [phasma](https://github.com/resonant-jovian/phasma) (caustic TUI).

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
- Perceptually uniform colormaps via sister crate prismatica (308 maps, 70 palettes)
- Theme system with presets (minimal, dark, publication, seaborn, ggplot) plus 1000+ editor themes via chromata

---

## For developers

### Positioning in the Rust ecosystem

starsight does not compete with plotters — it sits above it architecturally. Where plotters is a drawing library (analogous to matplotlib's backend layer), starsight is a visualization framework (analogous to matplotlib+seaborn+pyplot combined). Its primary rendering paths are wgpu and tiny-skia.

### Sister crate integration

- **prismatica** (https://docs.rs/prismatica) provides starsight's colormap system — 308 perceptually uniform colormaps + 70 discrete palettes as compile-time lookup tables from 10 scientific collections (Crameri, matplotlib, CET, CMOcean, ColorBrewer, CMasher, NCAR, CartoColors, Moreland, d3). Core types: `Colormap` (continuous LUT with `.eval(t: f32) -> Color`), `DiscretePalette` (categorical with `.get(i) -> Color`), `ReversedColormap` (zero-alloc reversed view). Enable `tiny-skia-integration` for `From<prismatica::Color> for tiny_skia::Color`.
- **chromata** (https://docs.rs/chromata) provides starsight's theme system — 1,104 editor/terminal color themes as compile-time `const` data from 5 collections (popular, base16, base24, vim, emacs). Core types: `Theme` (29 color fields + metadata), `Color` (RGB u8 with hex/CSS/luminance/contrast/lerp), `Variant` (Dark/Light), `Contrast` (High/Normal/Low). Enable `tiny-skia-integration` for `From<chromata::Color> for tiny_skia::PremultipliedColorU8`.
- **ratatui-plt** is the terminal-native widget library — starsight's terminal backend renders through ratatui-plt widgets when available

### License

GPL-3.0-only

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

| Backend       | Crate dependency              | Use case                              |
|---------------|-------------------------------|---------------------------------------|
| wgpu          | `wgpu`                        | GPU native windows, WebGPU WASM       |
| tiny-skia     | `tiny-skia`                   | CPU raster, headless CI, PNG export   |
| SVG           | `svg` crate                   | Scalable vector output                |
| PDF           | `krilla` / `pdf-writer`       | Publication vector export             |
| Terminal      | `ratatui` + `ratatui-image`   | TUI rendering with protocol detection |

### Layer 2 — Scale, axis, and coordinate system

Type-safe `Scale<Domain, Range>` implementations:

- **Continuous**: Linear, Log, Symlog, Logit, Sqrt, Power, Reverse
- **Temporal**: DateTime (with auto tick: year/month/day/hour/minute/second)
- **Discrete**: Categorical, Band, Point, Binned
- **Color**: Sequential, Diverging, Qualitative (backed by prismatica colormaps)

Axis generation from scales with automatic tick computation via pluggable `TickLocator` / `TickFormatter` traits. Coordinate systems: Cartesian, Polar, Geographic (via proj), Ternary, Parallel.

Tick generation uses the **Wilkinson Extended algorithm** (Talbot, Lin, Hanrahan 2010) — optimizes simplicity, coverage, density, and legibility to produce publication-quality axis labels. See the implementation reference section below for the complete algorithm.

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

### Required (core, default feature set)

| Crate           | Role                                       | Feature gate |
|-----------------|--------------------------------------------|--------------|
| `tiny-skia`     | CPU rasterization, headless rendering      | default      |
| `palette`       | Color space operations (sRGB, Oklab)       | default      |
| `chromata`      | Editor/terminal theme colors (1104 themes) | default      |
| `prismatica`    | Scientific colormaps (308 maps)            | default      |
| `image`         | PNG/JPEG/WebP I/O                          | default      |
| `svg`           | SVG document generation                    | default      |
| `cosmic-text`   | Text shaping and layout                    | default      |
| `ab_glyph`      | Font rasterization                         | default      |
| `thiserror`     | Error type derivation                      | default      |

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

## API examples

### Zero-config one-liners (Layer 5)

The `plot!()` macro is the primary entry point for quick visualization. It accepts raw data, Polars DataFrames, ndarray arrays, and Arrow RecordBatches. It auto-detects the appropriate chart type, scales axes, generates legends, and picks a default colormap.

```rust
use starsight::prelude::*;

// Scatter from raw slices — auto-opens a window or renders inline
let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
let y = vec![2.3, 4.1, 3.8, 5.7, 4.9];
plot!(x, y).show();

// Line chart — type inferred from sorted x
plot!(x, y).save("line.png");

// Histogram from a single series
let values: Vec<f64> = sample_normal(1000, 0.0, 1.0);
plot!(values).show();

// Heatmap from a 2D ndarray
let matrix = ndarray::Array2::<f64>::zeros((10, 10));
plot!(matrix).show();
```

### DataFrame-aware plotting (Layer 5 + Polars)

```rust
use starsight::prelude::*;
use polars::prelude::*;

let df = CsvReader::from_path("data.csv")?.finish()?;

// Scatter by column name — legends auto-generated from "species" values
plot!(&df, x = "sepal_length", y = "sepal_width", color = "species").show();

// Histogram of a single column
plot!(&df, x = "petal_length", kind = Histogram).show();

// Box plot grouped by category
plot!(&df, x = "species", y = "sepal_width", kind = BoxPlot).show();

// Faceted scatter — one subplot per species
plot!(&df, x = "sepal_length", y = "petal_length")
    .color("species")
    .facet_wrap("species")
    .show();
```

### Grammar of graphics (Layer 3)

The `Figure` builder exposes the full compositional power. Marks (geoms) are layered onto shared or independent axes.

```rust
use starsight::prelude::*;

let fig = Figure::new()
    .data(&df)
    .add(
        Geom::point()
            .aes(x("weight"), y("mpg"), color("origin"), size("horsepower"))
            .alpha(0.7),
    )
    .add(
        Geom::smooth()
            .aes(x("weight"), y("mpg"))
            .method(Loess { span: 0.75 })
            .color(Color::GRAY)
            .line_width(2.0),
    )
    .scale_x(Scale::linear().label("Weight (lbs)"))
    .scale_y(Scale::linear().label("Miles per Gallon"))
    .scale_color(Scale::categorical(prismatica::colorbrewer::SET2))
    .facet_wrap("cylinders", FacetOpts { ncol: 3, scales: FreeY })
    .theme(Theme::minimal())
    .title("Fuel Efficiency by Weight")
    .size(1200, 800);

fig.save("cars.pdf")?;
fig.save("cars.svg")?;
fig.show();
```

### Statistical charts (Layer 3 + stats feature)

```rust
use starsight::prelude::*;

// Violin plot
Figure::new()
    .data(&df)
    .add(Geom::violin().aes(x("species"), y("sepal_width")).inner(InnerMark::Box))
    .show();

// KDE density plot with rug
Figure::new()
    .data(&df)
    .add(Geom::density().aes(x("petal_length"), fill("species")).alpha(0.5))
    .add(Geom::rug().aes(x("petal_length"), color("species")))
    .show();

// Pair plot (scatterplot matrix)
PairPlot::new(&df)
    .columns(&["sepal_length", "sepal_width", "petal_length", "petal_width"])
    .hue("species")
    .diag(DiagKind::KDE)
    .upper(UpperKind::Scatter)
    .lower(LowerKind::Regression)
    .show();

// Joint plot (scatter + marginal distributions)
JointPlot::new(&df)
    .x("sepal_length")
    .y("sepal_width")
    .kind(JointKind::Hex)
    .marginal(MarginalKind::KDE)
    .show();

// Heatmap with hierarchical clustering
ClusterMap::new(&correlation_matrix)
    .method(Linkage::Ward)
    .cmap(prismatica::crameri::VIK)
    .annotate(true)
    .show();

// Regression plot with confidence band
Figure::new()
    .data(&df)
    .add(
        Geom::regression()
            .aes(x("weight"), y("mpg"))
            .method(Linear)
            .ci(0.95)
            .scatter(true),
    )
    .show();
```

### 3D visualization (Layer 3 + 3d feature)

```rust
use starsight::prelude::*;

// 3D scatter
Figure::new()
    .data(&df)
    .add(Geom::scatter3d().aes(x("x"), y("y"), z("z"), color("cluster")))
    .camera(Camera::orbit(45.0, 30.0, 5.0))
    .show();

// Surface plot from a function
let (xs, ys) = meshgrid(linspace(-3.0, 3.0, 50), linspace(-3.0, 3.0, 50));
let zs = xs.mapv(|x| x.sin()) * ys.mapv(|y| y.cos());

Figure::new()
    .add(
        Geom::surface3d()
            .data_xyz(&xs, &ys, &zs)
            .cmap(prismatica::crameri::BATLOW)
            .wireframe(false),
    )
    .title("sin(x) * cos(y)")
    .show();

// Isosurface from volumetric data
Figure::new()
    .add(
        Geom::isosurface()
            .data_volume(&volume_array)
            .level(0.5)
            .opacity(0.8)
            .cmap(prismatica::crameri::HAWAII),
    )
    .show();
```

### Terminal rendering (Layer 1 + terminal feature)

```rust
use starsight::prelude::*;

// Auto-detect terminal capabilities and render at max fidelity
// Kitty → Sixel → iTerm2 → half-block → Braille
let x = linspace(0.0, TAU, 200);
let y = x.mapv(f64::sin);
plot!(x, y).terminal().show();

// Force a specific protocol
plot!(x, y).terminal_protocol(Protocol::Braille).show();

// Render into a ratatui frame (for TUI applications)
let chart = starsight::terminal::RatatuiWidget::new(figure);
frame.render_widget(chart, area);
```

### Interactive features (Layer 6 + interactive feature)

```rust
use starsight::prelude::*;

// Open an interactive native window with default tools
Figure::new()
    .data(&df)
    .add(Geom::scatter().aes(x("x"), y("y"), color("label")))
    .interactive(Interactive {
        hover: true,
        zoom: Zoom::BoxAndWheel,
        pan: true,
        select: Select::Lasso,
        linked: vec![],
    })
    .show();

// Streaming real-time data
let mut stream_fig = Figure::new()
    .add(Geom::line().aes(x("time"), y("value")))
    .streaming(StreamOpts { window: Duration::from_secs(60) });

stream_fig.show();

loop {
    let (t, v) = read_sensor();
    stream_fig.append(row!["time" => t, "value" => v]);
    sleep(Duration::from_millis(16));
}
```

### Multi-backend export (Layer 7)

```rust
use starsight::prelude::*;

let fig = Figure::new()
    .data(&df)
    .add(Geom::scatter().aes(x("x"), y("y")))
    .theme(Theme::publication());

// Raster export
fig.save("plot.png")?;
fig.save_with("plot.png", SaveOpts { dpi: 300, width: 2400, height: 1600 })?;

// Vector export
fig.save("plot.svg")?;
fig.save("plot.pdf")?;

// Interactive HTML (self-contained)
fig.save("plot.html")?;

// Terminal inline
fig.print_terminal()?;

// Raw bytes for embedding
let png_bytes: Vec<u8> = fig.render_png(300)?;
let svg_string: String = fig.render_svg()?;
```

### Theming with chromata and prismatica

```rust
use starsight::prelude::*;
use chromata::popular::gruvbox;
use chromata::popular::catppuccin;

// Built-in presets
fig.theme(Theme::minimal());
fig.theme(Theme::dark());
fig.theme(Theme::publication());

// Theme from chromata — any of 1104 editor themes
fig.theme(Theme::from_chromata(&gruvbox::DARK_HARD));
fig.theme(Theme::from_chromata(&catppuccin::MOCHA));

// Element-level customization
fig.theme(
    Theme::minimal()
        .background(Color::WHITE)
        .axis_color(Color::GRAY_60)
        .grid(Grid { visible: true, color: Color::GRAY_90, width: 0.5 })
        .font_family("CMU Serif")
        .font_size(12.0)
        .title_size(16.0)
        .legend_position(LegendPosition::TopRight),
);

// Colormaps from prismatica
fig.scale_color(Scale::sequential(prismatica::crameri::BATLOW));
fig.scale_color(Scale::diverging(prismatica::crameri::VIK));
fig.scale_color(Scale::categorical_palette(prismatica::colorbrewer::SET2_PALETTE));
```

### Recipe system for custom chart types

```rust
use starsight::prelude::*;

#[starsight::recipe]
fn volcano_plot(
    data: &DataFrame,
    x: &str,           // log2 fold change column
    y: &str,           // -log10 p-value column
    threshold_x: f64,
    threshold_y: f64,
) -> Figure {
    let fig = Figure::new().data(data);
    fig.add(Geom::point().aes(x(x), y(y))
        .filter(|row| row[x].abs() < threshold_x || row[y] < threshold_y)
        .color(Color::GRAY_70).size(2.0));
    fig.add(Geom::point().aes(x(x), y(y))
        .filter(|row| row[x] >= threshold_x && row[y] >= threshold_y)
        .color(Color::RED).size(3.0));
    fig.add(Geom::hline(threshold_y).dash(Dash::Dashed).color(Color::GRAY_50));
    fig.add(Geom::vline(threshold_x).dash(Dash::Dashed).color(Color::GRAY_50));
    fig.add(Geom::vline(-threshold_x).dash(Dash::Dashed).color(Color::GRAY_50));
    fig
}
```


---

## Workspace and crate structure

```
starsight/
├── Cargo.toml                    # Workspace root (resolver = "3", edition 2024)
├── LICENSE                       # GPL-3.0-only
├── README.md
├── CONTRIBUTING.md
├── CHANGELOG.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── .clippy.toml
├── .rustfmt.toml
├── deny.toml
├── .github/
│   ├── FUNDING.yml
│   ├── PULL_REQUEST_TEMPLATE.md
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   ├── feature_request.md
│   │   └── config.yml
│   └── workflows/
│       ├── ci.yml
│       ├── release.yml
│       ├── coverage.yml
│       ├── gallery.yml
│       └── snapshots.yml
│
├── starsight/                    # Main facade crate (re-exports everything)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── prelude.rs
│
├── starsight-core/               # Layer 1-2: rendering, scales, axes, color, text
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs
│       ├── scene.rs
│       ├── backend/
│       │   ├── mod.rs            # DrawBackend trait
│       │   ├── tiny_skia.rs
│       │   ├── svg.rs
│       │   └── pdf.rs
│       ├── scale/
│       │   ├── mod.rs
│       │   ├── linear.rs
│       │   ├── log.rs
│       │   ├── symlog.rs
│       │   ├── time.rs
│       │   ├── categorical.rs
│       │   └── color.rs
│       ├── axis/
│       │   ├── mod.rs
│       │   ├── tick.rs           # TickLocator / TickFormatter / Wilkinson Extended
│       │   └── grid.rs
│       ├── coord/
│       │   ├── mod.rs
│       │   ├── cartesian.rs
│       │   ├── polar.rs
│       │   └── geographic.rs
│       ├── color/
│       │   ├── mod.rs
│       │   ├── mapping.rs        # Normalize + colormap pipeline
│       │   └── theme.rs          # Theme struct, presets, chromata bridge
│       └── text/
│           ├── mod.rs
│           └── layout.rs         # cosmic-text integration
│
├── starsight-marks/              # Layer 3: geom/mark system, stat transforms
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── aes.rs
│       ├── position.rs
│       ├── geom/
│       │   ├── mod.rs
│       │   ├── point.rs, line.rs, area.rs, bar.rs, rect.rs
│       │   ├── arc.rs, text.rs, rule.rs, polygon.rs
│       │   ├── contour.rs, surface.rs, volume.rs
│       │   └── (one file per geom type)
│       └── stat/
│           ├── mod.rs
│           ├── bin.rs, kde.rs, regression.rs, ecdf.rs
│           ├── boxplot.rs, density2d.rs, hexbin.rs
│           └── (one file per stat type)
│
├── starsight-layout/             # Layer 4: layout, faceting, legends
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── grid.rs, facet.rs, legend.rs
│       ├── colorbar.rs, inset.rs, compose.rs
│       └── (one file per layout concern)
│
├── starsight-figure/             # Layer 5: Figure builder, plot!(), data acceptance
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── figure.rs, plot_macro.rs, auto.rs, shorthand.rs
│       └── data/
│           ├── mod.rs, raw.rs, polars.rs, ndarray.rs, arrow.rs
│           └── (one file per data source)
│
├── starsight-interact/           # Layer 6: interactivity, streaming
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── hover.rs, zoom.rs, pan.rs, select.rs
│       ├── linked.rs, stream.rs, controls.rs
│       └── (one file per interaction mode)
│
├── starsight-export/             # Layer 7: animation, export, terminal
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── animation.rs, png.rs, svg.rs, pdf.rs
│       ├── html.rs, terminal.rs
│       └── (one file per export target)
│
├── starsight-gpu/                # wgpu + vello backend (optional)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── renderer.rs, pipeline.rs, camera.rs
│       ├── mesh.rs, window.rs
│       └── (GPU-specific rendering)
│
├── starsight-derive/             # Proc macros (#[starsight::recipe])
│   ├── Cargo.toml
│   └── src/lib.rs
│
├── examples/
│   ├── quickstart.rs, scatter.rs, statistical.rs
│   ├── surface3d.rs, terminal.rs, interactive.rs
│   ├── polars_integration.rs, streaming.rs, faceting.rs
│   ├── custom_theme.rs, recipe.rs, gallery.rs
│   └── (one example per use case)
│
└── xtask/
    ├── Cargo.toml
    └── src/main.rs               # Gallery generation, benchmarks, CI helpers
```

### Crate dependency graph

```
starsight (facade)
├── starsight-figure   (Layer 5)
│   ├── starsight-layout  (Layer 4)
│   │   └── starsight-marks  (Layer 3)
│   │       └── starsight-core  (Layer 1-2)
│   └── starsight-marks
├── starsight-interact (Layer 6, optional)
│   └── starsight-figure
├── starsight-export   (Layer 7)
│   └── starsight-figure
├── starsight-gpu      (optional)
│   └── starsight-core
└── starsight-derive   (proc macros)
```

Users depend only on `starsight`. Internal crate boundaries exist for compile-time isolation and optional feature gating — they are not published as independent crates unless there is demonstrated downstream demand.


---

## Implementation reference

This section provides foolproof, copy-pasteable reference material for every core technology starsight depends on. Every type signature, every gotcha, every conversion step.

### tiny-skia 0.12 — the CPU rasterization engine

**Docs**: https://docs.rs/tiny-skia/0.12.0/tiny_skia/ — **Repo**: https://github.com/linebender/tiny-skia — **License**: BSD-3-Clause

#### Three color types — when to use each

| Type | Components | Alpha | Construction | Use for |
|------|-----------|-------|-------------|---------|
| `Color` | f32 × 4 | Straight | `from_rgba(r,g,b,a) -> Option` or `from_rgba8(r,g,b,a) -> Self` | Paint, Pixmap::fill |
| `ColorU8` | u8 × 4 | Straight | `from_rgba(r,g,b,a) -> Self` (const, infallible) | Intermediate conversion |
| `PremultipliedColorU8` | u8 × 4 | Premultiplied | `from_rgba(r,g,b,a) -> Option` (fails if r>a, g>a, or b>a) | Pixmap pixel storage |

**Critical**: `Color::from_rgba()` returns `Option` (validates 0.0-1.0 range). `Color::from_rgba8()` does NOT return Option. `PremultipliedColorU8::from_rgba()` returns `Option` because premultiplied channels cannot exceed alpha.

**Premultiplication formula**: `premul_r = r * a / 255` (u8), `premul_r = r * a` (f32). Reverse: `straight_r = premul_r * 255 / a` (0 when a=0).

**`Pixmap::fill(color: Color)`** takes straight alpha, premultiplies internally. **`paint.set_color_rgba8(r, g, b, a)`** stores straight alpha Color in Shader::SolidColor, premultiplication happens in the rendering pipeline.

#### Core rendering — all draw methods take `Option<&Mask>` (not ClipMask)

```rust
pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
pixmap.fill_rect(rect, &paint, Transform::identity(), None);
pixmap.draw_pixmap(x: i32, y: i32, source: PixmapRef, &ppaint, Transform::identity(), None);
```

`Mask` replaces the old `ClipMask` in v0.12. Create with `Mask::new(w, h).unwrap()`, fill with `mask.fill_path(...)`.

#### PathBuilder — `finish()` returns `Option<Path>`

```rust
let mut pb = PathBuilder::new();
pb.move_to(50.0, 300.0);
pb.line_to(200.0, 100.0);
pb.cubic_to(200.0, 50.0, 600.0, 550.0, 750.0, 300.0);
pb.close();
let path = pb.finish().unwrap();  // None if empty
```

Static constructors: `PathBuilder::from_rect(rect) -> Path` (infallible), `PathBuilder::from_circle(cx, cy, r) -> Option<Path>`. Batch circles with `pb.push_circle(cx, cy, r)` and one `finish()`.

#### Stroke struct — all fields public

```rust
let stroke = Stroke {
    width: 2.0,
    miter_limit: 4.0,
    line_cap: LineCap::Round,   // Butt | Round | Square
    line_join: LineJoin::Round,  // Miter | MiterClip | Round | Bevel
    dash: StrokeDash::new(vec![10.0, 5.0], 0.0),  // returns Option
};
```

#### Transform — angle in DEGREES, not radians

```rust
Transform::identity()
Transform::from_translate(tx, ty)
Transform::from_scale(sx, sy)
Transform::from_rotate(degrees)           // NOT radians
Transform::from_rotate_at(degrees, cx, cy)
transform.pre_translate(tx, ty)           // apply before current
transform.post_concat(other)              // apply after current
```

#### LinearGradient — returns `Option<Shader<'static>>`

```rust
let gradient = LinearGradient::new(
    Point::from_xy(0.0, 0.0),
    Point::from_xy(100.0, 0.0),
    vec![
        GradientStop::new(0.0, Color::from_rgba8(255, 0, 0, 255)),
        GradientStop::new(1.0, Color::from_rgba8(0, 0, 255, 255)),
    ],
    SpreadMode::Pad,
    Transform::identity(),
).unwrap();
paint.shader = gradient;
```

#### PNG export

```rust
// To file
pixmap.save_png("output.png").unwrap();

// To bytes in memory
let png_bytes: Vec<u8> = pixmap.encode_png().unwrap();  // returns Result<Vec<u8>, png::EncodingError>
```

For DPI control, use the `png` crate directly: `encoder.set_pixel_dims(Some(png::PixelDimensions { xppu: 11811, yppu: 11811, unit: png::Unit::Meter }))` — 300 DPI = 11811 pixels/meter.

---

### cosmic-text 0.18 — text shaping and glyph compositing

**Docs**: https://docs.rs/cosmic-text/0.18.2/cosmic_text/ — **Repo**: https://github.com/pop-os/cosmic-text

#### Initialization

```rust
use cosmic_text::{FontSystem, SwashCache, Buffer, Metrics, Attrs, Shaping, Family, Weight};

let mut font_system = FontSystem::new();  // loads system fonts (~1s release, ~10s debug)
let mut swash_cache = SwashCache::new();  // no params
```

Embed custom fonts: `font_system.db_mut().load_font_data(include_bytes!("fonts/Inter.ttf").to_vec());`

#### Buffer setup and shaping

```rust
let metrics = Metrics::new(14.0, 20.0);  // font_size, line_height (f32 pixels)
let mut buffer = Buffer::new(&mut font_system, metrics);

// Option A: direct API
buffer.set_text(&mut font_system, "Hello", &Attrs::new(), Shaping::Advanced, None);
buffer.set_size(&mut font_system, Some(400.0), Some(200.0));
buffer.shape_until_scroll(&mut font_system, true);

// Option B: BorrowedWithFontSystem (elides font_system param)
let mut buf = buffer.borrow_with(&mut font_system);
buf.set_text("Hello", &Attrs::new(), Shaping::Advanced, None);
buf.set_size(Some(400.0), Some(200.0));
buf.shape_until_scroll(true);
```

#### Measuring text dimensions

```rust
let (mut max_w, mut total_h) = (0.0f32, 0.0f32);
for run in buffer.layout_runs() {
    max_w = max_w.max(run.line_w);
    total_h = run.line_top + run.line_height;
}
// max_w = width in pixels, total_h = height in pixels
```

#### Compositing onto tiny-skia — NO channel swap for file output

```rust
use cosmic_text::Color as CTextColor;

let text_color = CTextColor::rgb(0x33, 0x33, 0x33);

buffer.draw(&mut swash_cache, text_color, |x, y, w, h, color| {
    if color.a() == 0 { return; }
    let mut paint = Paint::default();
    // For file output (PNG/SVG): NO channel swap needed
    paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
    paint.anti_alias = false;  // pixels already rasterized by swash
    if let Some(rect) = Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
});
```

> [!WARNING]
> The cosmic-text rich-text example swaps R and B channels — that's for **softbuffer display**, not for file output. `softbuffer` uses `0xXXRRGGBB` u32 values (BGRA on little-endian) while tiny-skia stores RGBA bytes. For PNG/SVG rendering, pass channels straight through.

---

### prismatica — scientific colormaps for starsight

**Docs**: https://docs.rs/prismatica — **Repo**: https://github.com/resonant-jovian/prismatica — **License**: GPL-3.0

308 continuous colormaps + 70 discrete palettes from 10 collections. Both `no_std` and `no_alloc` compatible.

#### Core types

```rust
use prismatica::{Colormap, DiscretePalette, Color, ColormapKind};

// Continuous colormap — sample at any t in [0, 1]
let color: Color = prismatica::crameri::BATLOW.eval(0.5);
let color: Color = prismatica::crameri::BATLOW.eval_rational(5, 10);  // 5th of 10 samples

// Reversed view (zero allocation)
let rev = prismatica::crameri::BATLOW.reversed();
assert_eq!(rev.eval(0.0), prismatica::crameri::BATLOW.eval(1.0));

// Extract N discrete colors
let colors: Vec<Color> = prismatica::crameri::BATLOW.colors(8);  // requires alloc feature

// Discrete palette — categorical data
let c: Color = prismatica::colorbrewer::SET2_PALETTE.get(0);
let c: Color = prismatica::colorbrewer::SET2_PALETTE.get(8);  // wraps around

// Metadata
assert_eq!(prismatica::crameri::BATLOW.kind(), ColormapKind::Sequential);
assert!(prismatica::crameri::BATLOW.meta.perceptually_uniform);
assert!(prismatica::crameri::BATLOW.meta.cvd_friendly);
```

#### Choosing colormaps by data type

| Data type | Recommendation | Examples |
|-----------|---------------|----------|
| Sequential (temperature, elevation) | Sequential map | `BATLOW`, `VIRIDIS`, `OSLO`, `THERMAL` |
| Diverging (anomalies, residuals) | Diverging map | `BERLIN`, `VIK`, `BALANCE`, `SMOOTH_COOL_WARM` |
| Cyclic (phase, direction) | Cyclic map | `ROMA_O`, `PHASE`, `TWILIGHT` |
| Categorical (labels, classes) | Discrete palette | `SET2_PALETTE`, `DARK2_PALETTE`, `TABLEAU10` |

#### Color conversion to tiny-skia

With `tiny-skia-integration` feature enabled:

```rust
let pcolor: prismatica::Color = prismatica::crameri::BATLOW.eval(0.5);
let tcolor: tiny_skia::Color = pcolor.into();  // From<prismatica::Color> for tiny_skia::Color
paint.set_color(tcolor);
```

Without the integration feature:

```rust
let c = prismatica::crameri::BATLOW.eval(0.5);
paint.set_color_rgba8(c.r, c.g, c.b, 255);
```

#### Runtime lookup

```rust
use prismatica::{find_by_name, filter_by_kind, ColormapKind};

let cm = find_by_name("batlow").expect("batlow should exist");
let diverging = filter_by_kind(ColormapKind::Diverging);  // Vec<&Colormap>
```

---

### chromata — editor themes for starsight

**Docs**: https://docs.rs/chromata — **Repo**: https://github.com/resonant-jovian/chromata — **License**: GPL-3.0

1,104 editor/terminal color themes as compile-time constants. `no_std` compatible.

#### Core types

```rust
use chromata::{Theme, Color, Variant, Contrast};

let theme: &Theme = &chromata::popular::gruvbox::DARK_HARD;

// Theme metadata
assert_eq!(theme.name, "Gruvbox Dark Hard");
assert!(theme.is_dark());
assert_eq!(theme.variant, Variant::Dark);
assert_eq!(theme.contrast, Contrast::High);

// Core colors (always present)
let bg: Color = theme.bg;          // editor background
let fg: Color = theme.fg;          // default text
let accent: Color = theme.accent(); // first available accent (blue > purple > cyan > ...)

// Syntax colors (Option<Color>)
let kw: Option<Color> = theme.keyword;
let str_c: Option<Color> = theme.string;
let fn_c: Option<Color> = theme.function;

// Diagnostic colors
let err: Option<Color> = theme.error;
let warn: Option<Color> = theme.warning;

// Named accent palette
let red: Option<Color> = theme.red;
let blue: Option<Color> = theme.blue;
let green: Option<Color> = theme.green;
```

#### Color utilities

```rust
use chromata::Color;

let c = Color::new(255, 128, 0);           // from RGB u8
let c = Color::from_hex(0xFF8000);          // from 24-bit hex
let c = Color::from_css_hex("#ff8000").unwrap();  // from CSS string
let c: Color = "#FF8000".parse().unwrap();  // FromStr

let hex_str: String = c.to_css_hex();       // "#ff8000"
let (r, g, b): (f32, f32, f32) = c.to_f32(); // 0.0-1.0 range
let lum: f64 = c.luminance();               // WCAG relative luminance
let ratio: f64 = c.contrast_ratio(other);   // 1.0-21.0
let mid: Color = c.lerp(other, 0.5);        // linear interpolation
```

#### Building a starsight theme from chromata

```rust
/// Convert a chromata Theme to a starsight rendering theme.
fn chromata_to_starsight_theme(ct: &chromata::Theme) -> StarsightTheme {
    let bg = ct.bg;
    let fg = ct.fg;
    let accent = ct.accent();

    // Build the color cycle from available accent colors
    let mut color_cycle = Vec::new();
    for field in [ct.blue, ct.red, ct.green, ct.orange, ct.purple, ct.cyan, ct.yellow, ct.magenta] {
        if let Some(c) = field {
            color_cycle.push(c);
        }
    }
    if color_cycle.is_empty() {
        color_cycle.push(accent);
    }

    StarsightTheme {
        background: color_to_tiny_skia(bg),
        foreground: color_to_tiny_skia(fg),
        axis_color: ct.gutter.map(color_to_tiny_skia)
            .unwrap_or_else(|| color_to_tiny_skia(fg)),
        grid_color: ct.selection.map(color_to_tiny_skia)
            .unwrap_or_else(|| {
                // Derive from bg: slightly lighter for dark, slightly darker for light
                let shift = if ct.is_dark() { 30 } else { -30i16 };
                let lighten = |v: u8| (v as i16 + shift).clamp(0, 255) as u8;
                tiny_skia::Color::from_rgba8(lighten(bg.r), lighten(bg.g), lighten(bg.b), 40)
            }),
        color_cycle: color_cycle.into_iter()
            .map(color_to_tiny_skia)
            .collect(),
        error_color: ct.error.map(color_to_tiny_skia)
            .unwrap_or(tiny_skia::Color::from_rgba8(255, 80, 80, 255)),
        // ...
    }
}

fn color_to_tiny_skia(c: chromata::Color) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba8(c.r, c.g, c.b, 255)
}
```

#### Query APIs

```rust
let theme = chromata::find_by_name("Catppuccin Mocha").unwrap();
let dark_themes = chromata::filter_by_variant(chromata::Variant::Dark);
let all = chromata::collect_all_themes();  // Vec<&'static Theme>
```

---

### Wilkinson Extended tick algorithm

**Paper**: Talbot, Lin, Hanrahan (2010) "An Extension of Wilkinson's Algorithm for Positioning Tick Labels on Axes" — https://vis.stanford.edu/files/2010-TickLabels-InfoVis.pdf — **Reference R implementation**: https://rdrr.io/cran/labeling/src/R/labeling.R

#### Scoring function

**Score = w₁·simplicity + w₂·coverage + w₃·density + w₄·legibility**, with weights **w = [0.2, 0.25, 0.5, 0.05]**.

**Q preference list**: `[1, 5, 2, 2.5, 4, 3]` — ordered by human readability. Skip factor `j` generates step sizes: actual step = `j × q × 10^z`.

**Simplicity** = `1 - (i-1)/(|Q|-1) - j + v` where i = index in Q, j = skip, v = 1 if zero included.

**Coverage** = `1 - 0.5 × ((dmax-lmax)² + (dmin-lmin)²) / (0.1×(dmax-dmin))²`

**Density** = `2 - max(ρ/ρt, ρt/ρ)` where ρ = actual label density, ρt = target density.

The algorithm uses aggressive branch-and-bound pruning — computes upper bounds at each nesting level and breaks when no candidate can beat the current best. Averages ~41 inner iterations.

#### No Rust crate exists for this — starsight should implement it

Neither `plotters` nor any other Rust crate implements the Extended algorithm. D3 uses a simpler Heckbert formula with Q={1,2,5}. This is a genuine contribution starsight can make.

---

### SVG text positioning

SVG text `x`, `y` set the **baseline** position. Use `text-anchor` for horizontal alignment (`start`, `middle`, `end`) and `dominant-baseline` for vertical alignment (`alphabetic`, `central`, `hanging`).

For Y-axis label rotation: `<text transform="translate(15, {center_y}) rotate(-90)" text-anchor="middle" dominant-baseline="central">`.

**SVG cannot measure text width** without rendering. For static generation, estimate: digits ≈ 0.55 × font_size per character, average character ≈ 0.6 × font_size. For precision, use cosmic-text or fontdue to measure advance widths.

---

### Chart layout math

Standard margin calculation: `left_margin = pad + y_label_height + label_pad + max_ytick_width + tick_pad`. Where `max_ytick_width = max(len(format(tick))) × font_size × 0.6`. Plot area = figure dimensions minus all margins.

For facet grids: `cell_width = (available_width - (ncol-1) × spacing) / ncol`.

---

### Error handling pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum StarsightError {
    #[error("Rendering backend failure: {0}")]
    Render(String),
    #[error("Data shape/type mismatch: {0}")]
    Data(String),
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Scale domain/range error: {0}")]
    Scale(String),
    #[error("Export format error: {0}")]
    Export(String),
    #[error("Invalid configuration: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, StarsightError>;
```

Use `thiserror` (https://docs.rs/thiserror/2.0.18) for library errors, never `anyhow`. Wrap external errors with `Box<dyn std::error::Error + Send + Sync>` to avoid leaking dependency types.

---

### plot!() macro design

Use `macro_rules!` (not proc macros) — instant compilation, no extra deps:

```rust
#[macro_export]
macro_rules! plot {
    ($df:expr, x = $x:expr, y = $y:expr $(, $key:ident = $val:expr)* $(,)?) => {{
        let mut cfg = $crate::DataFramePlotConfig::new($df, $x, $y);
        $( cfg = cfg.$key($val); )*
        cfg.build()
    }};
    ($x:expr, $y:expr $(,)?) => {{
        $crate::PlotBuilder::from_arrays($x, $y).build()
    }};
    ($data:expr $(,)?) => {{
        $crate::PlotBuilder::from_single($data).build()
    }};
}
```

Literal tokens `x =` and `y =` disambiguate DataFrame from positional syntax. `$(,)?` handles trailing commas. Internal rules prefixed with `@` for sub-dispatch.


---

## Development guidelines

### Code style and conventions

**Naming**: All public types use full descriptive names. No abbreviations in public API (`Figure`, not `Fig`; `Histogram`, not `Hist`). Internal code may abbreviate where conventional (`ctx`, `opts`, `cfg`).

**Error handling**: All fallible operations return `Result<T, StarsightError>`. Never panic in library code. The `plot!()` macro and `.show()` may panic on unrecoverable display errors (no available backend) with a clear diagnostic message — this is the only exception.

**Builder pattern**: All complex types use the builder pattern with method chaining. Builders use `&mut self -> &mut Self` for optional configuration and consuming `self` for terminal `build()`. Provide `Default` implementations for all option structs.

**Trait design**: Prefer concrete types over trait objects in the public API. Use generics with trait bounds for data acceptance (`impl Into<DataSource>`). Reserve `dyn Trait` for the backend abstraction where dynamic dispatch is necessary.

**Generics**: Keep generic parameter counts low in public APIs. Use `impl Trait` in argument position for ergonomics. Never expose more than two generic parameters on a public type.

### Feature gating rules

Every optional dependency must be behind a feature gate. Feature gates use `#[cfg(feature = "...")]` at the **module level**, not scattered throughout functions. The `default` feature set must produce a useful crate with CPU rendering, SVG, and PNG export.

### Testing strategy

**Unit tests**: Every scale, mark, stat transform, and layout algorithm has unit tests with known-good reference values. Statistical transforms (KDE, regression) tested against scipy/R reference outputs.

**Snapshot tests** (via [insta](https://docs.rs/insta/1.47.1)): Every chart type has a snapshot test producing a reference PNG at fixed dimensions via the tiny-skia backend (deterministic). Binary snapshots with `assert_binary_snapshot!(".png", bytes)`. Use `cargo insta review` for interactive review. CI runs `cargo insta test --check --unreferenced reject`.

**Property tests** (via [proptest](https://docs.rs/proptest/1.11.0)): Scale round-trip (`inverse(transform(x)) == x`), axis tick generation (monotonically increasing), color mapping (monotonic for sequential colormaps).

**No GPU in CI**: CI runs only tiny-skia and SVG backends. GPU tests run locally or in a separate GPU-enabled pipeline. The starsight-gpu crate has mock backends for unit testing.

### Documentation requirements

Every public type, trait, method, and function must have a rustdoc comment. Examples mandatory for all Layer 5 API. Doc examples must compile and run (`cargo test --doc`). Module-level doc comments for every `mod.rs`. Top-level `lib.rs` doc comment includes a complete quickstart.

Feature-gated items use `#![cfg_attr(docsrs, feature(doc_auto_cfg))]` for automatic annotation on docs.rs.

---

## Hard restrictions

1. **No JavaScript runtime dependencies.** No Node.js, Deno, or any JS engine for any functionality in any configuration.
2. **No C/C++ system library dependencies in the default feature set.** Default configuration must compile with only a Rust toolchain.
3. **No `unsafe` code in Layers 3-5.** Unsafe permitted only in rendering backends (Layer 1) and GPU code with mandatory `// SAFETY:` comments.
4. **No runtime file I/O for core functionality.** Colormaps from prismatica (compile-time), themes from chromata (compile-time). Default font embedded via `include_bytes!()`.
5. **No `println!()` or `eprintln!()` in library code.** Use the `log` crate for diagnostics. Silent by default.
6. **No panics** except in `.show()` when no display backend is available.
7. **No nightly-only features required.** Must compile on stable Rust.
8. **No async in the public API.** Streaming uses push-based `fig.append()`, not async streams.

## Non-goals

1. **Not a GUI framework.** Produces charts, not applications.
2. **Not a game engine.** 3D is for data visualization only.
3. **Not a BI/dashboard platform.** No server, no database connectors.
4. **Not a notebook.** No REPL, no Jupyter kernel.
5. **Not a wrapper.** No gnuplot, no Plotly.js, no ECharts, no matplotlib. Every chart rendered by Rust in-process.

---

## Reference links

### Rendering and graphics

| Crate | docs.rs | crates.io | GitHub |
|-------|---------|-----------|--------|
| tiny-skia 0.12 | https://docs.rs/tiny-skia | https://crates.io/crates/tiny-skia | https://github.com/linebender/tiny-skia |
| wgpu | https://docs.rs/wgpu | https://crates.io/crates/wgpu | https://github.com/gfx-rs/wgpu |
| vello | https://docs.rs/vello | https://crates.io/crates/vello | https://github.com/linebender/vello |
| lyon | https://docs.rs/lyon | https://crates.io/crates/lyon | https://github.com/nical/lyon |
| cosmic-text | https://docs.rs/cosmic-text | https://crates.io/crates/cosmic-text | https://github.com/pop-os/cosmic-text |
| parley | https://docs.rs/parley | https://crates.io/crates/parley | https://github.com/linebender/parley |
| ab_glyph | https://docs.rs/ab_glyph | https://crates.io/crates/ab_glyph | https://github.com/alexheretic/ab-glyph |

### Data and math

| Crate | docs.rs | crates.io | GitHub |
|-------|---------|-----------|--------|
| polars | https://docs.rs/polars | https://crates.io/crates/polars | https://github.com/pola-rs/polars |
| ndarray | https://docs.rs/ndarray | https://crates.io/crates/ndarray | https://github.com/rust-ndarray/ndarray |
| nalgebra | https://docs.rs/nalgebra | https://crates.io/crates/nalgebra | https://github.com/dimforge/nalgebra |
| arrow | https://docs.rs/arrow | https://crates.io/crates/arrow | https://github.com/apache/arrow-rs |
| statrs | https://docs.rs/statrs | https://crates.io/crates/statrs | https://github.com/statrs-dev/statrs |
| contour | https://docs.rs/contour | https://crates.io/crates/contour | https://github.com/mthh/contour-rs |
| delaunator | https://docs.rs/delaunator | https://crates.io/crates/delaunator | https://github.com/mourner/delaunator-rs |

### Color and theming

| Crate | docs.rs | crates.io | GitHub |
|-------|---------|-----------|--------|
| palette | https://docs.rs/palette | https://crates.io/crates/palette | https://github.com/Ogeon/palette |
| chromata | https://docs.rs/chromata | https://crates.io/crates/chromata | https://github.com/resonant-jovian/chromata |
| prismatica | https://docs.rs/prismatica | https://crates.io/crates/prismatica | https://github.com/resonant-jovian/prismatica |

### Image, SVG, PDF output

| Crate | docs.rs | crates.io | GitHub |
|-------|---------|-----------|--------|
| image | https://docs.rs/image | https://crates.io/crates/image | https://github.com/image-rs/image |
| resvg | https://docs.rs/resvg | https://crates.io/crates/resvg | https://github.com/linebender/resvg |
| svg | https://docs.rs/svg | https://crates.io/crates/svg | https://github.com/bodoni/svg |
| krilla | https://docs.rs/krilla | https://crates.io/crates/krilla | https://github.com/LaurenzV/krilla |
| pdf-writer | https://docs.rs/pdf-writer | https://crates.io/crates/pdf-writer | https://github.com/typst/pdf-writer |

### Terminal, GUI, WASM

| Crate | docs.rs | crates.io | GitHub |
|-------|---------|-----------|--------|
| ratatui | https://docs.rs/ratatui | https://crates.io/crates/ratatui | https://github.com/ratatui/ratatui |
| crossterm | https://docs.rs/crossterm | https://crates.io/crates/crossterm | https://github.com/crossterm-rs/crossterm |
| ratatui-image | https://docs.rs/ratatui-image | https://crates.io/crates/ratatui-image | https://github.com/ratatui/ratatui-image |
| winit | https://docs.rs/winit | https://crates.io/crates/winit | https://github.com/rust-windowing/winit |
| egui | https://docs.rs/egui | https://crates.io/crates/egui | https://github.com/emilk/egui |
| wasm-bindgen | https://docs.rs/wasm-bindgen | https://crates.io/crates/wasm-bindgen | https://github.com/rustwasm/wasm-bindgen |

### Geospatial

| Crate | docs.rs | crates.io | GitHub |
|-------|---------|-----------|--------|
| geo | https://docs.rs/geo | https://crates.io/crates/geo | https://github.com/georust/geo |
| proj | https://docs.rs/proj | https://crates.io/crates/proj | https://github.com/georust/proj |
| geojson | https://docs.rs/geojson | https://crates.io/crates/geojson | https://github.com/georust/geojson |

### Utilities

| Crate | docs.rs | crates.io | GitHub |
|-------|---------|-----------|--------|
| thiserror | https://docs.rs/thiserror | https://crates.io/crates/thiserror | https://github.com/dtolnay/thiserror |
| serde | https://docs.rs/serde | https://crates.io/crates/serde | https://github.com/serde-rs/serde |
| insta | https://docs.rs/insta | https://crates.io/crates/insta | https://github.com/mitsuhiko/insta |
| proptest | https://docs.rs/proptest | https://crates.io/crates/proptest | https://github.com/proptest-rs/proptest |
| clap | https://docs.rs/clap | https://crates.io/crates/clap | https://github.com/clap-rs/clap |

### Rust community and standards

- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/ (checklist: https://rust-lang.github.io/api-guidelines/checklist.html)
- Cargo workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Cargo features: https://doc.rust-lang.org/cargo/reference/features.html
- Cargo SemVer: https://doc.rust-lang.org/cargo/reference/semver.html
- Cargo publishing: https://doc.rust-lang.org/cargo/reference/publishing.html
- Rustdoc book: https://doc.rust-lang.org/rustdoc/
- The Rustonomicon: https://doc.rust-lang.org/nomicon/
- Rust Edition Guide 2024: https://doc.rust-lang.org/edition-guide/rust-2024/index.html
- Effective Rust: https://effective-rust.com/
- Clippy lint list: https://rust-lang.github.io/rust-clippy/master/index.html
- rustfmt config: https://rust-lang.github.io/rustfmt/

### Versioning and release tooling

- Keep a Changelog: https://keepachangelog.com/en/1.1.0/
- Semantic Versioning: https://semver.org/
- Conventional Commits: https://www.conventionalcommits.org/en/v1.0.0/
- cargo-semver-checks: https://github.com/obi1kenobi/cargo-semver-checks
- git-cliff: https://github.com/orhun/git-cliff
- cargo-release: https://github.com/crate-ci/cargo-release
- cargo-deny: https://github.com/EmbarkStudios/cargo-deny
- cargo-audit: https://github.com/rustsec/rustsec
- cargo-llvm-cov: https://github.com/taiki-e/cargo-llvm-cov

### CI/CD

- dtolnay/rust-toolchain: https://github.com/dtolnay/rust-toolchain
- Swatinem/rust-cache: https://github.com/Swatinem/rust-cache
- EmbarkStudios/cargo-deny-action: https://github.com/EmbarkStudios/cargo-deny-action
- wasm-pack: https://github.com/rustwasm/wasm-pack
- trunk: https://github.com/trunk-rs/trunk

### Visualization theory

- Grammar of Graphics (Wilkinson, 2005): https://link.springer.com/book/10.1007/0-387-28695-0
- Wilkinson Extended tick algorithm: https://vis.stanford.edu/files/2010-TickLabels-InfoVis.pdf
- Vega-Lite spec: https://vega.github.io/vega-lite/docs/spec.html
- Observable Plot: https://observablehq.com/plot/
- ggplot2: https://ggplot2.tidyverse.org/
- Makie.jl: https://docs.makie.org/stable/
- D3.js: https://d3js.org/
- matplotlib: https://matplotlib.org/stable/index.html
- seaborn: https://seaborn.pydata.org/

### Terminal graphics protocols

- Kitty graphics protocol: https://sw.kovidgoyal.net/kitty/graphics-protocol/
- Sixel (VT3xx manual): https://vt100.net/docs/vt3xx-gp/chapter14.html
- iTerm2 inline images: https://iterm2.com/documentation-images.html

### Color science

- Crameri scientific colour maps: https://www.fabiocrameri.ch/colourmaps/
- CET perceptually uniform maps: https://colorcet.com/
- ColorBrewer: https://colorbrewer2.org/

### Licensing

- GPL-3.0 full text: https://www.gnu.org/licenses/gpl-3.0.html
- SPDX identifiers: https://spdx.org/licenses/
- GPL FAQ: https://www.gnu.org/licenses/gpl-faq.html

### Ecosystem trackers

- Are We GUI Yet: https://areweguiyet.com/
- Are We Learning Yet: https://www.arewelearningyet.com/
- Are We Game Yet: https://arewegameyet.rs/
- lib.rs visualization: https://lib.rs/visualization


---

## Roadmap: 0.1.0 through 1.0.0

Exhaustive, ordered task list for every milestone release. Tasks are grouped by release version and ordered by dependency.

## Pre-0.1.0 — Repository bootstrap

### Repository and workspace setup

- [x] Create `resonant-jovian/starsight` GitHub repository
- [ ] Write initial README.md with project description, badges, and resonant-jovian ecosystem overview
- [x] Add GPL-3.0-or-later LICENSE file
- [x] Create CONTRIBUTING.md with PR process, commit message conventions (Conventional Commits), and code review expectations
- [x] Create CODE_OF_CONDUCT.md (Contributor Covenant or Rust Code of Conduct reference)
- [x] Create CHANGELOG.md skeleton following Keep a Changelog format
- [x] Create SECURITY.md with vulnerability reporting instructions
- [x] Create `.github/ISSUE_TEMPLATE/` directory with templates for bug report, feature request, and chart type request
- [x] Create `.github/PULL_REQUEST_TEMPLATE.md` with checklist (tests, docs, changelog entry, snapshot update)
- [x] Create `.github/FUNDING.yml` pointing to Stripe donations and thanks.dev

### Workspace Cargo configuration

- [x] Initialize workspace root `Cargo.toml` with `resolver = "2"` and workspace members list
- [x] Create `starsight/Cargo.toml` (facade crate) with all workspace dependencies
- [x] Create `starsight-core/Cargo.toml` with tiny-skia, palette, cosmic-text, ab_glyph, svg, image, colorgrad, colorous as dependencies
- [x] Create `starsight-marks/Cargo.toml` depending on starsight-core
- [x] Create `starsight-layout/Cargo.toml` depending on starsight-marks
- [x] Create `starsight-figure/Cargo.toml` depending on starsight-layout and starsight-marks
- [x] Create `starsight-interact/Cargo.toml` depending on starsight-figure (optional feature)
- [x] Create `starsight-export/Cargo.toml` depending on starsight-figure
- [x] Create `starsight-gpu/Cargo.toml` depending on starsight-core (optional feature)
- [x] Create `starsight-derive/Cargo.toml` (proc-macro crate)
- [x] Create `xtask/Cargo.toml` for build automation tasks
- [x] Define all feature flags in workspace root: `default`, `full`, `minimal`, `science`, `dashboard`, `terminal`, `web`, `gpu`, `interactive`, `polars`, `ndarray`, `arrow`, `3d`, `pdf`, `stats`, `contour`, `geo`, `resvg`
- [x] Configure `[workspace.lints.clippy]` with `pedantic` + selective allows
- [x] Create `.rustfmt.toml` with project formatting rules
- [x] Create `.clippy.toml` if needed for non-default lint configuration
- [x] Create `deny.toml` for cargo-deny license and advisory configuration
- [x] Configure `[package.metadata.docs.rs]` in each crate for docs.rs feature flag builds
- [x] Add workspace-level `[profile.release]` with LTO and codegen-units=1
- [x] Add workspace-level `[profile.dev]` with opt-level=1 for tiny-skia performance during development

### CI/CD pipeline

- [x] Create `.github/workflows/ci.yml` — runs on every PR and push to main
  - [ ] Job: `check` — `cargo check --workspace --all-features`
  - [ ] Job: `test` — `cargo test --workspace` (default features)
  - [ ] Job: `test-all-features` — `cargo test --workspace --all-features`
  - [ ] Job: `test-minimal` — `cargo test --workspace --no-default-features`
  - [ ] Job: `clippy` — `cargo clippy --workspace --all-features -- -D warnings`
  - [ ] Job: `fmt` — `cargo fmt --all -- --check`
  - [ ] Job: `doc` — `cargo doc --workspace --all-features --no-deps` (verify doc builds)
  - [ ] Job: `deny` — `cargo deny check`
  - [ ] Job: `semver` — `cargo semver-checks` (after first publish)
  - [ ] Job: `wasm` — verify WASM compilation with `cargo build --target wasm32-unknown-unknown --features web` (placeholder)
  - [ ] Matrix: test on `ubuntu-latest`, `macos-latest`, `windows-latest`
  - [ ] Matrix: test on Rust `stable`, `beta`, and MSRV
- [x] Create `.github/workflows/release.yml` — triggered by version tags
  - [ ] Publish all workspace crates to crates.io in dependency order
  - [ ] Generate GitHub release with changelog extract via git-cliff
- [x] Create `.github/workflows/coverage.yml` — weekly or on-demand
  - [ ] Run cargo-llvm-cov and upload to Codecov or similar
- [ ] Create `.github/workflows/snapshots.yml` — runs snapshot tests and stores artifacts
- [ ] Create `.github/workflows/gallery.yml` — generates gallery images for documentation

### Skeleton source files

- [ ] Create `starsight/src/lib.rs` with top-level doc comment and feature-gated re-exports
- [ ] Create `starsight/src/prelude.rs` with `pub use` of all primary types
- [ ] Create `starsight-core/src/lib.rs` with module declarations
- [x] Create stub `mod.rs` for every module listed in the workspace file tree
- [x] Ensure `cargo check --workspace` passes with all stubs
- [ ] Ensure `cargo test --workspace` passes (zero tests, zero failures)
- [ ] Ensure `cargo doc --workspace --no-deps` builds cleanly

### Error type

- [ ] Define `starsight::Error` enum in `starsight-core/src/error.rs` using thiserror
  - [ ] Variant: `Render(String)` — rendering backend failures
  - [ ] Variant: `Data(String)` — data shape/type mismatches
  - [ ] Variant: `Io(std::io::Error)` — file I/O errors
  - [ ] Variant: `Scale(String)` — scale domain/range errors
  - [ ] Variant: `Export(String)` — export format errors
  - [ ] Variant: `Config(String)` — invalid configuration
- [ ] Define `pub type Result<T> = std::result::Result<T, Error>;`
- [ ] Implement `From<std::io::Error>` for `Error`
- [ ] Implement `From<tiny_skia::...>` conversions as needed
- [ ] Write unit tests for error creation and Display output

---

## 0.1.0 — Foundation (Phase 0)

> Exit criteria: `plot!([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]).save("test.png")` produces a correct line chart.

### starsight-core: Rendering abstraction (Layer 1)

#### DrawBackend trait

- [ ] Define `DrawBackend` trait in `starsight-core/src/backend/mod.rs`
  - [ ] Method: `fn draw_path(&mut self, path: &Path, style: &PathStyle) -> Result<()>`
  - [ ] Method: `fn draw_text(&mut self, text: &TextBlock, position: Point) -> Result<()>`
  - [ ] Method: `fn draw_image(&mut self, image: &ImageData, rect: Rect) -> Result<()>`
  - [ ] Method: `fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()>`
  - [ ] Method: `fn dimensions(&self) -> (u32, u32)`
  - [ ] Method: `fn save_png(&self, path: &std::path::Path) -> Result<()>`
  - [ ] Method: `fn save_svg(&self, path: &std::path::Path) -> Result<()>`
- [ ] Define `PathStyle` struct: stroke color, stroke width, fill color, dash pattern, line cap, line join, opacity
- [ ] Define `Path` type wrapping a sequence of `PathCommand` (MoveTo, LineTo, QuadTo, CubicTo, Close)
- [ ] Define `Point`, `Rect`, `Size` geometry primitives
- [ ] Define `Color` type with from_rgba, from_hex, named constants, and conversion to palette types
- [ ] Write unit tests for all geometry primitives

#### Scene graph

- [ ] Define `SceneNode` enum in `starsight-core/src/scene.rs`
  - [ ] Variant: `Path { path, style }`
  - [ ] Variant: `Text { block, position }`
  - [ ] Variant: `Group { children, transform }`
  - [ ] Variant: `Clip { rect, child }`
- [ ] Define `Scene` struct holding root `Vec<SceneNode>`
- [ ] Implement `Scene::render(&self, backend: &mut dyn DrawBackend) -> Result<()>`
- [ ] Write unit tests for scene construction and traversal

#### tiny-skia backend

- [ ] Implement `TinySkiaBackend` struct wrapping `tiny_skia::Pixmap`
- [ ] Implement `DrawBackend` for `TinySkiaBackend`
  - [ ] `draw_path`: convert Path to tiny_skia::Path, apply PathStyle to Paint/Stroke
  - [ ] `draw_text`: rasterize glyphs via ab_glyph, composite onto pixmap
  - [ ] `fill_rect`: use tiny_skia fill_rect
  - [ ] `save_png`: encode pixmap to PNG via image crate
- [ ] Handle anti-aliasing configuration
- [ ] Handle DPI scaling (logical vs physical pixels)
- [ ] Write integration test: draw a red rectangle, save PNG, verify pixel values
- [ ] Write integration test: draw a diagonal line, verify anti-aliased pixels exist
- [ ] Benchmark: render 1000 lines, measure time, establish baseline

#### SVG backend

- [ ] Implement `SvgBackend` struct wrapping svg::Document
- [ ] Implement `DrawBackend` for `SvgBackend`
  - [ ] `draw_path`: convert Path to SVG `<path d="...">` with style attributes
  - [ ] `draw_text`: emit `<text>` elements with font-family, font-size, fill
  - [ ] `fill_rect`: emit `<rect>` element
  - [ ] `save_svg`: serialize document to file
- [ ] Handle viewBox and dimensions
- [ ] Write integration test: draw shapes, parse output SVG, verify elements exist

#### Text rendering

- [ ] Initialize cosmic-text `FontSystem` with system font fallback
- [ ] Embed a default fallback font via `include_bytes!()` (DejaVu Sans or similar OFL-licensed font)
- [ ] Implement `TextBlock` struct: text content, font family, font size, color, alignment, line height
- [ ] Implement `text::measure(block: &TextBlock) -> Size` for layout calculations
- [ ] Implement glyph rasterization pipeline: cosmic-text shaping → ab_glyph rasterization → pixel buffer
- [ ] Write test: measure "Hello" at 12pt, verify width > 0 and height > 0
- [ ] Write test: render "Hello" on tiny-skia pixmap, verify non-white pixels exist

### starsight-core: Scale system (Layer 2)

#### Scale trait and linear scale

- [ ] Define `Scale` trait in `starsight-core/src/scale/mod.rs`
  - [ ] Method: `fn transform(&self, value: f64) -> f64` (domain → range)
  - [ ] Method: `fn inverse(&self, value: f64) -> f64` (range → domain)
  - [ ] Method: `fn domain(&self) -> (f64, f64)`
  - [ ] Method: `fn range(&self) -> (f64, f64)`
  - [ ] Method: `fn ticks(&self, count: usize) -> Vec<f64>`
  - [ ] Method: `fn nice(&mut self)` — round domain to nice values
- [ ] Implement `LinearScale` with domain, range, clamp option
- [ ] Implement `LinearScale::nice()` using Wilkinson's algorithm or Heckbert's
- [ ] Implement `LinearScale::ticks()` returning evenly spaced nice values
- [ ] Write property test: `inverse(transform(x)) == x` for all x in domain (within f64 epsilon)
- [ ] Write test: `LinearScale::new(0.0..100.0, 0.0..500.0).transform(50.0) == 250.0`
- [ ] Write test: `nice()` on domain (0.13, 97.4) produces (0.0, 100.0)

#### Categorical scale

- [ ] Implement `CategoricalScale` mapping string labels to evenly-spaced band positions
- [ ] Implement `band_width()` method for bar chart width calculation
- [ ] Write test: 3 categories map to positions 1/6, 3/6, 5/6 of range

#### Color scale

- [ ] Implement `ColorScale` mapping a continuous domain [0, 1] to colors via a colormap
- [ ] Accept prismatica colormaps as lookup tables
- [ ] Accept colorgrad gradients
- [ ] Implement `Normalize` trait with Linear and Diverging variants
- [ ] Write test: midpoint of a diverging colormap returns the center color

### starsight-core: Axis system (Layer 2)

#### Tick generation

- [ ] Define `TickLocator` trait: `fn locate(&self, scale: &dyn Scale, count: usize) -> Vec<f64>`
- [ ] Define `TickFormatter` trait: `fn format(&self, value: f64) -> String`
- [ ] Implement `AutoTickLocator` using extended Wilkinson algorithm
- [ ] Implement `NumericFormatter` with configurable decimal places and SI prefixes
- [ ] Write test: AutoTickLocator on [0, 100] with count=5 produces [0, 20, 40, 60, 80, 100]

#### Axis rendering

- [ ] Implement `Axis` struct: scale, position (Left/Right/Top/Bottom), label, tick_locator, tick_formatter, visibility flags
- [ ] Implement `Axis::render(&self) -> Vec<SceneNode>` producing:
  - [ ] Axis line (spine)
  - [ ] Tick marks (major)
  - [ ] Tick labels (formatted text)
  - [ ] Axis label (rotated for Y axis)
- [ ] Implement grid line generation as optional overlay
- [ ] Write snapshot test: X axis from 0 to 100 with label "Time (s)"
- [ ] Write snapshot test: Y axis from 0 to 1.0 with label "Amplitude"

### starsight-core: Theme system

- [ ] Define `Theme` struct with fields: background_color, axis_color, grid_color, grid_width, grid_visible, font_family, font_size, title_font_size, color_cycle (Vec<Color>), line_width, point_size, legend_position, margin (top/right/bottom/left)
- [ ] Implement `Theme::default()` (clean, light background, gray axes)
- [ ] Implement `Theme::minimal()` (no grid, thin axes)
- [ ] Implement `Theme::dark()` (dark background, light text)
- [ ] Implement `Theme::publication()` (high-contrast, serif font, thick lines, PDF-optimized)
- [ ] Implement builder methods for element-level customization
- [ ] Write test: `Theme::default()` has white background and non-zero margins

### starsight-marks: First mark types (Layer 3)

#### Aesthetic mapping

- [ ] Define `Aes` struct: x, y, color, size, shape, alpha, fill, label — each as `Option<AesMapping>`
- [ ] Define `AesMapping` enum: `Column(String)`, `Constant(Value)`, `Computed(Box<dyn Fn>)`
- [ ] Implement `x(col: &str) -> AesMapping`, `y(col: &str) -> AesMapping` shorthand functions
- [ ] Write test: `Aes::new().x("col_a").y("col_b")` stores correct column names

#### Geom trait and Line mark

- [ ] Define `Geom` trait in `starsight-marks/src/geom/mod.rs`
  - [ ] Method: `fn render(&self, data: &ResolvedData, scales: &Scales, theme: &Theme) -> Vec<SceneNode>`
  - [ ] Method: `fn required_aes(&self) -> &[&str]` (e.g., ["x", "y"] for line)
  - [ ] Method: `fn default_stat(&self) -> Option<Box<dyn Stat>>` (identity for line)
- [ ] Implement `LineMark` struct with aes, color, width, dash, alpha options
- [ ] Implement `Geom::line()` constructor with builder pattern
- [ ] Implement `LineMark::render()`: resolve x/y from data, transform through scales, emit Path nodes
- [ ] Write snapshot test: line chart with 5 points
- [ ] Write snapshot test: line chart with 100 sine wave points

#### Point/Scatter mark

- [ ] Implement `PointMark` struct with aes, color, size, shape, alpha options
- [ ] Define `PointShape` enum: Circle, Square, Triangle, Diamond, Cross, Plus, Star
- [ ] Implement `PointMark::render()`: emit circles/polygons at data positions
- [ ] Write snapshot test: scatter plot with 20 random points
- [ ] Write snapshot test: scatter with color mapped to a third variable

### starsight-figure: Figure builder and plot!() macro (Layer 5)

#### DataSource abstraction

- [ ] Define `DataSource` enum: `Slices { x: Vec<f64>, y: Vec<f64> }`, `Columns(HashMap<String, Vec<f64>>)`
- [ ] Implement `From<(Vec<f64>, Vec<f64>)>` for `DataSource`
- [ ] Implement `From<(&[f64], &[f64])>` for `DataSource`
- [ ] Implement data resolution: `DataSource::resolve(aes: &Aes) -> ResolvedData`
- [ ] Write test: resolve x="a", y="b" from column map

#### Figure struct

- [ ] Define `Figure` struct: data, marks (Vec<Box<dyn Geom>>), x_scale, y_scale, theme, title, size (width, height), margins
- [ ] Implement `Figure::new()` constructor
- [ ] Implement `Figure::data()` setter accepting DataSource
- [ ] Implement `Figure::add()` for adding marks
- [ ] Implement `Figure::title()`, `Figure::size()`, `Figure::theme()` setters
- [ ] Implement `Figure::scale_x()`, `Figure::scale_y()` setters
- [ ] Implement `Figure::build_scene() -> Scene` — orchestrates scale fitting, axis generation, mark rendering, title rendering
  - [ ] Step 1: Determine data extents from all marks
  - [ ] Step 2: Create/fit scales (auto if not user-provided)
  - [ ] Step 3: Calculate layout rectangles (margins, axis space, plot area)
  - [ ] Step 4: Render axes to scene nodes
  - [ ] Step 5: Render marks to scene nodes (clipped to plot area)
  - [ ] Step 6: Render title to scene node
  - [ ] Step 7: Render legend if needed
  - [ ] Step 8: Assemble into final Scene
- [ ] Implement `Figure::save(path: &str) -> Result<()>` dispatching by extension (.png, .svg)
- [ ] Implement `Figure::render_png(dpi: u32) -> Result<Vec<u8>>`
- [ ] Implement `Figure::render_svg() -> Result<String>`
- [ ] Write integration test: Figure with line mark → save PNG → verify file exists and is valid PNG
- [ ] Write integration test: Figure with line mark → save SVG → verify valid SVG document

#### plot!() macro

- [ ] Implement `plot!()` proc macro in starsight-derive or declarative macro in starsight-figure
  - [ ] Syntax: `plot!(x_data, y_data)` → creates Figure with Line mark
  - [ ] Syntax: `plot!(single_vec)` → creates Figure with Histogram mark
  - [ ] Return type: `Figure` (allowing `.save()`, `.show()`, chaining)
- [ ] Write test: `plot!([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]).render_png(150)` returns non-empty Vec
- [ ] Write doc test in top-level lib.rs demonstrating zero-config usage

### Testing and documentation for 0.1.0

- [ ] Set up insta snapshot testing infrastructure
  - [ ] Configure snapshot directory at `tests/snapshots/`
  - [ ] Create `tests/snapshot_tests.rs` runner
  - [ ] Generate reference snapshots for line and scatter charts at 400x300
- [ ] Write `examples/quickstart.rs` — minimal line chart saved to PNG
- [ ] Write `examples/scatter.rs` — scatter plot with color
- [ ] Write top-level `lib.rs` doc comment with complete quickstart example
- [ ] Write README.md quickstart section with code example
- [ ] Verify `cargo doc --workspace --no-deps` builds with no warnings
- [ ] Verify `cargo test --workspace` passes all tests
- [ ] Verify `cargo clippy --workspace -- -D warnings` passes
- [ ] Tag and publish `0.1.0` to crates.io

---

## 0.2.0 — Core chart types (Phase 1, part 1)

> Exit criteria: Bar, Area, Histogram, and Heatmap render correctly with snapshot tests.

### starsight-marks: Bar mark

- [ ] Implement `BarMark` struct with aes (x, y, fill), orientation (vertical/horizontal), width, gap
- [ ] Implement `BarMark::render()` using categorical X scale and linear Y scale
- [ ] Handle grouped bars (dodge position) — multiple series side by side
- [ ] Handle stacked bars (stack position) — series stacked vertically
- [ ] Write snapshot test: simple vertical bar chart (5 categories)
- [ ] Write snapshot test: grouped bar chart (3 categories, 2 series)
- [ ] Write snapshot test: stacked bar chart (4 categories, 3 series)
- [ ] Write snapshot test: horizontal bar chart

### starsight-marks: Area mark

- [ ] Implement `AreaMark` struct with aes (x, y, fill), stacked option, alpha
- [ ] Implement `AreaMark::render()` filling between baseline and data line
- [ ] Handle stacked areas (cumulative y values)
- [ ] Write snapshot test: single area chart
- [ ] Write snapshot test: stacked area chart (3 series)

### starsight-marks: Histogram

- [ ] Implement `Stat::Bin` — bin continuous data into count/frequency buckets
  - [ ] Auto bin count via Sturges' rule, Scott's rule, or Freedman-Diaconis
  - [ ] User-configurable bin count and bin edges
  - [ ] Normalization options: count, frequency, density, probability
- [ ] Implement `HistogramMark` as BarMark + Stat::Bin composition
- [ ] Write snapshot test: histogram of 1000 normal samples (30 bins)
- [ ] Write snapshot test: histogram with density normalization

### starsight-marks: Heatmap

- [ ] Implement `HeatmapMark` accepting 2D data (matrix or x/y/value triples)
- [ ] Map cell values to colors via ColorScale
- [ ] Render as grid of filled rectangles
- [ ] Add optional cell annotation (text values in each cell)
- [ ] Write snapshot test: 10x10 heatmap with sequential colormap
- [ ] Write snapshot test: annotated heatmap with values displayed

### starsight-core: Position adjustments

- [ ] Define `Position` enum: Identity, Dodge, Stack, Fill, Jitter, Nudge
- [ ] Implement `Position::Dodge` — offset grouped bars by series index
- [ ] Implement `Position::Stack` — cumulate y values across series
- [ ] Implement `Position::Jitter` — add small random offset to prevent overplotting
- [ ] Write unit tests for each position adjustment with known outputs

### Testing for 0.2.0

- [ ] Snapshot tests for all new chart types (bar, area, histogram, heatmap)
- [ ] Doc examples for each new Geom constructor
- [ ] Update gallery generator to include new types
- [ ] Update CHANGELOG.md

---

## 0.3.0 — Core chart types (Phase 1, part 2)

> Exit criteria: BoxPlot, Violin, KDE, ErrorBar, Pie, Contour, and Candlestick render correctly.

### starsight-marks: Statistical marks (requires `stats` feature)

#### BoxPlot

- [ ] Implement `Stat::Boxplot` — compute Q1, median, Q3, whiskers, outliers from grouped data
- [ ] Implement `BoxPlotMark` rendering box, median line, whiskers, and outlier points
- [ ] Handle orientation (vertical/horizontal)
- [ ] Handle notched box plots (confidence interval on median)
- [ ] Write snapshot test: box plot of 3 groups

#### Violin

- [ ] Implement `Stat::KDE` — kernel density estimation (Gaussian kernel, Silverman bandwidth)
  - [ ] Accept bandwidth parameter (auto, scalar, or per-group)
  - [ ] Support reflection at boundaries
- [ ] Implement `ViolinMark` rendering mirrored KDE curves per group
- [ ] Handle inner marks: box, quartile lines, stick, or none
- [ ] Write snapshot test: violin plot of 3 groups with inner box

#### KDE density plot

- [ ] Implement `DensityMark` rendering filled KDE curve
- [ ] Handle grouped/stacked density plots with alpha transparency
- [ ] Write snapshot test: overlapping density curves for 2 groups

#### ErrorBar

- [ ] Implement `ErrorBarMark` with x/y center, x_err/y_err extents
- [ ] Handle symmetric and asymmetric error bars
- [ ] Handle cap width
- [ ] Write snapshot test: scatter with symmetric Y error bars

#### Pie / Donut

- [ ] Implement `ArcMark` for pie and donut charts
- [ ] Compute angular extent from value proportions
- [ ] Handle donut hole (inner radius > 0)
- [ ] Handle label placement (inside arc, outside with leader lines)
- [ ] Handle explosion (offset individual slices)
- [ ] Write snapshot test: pie chart with 5 categories
- [ ] Write snapshot test: donut chart with labels

#### Contour

- [ ] Integrate `contour` crate for isoline computation
- [ ] Implement `ContourMark` for line contours
- [ ] Implement `FilledContourMark` for filled contours (isobands)
- [ ] Map contour levels to ColorScale
- [ ] Write snapshot test: filled contour plot of 2D Gaussian

#### Candlestick

- [ ] Implement `CandlestickMark` with open/high/low/close aesthetic mappings
- [ ] Color by up/down (close > open vs close < open)
- [ ] Handle wick/shadow lines and body rectangles
- [ ] Write snapshot test: 30-day candlestick chart

### starsight-figure: Polars DataFrame integration (requires `polars` feature)

- [ ] Implement `From<&DataFrame>` for `DataSource`
- [ ] Implement column name resolution: string name → Series → f64/str extraction
- [ ] Handle column types: f64, i64, String, Date, DateTime
- [ ] Handle null values (skip with warning via log crate)
- [ ] Handle categorical columns for grouping and color mapping
- [ ] Implement `plot!(&df, x = "col_a", y = "col_b")` syntax
- [ ] Implement `plot!(&df, x = "col_a", y = "col_b", color = "col_c")` syntax
- [ ] Write integration test: scatter from Polars DataFrame
- [ ] Write integration test: box plot grouped by string column

### starsight-figure: Auto-inference

- [ ] Implement chart type auto-detection from data shape:
  - [ ] Two numeric columns → scatter
  - [ ] One numeric column → histogram
  - [ ] 2D matrix → heatmap
  - [ ] One categorical + one numeric → bar
  - [ ] Sorted x + y → line
- [ ] Implement automatic axis label from column names
- [ ] Implement automatic legend generation from color column unique values
- [ ] Implement automatic color cycle from theme
- [ ] Write test: `plot!(&df, x = "a", y = "b")` with sorted "a" produces line chart

### Testing for 0.3.0

- [ ] Snapshot tests for all 7 new chart types
- [ ] Property tests for Stat::KDE (integral ≈ 1.0, non-negative)
- [ ] Property tests for Stat::Boxplot (Q1 <= median <= Q3)
- [ ] Doc examples for all new marks
- [ ] Write `examples/statistical.rs` demonstrating violin, box, KDE
- [ ] Write `examples/polars_integration.rs`
- [ ] Update gallery generator
- [ ] Update CHANGELOG.md

---

## 0.4.0 — Layout and composition (Phase 2)

> Exit criteria: Faceted plots, PairPlot, and multi-chart layouts render correctly.

### starsight-layout: Grid layout (Layer 4)

- [ ] Implement `GridLayout` struct: rows, cols, cell sizes (fixed/proportional), gaps
- [ ] Implement cell size negotiation: auto-size based on content, respect min/max constraints
- [ ] Implement `GridLayout::place(row, col, figure)` for placing figures in cells
- [ ] Implement `GridLayout::place_span(row, col, rowspan, colspan, figure)` for spanning cells
- [ ] Implement `GridLayout::render() -> Scene` composing all cells into a single scene
- [ ] Write snapshot test: 2x2 grid of scatter plots

### starsight-layout: Faceting

- [ ] Implement `FacetWrap` — wrap N subplots from a grouping variable into rows with configurable `ncol`
  - [ ] Compute unique values of facet column
  - [ ] Create one subplot per unique value with filtered data
  - [ ] Shared scales (fixed) or independent scales (free_x, free_y, free)
  - [ ] Render facet labels (strip text) above each subplot
- [ ] Implement `FacetGrid` — two-variable faceting (rows ~ var1, cols ~ var2)
  - [ ] Row variable labels on right margin
  - [ ] Column variable labels on top margin
  - [ ] Support free scales per row/column
- [ ] Write snapshot test: FacetWrap with 4 panels, 2 columns
- [ ] Write snapshot test: FacetGrid with 2x3 panels
- [ ] Write snapshot test: FacetWrap with free Y scales

### starsight-layout: Legend

- [ ] Implement `Legend` struct: entries (label, color, shape), position, orientation
- [ ] Auto-generate legend from color/shape aesthetic mappings
- [ ] Position options: TopLeft, TopRight, BottomLeft, BottomRight, OutsideRight, OutsideBottom
- [ ] Render: colored swatches + text labels
- [ ] Write snapshot test: scatter with 3-entry legend

### starsight-layout: Colorbar

- [ ] Implement `Colorbar` struct: color scale, label, orientation (vertical/horizontal), ticks
- [ ] Auto-generate from continuous color mapping
- [ ] Render as gradient rectangle with tick labels
- [ ] Write snapshot test: heatmap with vertical colorbar

### starsight-layout: Twin axes

- [ ] Implement `TwinAxes` — second Y axis (right side) with independent scale
- [ ] Maintain axis-to-mark association (left marks use left scale, right marks use right scale)
- [ ] Render both Y axes with labels
- [ ] Write snapshot test: line chart with two Y axes

### starsight-figure: Convenience types

#### PairPlot

- [ ] Implement `PairPlot` struct: columns, hue (grouping), diagonal kind, upper/lower triangle kind
- [ ] Diagonal: Histogram, KDE, or None
- [ ] Upper/lower: Scatter, Regression, KDE2D, or None
- [ ] Compose as GridLayout with shared axes
- [ ] Write snapshot test: 4-variable PairPlot with hue grouping

#### JointPlot

- [ ] Implement `JointPlot` struct: x, y, kind (Scatter/Hex/KDE2D), marginal kind (Histogram/KDE)
- [ ] Compose as GridLayout with main plot + two marginal axes
- [ ] Write snapshot test: JointPlot with scatter center and KDE margins

### Testing for 0.4.0

- [ ] Snapshot tests for grid, faceting, legend, colorbar, twin axes, PairPlot, JointPlot
- [ ] Write `examples/faceting.rs`
- [ ] Update CHANGELOG.md

---

## 0.5.0 — Scale infrastructure (Phase 3)

> Exit criteria: Log, datetime, and all specialized scales work with auto-ticking.

### starsight-core: Additional scales

#### Log scale

- [ ] Implement `LogScale` with base (10, 2, e), domain, range, clamp
- [ ] Implement `LogScale::ticks()` returning powers of base (1, 10, 100, ...)
- [ ] Implement minor ticks (2, 3, ..., 9 between major ticks)
- [ ] Handle zero/negative domain values gracefully (return error or clamp)
- [ ] Write test: LogScale(1, 1000) ticks → [1, 10, 100, 1000]
- [ ] Write snapshot test: log-scale Y axis

#### Symlog scale

- [ ] Implement `SymlogScale` — symmetric log scale handling zero and negative values
- [ ] Parameter: `linthresh` (linear threshold near zero)
- [ ] Write test: SymlogScale maps 0 to center, positive and negative symmetrically

#### Power scale

- [ ] Implement `PowerScale` with exponent parameter (sqrt = 0.5, square = 2)
- [ ] Write test: sqrt scale transforms 4 → 2 (normalized)

#### DateTime scale

- [ ] Implement `DateTimeScale` accepting chrono::NaiveDateTime or similar
- [ ] Implement auto-granularity tick generation: year, month, week, day, hour, minute, second
- [ ] Implement date formatters per granularity (e.g., "%Y" for year ticks, "%b %d" for day ticks)
- [ ] Write test: DateTimeScale over 1 year produces monthly ticks
- [ ] Write test: DateTimeScale over 1 hour produces 10-minute ticks
- [ ] Write snapshot test: line chart with datetime X axis

#### Band scale (for bar charts)

- [ ] Implement `BandScale` with categories, padding_inner, padding_outer
- [ ] Method: `bandwidth()` returns computed bar width
- [ ] Write test: 3 categories in 300px range produces 3 centered bands

### starsight-core: TickLocator and TickFormatter traits

- [ ] Implement `FixedTickLocator` — user-specified tick positions
- [ ] Implement `MultipleTickLocator` — ticks at multiples of a base (e.g., every 0.25)
- [ ] Implement `LogTickLocator` — ticks at powers of base
- [ ] Implement `DateTimeTickLocator` — auto-granularity date ticks
- [ ] Implement `PercentFormatter` — format as "50%"
- [ ] Implement `SIFormatter` — format with SI prefixes (k, M, G)
- [ ] Implement `DateTimeFormatter` — format dates at appropriate granularity
- [ ] Implement `CurrencyFormatter` — format with currency symbol
- [ ] Write unit tests for each formatter

### starsight-core: Secondary and broken axes

- [ ] Implement secondary X axis (top) and secondary Y axis (right) with independent scales
- [ ] Implement axis inversion (reversed scale direction)
- [ ] Write snapshot test: inverted Y axis (0 at top)

### Testing for 0.5.0

- [ ] Snapshot tests for every scale type with every axis position
- [ ] Property tests for log scale round-trip (within tolerance)
- [ ] Property tests for datetime scale tick spacing (monotonically increasing)
- [ ] Update CHANGELOG.md

---

## 0.6.0 — GPU and interactivity (Phase 4)

> Exit criteria: 100K-point scatter renders at 60fps with hover tooltips in a native window.

### starsight-gpu: wgpu backend

- [ ] Implement `WgpuBackend` struct managing wgpu Device, Queue, Surface
- [ ] Implement `DrawBackend` for `WgpuBackend`
  - [ ] `draw_path`: tessellate via lyon → upload vertex buffer → render with 2D shader pipeline
  - [ ] `draw_text`: rasterize glyphs to texture atlas → render as textured quads
  - [ ] `fill_rect`: render as two triangles with solid color
- [ ] Create 2D render pipeline (vertex + fragment shaders) for lines, fills, points
- [ ] Create point instancing pipeline for scatter plots with >10K points
- [ ] Implement GPU texture atlas for text glyph caching
- [ ] Implement GPU readback for `save_png()` (copy texture to staging buffer → read pixels)
- [ ] Write benchmark: render 100K points, measure frame time (target: <16ms)
- [ ] Write integration test: render scatter, readback pixels, verify content

### starsight-gpu: Window management

- [ ] Implement `Window` struct wrapping winit EventLoop and Window
- [ ] Implement event loop: process resize, close, key events
- [ ] Implement `Figure::show()` for GPU backend: open window, render figure, block until close
- [ ] Handle HiDPI scaling (winit scale_factor)
- [ ] Handle window resize (recreate swap chain)
- [ ] Write manual test: open window with scatter plot, close with Escape key

### starsight-interact: Interactivity (Layer 6)

#### Hover

- [ ] Implement point-under-cursor detection using spatial index (simple grid or k-d tree)
- [ ] Implement tooltip rendering: background rectangle + text with data values
- [ ] Format tooltip text from aesthetic values (x, y, color, size column values)
- [ ] Write manual test: hover over points, tooltips appear

#### Zoom and pan

- [ ] Implement box zoom: click-drag to define rectangle, update scale domains
- [ ] Implement scroll wheel zoom: centered on cursor, proportional scale change
- [ ] Implement pan: click-drag (middle button or modifier key) translates view
- [ ] Implement double-click reset to original scale domains
- [ ] Write manual test: zoom into cluster, pan around, reset

#### Selection

- [ ] Implement point selection: click to select nearest point, callback with data index
- [ ] Implement box selection: click-drag rectangle, callback with indices of enclosed points
- [ ] Implement lasso selection: freehand polygon, callback with enclosed point indices
- [ ] Define `SelectionCallback` trait for user-defined responses
- [ ] Write manual test: lasso select, verify callback fires with correct indices

### starsight-interact: Streaming data

- [ ] Implement `Figure::streaming(opts: StreamOpts) -> StreamingFigure`
- [ ] Implement `StreamingFigure::append(row: DataRow)` — add data point, shift window if needed
- [ ] Implement rolling window: keep last N seconds/points, auto-scroll X axis
- [ ] Implement efficient GPU buffer updates (ring buffer with partial upload)
- [ ] Write example: `examples/streaming.rs` with simulated real-time data

### Testing for 0.6.0

- [ ] GPU rendering snapshot tests (via readback, compared against tiny-skia reference)
- [ ] Performance benchmark: 1K, 10K, 100K, 1M points — frame time table
- [ ] Write `examples/interactive.rs`
- [ ] Update CHANGELOG.md

---

## 0.7.0 — 3D visualization (Phase 5)

> Exit criteria: Surface plot with colormapping renders in both wgpu and tiny-skia backends.

### starsight-gpu: 3D pipeline

- [ ] Implement 3D render pipeline with perspective projection
- [ ] Implement depth buffer
- [ ] Implement 3D vertex shader with model-view-projection matrices
- [ ] Implement basic Phong/Blinn lighting for surfaces

### starsight-gpu: Camera

- [ ] Implement `Camera` struct: position, target, up vector, FOV, near/far planes
- [ ] Implement `Camera::orbit(azimuth, elevation, distance)` constructor
- [ ] Implement `Camera::fly(eye, center, up)` constructor
- [ ] Implement interactive orbit: mouse drag rotates camera around target
- [ ] Implement interactive zoom: scroll wheel changes distance
- [ ] Write unit test: orbit camera at (45, 30, 5) produces correct view matrix

### starsight-marks: 3D marks

#### Scatter3D

- [ ] Implement `Scatter3DMark` with aes (x, y, z, color, size)
- [ ] Render as instanced spheres (GPU) or circles with depth sorting (CPU)
- [ ] Write snapshot test: 3D scatter with 100 points

#### Surface3D

- [ ] Implement `Surface3DMark` accepting meshgrid data (X, Y, Z 2D arrays)
- [ ] Triangulate grid into mesh
- [ ] Map Z values to face colors via ColorScale
- [ ] Handle wireframe overlay option
- [ ] Write snapshot test: sin(x)*cos(y) surface

#### Wireframe3D

- [ ] Implement `Wireframe3DMark` rendering only mesh edges
- [ ] Write snapshot test: wireframe of paraboloid

#### Isosurface

- [ ] Implement marching cubes algorithm for 3D scalar fields
- [ ] Implement `IsosurfaceMark` extracting surface at threshold value from 3D ndarray
- [ ] Map vertex values to colors
- [ ] Write snapshot test: isosurface of sphere function

#### VolumeRender

- [ ] Implement ray-marching volume renderer in wgpu fragment shader
- [ ] Implement `VolumeRenderMark` with transfer function (value → color + opacity)
- [ ] Implement `TransferFn` presets: ramp, threshold, Gaussian
- [ ] Write snapshot test: volume rendering of 3D Gaussian

### starsight-core: 3D axis rendering

- [ ] Implement 3D axis box (three axes at right angles)
- [ ] Implement 3D tick marks projected to screen space
- [ ] Implement 3D tick labels always facing camera (billboard text)
- [ ] Implement 3D grid planes (XY, XZ, YZ) as wireframe
- [ ] Write snapshot test: 3D axes with labels "X", "Y", "Z"

### starsight-core: CPU 3D fallback (tiny-skia)

- [ ] Implement simple 3D→2D projection for tiny-skia backend (no shading, painter's algorithm)
- [ ] Implement depth-sorted rendering of 3D scatter points
- [ ] Implement wireframe rendering of 3D surfaces via projected line segments
- [ ] Write test: same surface plot produces similar output on wgpu (readback) and tiny-skia

### Testing for 0.7.0

- [ ] Snapshot tests for all 5 3D chart types
- [ ] Write `examples/surface3d.rs`
- [ ] Write `examples/volume.rs`
- [ ] Performance benchmark: surface with 100x100, 500x500 grid sizes
- [ ] Update CHANGELOG.md

---

## 0.8.0 — Terminal backend (Phase 6)

> Exit criteria: Charts render inline in Kitty, WezTerm, iTerm2, and fallback terminals.

### starsight-export: Terminal rendering

#### Protocol detection

- [ ] Implement `TerminalCapability` enum: Kitty, Sixel, ITerm2, HalfBlock, Braille, Ascii
- [ ] Implement `detect_terminal() -> TerminalCapability` using $TERM, $TERM_PROGRAM, and escape sequence queries
- [ ] Handle tmux/screen passthrough
- [ ] Write test: mock TERM_PROGRAM=WezTerm → returns Sixel (or Kitty)

#### Kitty graphics protocol output

- [ ] Implement Kitty image transmission (base64 encoded, chunked for large images)
- [ ] Handle Kitty Unicode placeholders for ratatui cell integration
- [ ] Write manual test: render line chart inline in Kitty terminal

#### Sixel output

- [ ] Implement Sixel encoding from RGBA pixel buffer (via icy_sixel or custom encoder)
- [ ] Handle color quantization (256 color palette)
- [ ] Write manual test: render chart inline in WezTerm with Sixel

#### iTerm2 output

- [ ] Implement iTerm2 inline image protocol (ESC ]1337;File=...)
- [ ] Write manual test: render chart inline in iTerm2

#### Fallback: half-block and Braille

- [ ] Implement half-block (▀▄█) character rendering for moderate resolution
- [ ] Implement Braille dot (⠁⠂⠃...) character rendering for line charts
- [ ] Write snapshot test: Braille line chart output (text comparison)

#### Terminal integration API

- [ ] Implement `Figure::terminal() -> TerminalFigure`
- [ ] Implement `TerminalFigure::show()` — detect protocol, render via tiny-skia, output to stdout
- [ ] Implement `TerminalFigure::protocol(Protocol::Kitty)` — force specific protocol
- [ ] Implement `Figure::print_terminal() -> Result<()>` convenience method

#### ratatui widget adapter

- [ ] Implement `StarsightWidget` implementing `ratatui::Widget`
- [ ] Accept `Figure` and render to the allocated terminal area
- [ ] Handle resize: re-render at new cell dimensions
- [ ] Integrate with ratatui-image for protocol rendering within ratatui layouts
- [ ] Write example: ratatui app with starsight chart widget

### Testing for 0.8.0

- [ ] Manual testing matrix: Kitty, WezTerm, iTerm2, Alacritty (half-block fallback), xterm (Braille fallback)
- [ ] Snapshot test for Braille output (deterministic text)
- [ ] Write `examples/terminal.rs`
- [ ] Update CHANGELOG.md

---

## 0.9.0 — Remaining chart types and marks

> Exit criteria: All 66 chart types in the taxonomy have implementations and snapshot tests.

### starsight-marks: Additional 2D marks

#### Stem and Step

- [ ] Implement `StemMark` — vertical lines from baseline to data points with markers
- [ ] Implement `StepMark` — step function connecting points (pre, mid, post step styles)
- [ ] Write snapshot tests for both

#### Lollipop and Dot

- [ ] Implement `LollipopMark` — thin stem + circle marker (stem + point composition)
- [ ] Implement `DotMark` — Cleveland dot plot (horizontal, categorical Y)
- [ ] Write snapshot tests for both

#### Strip and Swarm

- [ ] Implement `StripMark` — jittered points along a categorical axis
- [ ] Implement `SwarmMark` — non-overlapping point packing (beeswarm algorithm)
- [ ] Write snapshot tests for both

#### Rug

- [ ] Implement `RugMark` — small tick marks along axis margin showing individual data points
- [ ] Support rug on X, Y, or both axes
- [ ] Write snapshot test: scatter with rug marks

#### Ridge / Joy plot

- [ ] Implement `RidgeMark` — vertically stacked overlapping KDE curves
- [ ] Handle overlap amount parameter
- [ ] Write snapshot test: 8-group ridge plot

#### RainCloud

- [ ] Implement `RainCloudMark` — half-violin + box + jittered points composition
- [ ] Write snapshot test: 3-group rain cloud plot

#### ECDF

- [ ] Implement `Stat::ECDF` — empirical cumulative distribution function
- [ ] Implement `ECDFMark` rendering step function from 0 to 1
- [ ] Handle complementary ECDF option
- [ ] Write snapshot test: ECDF of 500 samples

#### QQ plot

- [ ] Implement `QQMark` — quantile-quantile plot against theoretical distribution
- [ ] Handle Normal, Uniform, and custom reference distributions
- [ ] Add diagonal reference line
- [ ] Write snapshot test: QQ plot of normal samples

#### Regression

- [ ] Implement `Stat::Regression` — linear, polynomial, LOESS fit
  - [ ] Linear: OLS closed-form solution
  - [ ] Polynomial: degree parameter, solve via least squares
  - [ ] LOESS: local weighted regression with configurable span
- [ ] Implement `RegressionMark` rendering fit line + confidence band
- [ ] Write snapshot test: scatter with linear regression and 95% CI band

#### Hexbin

- [ ] Implement `Stat::Hexbin` — hexagonal binning of 2D points
- [ ] Implement `HexbinMark` rendering colored hexagons
- [ ] Write snapshot test: hexbin of 10K bivariate normal points

### starsight-marks: Network and hierarchical marks

#### Sankey

- [ ] Implement Sankey layout algorithm (iterative relaxation)
- [ ] Implement `SankeyMark` rendering nodes as rectangles, flows as bezier curves
- [ ] Write snapshot test: 3-level Sankey diagram

#### Force-directed graph

- [ ] Implement force simulation (Barnes-Hut or simple O(n²))
- [ ] Implement `ForceGraphMark` rendering nodes as circles, edges as lines
- [ ] Write snapshot test: 20-node graph

#### Treemap

- [ ] Implement squarified treemap layout algorithm
- [ ] Implement `TreemapMark` rendering nested rectangles with labels
- [ ] Write snapshot test: hierarchical data with 3 levels

#### Sunburst

- [ ] Implement sunburst layout (nested arc segments)
- [ ] Implement `SunburstMark` rendering concentric arcs
- [ ] Write snapshot test: 3-level sunburst

### starsight-marks: Financial and specialized

#### Waterfall

- [ ] Implement `WaterfallMark` — cumulative bar chart showing positive/negative contributions
- [ ] Color-code increase, decrease, and total bars
- [ ] Write snapshot test: 8-step waterfall chart

#### Funnel

- [ ] Implement `FunnelMark` — tapered horizontal bars showing conversion stages
- [ ] Write snapshot test: 5-stage funnel

#### Gantt

- [ ] Implement `GanttMark` — horizontal bars with start/end dates per task
- [ ] Use DateTime X scale
- [ ] Write snapshot test: 6-task project Gantt chart

#### Radar / Spider

- [ ] Implement `RadarMark` rendering data as polygon on polar axes
- [ ] Handle multiple overlaid series
- [ ] Write snapshot test: radar chart with 3 series on 6 axes

#### Parallel coordinates

- [ ] Implement `ParallelCoordinatesMark` — one vertical axis per variable, lines connecting values
- [ ] Handle color mapping for line grouping
- [ ] Write snapshot test: 5-variable parallel coordinates

#### Calendar heatmap

- [ ] Implement `CalendarHeatmapMark` — grid of days in month/week layout colored by value
- [ ] Handle year, month, day layout
- [ ] Write snapshot test: 1-year calendar heatmap

#### Sparkline

- [ ] Implement `SparklineMark` — minimal inline line chart with no axes
- [ ] Write snapshot test: 50-point sparkline

### starsight-marks: Geographic marks (requires `geo` feature)

#### Choropleth

- [ ] Implement `ChoroplethMark` accepting GeoJSON polygons + value column
- [ ] Implement polygon rendering (fill + stroke)
- [ ] Map values to ColorScale
- [ ] Handle map projections via proj crate (Mercator, Lambert, Albers, etc.)
- [ ] Write snapshot test: US states choropleth

#### ScatterMap and BubbleMap

- [ ] Implement `ScatterMapMark` placing points at lat/lon coordinates
- [ ] Implement `BubbleMapMark` with size-mapped points
- [ ] Write snapshot test: world scatter map

### Testing for 0.9.0

- [ ] Snapshot tests for every chart type in taxonomy (66 total)
- [ ] Update gallery generator to produce all chart types
- [ ] Write doc examples for all new marks
- [ ] Update CHANGELOG.md

---

## 0.10.0 — Export, animation, and WASM (Phase 7, part 1)

> Exit criteria: PDF export, HTML interactive export, and WASM compilation work.

### starsight-export: PDF backend

- [ ] Implement `PdfBackend` using krilla
- [ ] Implement `DrawBackend` for `PdfBackend`
  - [ ] `draw_path`: emit PDF path commands
  - [ ] `draw_text`: embed fonts and emit text operators
  - [ ] `fill_rect`: emit rectangle fill
- [ ] Handle font embedding (subset TrueType/OpenType fonts)
- [ ] Handle vector output (no rasterization)
- [ ] Handle multi-page output for figure collections
- [ ] Write integration test: save figure as PDF, verify PDF structure with a parser

### starsight-export: HTML interactive export

- [ ] Design minimal JS interactivity shim (hover tooltips, zoom/pan) — authored in starsight, not a bundled library
- [ ] Implement `HtmlExporter` generating self-contained HTML file
  - [ ] Embed SVG chart in HTML document
  - [ ] Embed data as JSON for tooltip data lookup
  - [ ] Embed interactivity shim as inline `<script>`
- [ ] Handle chart sizing (responsive or fixed)
- [ ] Write integration test: save figure as HTML, open in headless browser, verify chart renders

### starsight-export: Animation

- [ ] Implement `Animation` struct: frames (Vec<Figure>), duration_per_frame, transition
- [ ] Implement `Animation::record_gif(path: &str) -> Result<()>` using image crate's GIF encoder
- [ ] Implement frame interpolation for smooth transitions between states
- [ ] Implement `Transition` enum: None, Linear, EaseInOut
- [ ] Write test: generate 10-frame animation, save GIF, verify frame count

### starsight-gpu: WASM/WebGPU target

- [ ] Verify starsight-core compiles to `wasm32-unknown-unknown`
- [ ] Verify tiny-skia backend works in WASM (it should — pure Rust)
- [ ] Implement `WasmBackend` using web-sys Canvas2D API as alternative to tiny-skia
- [ ] Implement `WgpuBackend` initialization for WebGPU in browser
- [ ] Implement `Figure::show()` for WASM target: render to `<canvas>` element
- [ ] Create example WASM project with trunk build configuration
- [ ] Write `examples/wasm/` directory with HTML + Rust entry point
- [ ] Verify interactivity (hover, zoom) works in browser via web-sys events

### Testing for 0.10.0

- [ ] PDF output validation tests
- [ ] HTML export tests (parse HTML, check SVG and script presence)
- [ ] GIF animation frame count test
- [ ] WASM compilation test in CI (cargo build --target wasm32-unknown-unknown)
- [ ] Update CHANGELOG.md

---

## 0.11.0 — Recipe system, ndarray/Arrow support, and API polish (Phase 7, part 2)

> Exit criteria: Custom chart types via recipes, full data source coverage, clean API.

### starsight-derive: Recipe proc macro

- [ ] Implement `#[starsight::recipe]` attribute macro
- [ ] Transform annotated function into a registered chart type callable from Figure builder
- [ ] Generate documentation for recipe parameters
- [ ] Write example recipe: `volcano_plot` from STARSIGHT_ARCHITECTURE.md
- [ ] Write example recipe: `manhattan_plot` for genomics
- [ ] Write test: custom recipe renders correctly

### starsight-figure: ndarray support (requires `ndarray` feature)

- [ ] Implement `From<&Array1<f64>>` for DataSource (1D series)
- [ ] Implement `From<&Array2<f64>>` for DataSource (2D matrix → heatmap/surface)
- [ ] Implement `From<(&Array1<f64>, &Array1<f64>)>` for DataSource (x, y pair)
- [ ] Write integration test: plot from ndarray
- [ ] Write integration test: heatmap from 2D ndarray

### starsight-figure: Arrow RecordBatch support (requires `arrow` feature)

- [ ] Implement `From<&RecordBatch>` for DataSource
- [ ] Implement column name resolution from Arrow schema
- [ ] Handle Arrow data types: Float64, Int64, Utf8, Date32, Timestamp
- [ ] Write integration test: scatter from Arrow RecordBatch

### API consistency audit

- [ ] Review all public types for naming consistency (no abbreviations in public API)
- [ ] Review all builders for consistent method naming (`.color()` not `.set_color()` or `.with_color()`)
- [ ] Review all `Into<>` implementations for consistent data acceptance patterns
- [ ] Ensure all option structs implement `Default`
- [ ] Ensure all public types implement `Debug`, `Clone` where appropriate
- [ ] Ensure all error messages are descriptive and actionable
- [ ] Run `cargo semver-checks` against 0.10.0 to identify breaking changes
- [ ] Document all intentional breaking changes in CHANGELOG.md

### starsight-figure: Additional convenience APIs

- [ ] Implement `ClusterMap` convenience type (heatmap + hierarchical clustering + dendrograms)
- [ ] Implement `MosaicLayout` — named layout positions for complex dashboard arrangements
- [ ] Implement `Dashboard` builder — compose multiple figures with titles into a single exportable layout
- [ ] Write snapshot tests for each

### Testing for 0.11.0

- [ ] Recipe proc macro compilation tests
- [ ] ndarray integration tests
- [ ] Arrow integration tests
- [ ] Dashboard snapshot tests
- [ ] Update CHANGELOG.md

---

## 0.12.0 — Documentation, examples, and gallery (Phase 7, part 3)

> Exit criteria: Every public API item has docs, every chart type has a gallery entry, README is comprehensive.

### Documentation

- [ ] Write comprehensive top-level `lib.rs` doc comment (1000+ words) with:
  - [ ] Project overview and motivation
  - [ ] Quickstart example (zero-config line chart)
  - [ ] Grammar of graphics example
  - [ ] Feature flag reference table
  - [ ] Backend selection guide
  - [ ] Link to gallery
- [ ] Audit every public type for doc comment completeness
  - [ ] `Figure` — full builder API documented with examples
  - [ ] Every `Geom` variant — constructor, options, and visual example reference
  - [ ] Every `Scale` type — domain/range semantics and example
  - [ ] `Theme` — preset descriptions and customization guide
  - [ ] `DrawBackend` — implementor guide for custom backends
  - [ ] `DataSource` — accepted types and conversion guide
- [ ] Verify all doc examples compile and run (`cargo test --doc`)
- [ ] Write module-level doc comments for every `mod.rs`

### Examples

- [ ] Write `examples/quickstart.rs` — minimal one-liner (if not already done)
- [ ] Write `examples/scatter.rs` — scatter with color and size mappings
- [ ] Write `examples/statistical.rs` — violin, box, KDE, regression
- [ ] Write `examples/surface3d.rs` — 3D surface with colormapping
- [ ] Write `examples/volume.rs` — volume rendering
- [ ] Write `examples/terminal.rs` — terminal output with protocol detection
- [ ] Write `examples/interactive.rs` — GPU window with hover and zoom
- [ ] Write `examples/polars_integration.rs` — DataFrame-driven charts
- [ ] Write `examples/streaming.rs` — real-time data streaming
- [ ] Write `examples/faceting.rs` — FacetWrap and FacetGrid
- [ ] Write `examples/custom_theme.rs` — theme customization and chromata integration
- [ ] Write `examples/recipe.rs` — custom chart type via recipe macro
- [ ] Write `examples/geographic.rs` — choropleth map
- [ ] Write `examples/network.rs` — force-directed graph and Sankey
- [ ] Write `examples/dashboard.rs` — multi-chart dashboard layout
- [ ] Write `examples/animation.rs` — animated GIF generation
- [ ] Write `examples/pdf_export.rs` — publication-quality PDF output
- [ ] Write `examples/wasm/` — browser-based chart (trunk project)

### Gallery generator

- [ ] Implement `xtask gallery` command generating PNG for every chart type
- [ ] Output gallery images to `docs/gallery/` directory
- [ ] Generate `docs/GALLERY.md` with grid of thumbnails and chart type names
- [ ] Configure GitHub Actions to regenerate gallery on release
- [ ] Verify all 66+ chart types render without error

### README and project documentation

- [ ] Expand README.md with:
  - [ ] Feature comparison table (starsight vs plotters vs plotly-rs vs charming)
  - [ ] Gallery thumbnail grid (linking to full-size images)
  - [ ] Installation instructions for each feature preset
  - [ ] MSRV badge
  - [ ] License badge
  - [ ] crates.io badge
  - [ ] docs.rs badge
  - [ ] CI status badge
- [ ] Finalize CONTRIBUTING.md with:
  - [ ] Development setup instructions
  - [ ] How to add a new chart type (mark implementation guide)
  - [ ] How to add a new backend
  - [ ] Snapshot testing workflow (running, reviewing, updating)
  - [ ] PR review criteria

### Testing for 0.12.0

- [ ] `cargo test --doc` passes with zero failures
- [ ] All examples compile and run (CI job running each example)
- [ ] Gallery generates all images without error
- [ ] Update CHANGELOG.md

---

## 1.0.0-rc.1 — Release candidate

> Exit criteria: All features complete, no known critical bugs, API frozen.

### Final audit

- [ ] Run `cargo semver-checks` against 0.12.0 — no unintentional breaking changes
- [ ] Run `cargo deny check` — no license violations, no known advisories
- [ ] Run `cargo audit` — no security advisories
- [ ] Run full test suite on all three platforms (Linux, macOS, Windows)
- [ ] Run full test suite on MSRV
- [ ] Run clippy with `pedantic` — zero warnings
- [ ] Verify all feature flag combinations compile:
  - [ ] `--no-default-features`
  - [ ] `--features minimal`
  - [ ] `--features full`
  - [ ] `--features "gpu,interactive"`
  - [ ] `--features "terminal"`
  - [ ] `--features "web"`
  - [ ] `--features "science"`
  - [ ] `--features "polars,ndarray,arrow"`
- [ ] Manual testing: run every example, visually verify output
- [ ] Performance regression check: compare benchmarks against 0.6.0 baseline

### API freeze review

- [ ] Review all public types — is each name clear and consistent?
- [ ] Review all builder methods — are they discoverable and composable?
- [ ] Review all error types — are error messages helpful?
- [ ] Review feature flag surface — are features orthogonal and well-scoped?
- [ ] Document any known limitations in a `KNOWN_ISSUES.md`
- [ ] Write migration guide from plotters for common use cases
- [ ] Write migration guide from plotly-rs for common use cases

### Pre-release

- [ ] Publish `1.0.0-rc.1` to crates.io
- [ ] Announce on Reddit r/rust for community feedback
- [ ] Announce on Rust Users Forum
- [ ] Collect feedback for 2 weeks
- [ ] Address critical feedback as patch releases (1.0.0-rc.2, etc.)

---

## 1.0.0 — Stable release

> Exit criteria: Stable public API, comprehensive documentation, all chart types, all backends.

### Final changes

- [ ] Apply any remaining feedback from RC period
- [ ] Final CHANGELOG.md entry for 1.0.0
- [ ] Set version to 1.0.0 in all workspace Cargo.toml files
- [ ] Final `cargo test --workspace --all-features` pass
- [ ] Final `cargo doc --workspace --all-features --no-deps` pass

### Publish

- [ ] Publish workspace crates to crates.io in dependency order:
  1. `starsight-derive`
  2. `starsight-core`
  3. `starsight-marks`
  4. `starsight-layout`
  5. `starsight-figure`
  6. `starsight-interact`
  7. `starsight-export`
  8. `starsight-gpu`
  9. `starsight` (facade)
- [ ] Create GitHub release with full changelog and gallery images
- [ ] Tag `v1.0.0` in git

### Announce

- [ ] Announce on Reddit r/rust
- [ ] Announce on Rust Users Forum
- [ ] Announce on Hacker News (Show HN)
- [ ] Announce on Mastodon/X
- [ ] Submit to This Week in Rust
- [ ] Update Are We GUI Yet / lib.rs visualization listing
- [ ] Update resonant-jovian organization README to include starsight

---

## Post-1.0.0 — Ongoing

- [ ] Monitor GitHub issues for bug reports
- [ ] Patch releases (1.0.x) for bug fixes
- [ ] Minor releases (1.x.0) for new chart types via recipe system
- [ ] Track upstream dependency updates (wgpu, polars, ratatui) for compatibility
- [ ] Expand geographic chart support (more projections, tile map backgrounds)
- [ ] Explore LaTeX math rendering for axis labels and annotations
- [ ] Explore Jupyter/evcxr integration improvements
- [ ] Explore egui embedding convenience crate
- [ ] Explore ONNX-accelerated ML chart types (auto-clustering visualization)
- [ ] Community recipe registry (curated collection of domain-specific chart types)

---

## Versioning and release policy

starsight follows Rust ecosystem SemVer conventions. Pre-1.0 releases use 0.x.y where x increments on breaking changes and y on additions/fixes. The goal is to reach 1.0.0 after all phases with a stable public API.

After 1.0.0: patch releases (1.0.x) for bug fixes, minor releases (1.x.0) for new chart types and features, major releases (2.0.0) only for fundamental API redesigns. New chart types added via the recipe system do not require version bumps.

Changelogs follow the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format. Every PR must include a changelog entry categorized as Added, Changed, Deprecated, Removed, Fixed, or Security. Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) — `feat:` for minor, `fix:` for patch, `feat!:` / `BREAKING CHANGE:` for major.

---

## MSRV

The minimum supported Rust version is **1.85** (edition 2024). This will track the latest stable minus two releases, consistent with wgpu and ratatui MSRV policies.

---

## Community demand

The Rust Users Forum thread "Seeking Rust Alternative to Python's matplotlib for Plotting" (Aug 2025, 1,761 views) captures the pain. The top reply states plainly that feature parity with matplotlib does not exist. The Charton announcement thread (2025) notes existing options either feel like drawing on a canvas rather than analyzing data (plotters) or don't feel Rust-native (charming, plotly-rs). Plotters' own "Status of the project?" issue (Jul 2025, 9 reactions) and "Call for participations" signal the ecosystem's bandwidth problem. The pattern dates back to the earliest "What are our choices for a charting library?" thread (Nov 2017), making this an eight-year-old gap. starsight aims to close it.
