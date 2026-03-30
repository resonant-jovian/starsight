# starsight — API reference, architecture, development structure, and guidelines

> Internal development reference for the starsight unified visualization crate.

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

The `Figure` builder exposes the full compositional power. Marks (geoms) are layered onto shared or independent axes. Aesthetic mappings bind data columns to visual properties.

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
    .scale_color(Scale::categorical(prismatica::qualitative::SET2))
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
    .cmap(prismatica::diverging::BERLIN)
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
            .cmap(prismatica::sequential::BATLOW)
            .wireframe(false),
    )
    .title("sin(x) * cos(y)")
    .show();

// Isosurface from volumetric data
Figure::new()
    .add(
        Geom::isosurface()
            .data_volume(&volume_array)  // 3D ndarray
            .level(0.5)
            .opacity(0.8)
            .cmap(prismatica::sequential::HAWAII),
    )
    .show();

// Volume rendering
Figure::new()
    .add(
        Geom::volume_render()
            .data_volume(&density_field)
            .transfer_fn(TransferFn::ramp(0.0, 1.0))
            .cmap(prismatica::sequential::LAJOLLA),
    )
    .camera(Camera::fly(eye, center, up))
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

// Inline rendering in a ratatui-plt context
use ratatui_plt::StarsightPlot;
let widget = StarsightPlot::from(figure).protocol(Protocol::Kitty);
frame.render_widget(widget, area);
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

// Linked views — selection in one chart highlights in another
let (fig_a, fig_b) = linked_pair();

fig_a
    .data(&df)
    .add(Geom::scatter().aes(x("x"), y("y")));

fig_b
    .data(&df)
    .add(Geom::histogram().aes(x("x")));

Dashboard::new()
    .row(vec![fig_a, fig_b])
    .show();

// Streaming real-time data
let mut stream_fig = Figure::new()
    .add(Geom::line().aes(x("time"), y("value")))
    .streaming(StreamOpts { window: Duration::from_secs(60) });

stream_fig.show();

loop {
    let (t, v) = read_sensor();
    stream_fig.append(row!["time" => t, "value" => v]);
    sleep(Duration::from_millis(16)); // ~60fps
}
```

### Multi-backend export (Layer 7)

```rust
use starsight::prelude::*;

let fig = Figure::new()
    .data(&df)
    .add(Geom::scatter().aes(x("x"), y("y")))
    .theme(Theme::publication());

// Raster export (via tiny-skia CPU or wgpu readback)
fig.save("plot.png")?;            // Default DPI (150)
fig.save_with("plot.png", SaveOpts { dpi: 300, width: 2400, height: 1600 })?;

// Vector export
fig.save("plot.svg")?;
fig.save("plot.pdf")?;

// Interactive HTML (self-contained, embeds JS interactivity shim)
fig.save("plot.html")?;

// Terminal inline (prints directly to stdout)
fig.print_terminal()?;

// Raw bytes for embedding
let png_bytes: Vec<u8> = fig.render_png(300)?;
let svg_string: String = fig.render_svg()?;
```

### Theming and styling

```rust
use starsight::prelude::*;

// Built-in presets
fig.theme(Theme::minimal());
fig.theme(Theme::dark());
fig.theme(Theme::publication());
fig.theme(Theme::seaborn());
fig.theme(Theme::ggplot());

// Element-level customization
fig.theme(
    Theme::minimal()
        .background(Color::WHITE)
        .axis_color(Color::GRAY_60)
        .grid(Grid { visible: true, color: Color::GRAY_90, width: 0.5 })
        .font_family("CMU Serif")
        .font_size(12.0)
        .title_size(16.0)
        .legend_position(LegendPosition::TopRight)
        .colorbar_width(15.0),
);

// Theme from chromata
use chromata::popular::CATPPUCCIN_MOCHA;
fig.theme(Theme::from_chromata(CATPPUCCIN_MOCHA));
```

### Recipe system for custom chart types

```rust
use starsight::prelude::*;

