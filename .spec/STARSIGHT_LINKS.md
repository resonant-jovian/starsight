# starsight — Complete reference link collection

> Every external URL and resource needed for the starsight unified visualization library project.

---

## 1. Rust dependency crate documentation

Every crate below includes its docs.rs API reference, crates.io listing, and GitHub source repository.

---

### Rendering and graphics

**wgpu** — Cross-platform, safe WebGPU implementation running on Vulkan, Metal, D3D12, and OpenGL.
- docs.rs: https://docs.rs/wgpu
- crates.io: https://crates.io/crates/wgpu
- GitHub: https://github.com/gfx-rs/wgpu

**vello** — GPU compute-centric 2D vector graphics rendering engine built on wgpu.
- docs.rs: https://docs.rs/vello
- crates.io: https://crates.io/crates/vello
- GitHub: https://github.com/linebender/vello

**tiny-skia** — High-quality CPU-based 2D rendering library, a tiny Skia subset ported to pure Rust.
- docs.rs: https://docs.rs/tiny-skia
- crates.io: https://crates.io/crates/tiny-skia
- GitHub: https://github.com/linebender/tiny-skia

**lyon** — Path tessellation library that turns vector paths into GPU-friendly triangle meshes.
- docs.rs: https://docs.rs/lyon
- crates.io: https://crates.io/crates/lyon
- GitHub: https://github.com/nical/lyon

**cosmic-text** — Pure Rust multi-line text shaping, layout, and rendering with bidirectional text and font fallback.
- docs.rs: https://docs.rs/cosmic-text
- crates.io: https://crates.io/crates/cosmic-text
- GitHub: https://github.com/pop-os/cosmic-text

**parley** — Rich text layout library with font enumeration, shaping, line breaking, and bidi support.
- docs.rs: https://docs.rs/parley
- crates.io: https://crates.io/crates/parley
- GitHub: https://github.com/linebender/parley

**ab_glyph** — Fast OpenType font glyph loading, scaling, positioning, and rasterization.
- docs.rs: https://docs.rs/ab_glyph
- crates.io: https://crates.io/crates/ab_glyph
- GitHub: https://github.com/alexheretic/ab-glyph

---

### Data and math

**polars** — Extremely fast DataFrame query engine based on Apache Arrow with Python and Node.js bindings.
- docs.rs: https://docs.rs/polars
- crates.io: https://crates.io/crates/polars
- GitHub: https://github.com/pola-rs/polars

**ndarray** — N-dimensional array with views, multidimensional slicing, and efficient numerical operations.
- docs.rs: https://docs.rs/ndarray
- crates.io: https://crates.io/crates/ndarray
- GitHub: https://github.com/rust-ndarray/ndarray

**nalgebra** — General-purpose linear algebra with static/dynamic matrices, transformations, and decompositions.
- docs.rs: https://docs.rs/nalgebra
- crates.io: https://crates.io/crates/nalgebra
- GitHub: https://github.com/dimforge/nalgebra

**arrow** (arrow-rs) — Official native Rust implementation of Apache Arrow in-memory columnar format. The crate name on crates.io is `arrow`; the GitHub repo is `arrow-rs`.
- docs.rs: https://docs.rs/arrow
- crates.io: https://crates.io/crates/arrow
- GitHub: https://github.com/apache/arrow-rs

---

### Color

**palette** — Color management and conversion across RGB, HSL, L*a*b*, XYZ, and many other color spaces.
- docs.rs: https://docs.rs/palette
- crates.io: https://crates.io/crates/palette
- GitHub: https://github.com/Ogeon/palette

**colorgrad** — Color gradient library for data visualization, charts, generative art, and maps.
- docs.rs: https://docs.rs/colorgrad
- crates.io: https://crates.io/crates/colorgrad
- GitHub: https://github.com/mazznoer/colorgrad-rs

