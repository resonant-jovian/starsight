---
description: Locate where a feature, type, or symbol lives in the layered workspace. Faster than blind grep — uses the layer map + facade re-exports. Use when the user mentions a symbol you can't immediately place.
argument-hint: "<symbol-or-feature-name>"
---

# /scout — locate this in the layer stack

Steps, in order. Stop as soon as a step yields a confident answer.

## 1. Layer map

Map the symbol to a layer based on category:

| Concept | Layer | Crate |
|---|---|---|
| primitive (Color, Path, theme), backend, errors, colormap | 1 | `starsight-layer-1` |
| scale, coord, axis, tick | 2 | `starsight-layer-2` |
| mark (Line/Point/Bar/Area/Heatmap/Histogram/Step/BoxPlot/Violin/Pie/Candlestick), Kde, BoxPlotStats | 3 | `starsight-layer-3` |
| layout, legend dispatch | 4 | `starsight-layer-4` |
| Figure, `plot!` macro, snapshot tests | 5 | `starsight-layer-5` |
| winit, hover, zoom, pan | 6 | `starsight-layer-6` (mostly empty in 0.3.x) |
| PDF, GIF, HTML, WASM export | 7 | `starsight-layer-7` (mostly empty in 0.3.x) |

## 2. Facade re-exports

If the symbol is a public API the user calls via `starsight::…`, check `starsight/src/lib.rs` first — that file maps every public re-export to its layer. Three access patterns:

- `use starsight::prelude::*;` — common types
- `use starsight::marks::LineMark;` — by category
- `use starsight::components::marks::LineMark;` — by Latin layer alias (`background`/`modifiers`/`components`/`composition`/`common`/`interactivity`/`export`)

## 3. Grep the candidate layer

Once you have a candidate layer, narrow with ripgrep:

```bash
rg -n -t rust 'fn <symbol>|struct <symbol>|trait <symbol>|impl <symbol>' starsight-layer-N/src/
```

If multiple hits: prefer the one in `mod.rs` or `lib.rs` (likely the canonical definition over a re-export).

## 4. AGENTS.md "What works now"

For feature-level questions ("does it support log heatmap?"), check the AGENTS.md "What works now (0.3.x)" list — it's the canonical inventory. If it's there, it's implemented; if it's in "Not yet implemented", it isn't.

## 5. Master spec

For deep architectural context, `.spec/STARSIGHT.md` (~250KB) is the design document. Don't read it whole — `rg <symbol> .spec/STARSIGHT.md` first.

## Reporting

One short paragraph: where the symbol is defined (file:line), how it's re-exported through the facade (if at all), and the layer's role. Don't dump source — link the user to the right file.
