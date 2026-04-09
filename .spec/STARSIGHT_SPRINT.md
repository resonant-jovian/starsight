# Starsight — Thursday + Friday Sprint

> **Starting point**: 0.1.1 on crates.io. LineMark, PointMark, SkiaBackend, SvgBackend, Figure, Wilkinson ticks — all working. 7 snapshot tests passing. Next milestone: **0.2.0 — Core chart types part 1**.

---

## Thursday — BarMark + AreaMark

### 09:00–10:30 — BarMark implementation

- [ ] Define `BarMark` struct in `starsight-layer-3/src/marks.rs`
  - Fields: `x: Vec<f64>`, `y: Vec<f64>`, `color: Color`, `bar_width: f32`
  - Follow the same `new()` + builder pattern as `LineMark` and `PointMark`
  - `DataExtent` impl must force `y_min = y_min.min(0.0)` so the baseline is always visible
- [ ] Implement `Mark for BarMark` — iterate data, map through `CartesianCoord`, emit `fill_rect` calls
  - Handle y-values below zero (bars extend downward from axis)
  - Bar centering: offset x by `bar_width / 2`
- [ ] Unit tests: empty data, single bar, negative values, NaN skip

> [!warning] Design note
> The existing TODO stub says `categories: Vec<String>` — ignore that. The coord system is purely numeric (`CartesianCoord` only does `f64 → pixel`). Use `x: Vec<f64>` matching LineMark's pattern. Categorical x-axis is a 0.5.0 `BandScale` concern.

> [!tip] BarMark rendering path
> Use `backend.fill_rect()` directly for each bar rather than building a `Path` — it's more efficient and sidesteps the opacity question. `fill_rect` takes `Rect` + `Color` which is all you need for opaque bars. If you later want semi-transparent bars, switch to `draw_path` with a filled `PathStyle` (opacity field already works in SkiaBackend).

