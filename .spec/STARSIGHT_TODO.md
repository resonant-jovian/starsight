# starsight — TODO roadmap: 0.1.0 through 1.0.0

> Exhaustive, ordered task list for every milestone release of the starsight unified visualization crate.

Each task is marked with a checkbox. Tasks are grouped by release version and ordered by dependency (earlier tasks unblock later ones). Within each version, tasks are grouped by workspace crate.

---

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