**colorous** — Professional sequential, diverging, and categorical color schemes ported from d3-scale-chromatic.
- docs.rs: https://docs.rs/colorous
- crates.io: https://crates.io/crates/colorous
- GitHub: https://github.com/dtolnay/colorous

---

### Statistics and computational geometry

**statrs** — Statistical computing with probability distributions, statistical functions, and special functions.
- docs.rs: https://docs.rs/statrs
- crates.io: https://crates.io/crates/statrs
- GitHub: https://github.com/statrs-dev/statrs

**contour** — Computes isolines, contour polygons, and isobands via marching squares (ported from d3-contour).
- docs.rs: https://docs.rs/contour
- crates.io: https://crates.io/crates/contour
- GitHub: https://github.com/mthh/contour-rs

**delaunator** — Fast and robust 2D Delaunay triangulation, a Rust port of the JavaScript Delaunator library.
- docs.rs: https://docs.rs/delaunator
- crates.io: https://crates.io/crates/delaunator
- GitHub: https://github.com/mourner/delaunator-rs

---

### Image, SVG, and PDF output

**image** — Encoding, decoding, and basic processing for common image formats (PNG, JPEG, GIF, etc.).
- docs.rs: https://docs.rs/image
- crates.io: https://crates.io/crates/image
- GitHub: https://github.com/image-rs/image

**resvg** — High-quality SVG rendering library using tiny-skia with no system dependencies.
- docs.rs: https://docs.rs/resvg
- crates.io: https://crates.io/crates/resvg
- GitHub: https://github.com/linebender/resvg

**usvg** — SVG parser that simplifies the full SVG spec into a minimal, resolved tree representation. Lives in the resvg monorepo.
- docs.rs: https://docs.rs/usvg
- crates.io: https://crates.io/crates/usvg
- GitHub: https://github.com/linebender/resvg (monorepo, `crates/usvg`)

**svg** — Programmatic SVG composer and parser for generating and reading SVG documents.
- docs.rs: https://docs.rs/svg
- crates.io: https://crates.io/crates/svg
- GitHub: https://github.com/bodoni/svg

**krilla** — High-level, ergonomic PDF document creation library built on top of pdf-writer.
- docs.rs: https://docs.rs/krilla
- crates.io: https://crates.io/crates/krilla
- GitHub: https://github.com/LaurenzV/krilla

**pdf-writer** — Step-by-step, low-level PDF writer with type-safe builder APIs (by the Typst team).
- docs.rs: https://docs.rs/pdf-writer
- crates.io: https://crates.io/crates/pdf-writer
- GitHub: https://github.com/typst/pdf-writer

**printpdf** — Rust/WASM library for creating, reading, writing, and rendering PDF documents.
- docs.rs: https://docs.rs/printpdf
- crates.io: https://crates.io/crates/printpdf
- GitHub: https://github.com/fschutt/printpdf

---

### Terminal UI

**ratatui** — Rich terminal user interface library for Rust, the maintained successor of tui-rs.
- docs.rs: https://docs.rs/ratatui
- crates.io: https://crates.io/crates/ratatui
- GitHub: https://github.com/ratatui/ratatui

**crossterm** — Cross-platform terminal manipulation for input, styling, screen, and cursor control.
- docs.rs: https://docs.rs/crossterm
- crates.io: https://crates.io/crates/crossterm
- GitHub: https://github.com/crossterm-rs/crossterm

**ratatui-image** — Image widget for ratatui supporting Sixel, Kitty, iTerm2, and unicode halfblock protocols.
- docs.rs: https://docs.rs/ratatui-image
- crates.io: https://crates.io/crates/ratatui-image
- GitHub: https://github.com/ratatui/ratatui-image

**viuer** — Library for displaying images in the terminal via iTerm, Kitty, Sixel, or halfblock rendering.
- docs.rs: https://docs.rs/viuer
- crates.io: https://crates.io/crates/viuer
- GitHub: https://github.com/atanunq/viuer

---

### GUI and windowing