**Hints**
- [tiny-skia `Pixmap::fill_rect`](https://docs.rs/tiny-skia/latest/tiny_skia/struct.Pixmap.html#method.fill_rect) — the underlying call your `SkiaBackend::fill_rect` wraps
- [tiny-skia `Rect` constructors](https://docs.rs/tiny-skia/latest/tiny_skia/struct.Rect.html) — `from_xywh` and `from_ltrb`
- [plotters `Histogram` struct](https://docs.rs/plotters/latest/plotters/series/struct.Histogram.html) — mature reference for bar rendering (margins, baselines, style)
- [MDN: SVG fills and strokes](https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorials/SVG_from_scratch/Fills_and_strokes) — covers `fill`, `stroke`, `fill-opacity` on `<rect>`

### 10:30–10:45 — Break + small tasks

- [ ] **Fix CHANGELOG.md** — still says "No release has been published yet" and only has `[Unreleased]`. Add `[0.1.0]` and `[0.1.1]` sections (use `git-cliff` or write manually). Move the existing items under `0.1.0`, add a `0.1.1` patch section.
- [ ] **Update README roadmap** — check the `0.1.0` checkbox, it shipped

### 10:45–12:00 — BarMark snapshots + AreaMark struct

- [ ] Snapshot test: basic vertical bars (monthly rainfall data)
- [ ] Snapshot test: mixed positive/negative bars (profit/loss)
- [ ] Define `AreaMark` struct — `x: Vec<f64>`, `y: Vec<f64>`, `fill: Color`, `opacity: f32`
  - Baseline is always y=0 for now (0.2.0 scope)
  - `DataExtent` must include the baseline (`y_min = y_min.min(0.0)`)
- [ ] Implement `Mark for AreaMark` — build a closed `Path`: forward along data points, then back along baseline, then `Close`

> [!tip] Area path construction
> The SVG equivalent is `M x0,y0 L x1,y1 ... L xn,yn L xn,baseline L x0,baseline Z`. In starsight terms: `MoveTo(first_data_point)` → `LineTo` for each subsequent point → `LineTo(last_x, baseline_y)` → `LineTo(first_x, baseline_y)` → `Close`. Set `PathStyle.fill_color = Some(fill)` and `PathStyle.opacity` to the desired alpha.

### 12:00–13:00 — Lunch

### 13:00–14:30 — AreaMark rendering + snapshots

- [ ] Handle NaN gaps in AreaMark (split into separate closed polygons, same logic as LineMark's `need_move` but each segment needs its own baseline closure)
- [ ] Semi-transparent fill: `PathStyle.opacity` already works in SkiaBackend (it feeds into `set_color_rgba8`'s alpha byte)

> [!bug] SvgBackend ignores `PathStyle.opacity`
> The `draw_path` impl in `vectors.rs` never sets `fill-opacity` or `opacity` on the `<path>` element. Fix this now — add `.set("opacity", style.opacity)` (or separate `fill-opacity` / `stroke-opacity`) to the `SvgPath` builder. Without this, SVG snapshots for area charts will show fully opaque fills regardless of the opacity field.
>
> ```rust
> // In SvgBackend::draw_path, after setting fill:
> let p = SvgPath::new()
>     .set("d", data)
>     // ... existing stroke/fill ...
>     .set("opacity", style.opacity);  // ADD THIS
> ```

- [ ] Snapshot test: smooth area chart (temperature over a year)
- [ ] Snapshot test: area with NaN gap

**Hints**
- [tiny-skia `PathBuilder`](https://docs.rs/tiny-skia/latest/tiny_skia/struct.PathBuilder.html) — `move_to`, `line_to`, `close`, `finish`
- [tiny-skia `Color`](https://docs.rs/tiny-skia/latest/tiny_skia/struct.Color.html) — `from_rgba8(r, g, b, alpha)` for alpha; `from_rgba()` returns `Option`
- [MDN: SVG `fill-opacity`](https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Attribute/fill-opacity) — values 0–1 on shapes and text
- [`svg` crate docs](https://docs.rs/svg/latest/svg/) — `.set("fill-opacity", 0.5)` via the generic `.set()` method
- [MDN: SVG path commands](https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial/Paths) — M, L, C, Z for building area polygons

### 14:30–14:45 — Break + small tasks

- [ ] **Update marks.rs header comment** — currently lists StepMark under 0.3.0 but the TODO comment says 0.2.0. Pick one and be consistent.
- [ ] **Fix README "Coming from" table** — row says `BarMark::new(labels, vals)` but you just built it with `x: Vec<f64>`, not string labels. Update to `BarMark::new(x, y)`.
- [ ] **Update README Features table** — `LineMark` / `PointMark`, CPU rendering, and SVG export are all `working` now, not `wip`. Same for Wilkinson ticks.

### 14:45–16:00 — StepMark

- [ ] Define `StepMark` — `x: Vec<f64>`, `y: Vec<f64>`, `step: StepPosition`, `color: Color`, `width: f32`
  - `StepPosition` enum: `Pre`, `Mid`, `Post`
- [ ] Implement `Mark for StepMark` — emit horizontal then vertical (or vice versa) path segments depending on `StepPosition`
- [ ] Unit tests for each step variant
- [ ] Snapshot test: step chart (discrete event data)

> [!info] Step position semantics
> **Pre** (`steps-pre` / `curveStepBefore`): vertical transition happens *before* the data point → `(x_prev, y_prev)` → `(x_prev, y_curr)` → `(x_curr, y_curr)`.
> **Post** (`steps-post` / `curveStepAfter`): vertical transition happens *after* → `(x_prev, y_prev)` → `(x_curr, y_prev)` → `(x_curr, y_curr)`.
> **Mid** (`curveStep`): vertical transition at midpoint → `(x_prev, y_prev)` → `(mid_x, y_prev)` → `(mid_x, y_curr)` → `(x_curr, y_curr)`.

**Hints**
- [d3 `step.js` source](https://github.com/d3/d3-shape/blob/main/src/curve/step.js) — ~50 lines, cleanest reference to port
- [matplotlib `pyplot.step` API](https://matplotlib.org/stable/api/_as_gen/matplotlib.pyplot.step.html) — `where='pre'|'mid'|'post'`
- [matplotlib step demo gallery](https://matplotlib.org/stable/gallery/lines_bars_and_markers/step_demo.html) — visual comparison of all three modes
- [matplotlib `pyplot.stairs`](https://matplotlib.org/stable/api/_as_gen/matplotlib.pyplot.stairs.html) — edge-based step function, supports filled steps

### 16:00–17:00 — Cleanup + commit

- [ ] Run full `cargo test --workspace`, `cargo clippy`, `cargo fmt`
- [ ] Review all new public API: doc comments, `#[must_use]` where appropriate
- [ ] **Update facade re-exports:**
  - `starsight/src/marks.rs` — add `BarMark, AreaMark, StepMark` to the `pub use` line
  - `starsight/src/prelude.rs` — add `BarMark, AreaMark, StepMark` to prelude exports
- [ ] Commit: `feat(layer-3): add BarMark, AreaMark, StepMark`
- [ ] Update CHANGELOG.md draft under `[Unreleased]`

---

## Friday — Polish, Histogram, and stretch goals

### 09:00–10:30 — Histogram stat transform

- [ ] Implement `starsight-layer-3/src/statistics.rs` (file exists as stub)
  - `pub fn histogram_bins(data: &[f64], bin_count: Option<usize>) -> Vec<(f64, f64, f64)>` → (bin_start, bin_end, count)
  - Default bin count via Sturges' rule: `1 + ⌈log₂(n)⌉`
  - Consider also implementing Freedman-Diaconis (`h = 2·IQR·n⁻¹ᐟ³`) as an alternative — it's more robust to skewed data and only a few extra lines
  - Handle edge cases: empty data, single value, all identical values (return 1 bin)
- [ ] Unit tests for bin computation
- [ ] Wire histogram into Figure API — `HistogramMark` that internally computes bins and delegates to `BarMark`
- [ ] Snapshot test: histogram of normally distributed data (use deterministic seed)

> [!warning] No Rust crate implements auto bin selection
> Neither `ndhistogram` nor `histogram` implement Sturges/Scott/Freedman-Diaconis. Port directly from the formulas — NumPy's `histogram_bin_edges` source is the best reference.

**Hints**
- [NumPy `histogram_bin_edges`](https://numpy.org/doc/stable/reference/generated/numpy.histogram_bin_edges.html) — reference impl for `'sturges'`, `'scott'`, `'fd'`, `'auto'`
- [Wikipedia: Sturges' rule](https://en.wikipedia.org/wiki/Sturges's_rule) — `k = ⌈log₂(n) + 1⌉`
- [Wikipedia: Freedman–Diaconis rule](https://en.wikipedia.org/wiki/Freedman%E2%80%93Diaconis_rule) — `h = 2·IQR·n⁻¹ᐟ³`
- [Wikipedia: Scott's rule](https://en.wikipedia.org/wiki/Scott's_rule) — `h = 3.5σn⁻¹ᐟ³`
- [`ndhistogram` crate](https://docs.rs/ndhistogram/latest/ndhistogram/) — multi-dimensional, Uniform/Variable axes (no auto-bin, but good API reference)

### 10:30–10:45 — Break + small tasks

- [ ] **Update facade** `starsight/src/statistics.rs` — replace the TODO comment with actual re-exports: `pub use crate::components::statistics::histogram_bins;` (and `HistogramMark` if it lives in marks.rs)
- [ ] **Render reference screenshots** — run the new bar/area/step snapshot data through `SkiaBackend` → PNG and save to `docs/screenshots/`. Even rough ones are valuable for the README.

### 10:45–12:00 — Multi-series improvements

- [ ] Default color cycle: define 8–10 distinguishable default colors in `starsight-layer-1/src/primitives.rs` (or pull from prismatica if integration is ready)
- [ ] `Figure` auto-assigns colors from the cycle when marks don't specify one
- [ ] Snapshot test: multi-series line chart using auto colors
- [ ] Snapshot test: grouped bar chart (two `BarMark`s side-by-side with auto offset)

> [!tip] Color cycle approach
> Simplest path: hardcode Tableau 10 values as `Color` constants (they're in the public domain). If prismatica integration is ready, use `prismatica::matplotlib::TAB10` instead — but don't block on it.

### 12:00–13:00 — Lunch

### 13:00–14:30 — Documentation + examples

- [ ] Write `examples/quickstart.rs` — the 3-line `plot!` demo from README
- [ ] Write `examples/bar_chart.rs`
- [ ] Write `examples/area_chart.rs`
- [ ] Write `examples/histogram.rs`
- [ ] Verify all examples compile with `cargo build --examples`
- [ ] Update README.md: add chart type table showing what's available in 0.2.0
- [ ] **Update README roadmap** — check `0.2.0` items, correct the description (HeatmapMark moved out, StepMark and HistogramMark moved in)

### 14:30–14:45 — Break + small tasks

Pick 2–3 from this list:

- [ ] **Draw a data-to-pixel pipeline diagram** — ASCII art showing `DATA → Figure.add(mark) → DataExtent → Axis::auto_from_data → CartesianCoord → Mark::render → DrawBackend → OUTPUT`. Add to STARSIGHT.md or README architecture section. Style: Unicode box-drawing characters matching the existing architecture diagram.
- [ ] **Add `#[non_exhaustive]` to `StepPosition`** — future-proofs the enum (you'll want `Natural` and `CatmullRom` later)
- [ ] **Add `#[non_exhaustive]` to `StarsightError`** — already recommended by the spec but not yet applied
- [ ] **CONTRIBUTING.md "where to put what" table** — add rows for `StepMark`, `BarMark`, `AreaMark`, `HistogramMark`, and `statistics.rs`
- [ ] **CITATION.cff version** — bump if it still says `0.1.0`
- [ ] **Update layer-3 module doc** — the `//!` header in `marks.rs` lists `StepMark` under 0.3.0, `BarMark`/`AreaMark` under 0.2.0. Make it match reality after you're done.
- [ ] **Add doc-test to `BarMark::new`** — a 3-line `/// # Example` showing construction. Same for `AreaMark` and `StepMark`. These turn into compile-checked documentation.
- [ ] **`deny.toml` audit** — run `cargo deny check` and update the license allowlist if any new transitive deps landed

### 14:45–16:30 — Final sweep + release prep

- [ ] Full CI dry-run: `cargo test`, `cargo clippy`, `cargo deny check`, `cargo doc --no-deps`
- [ ] `cargo insta review` — accept all new snapshots, verify they look correct
- [ ] Bump version to `0.2.0` in workspace `Cargo.toml`
- [ ] Update CHANGELOG.md: move `[Unreleased]` items into `[0.2.0]` section with today's date
- [ ] Tag and push — let CI handle the crates.io publish

**Hints**
- [cargo-insta CLI reference](https://insta.rs/docs/cli/) — `cargo insta review`, `accept`, `reject`
- [insta snapshot types](https://insta.rs/docs/snapshot-types/) — SVG tested as plain text via `assert_snapshot!`
- [insta advanced features](https://insta.rs/docs/advanced/) — redactions, glob testing

### 16:30–17:00 — Buffer / stretch overflow

---

## Stretch goals

Pick from these if ahead of schedule. Ordered by impact.

- [ ] **Grid lines** — light gray horizontal/vertical lines at tick positions behind the data (add to `render_axes` in `renders.rs`)
- [ ] **Title rendering** — `Figure.title` is stored but not rendered; add a `draw_text` call above the plot area in `render_to`
- [ ] **Axis labels** — same for `x_label`/`y_label`; y-axis needs rotated text (or horizontal fallback)
- [ ] **Fill between two lines** — `AreaMark` variant with `y_low` + `y_high` instead of baseline
- [ ] **Grouped/stacked bars** — `BarMark` takes a `group` field; multiple series rendered side-by-side or stacked
- [ ] **prismatica default cycle** — use `prismatica::matplotlib::TAB10` as the default color cycle instead of hardcoded colors
- [ ] **Margin auto-computation** — measure tick label width to set left margin dynamically instead of the hardcoded 40px in `render_axes`
- [ ] **Architecture callout diagram** — draw an ASCII "call trace" showing what happens when `plot!(&x, &y).save("out.png")` executes: macro expansion → `Figure::from_arrays` → `SkiaBackend::new` → `render_to` → `render_background` → `render_axes` → `mark.render` → `backend.png_bytes` → `fs::write`. Add to STARSIGHT.md Part 1 or README.

---

## What not to touch

These are tempting but out of scope for this sprint:

- GPU backend / wgpu (0.6.0)
- Terminal backend / ratatui (0.8.0)
- Layout / faceting (0.4.0)
- Log scales (0.5.0)
- Interactivity (0.6.0)
- HeatmapMark (needs ColorScale infrastructure from 0.5.0 — moved from original 0.2.0 plan)

---

## End-of-sprint success criteria

**Minimum viable**: BarMark + AreaMark implemented, snapshot-tested, committed. 0.2.0 not necessarily published but the marks work.

**Target**: All of Thursday + Friday core blocks done. 0.2.0 tagged and on crates.io with BarMark, AreaMark, StepMark, HistogramMark, and 4+ examples.

**Stretch**: Grid lines, title rendering, and auto color cycle also landed.

---

## Small tasks checklist (at a glance)

Everything below is already woven into the schedule above at the appropriate break slot. Collected here for easy scanning.

### Stale docs that need updating
- [ ] CHANGELOG.md — add `[0.1.0]` and `[0.1.1]` sections, it still says unreleased
- [ ] README roadmap — check `0.1.0`, update `0.2.0` description (swap HeatmapMark for StepMark+Histogram)
- [ ] README Features table — `LineMark`, `PointMark`, CPU rendering, SVG export are `working` not `wip`
- [ ] README "Coming from" table — `BarMark::new(labels, vals)` → `BarMark::new(x, y)` (numeric, not string categories)
- [ ] CITATION.cff version bump
- [ ] CONTRIBUTING.md table — add new mark types

### Facade wiring
- [ ] `starsight/src/marks.rs` — add `BarMark, AreaMark, StepMark` to `pub use`
- [ ] `starsight/src/prelude.rs` — add new marks to prelude
- [ ] `starsight/src/statistics.rs` — replace TODO with real re-exports

### Code quality micro-fixes
- [ ] SvgBackend opacity bug — `draw_path` never emits `opacity`/`fill-opacity`
- [ ] `marks.rs` module doc header — milestone assignments out of sync with TODO comments
- [ ] `#[non_exhaustive]` on `StepPosition` and `StarsightError`
- [ ] Doc-tests on `BarMark::new`, `AreaMark::new`, `StepMark::new`
- [ ] `cargo deny check` — update license allowlist if new deps

### Visual / architecture
- [ ] Reference PNGs in `docs/screenshots/` for new chart types
- [ ] Data-to-pixel pipeline ASCII diagram
- [ ] Call trace ASCII diagram (`plot!` → `save` → backend)

---

## Quick reference links

### tiny-skia 0.12
| What | Link |
|---|---|
| Crate root | <https://docs.rs/tiny-skia/0.12.0/tiny_skia/> |
| `Paint` | <https://docs.rs/tiny-skia/latest/tiny_skia/struct.Paint.html> |
| `Pixmap` | <https://docs.rs/tiny-skia/latest/tiny_skia/struct.Pixmap.html> |
| `PathBuilder` | <https://docs.rs/tiny-skia/latest/tiny_skia/struct.PathBuilder.html> |
| `Rect` | <https://docs.rs/tiny-skia/latest/tiny_skia/struct.Rect.html> |
| `Color` | <https://docs.rs/tiny-skia/latest/tiny_skia/struct.Color.html> |
| GitHub | <https://github.com/linebender/tiny-skia> |

### SVG
| What | Link |
|---|---|
| `svg` crate docs | <https://docs.rs/svg/latest/svg/> |
| MDN `fill-opacity` | <https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Attribute/fill-opacity> |
| MDN path commands | <https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial/Paths> |

### Testing
| What | Link |
|---|---|
| insta docs | <https://docs.rs/insta> |
| insta CLI | <https://insta.rs/docs/cli/> |
| `assert_snapshot!` | <https://docs.rs/insta/latest/insta/macro.assert_snapshot.html> |

### Reference implementations
| What | Link |
|---|---|
| d3 `step.js` source | <https://github.com/d3/d3-shape/blob/main/src/curve/step.js> |
| matplotlib step demo | <https://matplotlib.org/stable/gallery/lines_bars_and_markers/step_demo.html> |
| plotters `Histogram` | <https://docs.rs/plotters/latest/plotters/series/struct.Histogram.html> |
| NumPy bin edges | <https://numpy.org/doc/stable/reference/generated/numpy.histogram_bin_edges.html> |
