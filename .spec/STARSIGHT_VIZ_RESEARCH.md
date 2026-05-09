# What Makes a Good Plot: A Compiled Reference for starsight

A synthesis of three independent deep-research passes on scientific and technical visualization, distilled into actionable design guidance for the starsight Rust visualization library. Findings are graded by how strongly the three passes agreed.

---

## How to read this document

Three independent research passes were conducted, each instructed to find sources the previous passes would miss. Where they converge, the recommendation is robust; where they diverge, the contested terrain is itself useful information.

Each major recommendation is tagged with a **confidence level**:

- **Triangulated** — all three passes independently arrived at the same conclusion via different sources. Treat as a hard default.
- **Two-of-three** — two passes converged; the third either omitted or qualified. Treat as a strong default with documented rationale.
- **Single-pass** — surfaced only once, but with strong primary sources. Treat as a recommendation worth implementing if it fits the architecture.
- **Contested** — passes disagreed substantively. Document both views; let users choose via themes or explicit opt-in.

starsight context is woven throughout: each finding is connected to the layer architecture, the release roadmap, and existing design decisions captured in STARSIGHT.md and LEARN.md.

---

## Part 1 — The strongest recommendations (triangulated across all three passes)

### 1.1 Adopt a layered grammar of graphics as the core abstraction

All three passes independently identified Wilkinson's Grammar of Graphics, Wickham's layered refinement (ggplot2), Vega-Lite's grammar of interactive graphics, and Observable Plot's marks-first reduction as the most successful API design idea in the field. Pass 2 added Bertin's predecessor framework (visual variables × implantations × impositions) as the deeper substrate. Pass 3 added Brehmer & Munzner's why/how/what task typology as orthogonal scaffolding.

**Implication for starsight:** the existing seven-layer architecture maps cleanly. Layer 3 (marks/stats/aesthetics) is the grammar layer; Layer 5's Figure builder is where users compose. This is already the right design — the research validates it strongly.

**Specific refinements all three passes endorse:**
- Reify each plot as a `PlotSpec` AST so the same spec can target any backend (Vello GPU / tiny-skia CPU / SVG / PDF / terminal). starsight's existing layer separation already supports this; make the AST explicit.
- Provide grammar-style composition for the common 80% but expose `geom_custom` / `mark_custom` escape hatches for charts that don't decompose cleanly (Sankey, sunburst, network, glyph-based, custom Minard-class layouts). Pass 3 emphasized this most strongly: Mike Bostock deliberately reduced the grammar in Observable Plot, and the Plotly community largely rejects strict GoG.
- Make the grammar layered such that each layer overrides plot-level defaults (Wickham 2010, JCGS 19(1):3–28).

### 1.2 Default to perceptually uniform colormaps; no rainbow

Triangulated unambiguously. Pass 1 cited Smith & van der Walt 2015 (viridis), Crameri 2020 (Nature Communications, "The misuse of colour in science communication"), and Borland & Taylor 2007 ("Rainbow Color Map Still Considered Harmful"). Pass 2 added Liu & Heer 2018 ("Somewhere Over the Rainbow," CHI), Stauffer et al. 2015 (BAMS) for HCL palettes, and confirmed cividis (Nuñez et al. 2018) as the deuteranopia-optimized choice. Pass 3 added Mosaic-era empirical work and noted Nature's 2024 figure guide explicitly recommends Wong's 8-color palette for categorical and Crameri-family for continuous.

**Implication for starsight:** prismatica is already the right dependency. The defaults that should ship in 0.1.0:

| Use case | Default | Source agreement |
|---|---|---|
| Sequential continuous | `prismatica::crameri::BATLOW` | All 3 passes |
| Diverging continuous | `prismatica::crameri::VIK` (or `BERLIN`) | All 3 passes |
| Cyclic (phase/angle) | `prismatica::crameri::ROMA_O` | Passes 1 and 2 |
| Categorical (≤8) | Okabe-Ito / Wong palette | All 3 passes; Nature recommends |
| Many-category (>8) | Glasbey/Polychrome max-distance algorithm | Pass 2 |

Categorical colormaps must hard-cap at ~8 distinguishable colors. Beyond that, switch to faceting, shape encoding, or direct labeling. starsight's existing default of Tableau10 should be reconsidered: Okabe-Ito (Wong 2011, *Nature Methods* 8:441) is the empirically-validated CVD-safe choice and is what Nature recommends.