**winit** — Cross-platform window creation and event loop management.
- docs.rs: https://docs.rs/winit
- crates.io: https://crates.io/crates/winit
- GitHub: https://github.com/rust-windowing/winit

**egui** — Easy-to-use immediate mode GUI library running on both web and native.
- docs.rs: https://docs.rs/egui
- crates.io: https://crates.io/crates/egui
- GitHub: https://github.com/emilk/egui

**egui_plot** — Immediate mode 2D plotting widget for egui (extracted into its own repo).
- docs.rs: https://docs.rs/egui_plot
- crates.io: https://crates.io/crates/egui_plot
- GitHub: https://github.com/emilk/egui_plot

---

### WASM

**wasm-bindgen** — Facilitates high-level interactions between Rust/Wasm modules and JavaScript.
- docs.rs: https://docs.rs/wasm-bindgen
- crates.io: https://crates.io/crates/wasm-bindgen
- GitHub: https://github.com/rustwasm/wasm-bindgen

**web-sys** — Raw bindings to Web APIs (DOM, Canvas, Fetch, WebGL, etc.) for use with wasm-bindgen. Lives in the wasm-bindgen monorepo.
- docs.rs: https://docs.rs/web-sys
- crates.io: https://crates.io/crates/web-sys
- GitHub: https://github.com/rustwasm/wasm-bindgen (monorepo, `crates/web-sys`)

---

### Geospatial

**geo** — Geospatial primitive types (Point, Polygon, etc.) and algorithms (distance, intersection, boolean ops).
- docs.rs: https://docs.rs/geo
- crates.io: https://crates.io/crates/geo
- GitHub: https://github.com/georust/geo

**proj** — High-level Rust bindings for the PROJ C library for coordinate transformation and projection.
- docs.rs: https://docs.rs/proj
- crates.io: https://crates.io/crates/proj
- GitHub: https://github.com/georust/proj

**geojson** — Serialization and deserialization of GeoJSON vector geographic data.
- docs.rs: https://docs.rs/geojson
- crates.io: https://crates.io/crates/geojson
- GitHub: https://github.com/georust/geojson

---

### Utilities

**serde** — The generic serialization/deserialization framework for Rust data structures.
- docs.rs: https://docs.rs/serde
- crates.io: https://crates.io/crates/serde
- GitHub: https://github.com/serde-rs/serde

**clap** — Full-featured command-line argument parser with derive and builder APIs.
- docs.rs: https://docs.rs/clap
- crates.io: https://crates.io/crates/clap
- GitHub: https://github.com/clap-rs/clap

**insta** — Snapshot testing library with inline and file-based snapshot support.
- docs.rs: https://docs.rs/insta
- crates.io: https://crates.io/crates/insta
- GitHub: https://github.com/mitsuhiko/insta

**proptest** — Property-based testing framework inspired by Hypothesis, generating random test inputs.
- docs.rs: https://docs.rs/proptest
- crates.io: https://crates.io/crates/proptest
- GitHub: https://github.com/proptest-rs/proptest

---

### Logging and error handling

**log** — Lightweight logging facade providing macros (info!, warn!, error!) with pluggable backends.
- docs.rs: https://docs.rs/log
- crates.io: https://crates.io/crates/log
- GitHub: https://github.com/rust-lang/log

**tracing** — Framework for structured, context-aware logging and diagnostics for async Rust.
- docs.rs: https://docs.rs/tracing
- crates.io: https://crates.io/crates/tracing
- GitHub: https://github.com/tokio-rs/tracing

**thiserror** — Derive macro for conveniently implementing `std::error::Error` on custom types.
- docs.rs: https://docs.rs/thiserror
- crates.io: https://crates.io/crates/thiserror
- GitHub: https://github.com/dtolnay/thiserror

**anyhow** — Flexible, easy error handling with a concrete `anyhow::Error` type and context chaining.
- docs.rs: https://docs.rs/anyhow
- crates.io: https://crates.io/crates/anyhow
- GitHub: https://github.com/dtolnay/anyhow

---

### Reference and comparison libraries