// Define a custom chart type via the recipe macro
#[starsight::recipe]
fn volcano_plot(
    data: &DataFrame,
    x: &str,           // log2 fold change column
    y: &str,           // -log10 p-value column
    threshold_x: f64,  // fold change threshold
    threshold_y: f64,  // significance threshold
) -> Figure {
    let fig = Figure::new().data(data);

    // Background points (non-significant)
    fig.add(
        Geom::point()
            .aes(x(x), y(y))
            .filter(|row| row[x].abs() < threshold_x || row[y] < threshold_y)
            .color(Color::GRAY_70)
            .size(2.0),
    );

    // Upregulated (significant, positive fold change)
    fig.add(
        Geom::point()
            .aes(x(x), y(y))
            .filter(|row| row[x] >= threshold_x && row[y] >= threshold_y)
            .color(Color::RED)
            .size(3.0),
    );

    // Downregulated
    fig.add(
        Geom::point()
            .aes(x(x), y(y))
            .filter(|row| row[x] <= -threshold_x && row[y] >= threshold_y)
            .color(Color::BLUE)
            .size(3.0),
    );

    // Threshold lines
    fig.add(Geom::hline(threshold_y).dash(Dash::Dashed).color(Color::GRAY_50));
    fig.add(Geom::vline(threshold_x).dash(Dash::Dashed).color(Color::GRAY_50));
    fig.add(Geom::vline(-threshold_x).dash(Dash::Dashed).color(Color::GRAY_50));

    fig
}

