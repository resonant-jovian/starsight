---
paths:
  - "starsight-layer-3/**/*.rs"
---

# Layer-3 marks

Layer-3 (`starsight-layer-3`, alias `components`) holds the mark types and the statistics primitives they consume.

## Dependency boundary

Layer-3 may import from `starsight_layer_1` (primitives, backends, paths, color, theme) and `starsight_layer_2` (scales, coords, axes, ticks). It must not import from layer-4..7. Use `cargo tree -e=normal -p starsight-layer-3` to verify before merging.

## Mark conventions

- **Builder shape**: `LineMark::new(x, y).color(...).label("...")`. Required data goes in `new()`; everything else is a chained setter.
- **NaN = gap** for `LineMark`, `AreaMark`. Don't filter NaNs at construction; the renderer breaks the path on NaN.
- **Per-element styling**: bars and points support `per_bar_color`, `per_point_color`, etc. — pass a `Vec<Color>` parallel to the data. If shorter than data, cycle.
- **Returns `Result<T>`** from any mark method that can fail at construction (mismatched array lengths, empty data). Don't panic.

## Legend & inference

- Each mark implements a `LegendGlyph` dispatch — when added to a `Figure` with a label, it emits a glyph variant (`line` / `point` / `bar` / `area`) that layer-4 renders into the legend rect. Adding a new mark? Choose an existing glyph or extend the enum.
- `infer_chart_kind` (in this layer) examines a mark to decide axis defaults, label autoplacement, and aspect-ratio hints. New marks should add an arm.

## Statistics

- `statistics::Kde` (Gaussian; `Bandwidth::{Silverman, Scott, Manual(f64)}`) drives `ViolinMark`.
- `statistics::BoxPlotStats` produces five-number summary + outliers for `BoxPlotMark`.
- `percentile`, `std_dev` helpers live alongside.
- These are the only stat primitives in 0.3.x — adding new ones (e.g. quantile regression) is a 0.4.x+ concern.

## Recent fix patterns

- `fix(pie)` 6c26aed: pie-only figures now suppress axes; slice labels auto-contrast against fill.
- `fix(candlestick)` 14fef6c: edge bars no longer half-clip — use `coord.range_padding()` semantics.
- `fix(legend)` b38698e: contrasting border around legend rect for visual separation from background theme.