**plotters** — Pure-Rust drawing/plotting library supporting bitmap, SVG, and WASM canvas backends.
- docs.rs: https://docs.rs/plotters
- crates.io: https://crates.io/crates/plotters
- GitHub: https://github.com/plotters-rs/plotters

**charming** — Rust visualization library leveraging Apache ECharts for chart rendering (HTML, SVG, image, WASM).
- docs.rs: https://docs.rs/charming
- crates.io: https://crates.io/crates/charming
- GitHub: https://github.com/yuankunzhang/charming

**plotly** — Rust plotting library powered by Plotly.js generating interactive HTML charts and static images.
- docs.rs: https://docs.rs/plotly
- crates.io: https://crates.io/crates/plotly
- GitHub: https://github.com/plotly/plotly.rs

**plotlars** — Plotly-based visualization library bridging Polars DataFrames to interactive charts.
- docs.rs: https://docs.rs/plotlars
- crates.io: https://crates.io/crates/plotlars
- GitHub: https://github.com/alceal/plotlars

---

## 2. Rust community guidelines and rules

**Rust API Guidelines** — Official checklist and topical chapters for designing idiomatic Rust APIs.
- Main: https://rust-lang.github.io/api-guidelines/
- Checklist: https://rust-lang.github.io/api-guidelines/checklist.html

**Rust RFC process** — Repository and process documentation for proposing substantial changes to Rust.
- Repository: https://github.com/rust-lang/rfcs
- RFC Book: https://rust-lang.github.io/rfcs/

**crates.io policies** — Naming rules, name squatting policy, acceptable content, and publishing guidelines.
- Policies: https://crates.io/policies
- Publishing guide: https://doc.rust-lang.org/cargo/reference/publishing.html

**Rust Code of Conduct** — The official behavior policy governing all Rust project venues.
- https://www.rust-lang.org/policies/code-of-conduct

**Rust Edition Guide** — Guide to Rust editions (2015, 2018, 2021, 2024) with migration tooling.
- Main: https://doc.rust-lang.org/edition-guide/
- 2024 edition: https://doc.rust-lang.org/edition-guide/rust-2024/index.html

**The Cargo Book** — Comprehensive reference for workspaces, features, dependencies, and crate publishing.
- Main: https://doc.rust-lang.org/cargo/
- Workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Features: https://doc.rust-lang.org/cargo/reference/features.html
- Publishing: https://doc.rust-lang.org/cargo/reference/publishing.html

**The Rust Reference** — Primary language reference detailing syntax, semantics, type system, and memory model.
- https://doc.rust-lang.org/reference/

**The Rustonomicon** — "The Dark Arts of Advanced and Unsafe Rust Programming," covering unsafe primitives, FFI, and UB.
- https://doc.rust-lang.org/nomicon/

**Rust Unsafe Code Guidelines** — UCG working group reference documenting type layout, validity invariants, and memory model.
- Repository: https://github.com/rust-lang/unsafe-code-guidelines
- Reference: https://rust-lang.github.io/unsafe-code-guidelines/

**Rust compiler error index** — Complete index of all compiler error codes with explanations and fixes.
- https://doc.rust-lang.org/error-index.html

---

## 3. Documentation best practices

**The rustdoc Book** — Official guide to writing documentation comments, Markdown, doc tests, and doc attributes.
- Main: https://doc.rust-lang.org/rustdoc/
- How to write docs: https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html

**docs.rs metadata and configuration** — Configuring docs.rs builds via `[package.metadata.docs.rs]` in Cargo.toml.
- https://docs.rs/about/metadata

**README best practices for Rust crates** — The API Guidelines documentation chapter covers crate-level docs and metadata conventions.
- https://rust-lang.github.io/api-guidelines/documentation.html

**The Rust Style Guide** — Official formatting style guide defining the default rustfmt style.
- https://doc.rust-lang.org/style-guide/

**Rust by Example** — Interactive tutorial teaching Rust through runnable, annotated code examples.
- https://doc.rust-lang.org/rust-by-example/