// Usage
volcano_plot(&de_results, "log2fc", "neg_log10_pval", 1.0, 2.0).show();
```

---

## Workspace and crate structure

```
starsight/
├── Cargo.toml                    # Workspace root
├── LICENSE                       # GPL-3.0-or-later
├── README.md                     # Project README (audience-based sections)
├── CONTRIBUTING.md               # Contributor guide
├── ARCHITECTURE.md               # This document
├── CHANGELOG.md                  # Keep-a-changelog format
│
├── starsight/                    # Main facade crate (re-exports everything)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── prelude.rs            # use starsight::prelude::*;
│
├── starsight-core/               # Layer 1-2: rendering abstraction, scales, axes
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── backend/
│       │   ├── mod.rs            # DrawBackend trait
│       │   ├── tiny_skia.rs      # CPU raster backend
│       │   ├── svg.rs            # SVG writer backend
│       │   └── pdf.rs            # PDF vector backend
│       ├── scale/
│       │   ├── mod.rs            # Scale<D, R> trait
│       │   ├── linear.rs
│       │   ├── log.rs
│       │   ├── symlog.rs
│       │   ├── time.rs
│       │   ├── categorical.rs
│       │   └── color.rs          # Color scale (maps value → color)
│       ├── axis/
│       │   ├── mod.rs
│       │   ├── tick.rs           # TickLocator / TickFormatter traits
│       │   └── grid.rs
│       ├── coord/
│       │   ├── mod.rs
│       │   ├── cartesian.rs
│       │   ├── polar.rs
│       │   └── geographic.rs     # Feature-gated on `geo`
│       ├── color/
│       │   ├── mod.rs
│       │   ├── mapping.rs        # Normalize + colormap pipeline
│       │   └── theme.rs          # Theme struct and presets
│       ├── text/
│       │   ├── mod.rs
│       │   └── layout.rs         # cosmic-text integration
│       └── scene.rs              # Backend-agnostic scene graph primitives
│
├── starsight-marks/              # Layer 3: geom/mark system, stat transforms
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── geom/
│       │   ├── mod.rs            # Geom enum and trait
│       │   ├── point.rs
│       │   ├── line.rs
│       │   ├── area.rs
│       │   ├── bar.rs
│       │   ├── rect.rs
│       │   ├── arc.rs            # Pie, donut
│       │   ├── text.rs
│       │   ├── rule.rs           # hline, vline
│       │   ├── polygon.rs
│       │   ├── contour.rs
│       │   ├── surface.rs        # 3D surface
│       │   └── volume.rs         # 3D volume rendering
│       ├── stat/
│       │   ├── mod.rs
│       │   ├── bin.rs
│       │   ├── kde.rs
│       │   ├── regression.rs
│       │   ├── ecdf.rs
│       │   ├── boxplot.rs
│       │   ├── density2d.rs
│       │   └── hexbin.rs
│       ├── aes.rs                # Aesthetic mapping (x, y, color, size, shape, etc.)
│       └── position.rs           # Dodge, Stack, Fill, Jitter, Nudge
│
├── starsight-layout/             # Layer 4: layout, faceting, legends
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── grid.rs               # GridLayout with variable cells
│       ├── facet.rs              # FacetWrap, FacetGrid
│       ├── legend.rs
│       ├── colorbar.rs
│       ├── inset.rs
│       └── compose.rs            # Layer composition, concatenation
│
├── starsight-figure/             # Layer 5: Figure builder, plot!() macro, data acceptance
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── figure.rs             # Figure struct and builder
│       ├── plot_macro.rs         # plot!() macro implementation
│       ├── auto.rs               # Auto-inference (chart type, scales, legends)
│       ├── data/
│       │   ├── mod.rs            # DataSource trait
│       │   ├── polars.rs         # Polars DataFrame/Series acceptance
│       │   ├── ndarray.rs        # ndarray acceptance
│       │   ├── arrow.rs          # Arrow RecordBatch acceptance
│       │   └── raw.rs            # Vec, slice, iterator acceptance
│       └── shorthand.rs          # PairPlot, JointPlot, ClusterMap convenience types
│
├── starsight-interact/           # Layer 6: interactivity, real-time, events
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── hover.rs
│       ├── zoom.rs
│       ├── pan.rs
│       ├── select.rs             # Box, lasso, point selection
│       ├── linked.rs             # Linked views / brushing
│       ├── stream.rs             # Streaming data append
│       └── controls.rs           # Range sliders, dropdowns (egui integration)
│
├── starsight-export/             # Layer 7: animation, export, terminal output
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── animation.rs          # Frame recording, transitions
│       ├── png.rs
│       ├── svg.rs
│       ├── pdf.rs
│       ├── html.rs               # Self-contained interactive HTML
│       └── terminal.rs           # Protocol detection, ratatui widget
│
├── starsight-gpu/                # wgpu + vello backend implementation
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── renderer.rs           # wgpu DrawBackend implementation
│       ├── pipeline.rs           # GPU render pipelines (2D, 3D)
│       ├── camera.rs             # 3D camera management
│       ├── mesh.rs               # Mesh generation for 3D charts
│       └── window.rs             # winit window management
│
├── starsight-derive/             # Proc macros
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                # #[starsight::recipe], derive macros
│
├── examples/
│   ├── quickstart.rs
│   ├── scatter.rs
│   ├── statistical.rs
│   ├── surface3d.rs
│   ├── terminal.rs
│   ├── interactive.rs
│   ├── polars_integration.rs
│   ├── streaming.rs
│   ├── faceting.rs
│   ├── custom_theme.rs
│   ├── recipe.rs
│   └── gallery.rs               # Generates all chart types as PNG for docs
│
└── xtask/                        # Build system tasks
    ├── Cargo.toml
    └── src/
        └── main.rs               # Gallery generation, benchmark, CI helpers
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

## Development guidelines

### Code style and conventions

**Naming**: All public types use full descriptive names. No abbreviations in public API (`Figure`, not `Fig`; `Histogram`, not `Hist`). Internal code may abbreviate where conventional (`ctx`, `opts`, `cfg`).

**Error handling**: All fallible operations return `Result<T, StarsightError>`. The error type is a single enum with variants per failure domain (rendering, data, IO, scale). Never panic in library code. The `plot!()` macro and `.show()` may panic on unrecoverable display errors (no available backend) with a clear diagnostic message — this is the only exception.

