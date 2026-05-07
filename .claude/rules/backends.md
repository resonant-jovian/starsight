---
paths:
  - "starsight-layer-1/**/*.rs"
---

# Layer-1 backends

Layer-1 (`starsight-layer-1`, alias `background`) is the foundation: primitives, backends, paths, errors, colormaps, theme. **No layer below it** — layer-1 may not depend on any other starsight crate.

## Backend split

- **`SkiaBackend`** (`backends::skia`): raster output via `tiny-skia`. The default for `Figure::save("foo.png")`. Anti-aliasing is per-path; the recent `fix(backend)` fb984d8 added auto-detection of axis-aligned paths to disable AA on them (sharper bars / axes / gridlines).
- **`SvgBackend`** (`backends::svg`): deterministic SVG output. Used by snapshot tests because it keeps text as `<text>` elements, no glyph rasterization, byte-exact across OS/fonts. Supports opacity.
- **`DrawBackend` trait**: the contract both implement. Adding a new backend means implementing this trait and adding dispatch in `Figure::save` (layer-5) keyed off file extension.

## When to use which

- User-facing `Figure::save` dispatches on extension: `.png` → Skia, `.svg` → Svg.
- Snapshot tests in layer-5 always go through SVG — never PNG-encode in tests.
- The PNG raster path is exercised by `starsight/tests/integration.rs` and the layer-1 `blue_rect_on_white` test, both of which avoid font rendering.

## Paths & primitives

- `paths::{Path, PathCommand, PathStyle}` is the IR every backend consumes. `PathCommand` is the move/line/cubic/close enum.
- AA decision lives at draw time: `Path::is_axis_aligned()` (added in fb984d8) returns `true` if every segment is horizontal or vertical with integer-aligned coords.

## Errors

`StarsightError` is here. Variants are added when a new failure mode crosses the public API; otherwise reuse `Generic(String)` or the closest existing variant.
