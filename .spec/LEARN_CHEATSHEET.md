# LEARN.md — One-Page Quick Reference

66 chapters, 33,858 words condensed to one page. Check what you know.

---

## Rust (Chapters 1–17)

Ownership: each value has exactly one owner. Pixmap owns its pixel buffer. When the owner goes out of scope the memory is freed. Borrowing: `&T` is shared (read-only, many at once), `&mut T` is exclusive (one at a time). The backend needs `&mut self` because rendering mutates the pixel buffer. Copy types (`f32`, `Point`, `Color`) are duplicated on assignment. Clone types (`Vec`, `String`, `Pixmap`) require explicit `.clone()`. Standard derives: `Debug` (print), `PartialEq` (compare), `Clone`, `Copy`, `Default`. Traits define shared behavior. `DrawBackend` declares `fill_rect`, `draw_path`, `draw_text`. Trait objects (`&dyn DrawBackend`, `Box<dyn DrawBackend>`) enable runtime backend selection but the trait must be object-safe: no generics on methods, no `Self` in return position, no `where Self: Sized`. Generics (`fn foo<T: Trait>`) for compile-time dispatch; trait objects for runtime dispatch. `From`/`Into` for type conversions. The `?` operator propagates errors by calling `.into()` on the error type. `thiserror` generates `From` impls and `Display` for error enums. Iterators: `.map()`, `.filter()`, `.collect()`. Closures: `Fn` (shared borrow), `FnMut` (mutable borrow), `FnOnce` (moves). cosmic-text's draw callback uses `FnMut`. Lifetimes ensure references stay valid. Module tree mirrors file tree; `pub(crate)` for internal visibility.

## Computer Graphics (Chapters 18–21)

A pixel is an RGB triplet in a grid. Pixmap is `width × height × 4` bytes (RGBA). Bresenham's algorithm draws lines by choosing the nearest pixel at each step. Bézier curves: quadratic (1 control point) and cubic (2 control points). tiny-skia flattens curves into line segments. Anti-aliasing: compute how much of each pixel is covered by the shape (0.0–1.0), blend proportionally. sRGB is nonlinear — gamma ~2.2. Blending must linearize first, blend, then re-encode. Premultiplied alpha: store `(r×a, g×a, b×a, a)` instead of `(r, g, b, a)`. tiny-skia uses premultiplied internally. WCAG contrast ratio: `(L1 + 0.05) / (L2 + 0.05)` where L = relative luminance. Minimum 4.5:1 for normal text. Luminance = `0.2126×R + 0.7152×G + 0.0722×B` (after linearizing sRGB).

## Visualization Theory (Chapters 22–28)

Grammar of graphics: data → scales → coordinate system → marks → output. A mark is a geometric primitive (line, point, bar, area) mapped from data. Scales map data domain to visual range. Linear: `(v - min) / (max - min)`. Log: `log(v)` then linear. Band: categories → equal-width bands with padding. Wilkinson Extended tick algorithm: triple nested loop scoring simplicity, coverage, density, legibility. Weights: `0.2, 0.25, 0.5, 0.05`. Nice numbers: `{1, 2, 2.5, 5} × 10^n`. No existing Rust implementation — starsight's is a genuine contribution. Axis = scale + ticks + labels. CartesianCoord bundles x-axis, y-axis, and plot area rectangle. `data_to_pixel(x, y)` maps data coordinates to screen `Point`. Y is inverted: `y_px = top + (1 - t) × height`. SVG backend: XML elements, CSS hex colors, `<clipPath>` for clipping. No font rasterization — text is `<text>` elements.

## Architecture & Tooling (Chapters 29–55)

Seven layers, each a separate crate. L1: primitives, error, backends. L2: scales, axes, coords. L3: marks, stats. L4: layout, faceting, legend. L5: Figure, `plot!()`, data acceptance. L6: interactivity. L7: animation, export. Each layer depends only on lower layers. `starsight` facade re-exports everything. Edition 2024, resolver 3, GPL-3.0-only. No async in the rendering pipeline. Scene graph: tree of `SceneNode` enums (Path, Text, Group, Clip). Builder pattern: methods take `self`, return `Self`. `#[non_exhaustive]` on public enums and config structs. Testing: snapshot tests with `insta`, property tests with `proptest`. CI: fmt → clippy → check → test matrix (stable + MSRV 1.85, Linux/macOS/Windows) → cargo-deny. Workspace: shared `[workspace.package]`, `[workspace.lints]`, `[workspace.dependencies]`. clippy pedantic warn, `unsafe_code` forbid. `thiserror` for errors, `tiny-skia 0.12` for CPU rendering (renamed `ClipMask` → `Mask`), `cosmic-text` for text (FontSystem + SwashCache). R↔B channel swap affects softbuffer display only, not PNG/SVG. `cargo xtask` for code generation. `git-cliff` for changelogs. Publish order: layer-1 first, facade last. 0.1.0 MVP: primitives + Skia backend + scales + ticks + axes + LineMark + PointMark + Figure + `plot!()`.

## Supplementary (Chapters 56–66)

Effective charts: high data-ink ratio, no chartjunk, colorblind-safe palettes (prismatica's `BATLOW`). Accessibility: alt text, patterns not just color, sufficient contrast. Affine transforms compose by matrix multiplication — order matters (`translate × rotate ≠ rotate × translate`). PNG: deflate compression on filtered scanlines. Filter choices: None, Sub, Up, Average, Paeth. SVG is vector XML; resolution-independent but no pixel-level control. Rust's type system prevents unit confusion: `Point` (position) vs `Vec2` (displacement) — `Point + Point` does not compile. `Result` and `Option` compose with iterators via `.filter_map()`, `.collect::<Result<Vec<_>, _>>()`. Module privacy: `pub` items in private modules are unreachable from outside the crate. Rendering allocations: Pixmap (~2MB), PathBuilder, cosmic-text buffers. Don't optimize before profiling.

---

*Full document: LEARN.md (66 chapters, 33,858 words)*