**Builder pattern**: All complex types use the builder pattern with method chaining. Builders consume `self` (not `&mut self`) to enable clean chaining. Provide `Default` implementations for all option structs.

```rust
// Correct — consuming self
impl Geom {
    pub fn alpha(self, alpha: f64) -> Self { ... }
    pub fn color(self, color: impl Into<Color>) -> Self { ... }
}

// Incorrect — mutable reference breaks chaining ergonomics
impl Geom {
    pub fn alpha(&mut self, alpha: f64) -> &mut Self { ... }
}
```

**Trait design**: Prefer concrete types over trait objects in the public API. Use generics with trait bounds for data acceptance (`impl Into<DataSource>`). Reserve `dyn Trait` for the backend abstraction where dynamic dispatch is necessary.

**Generics**: Keep generic parameter counts low in public APIs. Use `impl Trait` in argument position for ergonomics. Never expose more than two generic parameters on a public type.

```rust
// Good — impl Trait hides complexity
pub fn plot(data: impl Into<DataSource>) -> Figure { ... }

// Bad — leaks complexity to the user
pub fn plot<D: Into<DataSource>, S: Into<Scale<f64, f64>>>(data: D, scale: S) -> Figure { ... }
```

### Feature gating rules

Every optional dependency must be behind a feature gate. Feature gates use the names defined in the dependency stack section. Code behind feature gates uses `#[cfg(feature = "...")]` at the module level, not scattered throughout functions.

```rust
// Correct — module-level gating
#[cfg(feature = "polars")]
mod polars_support;

#[cfg(feature = "polars")]
pub use polars_support::*;

// Incorrect — inline gating that fragments logic
pub fn accept_data(data: DataSource) {
    #[cfg(feature = "polars")]
    if let DataSource::Polars(df) = data { ... }
}
```

The `default` feature set must produce a useful crate with CPU rendering, SVG, and PNG export. A user who types `cargo add starsight` without any feature flags must be able to create and save charts immediately.

### Testing strategy

**Unit tests**: Every scale, mark, stat transform, and layout algorithm has unit tests with known-good reference values. Statistical transforms (KDE, regression) are tested against scipy/R reference outputs with tolerance bounds.

**Snapshot tests**: Every chart type has a snapshot test (via `insta`) producing a reference PNG at fixed dimensions. Snapshot tests run against the tiny-skia backend only (deterministic, no GPU variance). The `examples/gallery.rs` binary generates all snapshots.

**Property tests**: Scale round-trip (domain→range→domain) and axis tick generation are property-tested via `proptest`. Color mapping must be monotonic for sequential colormaps and symmetric for diverging colormaps.

**Integration tests**: Polars DataFrame acceptance, ndarray acceptance, and Arrow RecordBatch acceptance each have integration tests that construct real data and render to PNG bytes without I/O.

**No GPU in CI**: CI runs only the tiny-skia and SVG backends. GPU backend tests are run locally or in a separate GPU-enabled CI pipeline. The `starsight-gpu` crate has a `#[cfg(test)]` mock backend for unit testing render pipeline logic without actual GPU access.

### Documentation requirements

Every public type, trait, method, and function must have a rustdoc comment. Examples are mandatory for all Layer 5 API (Figure, plot!() macro, convenience types). The top-level `lib.rs` doc comment includes a complete quickstart example.

Doc examples must compile and run (`cargo test --doc`). Examples that require optional features use `#[cfg(feature = "...")]` and are marked with the feature in the doc comment.