**Color interpolation** must happen in OKLab/OKLCh, never in sRGB. The `palette` crate (already in starsight's deps) supports this. All three passes converged.

### 1.3 The tick generation algorithm is the Talbot-Lin-Hanrahan extended Wilkinson

Triangulated. Pass 1 named it as the dominant algorithm, beating Heckbert's "Nice Numbers" Graphics Gems algorithm and R's `pretty()`. Pass 2 confirmed the four-component scoring function (simplicity, coverage, density, legibility). Pass 3 noted that no Rust crate currently implements this — starsight will be the first.

**Implication for starsight:** Layer 2's `tick.rs` should implement the extended Wilkinson algorithm (Talbot, Lin, Hanrahan, IEEE InfoVis 2010). This is already in the roadmap. The algorithm parameters are public; reference implementation is in the InfoVis 2010 paper PDF and a JavaScript port exists in the `ticks` npm package.

### 1.4 Bar chart y-axes must include zero; line/scatter axes need not

Triangulated. Pass 1 cited Correll, Bertini & Franconeri 2020 ("Truncating the Y-Axis: Threat or Menace?") and Long & Kay 2024 confirming that bar truncation inflates perceived effect sizes. Pass 2 added that Tufte himself recommended showing the data, not the zero point, for line charts — the truncation rule is bar-specific. Pass 3 connected this to LEARN.md's existing guidance and reinforced it as a misuse-prevention default.

**Implication for starsight:** the BarMark in 0.2.0 must default `y_min = min(0, data_min)`. Provide explicit `BarMark::without_zero_baseline()` for users who want non-zero (with a doc comment explaining the empirical risk). LineMark and PointMark default to data-tight bounds.

### 1.5 Banking to 45° for line charts

Triangulated as the right aspect ratio for slope-discrimination tasks. Pass 1 cited Cleveland 1988 and Heer & Agrawala 2006 (multi-scale banking). Pass 2 confirmed median-absolute-slope as the simplest defensible default: `aspect = 1 / median(|slopes|)`. Pass 3 noted that LEARN.md already commits to this principle.

**Implication for starsight:** Layer 4's auto-layout should compute a banking hint when LineMarks are present and the user hasn't fixed the aspect ratio explicitly. Multi-scale banking (Heer-Agrawala 2006) is the more sophisticated version for charts with features at multiple frequency bands.

### 1.6 Quote uncertainty honestly, not with bare error bars

Triangulated as a major theme:
- Pass 1: error bar SD/SEM/CI confusion (Cumming et al. 2007, JCB), HOPs (Hullman et al. 2015), raincloud plots (Allen et al. 2019)
- Pass 2: quantile dotplots (Kay et al. 2016, CHI 10.1145/2858036.2858558 — reduced probabilistic estimate variance ~1.15×), Fernandes et al. 2018 (CHI) showing quantile dotplots and CCDFs outperform textual uncertainty in incentivized decision tasks, multiverse visualization (Steegen et al. 2016)
- Pass 3: Bayesian Blocks for adaptive histograms, DKW inequality with Massart's C=2 for ECDF bands, ggdist's `dotsinterval` family

**Implication for starsight:** uncertainty visualization is a 0.3.0 / 0.4.0 priority. Specifically:
- `ErrorBarMark` must take `kind: ErrorBarKind { SD, SEM, CI(level: f64) }` and require explicit specification — no silent default. Default in API documentation: 95% CI for inferential comparisons.
- Add `geom_ribbon`, `geom_quantile_dotplot`, `geom_pointrange`, `geom_raincloud` as first-class. Raincloud is half-violin + jittered raw points + box summary.
- Build a `bayesplot`-style module (posterior density, trace, PPC, pair, energy) for Bayesian workflows, citing ArviZ and bayesplot as references.
- Provide `lineup()` for visual statistical inference (Buja-Cook-Hofmann-Lawrence-Lee-Swayne-Wickham 2009, Phil. Trans. R. Soc. A 367:4361–4383) — small to implement, unique to scientific viz libraries.

### 1.7 Above n ≈ 10⁴, switch to pixel-aware aggregation

Triangulated. Pass 1 noted hexbin/quad-tree spatial aggregation as a 0.4.0–0.5.0 optimization. Pass 2 referenced Datashader and VegaFusion. Pass 3 made this the central architectural point: the Datashader 5-stage pipeline (project → aggregate → transform → colormap → embed) plus Mosaic (Heer & Moritz 2024, IEEE TVCG) decoupled-compute architecture.

**Implication for starsight:** above some threshold, scatter and line marks should silently switch to aggregating-render mode. Concrete numbers (pass 3): ~10⁴ for points, ~10³ for line series. Below the threshold, render individual marks; above, rasterize-then-shade.

This is a 0.4.0–0.5.0 architectural commitment, but the abstraction must be designed in 0.2.0 so it can be slotted in. The Mark trait should expose a render strategy that the rendering pipeline can introspect; large-N marks can opt into aggregation paths.

### 1.8 Accessibility is a measurable, first-class constraint

Triangulated:
- Pass 1: WCAG 2.1 AA contrast (4.5:1 text, 3:1 non-text), Okabe-Ito CVD-safe palette
- Pass 2: Lundgard-Satyanarayan 2022 four-level alt-text model (TVCG 28(1):1073–1083, 10.1109/TVCG.2021.3114770), Chartability heuristic suite (Elavsky-Bennett-Moritz 2022 EuroVis, 10.1111/cgf.14522), Atkinson Hyperlegible bundleable
- Pass 3: WCAG 2.1 AA hard numbers, Atkinson Hyperlegible Next 2025 (variable font, 4,464 glyphs), bundleable under SIL Open Font License

**Implication for starsight:** accessibility primitives in 0.2.0 / 0.3.0:
- Every chart must produce a structured semantic-level-1-through-4 description (Lundgard-Satyanarayan).
- Default theme passes WCAG AA: contrast ≥4.5:1 for axis labels, ≥3:1 for non-text marks against the chart background.
- A `chartability_check()` function that lints a Plot for known Chartability heuristic violations.
- Bundle Atkinson Hyperlegible Next as an optional cargo feature `bundled-fonts` for guaranteed reproducibility and accessibility.
- SVG output must include `<title>`, `<desc>`, and `role="img"` markup; tab-navigable focus order; optional parallel data table.

### 1.9 Reproducible, deterministic output

Triangulated as a 2020s baseline expectation (Quarto, Observable Framework, Vega-Lite all enforce this). Pass 1: seeded RNG for jitter, deterministic sort, embedded fonts, canonical SVG. Pass 2: detailed treatment of font subsetting and SVG normalization. Pass 3: visual regression testing strategy (SSIM ≥ 0.99 with byte-identical canonical SVG diffing).

**Implication for starsight:** a `--reproducible` mode (or `Theme::reproducible()`) that pins jitter seeds, embeds fonts, canonicalizes SVG attribute order, and rounds float coordinates to 4 decimals. CI should snapshot-test on a single canonical Linux+freetype combo (with bundled Atkinson Hyperlegible) and treat cross-platform pixel-equality as out of scope.

### 1.10 Scale defaults that match user expectations

Triangulated:
- Pass 1: log scale appropriate when data spans ≥2 orders of magnitude or multiplicative effects matter; dual y-axes discouraged (Cleveland)
- Pass 2: scale-of-scales is where ggplot2/Vega-Lite struggle; OKLab continuous color
- Pass 3: Hyndman-Fan type 7 default for quantiles (matches numpy/R) but document type 8 as recommended; Freedman-Diaconis as default histogram bin selector; Improved Sheather-Jones as default KDE bandwidth

**Implication for starsight:** Layer 2 scales in 0.2.0 / 0.5.0 should default to:

| Decision | Default | Source agreement |
|---|---|---|
| Histogram bins | Freedman-Diaconis (matches numpy/seaborn) | Pass 3 (others did not specify) |
| KDE bandwidth | Improved Sheather-Jones | Pass 3 (strong primary source) |
| Quantile method | Hyndman-Fan type 7 (least surprise) | Pass 3 |
| Box plot promotion | Auto-upgrade to letter-value (boxen) above n ≈ 1000 | Pass 3 (Hofmann-Wickham-Kafadar 2017) |
| ECDF bands | DKW with Massart C=2 | Pass 3 |

These pass-3 defaults conflict with matplotlib/ggplot2/seaborn norms (which mostly use Silverman/Sturges). The conflict is empirically resolved in favor of starsight's choices — but document the divergence prominently so users from Python/R aren't surprised.

---

## Part 2 — Strong recommendations (two of three passes converged)

### 2.1 Backend abstraction modeled on Plotters/Makie

Passes 1 and 2 explicitly recommended the Plotters/Makie pattern: a `DrawingBackend` trait with plug-and-play backend crates, semver-stable backend interface, and feature-flagged backend selection. Pass 3 implicitly endorsed via the rerun.io reference architecture (egui + wgpu + Apache Arrow).

**Implication for starsight:** Layer 1's existing `DrawBackend` trait is the right abstraction. The backend roster should be (in implementation order):

1. **tiny-skia** (CPU raster): already chosen for 0.1.0. Right call.
2. **SVG**: 0.1.0 — canonical output, diff-friendly, vector-quality.
3. **wgpu** (GPU vector + 3D): 0.6.0 / 0.7.0. The pass-2 Linebender Vello recommendation conflicts with starsight's existing tiny-skia commitment; this is fine — Vello is GPU compute-centric and overlaps wgpu rather than tiny-skia.
4. **PDF** via krilla: 0.10.0 (matches starsight's roadmap).
5. **Terminal** (Kitty/Sixel/half-block/Braille): 0.8.0 (matches roadmap).

The Plotters lesson all three passes confirm: **never advertise features the chosen backend can't render** — emit a clear error at plot-construction time, not during rendering.

### 2.2 Faceting is a first-class concern, not an afterthought

Passes 1 and 2 emphasized small multiples (Tufte) / faceting (Wickham) as the most important compositional primitive. Pass 3 added Mosaic's vgplot grammar as the modern reference and noted that `patchwork` (R) is the model for arbitrary multi-panel layouts.

**Implication for starsight:** Layer 4's `FacetWrap` and `FacetGrid` are correctly scheduled for 0.4.0. The API should also support patchwork-style arbitrary composition: combining a scatter plot, a marginal histogram, and an inset zoom panel into one figure with explicit alignment. Use `taffy` or hand-rolled flexbox-like layout — don't reinvent.

Free vs. fixed scales in facets (`scales = "free_y"`, `"shared"`) is a real decision users want to make per-axis; expose it clearly.

### 2.3 Direct labeling is preferable to legends where space allows

Passes 1 and 2 cited Bryan Connor, the FT/BBC/NYT style guides, and Cédric Scherer / Lisa Charlotte Muth (Datawrapper) on the direct-labeling movement. Pass 3 added Burn-Murdoch's FT COVID work as the highest-profile recent example.

**Implication for starsight:** ship `geom_label_repel` (port the ggrepel iterative-repulsion algorithm — Kamil Slowikowski's documented force model) and `geom_endline_label` (label at the right end of each line, replacing legends). The algorithms are well-documented and the implementation is small.

### 2.4 Animation: default-off, congruence-driven

Passes 1 and 2 converged on Tversky-Morrison-Bétrancourt (2002) congruence/apprehension principles and Heer-Robertson (2007, IEEE TVCG 13(6):1240–1247) staged transitions. Pass 3 referenced Burn-Murdoch's COVID animations as a positive example of intentional motion.

**Implication for starsight:** Layer 7's animation module (0.10.0) should:
- Stage transitions when multiple operations occur (don't directly interpolate scatter→bar; align x first, then morph to bar)
- Default ~1 second total
- Maintain object identity (preserve color/key across frames)
- Use cubic ease-in-out; never linear (linear breaks identity perception)
- Maintain valid axes throughout transitions
- Provide replay controls

Animation is an interaction primitive, not a publication primitive. Static small multiples beat animation for cross-frame comparison.

### 2.5 Title-as-takeaway and structured captions

Passes 1 and 2 referenced Cole Nussbaumer Knaflic ("the title is the point") and Borkin et al. memorability studies. Pass 3 added structural support: Cell journal requires scale bars and stat annotations on every panel; Nature requires panel labels (a, b, c lowercase 8pt bold).

**Implication for starsight:** the Figure builder should expose:
- `.title(impl Into<String>)` — what the chart shows
- `.takeaway(impl Into<String>)` — the conclusion (rendered as subtitle or below title)
- `.caption(impl Into<String>)` — citation/source
- `.panel_label(char)` — auto-positioned A/B/C label for facet-grid panels

These are tiny API additions that close a major gap with publication conventions.

### 2.6 Domain-specific helpers as first-class chart types

Passes 2 and 3 enumerated the field-standard plots that deserve dedicated APIs rather than being assembled from primitives:

| Plot | Domain | Convention |
|---|---|---|
| Manhattan plot | GWAS / bioinformatics | -log10(p) vs genome position; 5×10⁻⁸ threshold line |
| Volcano plot | Differential expression | -log10(p) vs log2(fold-change) |
| Forest plot | Meta-analysis | Effect + 95% CI horizontal lines, size-encoded squares, pooled diamond |
| Q-Q plot | Statistics | Theoretical vs empirical quantiles |
| ROC + PR + calibration | ML | Standard axes, reference lines, AUC label |
| Kaplan-Meier | Survival | Step function with censoring tick marks |
| UpSet (Lex et al. 2014) | Set comparison | Replaces Venn beyond 4 sets |
| Bode + Nyquist + root locus | Control engineering | Standard axes, log-frequency |
| Spectrogram + scalogram | Signal processing | STFT vs CWT respectively |
| HR diagram | Astronomy | Temperature decreasing left-to-right |
| Hovmöller diagram | Climate | Time vs space cross-section |
| Taylor diagram | Climate model evaluation | Correlation as angle, normalized SD as radius |

**Implication for starsight:** ship a core domain helpers module in 0.5.0–0.6.0 with `manhattan()`, `volcano()`, `forest()`, `qq()`, `roc()`, `kaplan_meier()`, `upset()`, `bode()`, `spectrogram()`. Each should have field-standard defaults baked in — researchers shouldn't have to reinvent the conventions.

Pass 3 strongly recommended these as separate addon crates (`starsight-bio`, `starsight-astro`, etc.) to keep the core compile-time-friendly. starsight's seven-crate workspace makes this natural.

### 2.7 Avoid the matplotlib pyplot trap

Passes 1 and 2 explicitly named matplotlib's global state-machine API as the single most-cited frustration in Python data viz, lasting ~20 years. Pass 3 referenced rerun.io's immediate-mode pattern as the modern Rust analog.

**Implication for starsight:** the Figure builder is the only API. No global state, no `plt.gca()`-style implicit current-axes. The existing design is correct — don't add a "convenience" pyplot-style facade later.

### 2.8 Glyph-based viz (Chernoff, radar/spider) deserves skepticism

Passes 1 and 2 discouraged Chernoff faces and radar plots. Pass 3 confirmed empirical research finds humans don't spontaneously decode multivariate data from faces (Raciborski 2009; Kosara 2007 critique).

**Implication for starsight:** these can be implemented for completeness but should not be promoted in documentation. Mark them in docs as "discouraged for accuracy; use only when the audience expects them."

### 2.9 Test infrastructure

Passes 1 and 2 emphasized snapshot testing (matplotlib's `compare_images`, ggplot2's `vdiffr`). Pass 3 added concrete tooling: `insta` (already on starsight's radar), `image-compare` (SSIM ≥ 0.99 in Rust), Playwright-style canonical baselines.

**Implication for starsight:** the existing 0.1.0 commitment to `insta` snapshot testing is correct. Specifically:
- SSIM at 0.99 threshold for raster snapshots
- Byte-identical SVG snapshots after canonicalization (sort attribute order, normalize float precision to 4 decimals, strip random IDs)
- Run CI on one canonical OS+font combination
- Property-based tests for scale roundtrips (data → pixel → data is identity within float precision)

---

## Part 3 — Worth implementing if it fits (single-pass findings with strong sources)

### 3.1 Mosaic-style decoupled compute architecture (pass 3)

Heer & Moritz 2024 (IEEE TVCG, idl.uw.edu/papers/mosaic) decouple visualization specifications from data processing by pushing aggregation to a backing scalable database (DuckDB by default) over Apache Arrow. Their flagship demo interactively explores the entire 1.8-billion-star Gaia catalog.

**Implication for starsight:** for the >10⁴ aggregation path (Part 1.7), the right architecture is query-backed rather than iterator-backed. Polars + DataFusion is the natural Rust analog. starsight's Layer 5 data acceptance can compile a Plot to either iterator-mode (small N) or query-mode (large N) without the user noticing.

This is a 0.5.0+ architectural concern. Don't lock starsight into iterator-only data paths in 0.1.0–0.2.0; design the data abstraction such that a query-backed implementation can be slotted in.

### 3.2 Visual statistical inference (pass 3)

The `lineup()` protocol (Buja et al. 2009; Wickham et al. 2010 TVCG; Majumder et al. 2013 power proofs) — present a real plot among n-1 null permutations and let the viewer identify the real one. Provides formal hypothesis testing grounded in plots; matches uniformly most powerful tests in some scenarios.

**Implication for starsight:** small to implement, completely unique to scientific viz libraries, directly serves the academic user base. Add `Figure::lineup(real_data, null_generator, n=20)` in 0.4.0.

### 3.3 Mackinlay's expressiveness/effectiveness criteria (pass 3)

Jock Mackinlay's 1986 ACM TOG paper (5(2):110–141) defines two orthogonal evaluation criteria for visualization languages:
- **Expressiveness:** can the language express all and only the relations in the data?
- **Effectiveness:** does it exploit human visual capabilities?

**Implication for starsight:** as the API stabilizes toward 1.0, evaluate it against these criteria explicitly in design docs. They are the cleanest framework available for arguing that a Rust API is "correct" for visualization.

### 3.4 Bertin's organization levels (pass 2)

Bertin's four levels — selective (does it pop out?), associative (can elements be grouped?), ordered (is there an inherent order?), quantitative (can ratios be read?) — are richer than Cleveland's pure rank-ordered list. They tell you *what kind of question* a channel can answer, not just how accurately.

**Implication for starsight:** when documenting which encoding to use for which task, organize by Bertin's levels rather than Cleveland's ranking. For categorical data (nominal/associative), color hue and shape work; for ordinal data, position and size work; for quantitative data, only position on a common scale is fully ratio-faithful.

### 3.5 LIDA-friendly JSON spec (pass 3)

LIDA (Dibia, Microsoft, ACL 2023, arXiv:2303.02927) is grammar-agnostic LLM-driven viz generation. The lesson: a `starsight` *spec format* expressible as plain JSON is what makes LLM integration trivial. Tightly-coupled Rust builders are not.

**Implication for starsight:** in 0.5.0+, expose `Figure::to_spec() -> serde_json::Value` and `Figure::from_spec(spec) -> Figure`. Make the spec round-trippable. This costs little and positions starsight for the LLM-driven workflow that's now standard.

### 3.6 Interaction taxonomies as API checklist (pass 2)

Yi-Kang-Stasko-Jacko 2007 (IEEE TVCG 13(6):1224–1231) defines seven user-intent categories: Select, Explore, Reconfigure, Encode, Abstract/Elaborate, Filter, Connect. Heer-Shneiderman 2012 (CACM 55(4):45–54) extends to 12 dynamics across data/view specification, view manipulation, and analysis process/provenance.

**Implication for starsight:** when implementing Layer 6 interactivity (0.6.0), use Yi et al. as the API checklist. Each user intent should map to one named primitive (e.g., `Plot::brush(...)` for Select, `Plot::link(other)` for Connect).

### 3.7 Cartographic projection defaults (pass 3)

For thematic maps, equal-area is mandatory (Mollweide, Albers, Eckert IV); for reference world maps where compromise is acceptable, Winkel tripel (National Geographic standard since 1998). Mercator is forbidden for thematic data due to grossly inflated area distortion at high latitudes.

**Implication for starsight:** Layer 2's coordinate systems already include geographic (mentioned in LEARN.md). When implementing in 0.7.0, default to equal-area for choropleth/thematic maps and emit a warning when Mercator is requested with continuous data.

### 3.8 Letter-value (boxen) plot auto-promotion above n=1000 (pass 3)

Hofmann-Kafadar-Wickham 2017 (JCGS 26(3):469–477, 10.1080/10618600.2017.1305277) proves the conventional Tukey boxplot displays an expected `0.4 + 0.007n` "outliers" for Gaussian data — at n=10,000 that's ~70 spurious outlier marks. The letter-value plot displays additional letter values only out to depths where they remain reliable.

**Implication for starsight:** the BoxPlotMark in 0.3.0 should auto-promote to letter-value above n ≈ 1000 with a warn-on-promote message; provide explicit `BoxPlotMark::tukey()` for users who want the original.

### 3.9 Bundle Atkinson Hyperlegible Next (pass 3)

Released 2025 by the Braille Institute / Applied Design Works under SIL Open Font License: variable axis support, seven weights, italic, monospace, 4,464 glyphs per font, 150-language coverage. Specifically optimized for low-vision readability.

**Implication for starsight:** ship as `--features bundled-fonts`. Total bundle size is ~1–3 MB; the accessibility benefit is large; SIL OFL permits embedding.

### 3.10 Specific Rust stack choices (pass 2 and 3)

Both passes converged on a Linebender-heavy stack:

| Concern | Recommended | Status in starsight |
|---|---|---|
| CPU raster | tiny-skia | Already chosen ✓ |
| GPU vector (future) | Vello (with `vello_cpu` and `vello_hybrid`) | Layer 6 candidate |
| Text shaping | cosmic-text (with harfrust + skrifa) | Already chosen ✓ |
| Future text | Parley (when stable) | Track for 0.6.0+ |
| Color | `palette` crate (OKLab/OKLCh) | Already in deps ✓ |
| Vector tessellation (GPU) | lyon | When wgpu backend lands |
| 2D curve math | kurbo | Standard |
| Layout | taffy | When patchwork-style composition lands |

starsight's existing crate choices are well-aligned with the Rust ecosystem's converging stack.

---

## Part 4 — Contested terrain (passes disagreed)

### 4.1 Tufte's data-ink ratio: maximize or moderate?

- **Passes 1 and 2:** noted Bateman et al. 2010 ("Useful Junk?") and Borgo et al. 2012 (IEEE TVCG 18(12):2759–2768) showed embellishment can aid memory without harming accuracy. Tufte's blanket minimalism is not empirically defended at the extreme.
- **Pass 3:** added Borkin et al. 2013/2016 memorability studies (TVCG 19(12):2306–2315 and 22(1):519–528) confirming pictograms, multiple colors, low visual density, and human-recognizable objects all increased one-second and recall memorability across 2,070 visualizations. Tilt: in *applied* publication contexts, *some* embellishment actively helps recall.

**Resolution for starsight:** ship multiple themes, none of which is blank-Tufte:
- `Theme::publication()` — minimal but with structured titles and panel labels (Borkin findings respected)
- `Theme::exploratory()` — slightly more chrome, larger marks, more prominent labels
- `Theme::presentation()` — large fonts, high contrast
- `Theme::tufte()` — strict data-ink maximalism for those who want it

The default for new users should be `publication`, not `tufte`.

### 4.2 Pie charts: ban or qualify?

- **Pass 1:** Skau & Kosara 2016 (EuroVis) partially rehabilitated pies for ≤3–4 categories on part-to-whole tasks; angle is the *least* important cue, area and arc length carry most of the information; donut and waffle perform as well as or better than pie.
- **Pass 2:** confirmed pie charts remain "genuinely contested." Recommended starsight warn against misuse but not refuse.
- **Pass 3:** strongly recommended bar/dot/waffle alternatives but agreed pie shouldn't be banned outright.

**Resolution for starsight:** ship `PieMark` and `DonutMark` (already on the 0.3.0 roadmap). Warn when slice count > 5. Suggest `WaffleMark` or `BarMark` for higher counts. Don't refuse to draw.

### 4.3 Statistical graphics vs. infovis: one library or two stances?

- **Pass 2:** highlighted Gelman & Unwin 2013 (JCGS 22(1):2–28) vs. Kosara 2013 reply as the cleanest articulation of the explanatory/exploratory split.
- **Pass 3:** confirmed the split persists; suggested `Plot::explore` vs `Plot::present` as separate API surfaces (per Brehmer-Munzner why/how/what).

**Resolution for starsight:** the theme system handles this already (`exploratory` vs `publication`). No need for separate API surfaces; the same Plot type with different theme produces appropriately-different output. Document the distinction in the design guide.

### 4.4 Auto-recommendation of chart types

- **Pass 1:** mentioned Voyager / Show Me / AutoViz briefly.
- **Pass 3:** strongly recommended a Mackinlay-style auto-viz recommender taking a DataFrame and a `Task` hint, returning ranked chart suggestions (citing Saket-Endert-Demiralp 2018 IEEE TVCG 25(7):2505–2512).
- **Pass 2:** more skeptical — auto-recommendation can homogenize choices and hide better domain-specific options.

**Resolution for starsight:** post-1.0 feature. The risk-reward is uncertain enough that starsight should ship the grammar and helpers first, and add auto-recommendation only if community demand is clear.

### 4.5 GoG strict vs. escape hatches

All three passes agreed grammar-of-graphics is the right scaffolding but disagreed on how strict to be. Pass 1 leaned strict; passes 2 and 3 emphasized escape hatches for Sankey/network/glyph layouts.

**Resolution for starsight:** layered grammar for the common 80%; explicit `geom_custom` / `mark_custom` accepting raw `wgpu`/Vello primitives for the long tail. This is the Bostock approach in Observable Plot and matches Mosaic's vgplot. starsight's Mark trait already supports this — any user can implement Mark.

---

## Part 5 — Antipatterns starsight should warn about (consensus across passes)

The Mark API should detect and warn (not refuse) on these constructs:

| Antipattern | Detection rule | Suggested alternative |
|---|---|---|
| Bar chart with non-zero baseline | `BarMark` + `y_min != min(0, data_min)` | Add to-zero or use `LineMark` |
| Pie with > 5 slices | `PieMark` slice count > 5 | `BarMark` or `WaffleMark` |
| Rainbow/jet colormap on sequential data | colormap matches jet/rainbow signature | viridis or BATLOW |
| > 8 categorical colors | distinct color count > 8 | facet, shape encoding, or direct labeling |
| 3D bar / 3D pie | `BarMark3D` or `PieMark3D` constructed | 2D equivalent |
| Dual y-axis | two YAxes on one CartesianCoord | small multiples or normalized index |
| Single-error-bar dynamite plot | `BarMark` + `ErrorBarMark` with n ≤ 5 | strip/dot plot showing raw points |
| Mercator projection on thematic data | `Mercator` projection + continuous data layer | Mollweide/Albers/Eckert IV |
| Tukey boxplot on n > 1000 | `BoxPlotMark` + n > 1000 | letter-value plot (auto-promote in starsight) |
| Color-only categorical encoding | only color encodes class | add shape or pattern |

These warnings should be emitted at plot-construction time (not render time) so users can fix the issue before saving. Use the `tracing` crate (or starsight's own logging) at WARN level.

---

## Part 6 — Concrete starsight roadmap deltas

Mapped against starsight's existing release plan in STARSIGHT.md:

### 0.1.0 (current)
- ✓ Layer 1 + 2 + 3 vertical slice with Skia/SVG backends, LinearScale, LineMark, PointMark
- **Add:** Talbot-Lin-Hanrahan tick algorithm (currently planned, confirm priority)
- **Add:** Reproducibility flag (seeded jitter, font embedding, canonical SVG)
- **Add:** Default theme passes WCAG AA (already implied by 0x333333 text on white)
- **Reconsider:** swap default categorical palette from Tableau10 → Okabe-Ito (research-backed)

### 0.2.0 — Core chart types part 1
- BarMark with mandatory zero baseline; `BarMark::without_zero_baseline()` opt-out
- AreaMark, Histogram (Freedman-Diaconis default bins; expose Knuth Bayesian Blocks)
- HeatmapMark with BATLOW default sequential colormap
- **Add:** ErrorBarMark with mandatory `kind` parameter (no silent default)
- **Add:** Antipattern warnings infrastructure

### 0.3.0 — Core chart types part 2
- BoxPlotMark with auto-promotion to letter-value above n=1000
- ViolinMark with KDE (Improved Sheather-Jones default bandwidth)
- RaincloudMark (half-violin + jitter + box) — research strongly endorses
- KDE stat with ISJ default
- PieMark / DonutMark with slice-count warning
- ContourMark, CandlestickMark
- **Add:** QuantileDotplotMark, GradientPlotMark for uncertainty
- **Add:** Polars DataFrame integration

### 0.4.0 — Layout and composition
- GridLayout, FacetWrap, FacetGrid (already planned)
- Legend, Colorbar with continuous BATLOW, "extend" arrows for clipping
- PairPlot, JointPlot
- **Add:** Patchwork-style composition (taffy or hand-rolled flexbox)
- **Add:** `Figure::lineup()` for visual statistical inference
- **Add:** Direct labeling (`label_repel`, `endline_label`)
- **Add:** `Figure::takeaway()`, `panel_label()` API

### 0.5.0 — Scale infrastructure
- LogScale, SymlogScale, DateTimeScale, BandScale, CategoricalScale (already planned)
- ColorScale backed by prismatica
- **Add:** Above-N pixel-aware aggregation path (Datashader-style)
- **Add:** Mosaic-compatible JSON spec emission (`Figure::to_spec()`)
- **Add:** Domain helpers crate `starsight-domain` (Manhattan, volcano, forest, qq, roc, kaplan_meier, upset, bode, spectrogram)

### 0.6.0 — GPU and interactivity
- wgpu DrawBackend, native window via winit
- Yi et al. 2007 seven user-intent categories as named API primitives
- Hover, brush, link, zoom, pan, legend toggle
- Streaming data with rolling window

### 0.7.0 — 3D
- Scatter3D, Surface3D, Wireframe3D, Line3D
- Marching cubes (Chernyaev-corrected, not vanilla Lorensen-Cline) for isosurfaces
- Volume rendering
- Camera orbit/pan
- **Add:** Equal-area cartographic projections (Mollweide, Albers, Eckert IV) for geographic Coord
- **Add:** Mercator-with-continuous-data warning

### 0.8.0 — Terminal
- Kitty graphics protocol, Sixel, iTerm2 inline
- Half-block, Braille, octant (Unicode 16) — pass 2 noted octant has better font support than Braille
- ratatui Widget integration
- Auto-detect protocol

### 0.9.0 — All chart types
- Long tail per starsight taxonomy (Sankey, sunburst, chord, parallel coordinates, slope graphs, bullet, treemap with squarified algorithm Bruls-Huizing-van Wijk 2000, streamgraph with Byron-Wattenberg 2008 baseline geometry options, horizon graph with Heer-Kong-Agrawala 2009 layering thresholds)
- Connected scatterplot with explanatory-only warning (Haroz-Kosara-Franconeri 2016)
- Glyph viz (Chernoff, radar) marked as "discouraged"

### 0.10.0 — Export and animation
- PDF via krilla
- Self-contained interactive HTML (with Lundgard-Satyanarayan four-level alt-text)
- GIF via Heer-Robertson 2007 staged transitions, ~1s default, cubic ease-in-out
- WASM + WebGPU
- **Add:** PGFPlots/TikZ emitter for direct LaTeX inclusion (pass 2 strongly recommended)

### 0.11.0 — Polish
- Recipe proc macro
- ndarray, Arrow RecordBatch acceptance
- API audit
- **Add:** `chartability_check()` lint
- **Add:** `simulate_cvd()` color tooling

### 0.12.0 — Documentation
- Per existing roadmap
- **Add:** Cite primary research (Cleveland-McGill, Tufte, Crameri, Heer-Bostock, Munzner, Franconeri, Wickham) in API docs themselves
- **Add:** "Design rationale" section explaining defaults

### 1.0.0 — Stable
- Per existing roadmap

---

## Part 7 — Things all three passes deliberately did *not* recommend

A few things appeared in early drafts of one pass and were either contradicted by another pass or fell out under scrutiny. Worth noting so they don't get adopted by accident:

- **Don't make all defaults overridable equally.** Some defaults are misuse-preventing (bar zero baseline, no rainbow); these should be hard to override and warn loudly when overridden. Others are aesthetic (font choice, theme); these should be easy to swap.
- **Don't ship a pyplot-style facade.** All three passes warned that matplotlib's pyplot/OO confusion has lasted 20 years and is the single most-cited frustration in Python data viz. Stick to one explicit Figure-builder API.
- **Don't auto-recommend chart types in 1.0.** The risk-reward is unclear; ship after community demand, not before.
- **Don't ban pie charts; warn when misused.** Banning is paternalistic; warnings respect users.
- **Don't trust LLM correctness.** LIDA's <3.5% error claim is self-reported. Design for LLM-friendliness (JSON spec, stable API) but don't rely on LLM output being correct.
- **Don't cross-platform-test pixel equality.** Sub-pixel rendering, color management, and OS gamma curves create unavoidable drift. Test on one canonical Linux+freetype combo with bundled fonts.
- **Don't bake one dataframe library into core.** Polars, ndarray, Arrow, plain `Vec<(f64, f64)>` should all work via adapter traits. Hard-depending on Polars limits the user base.
- **Don't enforce one color space everywhere.** Interpolate in OKLab; output in sRGB; allow CIE Lab as opt-in. The complexity is unavoidable but encapsulatable.

---

## Part 8 — Caveats

- **Many "settled" results are thinly replicated.** Kosara's "Empire Built On Sand" critique applies broadly. starsight should cite, not preach.
- **Authorities openly disagree.** Tufte vs. Few vs. Knaflic on titles and dashboards; Skau-Kosara vs. Few on pies; Gelman-Unwin vs. Kosara on infovis vs. statistical graphics. Themes encode the disagreement, not the API surface.
- **Forward-looking tools are still maturing.** Vello is alpha; Parley is pre-1.0; Mosaic Selections is 2026 preprint. Track them but don't depend on unstable parts.
- **Many commercial sources are sponsored** (Datawrapper, Tableau, Inforiver). Their advice is generally sound but selectively framed.
- **The Rust ecosystem moves quickly.** Crate API specifics in this report reflect late-2025 / early-2026 state; re-verify before commitment.
- **Some specific empirical numbers are reported as the original authors stated them** (Kay et al.'s ~1.15× variance reduction; Heer-Robertson's ~1-second transitions; Mosaic's billion-row Gaia demo; LIDA's <3.5% error). Treat as orienting estimates, not exact targets.
- **Domain-specific defaults may surprise users from other tools.** Auto-promoting to letter-value at n=1000, defaulting to ISJ over Silverman, defaulting to Freedman-Diaconis over Sturges all diverge from matplotlib/seaborn norms. Document the divergence prominently in user-facing docs.

---

## Appendix A — The 30 most-cited primary sources across all three passes

Listed alphabetically by first author. All three passes cited these (or one of them did with strong support and the others didn't contradict).

1. Allen et al. 2019, "Raincloud plots," *Wellcome Open Research*
2. Becker & Cleveland 1987, "Brushing scatterplots," *Technometrics* 29(2):127–142
3. Bertin 1967/1983, *Sémiologie graphique* / *Semiology of Graphics*
4. Borkin et al. 2013, TVCG 19(12):2306–2315 + 2016, TVCG 22(1):519–528 (memorability)
5. Brehmer & Munzner 2013, TVCG 19(12):2376–2385 (multi-level task typology)
6. Buja et al. 2009, "Statistical inference for exploratory data analysis," Phil. Trans. R. Soc. A 367:4361–4383
7. Cleveland & McGill 1984, JASA 79:531–554 (perceptual hierarchy)
8. Correll, Bertini & Franconeri 2020 CHI, "Truncating the Y-Axis"
9. Crameri 2020, *Nature Communications* 11:5444 ("misuse of colour in science communication")
10. Cumming, Fidler & Vaux 2007, *Journal of Cell Biology* (error bar conventions)
11. Elavsky, Bennett & Moritz 2022 EuroVis (Chartability)
12. Fernandes et al. 2018 CHI 10.1145/3173574.3173718 (uncertainty displays)
13. Franconeri, Padilla, Shah, Zacks & Hullman 2021, *Psychological Science in the Public Interest* 22(3):110–161
14. Friendly & Denis 2001+ (Milestones)
15. Heer, Bostock & Ogievetsky 2010 ACM Queue ("tour through the visualization zoo")
16. Heer & Moritz 2024 IEEE TVCG (Mosaic)
17. Heer & Robertson 2007 IEEE TVCG 13(6):1240–1247 (animated transitions)
18. Heer & Shneiderman 2012 CACM 55(4):45–54 (interactive dynamics)
19. Hofmann, Kafadar & Wickham 2017 JCGS 26(3):469–477 (letter-value plots)
20. Hullman, Resnick & Adar 2015 PLoS ONE 10(11) (HOPs)
21. Kay, Kola, Hullman & Munson 2016 CHI 10.1145/2858036.2858558 (quantile dotplots)
22. Kosara 2016 BELIV ("Empire Built On Sand")
23. Kosslyn 2006, *Graph Design for the Eye and Mind*
24. Lex, Gehlenborg, Strobelt, Vuillemot & Pfister 2014 TVCG 20(12):1983–1992 (UpSet)
25. Lundgard & Satyanarayan 2022 IEEE TVCG 28(1):1073–1083 (4-level alt-text)
26. Mackinlay 1986 ACM TOG 5(2):110–141 (expressiveness/effectiveness)
27. Munzner 2014, *Visualization Analysis and Design* (CRC)
28. Smith & van der Walt 2015 (viridis colormap family)
29. Talbot, Lin & Hanrahan 2010 IEEE InfoVis (extended Wilkinson tick algorithm)
30. Wickham 2010 JCGS 19(1):3–28 (layered grammar of graphics)

---

## Appendix B — Convergence matrix

How strongly each major recommendation triangulated:

| Recommendation | Pass 1 | Pass 2 | Pass 3 | Confidence |
|---|:-:|:-:|:-:|---|
| Layered grammar of graphics | ✓ | ✓ | ✓ | Triangulated |
| Perceptually uniform colormaps default; no rainbow | ✓ | ✓ | ✓ | Triangulated |
| Talbot-Lin-Hanrahan tick algorithm | ✓ | ✓ | ✓ | Triangulated |
| Bar chart y-axis includes zero | ✓ | ✓ | ✓ | Triangulated |
| Banking-to-45° for line charts | ✓ | ✓ | ✓ | Triangulated |
| Honest uncertainty viz (raincloud, HOPs, quantile dotplots) | ✓ | ✓ | ✓ | Triangulated |
| Pixel-aware aggregation above N≈10⁴ | ✓ | ✓ | ✓ | Triangulated |
| WCAG-AA + Lundgard-Satyanarayan alt-text | ✓ | ✓ | ✓ | Triangulated |
| Reproducible deterministic output | ✓ | ✓ | ✓ | Triangulated |
| Auto-promote boxplot to letter-value at n>1000 | – | – | ✓ | Single-pass strong |
| ISJ KDE default | – | – | ✓ | Single-pass strong |
| Freedman-Diaconis histogram default | – | – | ✓ | Single-pass strong |
| DKW-banded ECDFs | – | – | ✓ | Single-pass strong |
| Plotters/Makie backend trait pattern | ✓ | ✓ | – | Two-of-three |
| Faceting first-class | ✓ | ✓ | – | Two-of-three |
| Direct labeling over legends | ✓ | ✓ | – | Two-of-three |
| Animation default-off, congruence-driven | ✓ | ✓ | – | Two-of-three |
| Title-as-takeaway + panel labels | ✓ | ✓ | ✓ | Triangulated |
| Domain helpers (Manhattan, forest, ROC, etc.) | – | ✓ | ✓ | Two-of-three |
| Avoid pyplot-style global state | ✓ | ✓ | ✓ | Triangulated |
| Glyph viz (Chernoff, radar) discouraged | ✓ | ✓ | ✓ | Triangulated |
| Snapshot testing infrastructure | ✓ | ✓ | ✓ | Triangulated |
| Mosaic-style decoupled compute | – | – | ✓ | Single-pass strong |
| `lineup()` visual statistical inference | – | – | ✓ | Single-pass strong |
| Mackinlay expressiveness/effectiveness | – | – | ✓ | Single-pass strong |
| Bertin organization levels | – | ✓ | – | Single-pass |
| LIDA-friendly JSON spec | – | – | ✓ | Single-pass strong |
| Yi et al. interaction taxonomy as API checklist | – | ✓ | – | Single-pass |
| Equal-area cartographic projections default | – | – | ✓ | Single-pass strong |
| Bundle Atkinson Hyperlegible Next | – | ✓ | ✓ | Two-of-three |
| Linebender Rust stack alignment | ✓ | ✓ | ✓ | Triangulated |
| Tufte data-ink: maximize? | – | qualified | qualified | Contested |
| Pie charts: ban? | qualified | qualified | qualified | Contested (don't ban) |

Empty cells mean "not addressed" rather than "disagreed" unless explicitly qualified.
