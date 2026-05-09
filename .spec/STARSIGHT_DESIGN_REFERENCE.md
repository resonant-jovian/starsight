# Starsight Design Reference

A compiled synthesis of three research passes on what makes a good scientific plotting library. Pass 1 surveyed the major library landscape (matplotlib, ggplot2, Vega-Lite, Makie, Plot, D3, plotly, Bokeh, gnuplot, PGFPlots, ECharts, Datashader, plotters). Pass 2 added the academic foundation (Bertin → Cleveland → Munzner → Heer; library-author retrospectives; uncertainty visualization; GPU/terminal/accessibility/animation; reactive dataflow; LLM-to-chart). Pass 3 added empirical numbers, infrastructure, and underweighted territory (Arrow/Polars/DataFusion; Makie's compile-time saga; provenance; RTL/CJK; ArviZ; misleading-chart detection; sustainability).

This document compiles the actionable conclusions and resolves tensions between passes. It is the design reference for starsight v0.1 → v1.0.

---

## Executive summary

**The thesis.** No Rust library combines grammar-of-graphics architecture with: (a) Apache Arrow as the canonical data substrate; (b) dimensional types for axes; (c) first-class uncertainty as an encoding channel; (d) RTL/CJK and locale-aware text out of the box; (e) automatic provenance; (f) misleading-chart linting; (g) backends spanning terminal cascade, vector publication, GPU interactive, and PDF/A-3u archival. Each is independently valuable; together they describe a position no current library holds and that the Rust/Arrow/wgpu/icu4x stack uniquely enables in 2026.

**The architectural commitments.** Layered crates with explicit Munzner-level declarations. `starsight-spec` as a serde-friendly AST that is the durable artifact. Arrow as the only column primitive (zero-copy, never owned). Monomorphize over scalar/column types; dynamic-dispatch over plot pieces (avoiding Makie's compile-time failure mode). Backends as protocol via `Renderer` trait. Static-first with optional reactive layer. Dimensional types via `uom`-style phantoms. Provenance auto-attached to every figure.

**The roadmap.** Phase 0: lock foundations (Arrow, units, spec format). Phase 1: ship Cairo backend with CJK/RTL, linter, and provenance. Phase 2: add wgpu for >1M points; anywidget for Python notebooks; terminal cascade; ArviZ-equivalent. Phase 3: NumFOCUS path, optional dual licensing, ecosystem partnerships.

---

## Part I: Foundations

### 1. The theoretical lineage

The field has a load-bearing intellectual stack that any serious plotting library either inherits explicitly or reinvents implicitly. Starsight inherits explicitly.

**Bertin (1967, *Sémiologie Graphique*)** identified two planar variables (x, y) and six retinal variables (size, value, texture, color, orientation, shape), each with associativity properties. This was typological — the first systematic decomposition of "graphic" into components.

**Cleveland and McGill (*JASA* 1984)** added empirical experiments, producing the perceptual-task hierarchy: position-on-common-scale outperforms position-on-non-aligned, which outperforms length, which outperforms angle/slope, which outperforms area, which outperforms volume/curvature, which outperforms shading/saturation. Heer and Bostock (CHI 2010) replicated this via Mechanical Turk; Kale et al. (TVCG 2023) refined it with Bayesian multilevel models showing meaningful individual variation but a stable population ordering.

**Mackinlay (*ACM TOG* 1986, "APT")** used these rankings algorithmically, automatically generating presentations by ranking encodings against task. This is the proximate ancestor of Vega-Lite's compiler, Tableau's "Show Me," Voyager (Wongsuphasawat & Heer 2016), and Draco (Moritz et al. 2019), which formalizes encoding choice as constraint satisfaction with weighted soft constraints derived from Cleveland-McGill.

**Wilkinson (*The Grammar of Graphics*, Springer 1999/2005)** provided the syntactic decomposition: any chart is a composition of data, transforms, scales, geometries, statistics, coordinates, aesthetics, and facets. Without this grammar, plotting libraries are ad hoc collections of chart-type functions.

**Wickham (*JCGS* 2010, "A Layered Grammar of Graphics")** refined Wilkinson by making layers first-class compositional units — each with its own data/aesthetic/stat/geom/position — and providing a hierarchy of defaults so any layer component can be omitted. ggplot2 is the implementation; the design document is essential reading.

**Munzner (*TVCG* 2009, "A Nested Model for Visualization Design and Validation"; *Visualization Analysis & Design*, 2014)** provided the four-level evaluation framework: domain problem → data/task abstraction → idiom → algorithm. This is the canonical organizing principle for an opinionated library. Use Munzner's levels as documentation labels for every starsight crate. The "what-why-how" framing is the lingua franca of contemporary InfoVis pedagogy.

**Heer and Shneiderman (*CACM* 2012, "Interactive Dynamics for Visual Analysis")** gave the 12-task interactive taxonomy: visualize, filter, sort, derive, select, navigate, coordinate, organize, record, annotate, share, guide. This is the checklist for what an "interactive backend" actually means.

### 2. Library-author retrospectives

The retrospectives reveal failure modes that headline comparisons miss.

**John Hunter on matplotlib.** The explicit philosophy was an imperative MATLAB-style convenience layer above an object-oriented core; "histograms shouldn't require objects." Its persistence is not a defect but the answer to *why* matplotlib has not been displaced. When the goal is "histogram of this array, now," the API is hard to beat. The corollary is that the OO `Axes`/`Figure` core *is* the grammar; pyplot is convenience. Starsight needs an equivalent terse top layer above the grammar — a `plot!()` macro that returns a manipulable `Figure`, not a black box.

**Hadley Wickham on ggplot2.** The 2010 layered-grammar paper makes explicit what ggplot2 fixed in Wilkinson: layers as compositional units, hierarchical defaults so any component is optional, faceting as orthogonal to layering, `+` as syntactic sugar. The known weak points are catalogued in the critique literature: ternary plots, Sankey, network/hierarchical layouts, non-Cartesian coordinate systems. The grammar isn't broken, it's *incomplete*. Extension packages (ggtern, ggraph, ggalluvial, ggalign) address each gap. Starsight should make `Coord` a trait, not an enum, so coordinate systems are pluggable.

**Mike Bostock on D3 → Plot.** The 2011 *D3: Data-Driven Documents* TVCG paper argues for *representational transparency*: D3 binds data to DOM nodes rather than maintaining an abstract scene graph, which improves debuggability — a deliberate reaction against Protovis's implicit re-evaluation closures. The "Future of Data Work" Q&A frames the *ladder of abstraction*: WebGL fragment shaders at the bottom, D3 above, Plot above that, Shneiderman-mantra GUIs at the top — the goal is to climb up and down without restarting.

**Simon Danisch on Makie.** The architecture is correct (reactive Observables core; multiple backends sharing recipe code; separable layout via GridLayoutBase; recipes as the integration mechanism). But the *Julia execution* of that architecture revealed a precise warning about monomorphization that translates directly to Rust. GitHub issues #792, #654, #1636 and others document multi-second compilation latencies and multi-GB memory blowups during inference of single methods like `draw_axis2d` (4.7 seconds, 681 MiB, 11.49M allocations for a 43-argument function). The pattern was over-specialization on `StaticArrays`/`GeometryBasics` types. **Lesson for starsight: monomorphize at the data-type boundary; erase at the geometry/style/aesthetic boundary.** Use `&dyn Drawable` for plot pieces, generic only over the numeric scalar `T: Float` and the Arrow column type. Avoid `nalgebra::SMatrix<f32, R, C>`-style fixed-size generics in public API surfaces.

**Anaconda/HoloViz (Bednar, Rudiger).** The explicit thesis: no single library suffices, and the ecosystem must be layered — Bokeh for interactive, matplotlib for publication, Datashader for big-data rasterization, Panel for assembly, hvPlot as the unifier. This validates the layered-crates instinct as the only known sustainable approach to coverage.

### 3. The grammar of graphics: power and limits

The grammar of graphics is the highest-leverage organizing principle in the field. Once a chart decomposes into marks, stats, scales, coords, and facets, adding a new mark type is local — it does not require changes elsewhere. ggplot2 extensions (ggrepel, ggraph, ggalign) are easy to write because they only add new geoms or stats. Vega-Lite's compiler synthesizes legends, scales, and tick positions from declared encodings — the user never asks for a legend and one appears, correctly, every time.

The limits are documented and real. The Cartesian assumption fails for ternary, parallel-coordinate, polar with full angular sweep, and geographic projections beyond the basic. The mark-as-glyph assumption fails for graph viz (where edges are *between* glyphs and the layout problem dominates the rendering problem) and for Sankey/sunburst/treemap (where the "mark" is a layout cell, not a glyph). The flat-table assumption forces awkward long-form pivots for naturally hierarchical data. The static-encoding assumption requires extension to handle uncertainty, animation, and interaction.

Starsight inherits the grammar and addresses its limits structurally: pluggable `Coord`; first-class `RasterMark` and `LayoutMark` for non-glyph cases; uncertainty as an encoding channel; animation as either a data channel or an event stream (per Animated Vega-Lite).

---

## Part II: The design space

### 4. Library landscape (best-in-class)

Each library represents a design philosophy. Best-in-class assignments below identify what each one genuinely won at, separated from what it disappoints.

**matplotlib.** Best at ecosystem reach and format coverage. Disappoints on API. Lesson: ecosystem integration matters more than API beauty; longevity comes from escape-hatchability.

**ggplot2.** Best at layered grammar and extensibility. Disappoints at performance and 3D. Lesson: invest in the grammar; harvest extensibility forever.

**Vega-Lite.** Best at the formal grammar of interactive graphics and at compiler-synthesized defaults. Disappoints because JSON is poor authoring (Altair wraps it for this reason). Lesson: defaults that *compose* outperform defaults that look pretty; selections-as-predicates is the right interactivity model.

**Makie.jl.** Best at unified 2D/3D pipelines and scientific 3D. Disappoints at compile-time latency. Lesson: unified frontend with multiple backends works; auto-growing layout grammar (`f[1,1]`, nested `f[1:2, 2][2, 1:2]`) is more ergonomic than `subplot(2,3,4)`; recipe systems for domain types are how you get ecosystem buy-in.

**Observable Plot.** Best at the modern declarative-with-escape-hatches synthesis. Lesson: the ladder-of-abstraction framing is the right design test — a user must be able to ascend or descend without rewriting from scratch.

**D3.js.** Best at primitive composition and bespoke visualization. Disappoints at time-to-first-chart. Lesson: a primitive layer that is genuinely complete is enormously valuable as a foundation; D3's primitives are so complete that two completely different higher-level libraries (Vega and Plot) sit on top without modification.

**plotly.** Best at interactive 2D/3D out of the box. Disappoints because the SVG/WebGL split is leaky and the Express/Graph Objects split mirrors matplotlib's pyplot/OO split. Lesson: dual-backend architectures are necessary at scale but require painful design work to make the swap transparent.

**Bokeh.** Best at server-driven applications. Lesson: if you want server-driven dashboards, commit to a wire protocol between authoring and rendering early.

**gnuplot.** Best at command-line scripting and batch processing. Lesson: separation of data generation from visualization is real value; reproducibility — a script that produces identical output years later — matters disproportionately for scientific users.

**PGFPlots/TikZ.** Best at publication quality and math typography. Disappoints past ~10,000 points. Lesson: math typography matters more in scientific publishing than data-app libraries weight; fonts matter more than colors for publication credibility.

**Datashader/HoloViews.** Best at large-data visualization (billion points in seconds). Lesson: at the billion-point scale, the design moves from "draw each mark" to "compute a 2D histogram of marks per pixel and shade." Different code path; cannot be retrofitted.

**plotters (Rust).** Best at pure-Rust drawing with pluggable backends. Disappoints because there is no grammar layer. Lesson: the backend trait architecture is sound; what plotters chose *not* to build (a marks-and-stats grammar) is precisely what starsight's value-add is.

**ECharts.** Best at feature breadth and polish. Lesson: a long-tail of mark types is achievable if there is a good extensibility hook (ECharts' `renderItem` callback; ggplot2's `geom` system; Makie's `@recipe`).

**Less-cited but instructive.** GR Framework (GKS-based; >15 backend drivers from one core; existence proof that abstract device interfaces scale to print, raster, vector, GUI, web, animation). CERN ROOT (typed histogram hierarchy with explicit overflow/underflow bins; weighted histograms; the canvas/pad model). Trellis (Becker, Cleveland, Shyu *JCGS* 1996 — the original small-multiples paper; faceting in ggplot2 is the direct descendant). VegaFusion (proves push-down to DataFusion plus zero-copy Polars input is the durable architecture for large-data Vega-Lite). Mosaic (UW IDL; DuckDB-backed; outperforms VegaFusion and pure browser rendering by orders of magnitude on static plots — including DuckDB-WASM in-browser).

### 5. Design axes and tradeoffs

Every plotting library can be characterized along orthogonal axes. Naming them makes the analysis tractable.

**API style.** Imperative (matplotlib pyplot) versus declarative (ggplot2, Vega-Lite, Plot, Altair). Modern libraries are hybrid: high-level declarative entry plus low-level imperative one. Plotly Express plus Graph Objects is the clearest example. Starsight should follow this pattern: a `plot!()` macro at the top, a `Figure` builder below, marks/scales/coords below that.

**Composition.** Layered (ggplot2, Vega-Lite, Makie) versus monolithic (Chart.js, early ScottPlot). Layered systems eliminate combinatorial explosion and reward investment in primitives.

**Rendering target.** Vector (SVG, PDF, PostScript) versus raster (PNG, in-memory pixmap). The best scientific libraries support both transparently. Starsight commits to both via separate backends.

**Backend topology.** Single (gnuplot, in practice) versus pluggable (plotters, Makie, matplotlib). Pluggable means rendering and pipeline are decoupled. Starsight commits to pluggable via the `Renderer` trait.

**Performance ceiling.** SVG-bound libraries cap around 10⁵ marks. Canvas/Cairo reaches a few hundred thousand. WebGL/wgpu reaches 10⁶ at 60 FPS, low 10⁷ at degraded frame rates. Server-side aggregation pipelines (Datashader, Mosaic) reach 10⁹+. The architecture choice locks in the ceiling; you cannot retrofit.

**Interactivity model.** None (PGFPlots), post-hoc (matplotlib pan/zoom), first-class (Vega-Lite selections; Makie Observables; Bokeh server). First-class interactivity is the only one that composes; selections-as-predicates is the right model.

**State management.** Stateless (`plot(x, y)`) versus builder-with-state versus reactive. Reactive is the only foundation for live dashboards but adds significant complexity. Starsight's reactive layer should be optional, behind a feature flag.

**Type discipline.** Dynamic (Python, JavaScript) versus statically typed (Makie via multiple dispatch, plotters via Rust). The hard part is making aesthetic mappings ergonomic — `aes(x = "sepal_length", color = "species")` works in R because of lazy evaluation; in Rust the equivalent requires explicit column references but can still be ergonomic via Polars' `col()`.

**Scope.** Chart library (fixed taxonomy) versus toolkit (primitives composed into charts). D3 is explicitly not a charting library. The toolkit approach maximizes flexibility but raises the floor.

**Mutually exclusive or nearly so.** Pure-vector output and billion-point performance (SVG cannot represent a billion shapes; PDF cannot either). Publication-grade math typography and runtime-flexible authoring (PGFPlots executes inside LaTeX; nothing else matches). Stateless functional API and reactive live updates. Single API surface and full backend flexibility. Tiny dependency footprint and complete out-of-box experience.

**Strongly correlated.** Grammar of graphics architecture and easy extensibility. Declarative API and good defaults. Color science correctness and scientific user buy-in. Snapshot testing infrastructure and architectural quality.

**Achievable together with effort.** Static publication output and interactive exploration (Makie demonstrates). Layered architecture and fast time-to-first-plot (ggplot2 demonstrates). Type safety and ergonomic data binding (achievable via lifetime-based binding plus generics over Arrow columns).

### 6. What scientific use weighs most

After surveying the libraries and the academic record, here is the ranking of which dimensions matter most for scientific visualization.

1. **Correct, perceptually uniform default colormaps and scientific honesty.** Highest stakes because bad colormaps mislead. Jet/rainbow create false discontinuities, hide structure in dark regions, exclude colorblind users. Starsight via prismatica is in good shape.

2. **Publication-quality static output.** PDF, SVG, PNG at high DPI. Scientific work eventually goes into a paper. PDF/A-3u with embedded source data is a structural archival upgrade no Python plotter currently does well.

3. **Grammar-of-graphics layered architecture.** Long-tail chart types no library can pre-implement (phase portraits, Hovmöller diagrams, ridgeplots, ROC curves, persistence diagrams) become expressible from primitives.

4. **Faceting and small multiples.** Tufte-Cleveland-Becker tradition. Scientific datasets are usually multi-conditional and small multiples are the right comparison structure.

5. **Reproducibility and deterministic output.** Scientific users care disproportionately about reproducing figures years later. Requires deterministic font rendering, stable color quantization, locale-independent number formatting.

6. **Performance up to ~10⁷ points.** Most scientific visualizations are well under this. Beyond it, rasterization becomes necessary; the architecture should not preclude it.

7. **Coordinate system flexibility.** Cartesian, log, polar, geographic projections, ternary, parametric. Pluggable `Coord` trait.

8. **Math and unicode in labels.** Greek letters, subscripts, integrals, chemical formulas. Cosmic-text plus careful font stack handles this.

9. **Animation for time-series and parameter sweeps.** Useful but secondary. Most publications still use static figures.

10. **Live-updating and dashboard integration.** Lower priority for scientific use. Optional layer behind a feature flag.

The first six are non-negotiable for scientific credibility. The remaining four are differentiators where strategic investment pays off.

---

## Part III: Underweighted ground

These are the territories where starsight has the clearest competitive opportunity — areas where existing libraries are weak and where the Rust/Arrow/wgpu/icu4x stack offers genuine advantages.

### 7. Uncertainty as a first-class encoding channel

This is the largest single underweighted topic in plotting library design.

The Hullman-Kay-Correll-Padilla research program established the case. Hullman's "Why Authors Don't Visualize Uncertainty" (IEEE VIS 2020) documents that authors who acknowledge uncertainty's importance routinely omit it — a tooling failure, not a cultural one. Most grammars treat error bars as a `geom_errorbar` afterthought rather than a first-class channel.

The taxonomy of uncertainty visualization (Padilla, Kay, Hullman 2020 review):
- *Interval/boundary*: error bars, confidence bands.
- *Density*: violins, gradient ribbons.
- *Hypothetical outcome plots (HOPs)*: animated frame sampling. Hullman, Resnick, Adar (PLOS ONE 2015) and Kale, Nguyen, Kay, Hullman (TVCG 2018) showed HOPs beat error bars and violins for multivariate probability judgments and trend reliability.
- *Quantile dotplots*: static frequency framing. Kay, Kola, Hullman, Munson (CHI 2016, "When (ish) is my bus?") demonstrated for static media.
- *Ensemble*: multiple sample lines or surfaces.

**Starsight implication.** Treat uncertainty as a typed channel where any quantitative encoding has an optional parameterization carrying a distribution, samples, an interval, or a standard error. The renderer chooses the visual idiom based on backend capability and user instruction. Concretely:

```rust
let plot = scatter()
    .x(times.with_uncertainty(Uncertainty::standard_error(times_se)))
    .y(distances.with_uncertainty(Uncertainty::samples(distance_posterior)));
```

This is a grammar-level commitment, not a `geom_uncertainty` afterthought. The `starsight-stats` and `starsight-bayes` crates extend this: ArviZ-equivalent plots (`plot_ppc`, `plot_loo_pit` ECDF-difference, `plot_rank` fractional rank histograms, `plot_ess` quantile ESS, `plot_pareto_k` PSIS-LOO diagnostics, prior/posterior power-scaling sensitivity) become primitives over the uncertainty channel. There is no good Rust answer for ArviZ today; this is a defensible vertical.

### 8. Color science, dimensional types, and scientific honesty

**Color.** Default sequential colormap should be viridis or a Crameri scientific colormap from prismatica. Default categorical palette should be Paul Tol's qualitative or Wong's eight-color (the most-recommended colorblind-safe categorical palette, published in *Nature Methods*). Cividis (Nuñez, Anderton, Renslow) is perceptually uniform in hue and brightness, increases linearly in brightness, and is nearly identical for those with and without red-green colorblindness — a stronger choice than viridis where the only goal is colorblind safety.

**Color management.** Default to sRGB-tagged output. Embed a sRGB ICC v4 micro profile (~400 bytes from the Compact-ICC-Profiles repo) in PNG output. Opt-in Display-P3 mode for wide-gamut displays. HDR plotting (Rec. 2020 + PQ/HLG) is a 5-year horizon; ignore for now.

**Dimensional types.** The `uom` crate (~7M downloads, no_std-capable, full SI/ISQ support, supports `usize..i128`, BigRational, complex64) demonstrates zero-cost dimensional analysis using phantom types — the entire dimensional algebra is a compile-time monoid. The yaiouom proof-of-concept extends this to a clippy-style refinement-type checker. Make `Quantity<D, U, V>` a first-class input to axes. A `ContinuousScale<Length>` and a `ContinuousScale<Time>` are different types; auto-format the unit on the axis label; reject `plot(km_series, sec_series)` at the type level *unless* a user wires up an explicit affine map. This is correctness at the type level; no other plotting library does this.

**Scientific honesty defaults.** Y-axes never auto-truncate for bar charts (Pandey, Rall, Satyanarayan, Adar CHI 2015 documents the deception effect; Yang et al. 2021 and Rho et al. CogSci 2024 confirm it persists across chart types and after explicit warnings). Confidence/credible intervals shown by default if input has uncertainty metadata. Histograms always show overflow/underflow bins (ROOT convention). The Misviz benchmark (arXiv 2508.21675) and McNutt-Kindlmann's *VisuaLint* (CGF 2020) provide a corpus of misleading-chart heuristics; ship a built-in linter that runs against the spec at render time and refuses to render the worst-offender chart types without an explicit `acknowledged_misleading: true` flag.

### 9. Accessibility as structural, not cosmetic

Accessibility is a structural concern, not an `alt_text=` parameter.

The W3C **ARIA Graphics Module** (2018) standardized roles for `graphics-document`, `graphics-object`, `graphics-symbol`. These are the semantic primitives for SVG charts, not generic `img` with alt-text.

**MIT Olli library** (Zong, Lundgard, Chan, Choi, Subramonyam, Satyanarayan; ASSETS 2022, "Rich Screen Reader Experiences for Accessible Data Visualization"): turns any chart into a tree of axes/legend/marks/data-points, navigable via keyboard with screen-reader output. Sharif et al. (CHI 2021) measured the cost of failing at this: 211% time overhead and 61% accuracy drop for screen reader users on web charts.

**Microsoft Chart Reader** (Thompson, Martinez, Sarikaya, Cutrell, Lee; CHI 2023): hierarchical keyboard navigation plus sonification plus cross-cutting insights, co-designed with blind/low-vision Microsoft employees. Validates the multimodal approach. Highcharts' Sonification Studio and Apple's Audio Graphs API show commercial markets validating sonification.

**Starsight implication.** SVG backend emits structured semantic tree (axes, legend, marks, data points) with ARIA roles. API accepts a `description` slot at every grammar level (chart, layer, encoding) so screen-reader output is generated, not hand-authored. Sonification hook in the interactive backend pitch-maps quantitative axes.

### 10. Internationalization and locale

This is one of starsight's clearest competitive opportunities and is essentially unaddressed in the existing scientific plotting ecosystem.

**The state of the field.** matplotlib has no native RTL support (open issue #30557, October 2025); users compose `arabic_reshaper` and `python-bidi` manually, which then breaks LaTeX expressions. CJK vertical text requires explicit handling for fullwidth punctuation, emphasis-mark normalization, kinsoku line-break rules, ruby annotations, tate-chu-yoko, warichu, and shatai obliques. matplotlib falls back to "set the font and pray." The 2026 Python ecosystem has no clean answer.

**The Rust opportunity.** `rustybuzz` (pure-Rust HarfBuzz port) provides full OpenType shaping including kerning, ligatures, contextual alternates, language-tagged glyphs via the `'locl'` feature. `cosmic-text` provides text layout with bidi, line-break, and font-fallback. `icu4x` (Rust port of ICU4X, used by Servo) provides locale-aware number formatting, message formatting, and bidirectional text. Together these enable a pure-Rust text pipeline that handles every major writing system correctly.

**Starsight commitment.** Ship RTL, CJK, and locale-aware number formatting in v1.0 as a launch differentiator. Default to fontconfig fallback chain including Noto Sans/Serif CJK and Noto Sans Arabic. Detect locale from `LANG`/`LC_ALL`; respect decimal separator and digit grouping (`1,234.5` en-US vs `1.234,5` de-DE vs `1 234,5` fr-FR with NBSP vs `1٬234٫5` Arabic). Validate against the W3C i18n test suite before claiming correctness.

This positions starsight as the only mainstream scientific plotting library that works correctly out of the box for a Persian climate scientist or a Japanese physicist labeling axes in their native script.

### 11. Provenance and reproducibility

Scientific plotting libraries underweight this dramatically.

**The reference work.** Alpaca (Sprenger et al., NeuroInformatics 2024) auto-records inputs, function parameters, and outputs to W3C PROV-JSON with minimal user intervention. yProv4DV (arXiv 2603.20437, 2026) packages source code, inputs, execution context, and outputs as RO-Crate per figure call. trackr (Becker et al.) and VisTrails are earlier work in metadata-annotated artifacts. pylustrator does the inverse: records GUI manipulations and emits Python source.

**Starsight commitment.** Attach a `Provenance` struct to every `Figure`, populated automatically with: spec hash (deterministic), input data hashes (xxhash on Arrow buffers), library version, host OS, timestamp, and an opaque user-extensible `metadata: BTreeMap<String, Value>`. Serialize to PROV-JSON or RO-Crate on `save()`. Concretely, `fig.save("plot.svg")` also writes `plot.svg.prov.json` next to it; `fig.save("plot.pdf")` produces PDF/A-3u with embedded provenance plus parquet of input data. This is a small amount of code that makes scientific reproducibility a property of the library, not a discipline of the user.

**Git-friendly output.** SVG with sorted attributes and fixed decimal precision produces consistently diffable text. PDFs from matplotlib are non-deterministic by default (timestamps, font subsets); set `SOURCE_DATE_EPOCH` or use deterministic metadata. Starsight's SVG backend defaults to deterministic, sorted-attribute, fixed-precision output.

**PDF/A-3u (ISO 19005-3)** is now the archival profile of choice for journals because it allows embedded files (CSV of plot data, LaTeX of math labels, source script). PLOS ONE has shown 3D PDF figures via PRC/U3D as a viable archival pattern. Starsight's PDF backend targets PDF/A-3u with the spec embedded as `application/json` and source data as `application/vnd.apache.parquet`. veraPDF can validate. Every starsight PDF is self-archiving.

### 12. Performance: empirical ceilings

Vendor-stated numbers are reliable for orders-of-magnitude characterization. The concrete ceilings:

- **deck.gl** (production WebGL/WebGPU): ~1M points at 60 FPS for ScatterplotLayer; 10–20 FPS at 10M; browser hard-crash between 10M–100M owing to Chrome's 1 GB single-allocation cap. Picking is bounded at 16M items per layer. Binary `Float32Array` attribute upload is the only way to avoid per-render accessor recomputation.
- **Datashader**: a billion points in seconds via bin-aggregate-shade pipeline. Trades vector output entirely.
- **VegaFusion 2.0**: zero-copy Polars input via `arro3`; native DataFusion ops replace custom UDFs; transforms can push down to DuckDB/Postgres.
- **Mosaic** (UW IDL, DuckDB-backed): outperforms VegaFusion and pure browser rendering by orders of magnitude on static plots. DuckDB-WASM in-browser approaches the same performance modulo WASM's lack of parallel processing.
- **Cairo / matplotlib Agg**: reaches a few hundred thousand marks before degrading.
- **SVG**: caps around 10⁵ marks before browser performance collapses.

**Memory layout.** A 24-byte `(Vec3, _)` AoS layout straddles cache lines. AoSoA tile size = SIMD width is the empirical sweet spot for both CPU SIMD and GPU coalescing. Starsight's vertex buffers should be SoA-by-default with a documented `pack_aosoa::<8>()` adapter for AVX2.

**Compile-time cost.** Every `derive(Serialize, Deserialize, Debug, Clone)` and every blanket trait impl increases monomorphization time. The Bevy/wgpu communities split traits into `*-trait` crates (no impls) and `*-impl` crates (blanket). Apply this to starsight: keep aesthetics, scales, and the spec AST in a tiny no-deps crate; rendering, IO, and statistics depend on it. Target ≤ 5s for a debug build of an example "scatter from CSV" using default features. Gate Cairo, GPU, terminal, and serde-yaml behind features.

**Build-time prescription.** `starsight-spec` < 0.5s; `starsight-core` < 2s; `starsight` (full default) < 8s on a 2025 laptop. Continuous regression test via `cargo bench --bench compile_time`.

---

## Part IV: Synthesis for starsight

### 13. Crate architecture

```
starsight-spec       AST: Plot, Layer, Scale, Mark, Aesthetic, Legend, Coord
                     Deps: serde + uom traits only. Compile <0.5s.
starsight-core       Engine: layout (Cassowary via casuarius), scales, statistics
                     Deps: starsight-spec + arrow + polars-core. Compile <2s.
starsight-stats      Histograms (with overflow), KDE, ECDF, CIs, Bayesian primitives
starsight-render     Trait: Renderer; geometry + tessellation
starsight-text       Text shaping (rustybuzz + cosmic-text + icu4x for bidi/locale)
starsight-prismatica Already exists: compile-time scientific colormaps
starsight-chromata   Already exists: compile-time editor themes
starsight-cairo      Cairo backend (raster + SVG + PDF/A-3u)
starsight-wgpu       GPU backend (instanced quads + MSDF text + zero-copy Arrow upload)
starsight-term       Terminal cascade (Unicode → Braille → Sixel → Kitty)
starsight-svg        Pure SVG (deterministic output for Git diffs)
starsight-anywidget  Notebook integration via AFM standard
starsight-bayes      ArviZ-style PPCs, trace plots, rank plots, ESS/MCSE, PIT-ECDFs
starsight-graph      Force/hierarchical/edge-bundling layouts (depends on petgraph)
starsight-geo        Geospatial: PROJ bindings, MVT/PMTiles, CF conventions
starsight-provenance Auto-record per-figure metadata to PROV-JSON / RO-Crate
starsight            Convenience reexports + opinionated default
```

Each crate declares its Munzner level(s) in its README: domain → abstraction → idiom → algorithm. The `starsight-traits` micro-crate (sketched here within `starsight-spec`) plays the role RecipesBase plays in Julia: zero-deps, defines `IntoLayer`/`IntoMark`/`Recipe`, depended on by both backends and downstream consumers.

### 14. API axioms

**Axiom 1: Arrow is the only column primitive.** Take `&dyn arrow::array::Array` everywhere; reject any internal `Vec<f64>` path. Provide a `From<Vec<f64>>` blanket impl that wraps in `Float64Array` at zero cost for casual use, but the canonical type is Arrow.

**Axiom 2: Monomorphize over data, dynamic-dispatch over plot pieces.**

```rust
// Good: one specialization per scalar/column type
pub trait Renderer {
    fn render(&mut self, scene: &Scene) -> Result<(), Error>;
}
pub struct Scene {
    pub items: Vec<Box<dyn Drawable>>,  // erased
    pub viewport: Viewport,
    pub theme: Theme,
}

// Bad (Makie failure mode replicated):
pub struct Plot<T, X, Y, M, S, A1, A2, A3, ...> { ... }  // type explosion
```

**Axiom 3: Units are mandatory at API edges.**

```rust
use uom::si::f64::*;

pub fn scatter<X: Quantity, Y: Quantity>(
    x: impl IntoIterator<Item = X>,
    y: impl IntoIterator<Item = Y>,
) -> ScatterBuilder<X, Y> { ... }
```

A user cannot accidentally pass mixed units. Axis labels are auto-set from the unit. Uncertainty is automatically rendered if metadata is present.

**Axiom 4: The spec is the durable artifact.** `Chart` is `serde::Serialize`. JSON-Schema-validated, deterministically serialized, content-hashable. The fluent builder is one of several construction front-ends; the spec is the source of truth. This unlocks reproducible figures, LLM-authored plots, cross-language bindings, Quarto/Jupyter notebook integration, and meaningful version-control diffs.

**Axiom 5: Provenance is automatic, not opt-in.**

```rust
let fig = scatter(...).render();
fig.save("plot.svg")?;  // also writes plot.svg.prov.json next to it
fig.save("plot.pdf")?;  // PDF/A-3u with embedded prov + parquet of input data
```

**Axiom 6: Reactive is signals + dataflow, not callbacks.** When the reactive layer ships, model it on Reactive Vega + Makie Observables + leptos signals. Event streams as data, not event handlers.

### 15. Backend strategy

**Vector (Cairo + SVG + PDF/A-3u): ship first.** Scientific users need print-quality vector output more than 60 FPS scatter of 10M points. Polars/matplotlib usage data confirms the ratio. Use `tiny-skia` for pure-Rust path; consider `skia-safe` or Cairo for PDF text quality (font subsetting is a pain in pure-Rust paths).

**GPU (wgpu): ship after vector is stable.** SDF marks in a texture atlas, instanced quads, compute shaders for binning/density. Direct Arrow upload as `Float32Array`-equivalent buffers. WebGPU is now in Chrome (since 113), Edge (113), Safari (since 17.4), and Firefox (Nightly); compiling to `wasm32-unknown-unknown` plus WebGPU via wgpu is an obvious deployment target where Rust is uniquely well-suited.

**Terminal cascade.** Probe at runtime: Unicode blocks → Braille (2×4 dots/cell mono) → Sixel (palette-limited, broad support) → Kitty/iTerm2 (full RGBA). Expose the level explicitly for forced lowest-common-denominator output (CI logs).

**Datashader-style rasterization.** A per-geom option for >10⁶ points; implement on wgpu compute shaders. Plan for it now; ship it when needed.

### 16. Defaults aligned with the empirical literature

- Y-axes never auto-truncate for bar charts. Refuse, citing Pandey 2015.
- Confidence/credible intervals shown by default if input has uncertainty metadata.
- Locale auto-detected from `LANG`/`LC_ALL`; decimal separator and digit grouping respected.
- RTL text shaped natively via `rustybuzz`; no `arabic_reshaper` required.
- CJK fullwidth punctuation, emphasis marks, kinsoku line-breaks handled correctly.
- Histograms always show overflow/underflow bins (ROOT convention).
- Color-blind-safe colormaps from prismatica are the defaults.
- Default raster output is 300 DPI for `save("fig.png")` rather than matplotlib's 100 DPI.
- WebP-lossless as the default raster web format (25% smaller than PNG).
- PDF/A-3u with embedded source data as the default PDF.
- SVG output is deterministic, sorted-attribute, fixed-precision for Git diffs.
- Viridis-equivalents only for sequential colormaps; no jet/rainbow shipped.

### 17. Roadmap (phased with revisit thresholds)

**Phase 0 (now, before v0.1): lock the foundations.**
1. Adopt Arrow as the only column primitive.
2. Bake uom-style phantom units into scales.
3. Decide the spec format: JSON-Schema-validated, deterministically serialized, content-hashable.

*Revisit if:* 30%+ of users want `Vec<f64>` ergonomic input — ship a `From` blanket impl that wraps zero-cost.

**Phase 1 (v0.1–v0.5): single backend, broad correctness.**

4. Ship `starsight-cairo` first (raster + SVG + PDF/A-3u).
5. Ship CJK + RTL support in v0.1. Make this a launch differentiator.
6. Ship the misleading-chart linter (Misviz-inspired).
7. Ship the provenance writer.
8. Ship `starsight-stats` with histograms (overflow/underflow), KDE, ECDF, CIs.

*Revisit GPU priority if:* a benchmark on a real scientific dataset (ZTF DR23 stellar catalog ~3 billion rows; NCEP reanalysis) takes >500ms in Cairo for a single-frame render. Until then, GPU is optional.

**Phase 2 (v0.5–v1.0): performance and ecosystem.**

9. Add `starsight-wgpu` with binary Arrow upload. Target deck.gl numbers as reference: 1M points @ 60 FPS minimum on integrated graphics.
10. Add `starsight-anywidget` for Jupyter/marimo. Unlocks the entire Python notebook ecosystem with one crate.
11. Add `starsight-term` for Unicode/Braille/Sixel/Kitty cascade.
12. Add `starsight-bayes`. Unique value proposition: no other Rust library covers ArviZ.
13. Optional reactive layer behind a feature flag.

*Revisit graph/geo timing if:* `starsight-bayes` has shipped and ≥50 GitHub Sponsors back the project. Graph and geo are deep specialty domains; do not divert focus until the core is stable.

**Phase 3 (v1.0+): sustainability.**

14. Apply for NumFOCUS Affiliated Project status; once stable, apply for sponsored-project status. Unlocks CZI grants and small development grants.
15. Investigate dual licensing (BSL or PolyForm Perimeter) for an enterprise feature — likely the streaming/incremental visualization piece for observability vendors. Keep `starsight-core` permissive (Apache-2.0/MIT).
16. Reach out to Hex, Posit, Polars Inc., and rerun with explicit integration proposals.

### 18. Sustainability

Pure GitHub Sponsors will fund roughly 0.2–0.5 FTE if starsight becomes well-known. The sustainable patterns observed in the field:

- **NumFOCUS sponsorship** (matplotlib's path; reduces legal/admin overhead; access to CZI Essential Open Source Software grants typically ≥$200k cycles).
- **Acquired/employed by a data-analytics startup** (Polars/Hex/Observable/VegaFusion-via-Hex model).
- **Dual licensing commercial extensions** (Highcharts model; works for Rust crates with BSL or PolyForm Perimeter).
- **Consulting on integrations**.

Successful Rust crate funding patterns: tokio-rs (Foundation + corporate sponsorship from AWS, Microsoft, Discord, Embark — ~$200k/yr); rust-analyzer (Ferrous Systems split: 80% employee time, 20% to non-FS contributors); Polars (Polars Inc. spinoff, raised seed funding 2024); rerun (Series A 2023, $5M from Costanoa Ventures); bevy (donations + Foresight Institute grants, Cart full-time); cargo-mutants and other niche tools sustained by individual GitHub Sponsors at $1–2k/month.

The starsight plan: GitHub Sponsors as supplementary, NumFOCUS as the medium-term target, optional commercial integration with a data-analytics partner as the long-term path.

### 19. What starsight is — and what it is not

**Starsight is.** A grammar-of-graphics scientific plotting library for Rust, built on Arrow with dimensional types, first-class uncertainty, locale-aware text, automatic provenance, multiple backends spanning terminal to GPU to PDF/A-3u archival. The first library to combine these — a defensible product position no current library holds.

**Starsight is not.** A medical imaging library (out of scope; integrate with `dicom-rs`/ITK). A graph visualization framework (out of scope for core; `starsight-graph` is a separate crate depending on `petgraph`). A scrollytelling framework (the spec supports multi-state encoding; the JS framework is downstream). An interactive dashboard server (the reactive layer is optional and engine-agnostic; full dashboards belong elsewhere). A drop-in matplotlib replacement (different design, different Munzner level commitments).

---

## Appendix: Cross-pass convergence and tensions

This section flags where the three passes agreed, refined each other, or surfaced genuine tensions. It is the audit trail for the synthesis.

**Convergence: layered architecture.** Pass 1 deduced layered crates from the library survey. Pass 2 confirmed via Munzner's nested model and HoloViz's explicit "no single library suffices" thesis. Pass 3 added the concrete Rust crate layout and the specific compile-time discipline (`*-trait` plus `*-impl` split) the Bevy/wgpu communities converged on. All three agree.

**Convergence: grammar of graphics.** Pass 1 ranked it as the highest-leverage organizing principle. Pass 2 traced its lineage (Wilkinson → Wickham). Pass 3 confirmed the choice while adding the concrete graph-viz exception (layout dominates rendering; ship a separate crate). All three agree.

**Convergence: scientific defaults matter more than aesthetics.** Pass 1 ranked colormaps and publication output as the top two priorities. Pass 2 added the Cleveland-McGill perceptual basis and the Bertin-Mackinlay-Munzner formalization. Pass 3 added the empirical evidence that bad defaults persist as deception even after explicit warnings (Pandey, Yang, Rho) and the Misviz/VisuaLint linters as the operationalization. All three agree, with strengthening empirical support.

**Refinement: interactivity is conditional.** Pass 1 treated interactive backends as a desirable feature. Pass 2 added the contrarian view (van Wijk; Hullman/Kale on probability communication; Lam's cost-of-interaction framework). Pass 3 confirmed via Mosaic showing static plots with server-side aggregation can outperform interactive browser rendering by orders of magnitude. The compiled stance: static-first, interactive layer optional and engine-agnostic.

**Refinement: GPU performance ceilings are concrete.** Pass 1 noted the ceiling-by-architecture pattern. Pass 2 quantified it informally. Pass 3 surfaced exact numbers (deck.gl 1M @ 60 FPS, 10M crash, 16M picking limit, Chrome 1 GB cap). Design around the concrete thresholds; do not pretend they don't exist.

**Refinement: Makie's design is sound but its execution is a warning.** Pass 1 cited Makie favorably for unified 2D/3D and layout grammar. Pass 2 added Danisch's design philosophy. Pass 3 surfaced the GitHub-issue trail of compile-latency failures (4.7s for one method, 681 MiB inference) and the precise Rust analog (avoid `nalgebra::SMatrix<f32, R, C>` in public APIs; prefer `&dyn Drawable`). The compiled lesson: Makie's architecture is the model; its monomorphization is the anti-pattern.

**Tension resolved: single API surface versus full backend flexibility.** Pass 2 listed these as mutually exclusive. Pass 3 surfaced GR Framework's GKS as an existence proof that an abstract device interface can scale to >15 backends. Resolution: it is possible *if* the API is constrained to the lowest common denominator. Starsight should declare a core feature set every backend must support and a set of optional capabilities backends advertise. Honesty about backend differences (Plotly's WebGL traces don't support area fills) is better than pretending uniformity.

**Tension resolved: server roundtrips versus local-first.** Pass 2 leaned local-first (Ink & Switch framing). Pass 3 contradicted via Mosaic showing DuckDB push-down (in-process *or* remote) outperforms both. Resolution: the right axis is "engine that can push aggregation down," not "local versus remote." Starsight's data-source abstraction is engine-agnostic; DataFusion (in-process), DuckDB (in-process or remote), and cloud SQL endpoints all look the same from the spec.

**New gap surfaced: internationalization.** Not present in Pass 1; surfaced in Pass 3. matplotlib's open RTL issue (#30557) and the absence of correct CJK vertical typesetting in any Python plotting library are unaddressed in the field. Starsight has a clear competitive opportunity here via rustybuzz + cosmic-text + icu4x. Compiled into a launch-differentiator commitment.

**New gap surfaced: provenance.** Lightly mentioned in Pass 2 (reproducibility); operationalized in Pass 3 via Alpaca/yProv4DV/PDF/A-3u. Compiled into Axiom 5 (provenance is automatic, not opt-in).

**New gap surfaced: Bayesian visualization.** Lightly mentioned in Pass 2 (uncertainty); operationalized in Pass 3 via ArviZ's 25+ specific plot types and the Gabry/Vehtari/Gelman Bayesian Workflow conventions. Compiled into the `starsight-bayes` crate as a Phase 2 deliverable.

**New gap surfaced: misleading-chart detection at render time.** Not in Pass 1. Pass 2 mentioned the perception literature on truncation. Pass 3 added the Misviz benchmark and McNutt-Kindlmann VisuaLint linter as concrete operationalizations. Compiled into the linter shipped in Phase 1.

**New gap surfaced: PDF/A-3u archival.** Not in Pass 1 or Pass 2. Pass 3 added it as a structural archival upgrade no Python plotter does well. Compiled into the default PDF backend output.

**New gap surfaced: sustainability path.** Not in Pass 1 or Pass 2. Pass 3 added the NumFOCUS / acquisition / dual-license / consulting matrix observed in the field. Compiled into the Phase 3 plan.

---

*This document is the design reference for starsight v0.1 → v1.0. It should be revisited annually as the field evolves and as starsight's own implementation reveals which assumptions held and which did not. Companion documents: STARSIGHT.md (development reference), LEARN.md (TTS-compatible teaching narrative).*