```rust
/// Create a scatter plot from two slices.
///
/// # Examples
///
/// ```
/// use starsight::prelude::*;
///
/// let x = [1.0, 2.0, 3.0];
/// let y = [4.0, 5.0, 6.0];
/// let png_bytes = plot!(x, y).render_png(150)?;
/// assert!(!png_bytes.is_empty());
/// # Ok::<(), starsight::Error>(())
/// ```
pub fn scatter(x: impl IntoIterator<Item = f64>, y: impl IntoIterator<Item = f64>) -> Figure {
    ...
}
```

### README and public documentation style

Follows the resonant-jovian standard: audience-based sections (For Everyone / For Users / For Developers), Highlights block, GitHub callout syntax, ASCII diagrams, tables, no emojis, horizontal rule separators, standalone MSRV section.

---

## Restrictions and non-goals

### Hard restrictions

**No JavaScript runtime dependencies.** starsight must not require Node.js, Deno, or any JS engine for any functionality in any configuration. The HTML export feature generates self-contained HTML with a minimal embedded JS interactivity shim (authored as part of starsight, not a bundled library), but this is output — not a dependency.

**No C/C++ system library dependencies in the default feature set.** The default configuration (tiny-skia + SVG + PNG) must compile with `cargo build` on a clean system with only a Rust toolchain. Feature-gated backends (wgpu, proj) may depend on system libraries but must document this clearly.

**No unsafe code in the marks, layout, or figure layers (Layers 3-5).** Unsafe is permitted only in the rendering backends (Layer 1) and GPU code (starsight-gpu) where FFI or GPU buffer management requires it. Every `unsafe` block must have a `// SAFETY:` comment explaining the invariant.

**No runtime file I/O for core functionality.** Colormaps come from prismatica (compile-time constants). Themes are built-in structs. Font data is embedded via `include_bytes!()` for a default fallback font. Users may load custom fonts from files, but the library must function without any filesystem access.