**The Rust Programming Language ("The Rust Book")** — The official introductory book covering ownership, lifetimes, traits, concurrency, and projects.
- https://doc.rust-lang.org/book/

**Effective Rust** — "35 Specific Ways to Improve Your Rust Code" by David Drysdale (O'Reilly, 2024), modeled after Effective C++.
- Free online: https://effective-rust.com/
- O'Reilly: https://www.oreilly.com/library/view/effective-rust/9781098151393/

---

## 4. Changelog and versioning

**Keep a Changelog** — Standardized format for human-readable changelogs with Added, Changed, Fixed, etc. categories.
- https://keepachangelog.com/en/1.1.0/

**Semantic Versioning 2.0.0** — The MAJOR.MINOR.PATCH versioning specification.
- https://semver.org/

**Cargo SemVer compatibility** — Official Cargo reference detailing what constitutes breaking vs. compatible changes for Rust crates.
- https://doc.rust-lang.org/cargo/reference/semver.html

**cargo-semver-checks** — Lint tool that scans rustdoc JSON output to detect API breaking changes before publishing.
- GitHub: https://github.com/obi1kenobi/cargo-semver-checks
- crates.io: https://crates.io/crates/cargo-semver-checks
- GitHub Action: https://github.com/obi1kenobi/cargo-semver-checks-action

**Conventional Commits** — Lightweight convention for structured commit messages that dovetails with SemVer.
- https://www.conventionalcommits.org/en/v1.0.0/

**git-cliff** — Highly customizable changelog generator following Conventional Commits, written in Rust.
- GitHub: https://github.com/orhun/git-cliff
- Website/docs: https://git-cliff.org/

---

## 5. CI/CD and publishing

### GitHub Actions for Rust

**dtolnay/rust-toolchain** — The most popular, concise GitHub Action for installing a Rust toolchain via rustup.
- https://github.com/dtolnay/rust-toolchain

**actions-rust-lang/setup-rust-toolchain** — Official Rust community Action with caching, problem matchers, and RUSTFLAGS support.
- https://github.com/actions-rust-lang/setup-rust-toolchain

**actions-rs (legacy)** — Original GitHub Actions for Rust toolchain and cargo commands; now largely unmaintained.
- https://github.com/actions-rs/toolchain
- https://github.com/actions-rs/cargo

### Release and auditing

**cargo-release** — Automates version bumping, git tagging, changelog updates, and `cargo publish`.
- https://github.com/crate-ci/cargo-release

**cargo-deny** — Lints dependencies for license compliance, banned crates, security advisories, and source validation.
- GitHub: https://github.com/EmbarkStudios/cargo-deny
- GitHub Action: https://github.com/EmbarkStudios/cargo-deny-action

**cargo-audit** — Audits Cargo.lock against the RustSec Advisory Database for known vulnerabilities.
- GitHub (monorepo): https://github.com/rustsec/rustsec
- crates.io: https://crates.io/crates/cargo-audit
- GitHub Action: https://github.com/actions-rust-lang/audit

### Code coverage

**cargo-tarpaulin** — Code coverage tool using ptrace or LLVM instrumentation, supporting HTML/XML/LCOV reports.
- https://github.com/xd009642/tarpaulin

**cargo-llvm-cov** — LLVM source-based code coverage via `-C instrument-coverage`; the recommended modern coverage tool.
- https://github.com/taiki-e/cargo-llvm-cov

### Code quality configuration

**Clippy** — The official Rust linter with configurable lint categories and per-project settings.
- Book: https://doc.rust-lang.org/clippy/
- Configuration: https://doc.rust-lang.org/clippy/configuration.html
- Lint configuration reference: https://doc.rust-lang.org/clippy/lint_configuration.html
- Searchable lint list: https://rust-lang.github.io/rust-clippy/master/index.html

**rustfmt** — The official Rust code formatter with extensive configuration options.
- GitHub: https://github.com/rust-lang/rustfmt
- Configuration reference: https://rust-lang.github.io/rustfmt/
- Configurations.md: https://github.com/rust-lang/rustfmt/blob/main/Configurations.md

### Cross-compilation and WASM

**cross-rs** — "Zero setup" cross compilation using Docker/Podman containers with pre-built toolchains.
- https://github.com/cross-rs/cross

**wasm-pack** — Builds Rust-generated WebAssembly packages for npm/browser/Node.js use.
- GitHub: https://github.com/rustwasm/wasm-pack
- Docs: https://rustwasm.github.io/docs/wasm-pack/

**trunk** — Build, bundle, and ship Rust WASM applications to the web with dev server and asset pipeline.
- GitHub: https://github.com/trunk-rs/trunk
- Docs: https://trunkrs.dev/

---

## 6. Licensing

**GPL-3.0 full text** — The canonical GNU General Public License v3.0 on gnu.org.
- https://www.gnu.org/licenses/gpl-3.0.html

**SPDX license identifier list** — Complete list of standardized short identifiers for all recognized licenses.
- https://spdx.org/licenses/

**choosealicense.com GPL-3.0** — GitHub's summary of GPL-3.0 permissions, conditions, and limitations.
- https://choosealicense.com/licenses/gpl-3.0/

**FSF licensing resources** — Comprehensive FAQ, compatibility guides, and license documentation from the FSF.
- GPL FAQ: https://www.gnu.org/licenses/gpl-faq.html
- License list and compatibility: https://www.gnu.org/licenses/license-list.html
- Quick guide to GPLv3: https://www.gnu.org/licenses/quick-guide-gplv3.html
- GNU Licenses main page: https://www.gnu.org/licenses/

**Rust crate license conventions** — Cargo manifest documentation for the `license` and `license-file` fields using SPDX 2.3 expressions.
- https://doc.rust-lang.org/cargo/reference/manifest.html

---

## 7. Visualization and graphics references

### Foundational theory and specifications

**The Grammar of Graphics** — Leland Wilkinson's foundational 2005 book defining a formal grammar for statistical graphics.
- Springer (2nd ed.): https://link.springer.com/book/10.1007/0-387-28695-0
- Author's page: https://www.cs.uic.edu/~wilkinson/TheGrammarOfGraphics/GOG.html
- EU summary: https://data.europa.eu/apps/data-visualisation-guide/foundation-of-the-grammar-of-graphics

**Vega-Lite** — High-level grammar of interactive graphics providing a concise JSON syntax for rapid visualization.
- Homepage: https://vega.github.io/vega-lite/
- Full spec: https://vega.github.io/vega-lite/docs/spec.html
- Docs index: https://vega.github.io/vega-lite/docs/

**Observable Plot** — JavaScript library for exploratory data visualization built by the D3 team.
- Homepage: https://observablehq.com/plot/
- What is Plot: https://observablehq.com/plot/what-is-plot
- Getting started: https://observablehq.com/plot/getting-started

### Major visualization libraries for comparison

**matplotlib** — Python's comprehensive 2D plotting library for publication-quality figures.
- https://matplotlib.org/
- Full docs: https://matplotlib.org/stable/index.html

**seaborn** — Python statistical visualization library built on matplotlib with a high-level interface.
- https://seaborn.pydata.org/

**ggplot2** — R's declarative graphics system based on The Grammar of Graphics, part of the tidyverse.
- https://ggplot2.tidyverse.org/

**Makie.jl** — Julia's high-performance data visualization with multiple backends (GL, Cairo, WGL, RPR).
- https://docs.makie.org/stable/

**D3.js** — The foundational JavaScript library for bespoke, bindable data-driven visualizations.
- https://d3js.org/

### Graphics and rendering

**Learn Wgpu** — Popular community tutorial for learning wgpu covering pipelines, textures, buffers, and rendering.
- Tutorial: https://sotrh.github.io/learn-wgpu/
- GitHub: https://github.com/sotrh/learn-wgpu

**WebGPU specification** — W3C Candidate Recommendation for the GPU hardware access API on the web.
- Editor's draft: https://gpuweb.github.io/gpuweb/
- W3C TR: https://www.w3.org/TR/webgpu/
- Explainer: https://gpuweb.github.io/gpuweb/explainer/

### Terminal graphics protocols

**Kitty graphics protocol** — Official specification for image transmission, display, animation, and Unicode placeholders.
- https://sw.kovidgoyal.net/kitty/graphics-protocol/

**Sixel graphics** — The original DEC VT3xx bitmap graphics format.
- VT3xx manual: https://vt100.net/docs/vt3xx-gp/chapter14.html
- Wikipedia: https://en.wikipedia.org/wiki/Sixel
- libsixel: https://saitoha.github.io/libsixel/

**iTerm2 inline images** — Documentation for the ESC 1337 escape sequence protocol for inline image display.
- https://iterm2.com/documentation-images.html

### Color science

**ColorBrewer** — Interactive tool for perceptually designed sequential, diverging, and qualitative color schemes for cartography.
- https://colorbrewer2.org/

**Perceptually uniform colormaps (Peter Kovesi)** — CET colour maps with test methodology for evaluating perceptual uniformity.
- Gallery and downloads: https://colorcet.com/
- Python package: https://colorcet.holoviz.org/
- Paper: https://arxiv.org/abs/1509.03700

**Crameri scientific colour maps** — Perceptually uniform, colour-blind friendly, and B&W-print readable colour gradients by Fabio Crameri.
- Main: https://www.fabiocrameri.ch/colourmaps/
- User guide: https://www.fabiocrameri.ch/colourmaps-userguide/
- Zenodo archive: https://zenodo.org/records/8409685

---

## 8. Rust ecosystem status trackers

**Are We GUI Yet?** — Tracks the state of GUI development in Rust, cataloging frameworks and widget toolkits.
- https://areweguiyet.com/

**Are We Learning Yet?** — Catalogs the Rust machine learning ecosystem with quality scores and metadata.
- https://www.arewelearningyet.com/

**Are We Game Yet?** — Comprehensive guide to the Rust game development ecosystem across engines, graphics, audio, and physics.
- https://arewegameyet.rs/

**Are We Web Yet?** — Tracks web development in Rust covering frameworks, HTTP, databases, and templating.
- https://www.arewewebyet.org/

**lib.rs visualization category** — Curated, quality-ranked listing of Rust visualization crates.
- https://lib.rs/visualization

---

## Quick-reference naming notes

Some crates have naming discrepancies between crates.io, Rust import names, and GitHub repos:

| crates.io name  | Rust import      | GitHub repo                   |
|-----------------|------------------|-------------------------------|
| `tiny-skia`     | `tiny_skia`      | `linebender/tiny-skia`        |
| `cosmic-text`   | `cosmic_text`    | `pop-os/cosmic-text`          |
| `ab_glyph`      | `ab_glyph`       | `alexheretic/ab-glyph`        |
| `arrow`         | `arrow`          | `apache/arrow-rs`             |
| `colorgrad`     | `colorgrad`      | `mazznoer/colorgrad-rs`       |
| `contour`       | `contour`        | `mthh/contour-rs`             |
| `delaunator`    | `delaunator`     | `mourner/delaunator-rs`       |
| `pdf-writer`    | `pdf_writer`     | `typst/pdf-writer`            |
| `wasm-bindgen`  | `wasm_bindgen`   | `rustwasm/wasm-bindgen`       |

Several crates live in monorepos: **usvg** is inside `linebender/resvg`, **web-sys** is inside `rustwasm/wasm-bindgen`, and **egui_plot** was extracted from the egui repo into its own repository at `emilk/egui_plot`. The `arrow` crate on crates.io is published from the `apache/arrow-rs` GitHub repo, which also contains sub-crates like `arrow-array`, `arrow-buffer`, and `parquet`.