**No dynamic allocation in the hot rendering path for simple charts.** The tiny-skia backend for a basic line chart with <1000 points must not allocate after initial setup. This is enforced by benchmarking, not by `#[no_alloc]` (which doesn't exist), but is a design principle for the rendering layer.

**No `println!()` or `eprintln!()` in library code.** Use the `log` crate (or `tracing` if adopted) for diagnostic output. The library must be silent by default.

**No panics in library code except in the `plot!()` macro's `.show()` method** when no display backend is available (documented clearly). All other code paths return `Result`.

### Soft restrictions

**Minimize total dependency count in the default feature set.** Every new dependency in the default feature set must be justified in a PR description with the rationale and the crate's maintenance status. Prefer crates with >100K downloads/month and active maintenance.

**Avoid re-inventing solved problems.** If a mature Rust crate exists for a capability (tiny-skia for rasterization, cosmic-text for text shaping, statrs for distributions), depend on it rather than reimplementing. Only reimplement when the existing crate is abandoned, has an incompatible license, or introduces unacceptable dependency weight.

**No async in the public API.** The `plot!()` macro and `Figure` builder are synchronous. Streaming data (Layer 6) uses a push-based API (`fig.append()`), not async streams. Internally, the wgpu backend may use async for GPU operations, but this is hidden behind a synchronous `DrawBackend` trait with `pollster::block_on()` or equivalent.

**No nightly-only features.** starsight must compile on stable Rust. If a nightly feature would be beneficial (e.g., SIMD intrinsics, const generics improvements), it may be exposed behind a `nightly` feature flag but must not be required.

### Non-goals

**starsight is not a GUI framework.** It produces charts, not applications. The egui integration (Layer 6) embeds charts into egui applications but does not provide windows, menus, or application lifecycle management.

**starsight is not a game engine.** The 3D capabilities are for data visualization (scatter, surface, volume), not for real-time 3D scene rendering. No PBR materials, no skeletal animation, no physics. Use bevy or wgpu directly for those.

**starsight is not a BI/dashboard platform.** It does not include a server, database connectors, or a layout designer. The `Dashboard` builder composes charts into layouts, but deployment (serving HTML, updating on schedule) is the user's responsibility.

**starsight is not a notebook.** It does not provide a REPL, incremental compilation, or Jupyter kernel. It integrates with evcxr where possible but does not attempt to fix Rust's compilation-speed limitations for interactive use.

**starsight does not wrap any external plotting library.** It does not shell out to gnuplot, embed Plotly.js, bundle ECharts, or call matplotlib via PyO3. Every chart is rendered by Rust code running in-process. The only exception is the HTML export feature, which includes a small JS interactivity shim for browser-based hover/zoom.

---

## Development phases

### Phase 0 — Foundation (starsight-core)

Implement the `DrawBackend` trait with the tiny-skia backend. Implement linear and categorical scales with automatic tick generation. Implement basic text rendering via cosmic-text. Produce a single working chart type (line) rendered to PNG and SVG. Establish the snapshot testing infrastructure with insta.

**Exit criteria**: `plot!([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]).save("test.png")` produces a correct line chart.

### Phase 1 — Core chart types (starsight-marks + starsight-figure)

Implement the 14 most-used geom types: Point, Line, Area, Bar, Histogram, BoxPlot, Heatmap, Scatter (as Point alias), Pie, ErrorBar, Contour, Violin, KDE, and Candlestick. Implement the `Figure` builder and `plot!()` macro. Add Polars DataFrame acceptance.

**Exit criteria**: Every chart type has a snapshot test. The `plot!(&df, x = "col_a", y = "col_b")` syntax works.

### Phase 2 — Layout and composition (starsight-layout)

Implement GridLayout, FacetWrap, FacetGrid, legends, colorbars, and twin axes. Implement PairPlot and JointPlot convenience types.

**Exit criteria**: Faceted plots render correctly with free and fixed scales. PairPlot produces a scatterplot matrix.

### Phase 3 — Scale infrastructure

Implement log, symlog, logit, power, and datetime scales. Implement the pluggable TickLocator/TickFormatter trait system. Implement secondary axes.

**Exit criteria**: Log-scale axes with minor ticks render correctly. DateTime axes auto-format at year/month/day/hour granularity.

### Phase 4 — GPU and interactivity (starsight-gpu + starsight-interact)

Implement the wgpu DrawBackend. Implement hover, zoom, pan, and selection. Implement the winit window loop. Target: 60fps with 100K data points.

**Exit criteria**: A scatter plot with 100K points renders interactively with hover tooltips in a native window.

### Phase 5 — 3D visualization

Implement Scatter3D, Surface3D, Wireframe3D, Isosurface, and VolumeRender. Implement 3D camera management (orbit, fly, turntable). Implement 3D axes with labels and ticks.

**Exit criteria**: A surface plot with colormapping and labeled axes renders correctly in both wgpu and tiny-skia backends.

### Phase 6 — Terminal backend (starsight-export terminal path)

Implement terminal protocol detection (Kitty/Sixel/iTerm2/half-block/Braille). Implement the ratatui widget adapter. Integrate with ratatui-plt for terminal-native chart widgets.

**Exit criteria**: `plot!(x, y).terminal().show()` renders a chart inline in Kitty, WezTerm, and iTerm2 terminals.

### Phase 7 — Export, animation, and polish

Implement PDF export via krilla. Implement frame-based animation recording. Implement the HTML interactive export. Write comprehensive documentation and examples. Run the full gallery generator. Audit the public API for consistency.

**Exit criteria**: 1.0.0 release candidate. All 60+ chart types have snapshot tests, doc examples, and gallery entries.

---

## Versioning and release policy

starsight follows Rust ecosystem SemVer conventions. Pre-1.0 releases use 0.x.y where x increments on breaking changes and y on additions/fixes. The goal is to reach 1.0.0 after Phase 7 with a stable public API.

After 1.0.0: patch releases (1.0.x) for bug fixes, minor releases (1.x.0) for new chart types and features, major releases (2.0.0) only for fundamental API redesigns. New chart types added via the recipe system do not require version bumps.

Changelogs follow the Keep a Changelog format. Every PR must include a changelog entry categorized as Added, Changed, Deprecated, Removed, Fixed, or Security.
