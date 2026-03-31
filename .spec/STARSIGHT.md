# starsight — development reference

> Authored by Claude Opus 4.6 (Anthropic) with Albin Sjögren. Last generated: 2026-03-31.

This document has four parts. Use the one you need.

**Part 1 — Listen** is pure prose with no formatting, no code blocks, no tables. Read it or pipe it through text-to-speech as preparation before sitting down to code. It explains every concept, every decision, every tricky bit in plain sentences.

**Part 2 — Build** is the task list with code blocks. Every item has the level of detail needed to implement it without looking anything else up. When it says create a struct, it shows you the exact code.

**Part 3 — Look up** is the reference section. Type signatures, dependency APIs, conversion formulas, algorithm pseudocode. Come here when you are mid-implementation and need to check a specific detail.

**Part 4 — Navigate** is the architecture map. Tree structures showing what goes where, crate dependency graphs, module layouts. Come here when you need to know which file to create or which crate a type belongs in.

---
---

# Part 1 — Listen

Read this section or have it read to you. No code blocks, no tables, no formatting that breaks text-to-speech. Just the full story of what starsight is, how it works, and what you need to know before writing a single line.

## What starsight is

starsight is a scientific visualization library for Rust. It exists because Rust has no equivalent of Python's matplotlib. The current options are plotters (powerful but verbose and stagnating), plotly-rs and charming (which secretly bundle JavaScript engines), egui_plot (locked to the egui framework), and textplots (terminal only). Researchers working in Rust end up exporting CSV and plotting in Python. starsight fixes this.

The library provides one import, sixty chart types, and five rendering backends. A user writes "plot x y dot save chart dot png" and gets a chart. A power user writes a grammar-of-graphics figure with layered marks, custom scales, faceting, and publication-quality PDF export. Both use the same library.

starsight belongs to the resonant-jovian ecosystem. Its sister crates are prismatica, which provides 308 scientific colormaps as compile-time lookup tables, and chromata, which provides 1104 editor color themes as compile-time constants. These are not optional integrations. They are the actual color and theme systems starsight uses. When starsight needs a viridis colormap, it calls prismatica dot crameri dot BATLOW dot eval of 0.5 and gets an RGB color back. When starsight needs a dark theme background color, it reads chromata dot popular dot gruvbox dot DARK HARD dot bg and gets three bytes.

## The layer architecture

The library is organized into seven layers, each a separate crate. Layer one is the foundation. Layer seven is the roof. Each layer depends only on layers below it. This is enforced by Cargo dependencies, not by convention. starsight-layer-3 literally cannot import anything from starsight-layer-5 because it is not in its dependency list.

Layer one is the rendering abstraction. It contains geometry primitives like Point, Rect, Size, and Color. It contains the DrawBackend trait that all rendering backends implement. It contains the Scene type that accumulates drawing commands. It contains the error types. It contains the backend implementations for tiny-skia (CPU), SVG, PDF, wgpu (GPU), and terminal (Kitty, Sixel, iTerm2, half-block, Braille). Everything in starsight ultimately bottoms out at layer one.

Layer two is the scale, axis, and coordinate system. A scale maps data values to pixel positions. A linear scale maps the range zero to one hundred onto the range zero to eight hundred pixels. A log scale does the same but logarithmically. Layer two also contains the tick generation algorithm, which decides where to place axis labels. starsight uses the Wilkinson Extended algorithm, which optimizes a scoring function over simplicity, coverage, density, and legibility. No Rust crate implements this algorithm. starsight will be the first. Layer two also contains coordinate systems. Cartesian is the default. Polar wraps angles. Geographic projects latitude and longitude.

Layer three is the mark and stat system. This is the grammar of graphics layer. A mark is a visual element: a point, a line, a bar, an area, a rect, an arc. A stat is a data transform: binning, kernel density estimation, regression, boxplot summary. An aesthetic mapping connects data columns to visual properties: x position, y position, color, size, shape. Position adjustments handle overlapping marks: dodge, stack, jitter. This layer does not render anything. It describes what should be rendered.

Layer four is layout and composition. Grid layouts arrange multiple charts in rows and columns. Faceting splits data by a categorical variable and creates one chart per value. Legends map visual encodings back to data values. Colorbars show the continuous color scale. Inset axes place a small chart inside a bigger one. This layer arranges charts but does not render them.

Layer five is the high-level API. The plot macro lives here. The Figure builder lives here. Data acceptance for Polars DataFrames, ndarray arrays, and Arrow RecordBatches lives here. Auto-inference of chart types from data shape lives here. This is the layer most users interact with.

Layer six is interactivity. Hover tooltips, box zoom, wheel zoom, pan, lasso selection, linked views between multiple charts, streaming data with rolling windows. This layer requires a windowing system (winit for native, web-sys for browser) and is entirely optional.

Layer seven is animation and export. Frame recording for GIF and MP4. Transition animations between chart states. Static export to PNG, SVG, PDF. Interactive HTML export. Terminal inline output with automatic protocol detection.

## Why Point and Vec2 are different types

This is a pattern from egui and from game engine math libraries. A Point is a position in space. The pixel at x equals 100, y equals 200. A Vec2 is a displacement. Fifty pixels to the right, thirty pixels down.

They are both two floats. But the valid operations are different. Subtracting one point from another gives a displacement, a Vec2. The distance from your house to the grocery store is a displacement, not a location. Adding a displacement to a point gives a new point. Your house plus the displacement to the grocery store gives the grocery store's location. But adding two points together is meaningless. Your house plus the grocery store is not a place.

The type system enforces this. Point minus Point returns Vec2. Point plus Vec2 returns Point. Point plus Point does not compile. This catches real bugs. In chart layout code, you deal with positions (where does this axis label go) and offsets (how much margin do I add). If they are both just float tuples, nothing stops you from accidentally adding two positions together and getting garbage coordinates. With separate types, the compiler catches this.

Vec2 also supports scalar multiplication. A displacement times two is twice as far in the same direction. A position times two is nonsensical. So Vec2 implements multiplication by f32, and Point does not.

## Why Color has no alpha field in the current code

The Color struct in primitives dot rs has three fields: r, g, b, all u8. There is no alpha channel. This is deliberate for the initial implementation. Most chart elements are fully opaque. The backgrounds, the axis lines, the tick labels, the titles. Alpha becomes important later for overlapping scatter points, area fill transparency, and hover highlight overlays. When alpha is needed, it should be a separate type or an optional wrapper, not baked into the base Color struct, because premultiplied alpha and straight alpha are different things and conflating them causes bugs. Tiny-skia internally uses premultiplied alpha (each RGB channel is already multiplied by the alpha value). The image crate expects straight alpha. If you store alpha in your Color type without tracking which kind it is, you will get wrong colors when converting between libraries.

For now, the Color struct matches chromata's Color and prismatica's Color, both of which are three u8 fields with no alpha. Conversion between them is zero-cost: just move the bytes.

## How tiny-skia rendering actually works

tiny-skia is a CPU rasterizer. You create a Pixmap (a pixel buffer), you draw paths and shapes onto it, you encode it as PNG. The Pixmap stores premultiplied RGBA pixels. Every pixel is four bytes: red, green, blue, alpha, where each RGB byte has already been multiplied by the alpha value divided by 255.

To draw a line, you build a Path. You call PathBuilder new, then move to the start point, then line to the end point, then finish. The finish method returns Option of Path. It returns None if the path is empty, which happens if you called finish without adding any segments.

To actually paint the path onto the Pixmap, you need a Paint struct and a Stroke struct. The Paint holds the color (via a Shader, which defaults to solid color) and the blend mode (default SourceOver). The Stroke holds the line width, line cap (Butt, Round, or Square), line join (Miter, Round, or Bevel), and optional dash pattern.

Then you call pixmap dot stroke path, passing the path, the paint, the stroke, a Transform (use identity for no transformation), and an optional Mask (pass None for no clipping, or pass Some of a Mask to restrict drawing to a region).

The critical thing about Transform is that its rotation method takes degrees, not radians. This is unlike virtually every other math library. If you pass pi divided by two expecting a 90-degree rotation, you will get a 1.57-degree rotation instead.

For text, starsight uses cosmic-text. You create a FontSystem (which loads system fonts and takes about one second in release mode), a SwashCache (no arguments), and a Buffer (with a Metrics struct specifying font size and line height in pixels). You set the text, call shape until scroll to lay it out, then call draw with a callback that receives individual glyph rectangles. Each callback invocation gives you an x, y, width, height, and color. You paint each rectangle onto the Pixmap using fill rect.

There is a persistent myth that you need to swap the red and blue channels between cosmic-text and tiny-skia. You do not. That swap exists in the cosmic-text example code because the example renders to softbuffer, which uses a different byte order. For PNG and SVG output, pass the channels straight through.

## How prismatica colormaps work

A Colormap in prismatica is a lookup table. It stores 256 RGB triplets as a static array of u8 three-element arrays compiled into the binary. When you call eval with a float between zero and one, it scales the float to the array index, interpolates linearly between the two nearest entries, and returns a Color.

The interpolation is in sRGB space, not linear space. This matches matplotlib, ParaView, and most scientific tools. Perceptual uniformity comes from how the lookup table was constructed (by Crameri, or the CET group, or matplotlib's team), not from the interpolation method.

eval rational takes two integers, i and n, and returns the i-th of n evenly spaced samples. This is useful when you have categorical data with n categories and want n distinct colors from a sequential map.

reversed returns a ReversedColormap, which is a zero-allocation wrapper that internally calls eval with one minus t. It does not copy or reverse the lookup table.

A DiscretePalette is different from a Colormap. It stores a fixed set of distinct colors for categorical data. It has get which takes an index and wraps around if the index exceeds the palette size. It has iter which returns an iterator over all colors without allocation.

## How chromata themes work

A Theme in chromata has 29 color fields plus metadata. The bg and fg fields are always present. Everything else is Option of Color because not every source theme defines every semantic role. The accent method returns the first available accent color, checking blue, then purple, then cyan, then green, then orange, then red, falling back to fg if none are defined.

The Theme struct is non-exhaustive, meaning you cannot construct it with struct literal syntax outside the crate. Use the builder: Theme builder of name, author, bg color, fg color, then chain optional setters, then call build. The build method auto-detects variant (dark if background luminance is 0.5 or below) and contrast level (from the WCAG contrast ratio between bg and fg).

## The Wilkinson Extended tick algorithm

This is the algorithm that decides where to put tick marks on an axis. Given a data range (say 3.7 to 97.2) and a desired number of ticks (say 5 to 10), it finds the "nicest" set of tick positions. Nice means: prefer round numbers (10, 20, 30 over 13.7, 27.4, 41.1), cover the data range without too much whitespace, get close to the desired tick count, and include zero if the data range spans zero.

The algorithm searches over a preference-ordered list of step bases: 1, 5, 2, 2.5, 4, 3. These are ordered by human readability. Steps of 1 (giving ticks at 10, 20, 30) are preferred over steps of 5 (giving ticks at 5, 10, 15, 20) which are preferred over steps of 2 (giving ticks at 2, 4, 6, 8, 10). The skip factor j multiplies these: skip 2 with base 5 gives step 10, which normalizes to base 1 at the next order of magnitude.

The scoring function combines four components. Simplicity (weight 0.2) rewards earlier entries in the preference list and lower skip factors. Coverage (weight 0.25) penalizes whitespace between the data range and the label range. Density (weight 0.5, the heaviest) penalizes having too many or too few ticks compared to the target count. Legibility (weight 0.05) is simplified to a constant.

The algorithm uses nested loops over j, q, k (number of ticks), z (power of ten), and start position, with aggressive pruning. At each nesting level, it computes an upper bound on the score achievable by any remaining candidate. If that upper bound is below the best score found so far, it breaks out of the loop. This makes the average iteration count about 41, which is fast enough for real-time use.

No Rust crate implements this algorithm. D3 uses a simpler formula with only three step bases. Plotters uses basic rounding. starsight will be the first Rust implementation of the full Extended Wilkinson algorithm.

## What SVG cannot do

SVG is a text format for vector graphics. starsight generates SVG documents using the svg crate, which provides a builder API: Document new, set viewBox, add elements. Each element (Path, Rectangle, Circle, Text, Group) is built with chained set calls.

The critical limitation of SVG is that you cannot measure text width without a rendering engine. The width of the string "123.45" depends on the font, the font size, kerning tables, and ligature rules. A browser can measure this after layout. A static SVG generator cannot. starsight works around this by estimating: digits are approximately 0.55 times the font size wide, average characters approximately 0.6 times. For precise measurement when generating PNG (not SVG), cosmic-text handles measurement after shaping.

Text positioning in SVG uses the baseline, not the bounding box. The x and y attributes set where the text baseline starts. To center text horizontally, set text-anchor to middle. To center vertically, set dominant-baseline to central. To rotate a Y-axis label, apply a transform: translate to the label position, then rotate negative 90 degrees.

## Edition 2024 things that matter

Rust edition 2024 (shipped with Rust 1.85) changed several things relevant to starsight. The gen keyword is now reserved for future generators, so any identifier named gen must become r#gen. The unsafe_op_in_unsafe_fn lint is now warn by default, meaning unsafe operations inside unsafe functions need explicit unsafe blocks. RPIT (return position impl trait) lifetime capture rules changed: functions returning impl Trait now capture all in-scope lifetimes by default, which can affect public API signatures.

Resolver 3 (implied by edition 2024) adds MSRV-aware dependency resolution. If a dependency's latest version requires a newer Rust than your declared rust-version, Cargo falls back to an older compatible version. Feature unification behavior is unchanged from resolver 2.


---
---

# Part 2 — Build

Every task below has enough detail to implement it without looking anything else up. Items are ordered by dependency. Do not skip ahead. When a task says to create a struct, it tells you the fields, the derives, the trait implementations, and why.

Checked items reflect the current state of the codebase as of 2026-03-31.

---

## Pre-0.1.0 — Workspace bootstrap

These are done. Listed for audit completeness.

- [x] Create resonant-jovian/starsight GitHub repository
- [x] Add GPL-3.0-only LICENSE
- [x] Create CONTRIBUTING.md, CODE_OF_CONDUCT.md, CHANGELOG.md, SECURITY.md
- [x] Create .github/ISSUE_TEMPLATE/ (bug_report.md, feature_request.md, config.yml)
- [x] Create .github/PULL_REQUEST_TEMPLATE.md
- [x] Create .github/FUNDING.yml
- [x] Initialize workspace Cargo.toml with resolver 3, edition 2024, all workspace members
- [x] Create all 8 crate Cargo.toml files (starsight, layer-1 through layer-7) with workspace inheritance
- [x] Create xtask/Cargo.toml
- [x] Define all feature flags in starsight/Cargo.toml
- [x] Configure workspace lints: unsafe_code forbid, clippy pedantic warn
- [x] Create .rustfmt.toml and .clippy.toml with full config
- [x] Create deny.toml for cargo-deny
- [x] Configure profile.release (LTO, codegen-units 1) and profile.dev (opt-level 1)
- [x] Create .github/workflows/ci.yml (fmt, clippy, check, test matrix, deny)
- [x] Create .github/workflows/release.yml (publish, GitHub release with git-cliff)
- [x] Create .github/workflows/coverage.yml (cargo-llvm-cov, Codecov upload)
- [x] Create .github/workflows/snapshots.yml (cargo insta test, artifact upload on failure)
- [x] Create .github/workflows/gallery.yml (xtask gallery, artifact upload)
- [x] Create README.md with badges, feature table, roadmap
- [x] Create starsight-layer-1/src/error.rs with StarsightError enum (7 variants) and Result type alias
- [x] Create starsight-layer-1/src/backend/mod.rs with DrawBackend trait (partial, some methods commented)
- [x] Create starsight-layer-1/src/primitives.rs with Color (r/g/b u8), Point (x/y f32), Rect (ltrb f32), Size (wh f32)
- [x] Create From<tiny_skia::Point> for Point, From<tiny_skia::Rect> for Rect, From<tiny_skia::Size> for Size
- [x] Create all stub module files for every backend (skia/, svg/, pdf/, wgpu/, terminal/)
- [x] Create all stub lib.rs files for layers 2-7
- [x] Verify cargo check --workspace passes
- [x] Verify cargo test --workspace passes (zero tests, zero failures)

---

## 0.1.0 — Foundation

Exit criteria: plot!([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]).save("test.png") produces a correct line chart PNG.

### Layer 1: Complete the primitive types

#### Add Vec2 with semantic arithmetic

- [ ] Create `Vec2` in `starsight-layer-1/src/primitives.rs`. A Vec2 is a displacement, not a position. The grocery store minus your house is a Vec2. The grocery store itself is a Point.

    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct Vec2 {
        pub x: f32,
        pub y: f32,
    }

    impl Vec2 {
        pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
        pub const X: Self = Self { x: 1.0, y: 0.0 };
        pub const Y: Self = Self { x: 0.0, y: 1.0 };

        pub const fn new(x: f32, y: f32) -> Self { Self { x, y } }

        pub fn length(self) -> f32 { (self.x * self.x + self.y * self.y).sqrt() }

        pub fn normalize(self) -> Self {
            let len = self.length();
            if len == 0.0 { Self::ZERO } else { Self { x: self.x / len, y: self.y / len } }
        }
    }
    ```

- [ ] Implement the semantic arithmetic. This is the entire point of having two types. `Point - Point = Vec2` (displacement between positions). `Point + Vec2 = Point` (shift a position). `Point + Point` does not compile (meaningless). `Vec2 + Vec2 = Vec2` (compose displacements). `Vec2 * f32 = Vec2` (scale a displacement). `Point * f32` does not compile (scaling a position is meaningless).

    ```rust
    impl std::ops::Sub for Point {
        type Output = Vec2;
        fn sub(self, rhs: Point) -> Vec2 {
            Vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
        }
    }

    impl std::ops::Add<Vec2> for Point {
        type Output = Point;
        fn add(self, rhs: Vec2) -> Point {
            Point { x: self.x + rhs.x, y: self.y + rhs.y }
        }
    }

    impl std::ops::Sub<Vec2> for Point {
        type Output = Point;
        fn sub(self, rhs: Vec2) -> Point {
            Point { x: self.x - rhs.x, y: self.y - rhs.y }
        }
    }

    impl std::ops::Add for Vec2 {
        type Output = Vec2;
        fn add(self, rhs: Vec2) -> Vec2 {
            Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
        }
    }

    impl std::ops::Sub for Vec2 {
        type Output = Vec2;
        fn sub(self, rhs: Vec2) -> Vec2 {
            Vec2 { x: self.x - rhs.x, y: self.y - rhs.y }
        }
    }

    impl std::ops::Mul<f32> for Vec2 {
        type Output = Vec2;
        fn mul(self, rhs: f32) -> Vec2 {
            Vec2 { x: self.x * rhs, y: self.y * rhs }
        }
    }

    impl std::ops::Mul<Vec2> for f32 {
        type Output = Vec2;
        fn mul(self, rhs: Vec2) -> Vec2 {
            Vec2 { x: self * rhs.x, y: self * rhs.y }
        }
    }

    impl std::ops::Neg for Vec2 {
        type Output = Vec2;
        fn neg(self) -> Vec2 {
            Vec2 { x: -self.x, y: -self.y }
        }
    }
    ```

- [ ] Add `From`/`Into` conversions for interop with other libraries:

    ```rust
    impl From<[f32; 2]> for Point { fn from([x, y]: [f32; 2]) -> Self { Self { x, y } } }
    impl From<(f32, f32)> for Point { fn from((x, y): (f32, f32)) -> Self { Self { x, y } } }
    impl From<Point> for [f32; 2] { fn from(p: Point) -> Self { [p.x, p.y] } }
    impl From<Point> for (f32, f32) { fn from(p: Point) -> Self { (p.x, p.y) } }
    // Same four impls for Vec2
    ```

- [ ] Write tests:

    ```rust
    #[test]
    fn point_minus_point_is_vec2() {
        let a = Point::new(10.0, 20.0);
        let b = Point::new(3.0, 5.0);
        let v: Vec2 = a - b;
        assert_eq!(v, Vec2::new(7.0, 15.0));
    }

    #[test]
    fn point_plus_vec2_is_point() {
        let p = Point::new(1.0, 2.0);
        let v = Vec2::new(10.0, 20.0);
        let result: Point = p + v;
        assert_eq!(result, Point::new(11.0, 22.0));
    }

    #[test]
    fn vec2_scale() {
        assert_eq!(Vec2::new(3.0, 4.0) * 2.0, Vec2::new(6.0, 8.0));
        assert_eq!(2.0 * Vec2::new(3.0, 4.0), Vec2::new(6.0, 8.0));
    }

    #[test]
    fn vec2_length() {
        assert!((Vec2::new(3.0, 4.0).length() - 5.0).abs() < f32::EPSILON);
    }
    ```

#### Complete the Rect type

- [ ] Add convenience constructors and accessors:

    ```rust
    impl Rect {
        pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
            Self { left: x, top: y, right: x + width, bottom: y + height }
        }

        pub fn from_center_size(center: Point, size: Size) -> Self {
            let half_w = size.width * 0.5;
            let half_h = size.height * 0.5;
            Self { left: center.x - half_w, top: center.y - half_h,
                   right: center.x + half_w, bottom: center.y + half_h }
        }

        pub fn width(&self) -> f32 { self.right - self.left }
        pub fn height(&self) -> f32 { self.bottom - self.top }
        pub fn size(&self) -> Size { Size::new(self.width(), self.height()) }
        pub fn center(&self) -> Point {
            Point::new((self.left + self.right) * 0.5, (self.top + self.bottom) * 0.5)
        }
        pub fn top_left(&self) -> Point { Point::new(self.left, self.top) }
        pub fn bottom_right(&self) -> Point { Point::new(self.right, self.bottom) }

        pub fn contains(&self, p: Point) -> bool {
            p.x >= self.left && p.x <= self.right && p.y >= self.top && p.y <= self.bottom
        }

        pub fn intersection(&self, other: &Rect) -> Option<Rect> {
            let r = Rect {
                left: self.left.max(other.left), top: self.top.max(other.top),
                right: self.right.min(other.right), bottom: self.bottom.min(other.bottom),
            };
            if r.left < r.right && r.top < r.bottom { Some(r) } else { None }
        }

        pub fn pad(&self, amount: f32) -> Rect {
            Rect { left: self.left - amount, top: self.top - amount,
                   right: self.right + amount, bottom: self.bottom + amount }
        }

        /// Returns None if left >= right or top >= bottom.
        pub fn to_tiny_skia(&self) -> Option<tiny_skia::Rect> {
            tiny_skia::Rect::from_ltrb(self.left, self.top, self.right, self.bottom)
        }
    }
    ```

- [ ] Add derives: `#[derive(Debug, Clone, Copy, PartialEq)]` (already have some, verify all present). Add `Display`:

    ```rust
    impl std::fmt::Display for Rect {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Rect({}, {}, {}, {})", self.left, self.top, self.right, self.bottom)
        }
    }
    ```

#### Complete the Color type

- [ ] Add `ColorAlpha` and core Color methods:

    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Color { pub r: u8, pub g: u8, pub b: u8 }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ColorAlpha { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

    impl Color {
        pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
        pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
        pub const RED: Self = Self { r: 255, g: 0, b: 0 };
        pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
        pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };

        pub const fn to_f32(self) -> (f32, f32, f32) {
            (self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0)
        }

        pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
            Self {
                r: (r.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                g: (g.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
                b: (b.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            }
        }

        pub fn to_tiny_skia(self) -> tiny_skia::Color {
            tiny_skia::Color::from_rgba8(self.r, self.g, self.b, 255)
        }

        pub fn with_alpha(self, a: u8) -> ColorAlpha {
            ColorAlpha { r: self.r, g: self.g, b: self.b, a }
        }
    }
    ```

- [ ] Add `from_css_hex` and `to_css_hex`:

    ```rust
    impl Color {
        pub fn from_css_hex(s: &str) -> Option<Self> {
            let hex = s.strip_prefix('#').unwrap_or(s);
            match hex.len() {
                6 => {
                    let val = u32::from_str_radix(hex, 16).ok()?;
                    Some(Self::from_hex(val))
                }
                3 => {
                    let mut chars = hex.chars();
                    let r = chars.next().and_then(|c| c.to_digit(16))? as u8;
                    let g = chars.next().and_then(|c| c.to_digit(16))? as u8;
                    let b = chars.next().and_then(|c| c.to_digit(16))? as u8;
                    Some(Self { r: r << 4 | r, g: g << 4 | g, b: b << 4 | b })
                }
                _ => None,
            }
        }

        pub fn to_css_hex(self) -> String {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        }
    }

    impl std::fmt::Display for Color {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        }
    }
    ```

- [ ] Add `luminance`, `contrast_ratio`, `lerp`:

    ```rust
    impl Color {
        pub fn luminance(self) -> f64 {
            fn linearize(c: f64) -> f64 {
                if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
            }
            let r = linearize(self.r as f64 / 255.0);
            let g = linearize(self.g as f64 / 255.0);
            let b = linearize(self.b as f64 / 255.0);
            0.2126 * r + 0.7152 * g + 0.0722 * b
        }

        pub fn contrast_ratio(self, other: Color) -> f64 {
            let l1 = self.luminance();
            let l2 = other.luminance();
            let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
            (lighter + 0.05) / (darker + 0.05)
        }

        pub fn lerp(self, other: Color, t: f32) -> Color {
            let t = t.clamp(0.0, 1.0);
            Color {
                r: (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
                g: (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
                b: (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
            }
        }
    }
    ```

- [ ] Add sister crate conversions:

    ```rust
    impl From<chromata::Color> for Color {
        fn from(c: chromata::Color) -> Self { Self { r: c.r, g: c.g, b: c.b } }
    }
    impl From<prismatica::Color> for Color {
        fn from(c: prismatica::Color) -> Self { Self { r: c.r, g: c.g, b: c.b } }
    }
    ```

- [ ] Write tests: `from_hex` roundtrip, `from_css_hex` with all formats, luminance black ≈ 0, luminance white ≈ 1, contrast black/white ≈ 21, lerp at 0.0 returns self, lerp at 1.0 returns other.

#### Add the Transform type

- [ ] Create a `Transform` newtype wrapping `tiny_skia::Transform`:

    ```rust
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Transform(pub(crate) tiny_skia::Transform);

    impl Transform {
        pub fn identity() -> Self { Self(tiny_skia::Transform::identity()) }
        pub fn translate(dx: f32, dy: f32) -> Self { Self(tiny_skia::Transform::from_translate(dx, dy)) }
        pub fn scale(sx: f32, sy: f32) -> Self { Self(tiny_skia::Transform::from_scale(sx, sy)) }
        /// NOTE: tiny-skia takes DEGREES, not radians.
        pub fn rotate_degrees(angle: f32) -> Self { Self(tiny_skia::Transform::from_rotate(angle)) }

        pub fn then(self, other: Transform) -> Self { Self(self.0.post_concat(other.0)) }
        pub fn pre_translate(self, dx: f32, dy: f32) -> Self { Self(self.0.pre_translate(dx, dy)) }

        pub(crate) fn as_tiny_skia(self) -> tiny_skia::Transform { self.0 }
    }
    ```

### Layer 1: Implement the tiny-skia backend

#### Create the SkiaBackend struct

- [ ] Create `starsight-layer-1/src/backend/skia/raster/mod.rs`:

    ```rust
    use tiny_skia::{Pixmap, Paint, FillRule, Stroke, LineCap, LineJoin, PathBuilder};
    use crate::error::{Result, StarsightError};
    use crate::primitives::{Color, Point, Rect, Transform};
    use super::super::DrawBackend;

    pub struct SkiaBackend {
        pixmap: Pixmap,
        font_system: cosmic_text::FontSystem,
        swash_cache: cosmic_text::SwashCache,
    }

    impl SkiaBackend {
        pub fn new(width: u32, height: u32) -> Result<Self> {
            let pixmap = Pixmap::new(width, height)
                .ok_or_else(|| StarsightError::Render(
                    format!("Failed to create {width}x{height} pixmap")
                ))?;
            Ok(Self {
                pixmap,
                font_system: cosmic_text::FontSystem::new(),
                swash_cache: cosmic_text::SwashCache::new(),
            })
        }

        pub fn fill(&mut self, color: Color) {
            self.pixmap.fill(color.to_tiny_skia());
        }

        pub fn png_bytes(&self) -> Result<Vec<u8>> {
            self.pixmap.encode_png().map_err(|e| StarsightError::Export(e.to_string()))
        }
    }
    ```

- [ ] Implement `DrawBackend` for `SkiaBackend`. The key methods:

    ```rust
    impl DrawBackend for SkiaBackend {
        fn dimensions(&self) -> (u32, u32) {
            (self.pixmap.width(), self.pixmap.height())
        }

        fn save_png(&self, path: &std::path::Path) -> Result<()> {
            self.pixmap.save_png(path)
                .map_err(|e| StarsightError::Export(e.to_string()))
        }

        fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
            let sk_rect = rect.to_tiny_skia()
                .ok_or_else(|| StarsightError::Render("Invalid rect".into()))?;
            let mut paint = Paint::default();
            paint.set_color_rgba8(color.r, color.g, color.b, 255);
            self.pixmap.fill_rect(sk_rect, &paint,
                tiny_skia::Transform::identity(), None);
            Ok(())
        }

        fn draw_path(&mut self, path: &crate::backend::Path,
                     style: &crate::backend::PathStyle) -> Result<()> {
            // Convert PathCommand sequence to tiny_skia::Path
            let mut pb = PathBuilder::new();
            for cmd in path.commands() {
                match cmd {
                    PathCommand::MoveTo(p) => pb.move_to(p.x, p.y),
                    PathCommand::LineTo(p) => pb.line_to(p.x, p.y),
                    PathCommand::QuadTo(c, p) => pb.quad_to(c.x, c.y, p.x, p.y),
                    PathCommand::CubicTo(c1, c2, p) =>
                        pb.cubic_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y),
                    PathCommand::Close => pb.close(),
                }
            }
            let sk_path = pb.finish()
                .ok_or_else(|| StarsightError::Render("Empty path".into()))?;

            let mut paint = Paint::default();
            paint.set_color_rgba8(style.stroke_color.r, style.stroke_color.g,
                                  style.stroke_color.b, 255);
            let stroke = Stroke {
                width: style.stroke_width,
                line_cap: style.line_cap,
                line_join: style.line_join,
                dash: style.dash_pattern.and_then(|(len, gap)|
                    tiny_skia::StrokeDash::new(vec![len, gap], 0.0)),
                ..Stroke::default()
            };
            self.pixmap.stroke_path(&sk_path, &paint, &stroke,
                tiny_skia::Transform::identity(), None);
            Ok(())
        }

        // draw_text and save_svg omitted for brevity — see Look up section
    }
    ```

- [ ] Uncomment the commented-out methods and `PathCommand` variants in `backend/mod.rs`:

    ```rust
    pub enum PathCommand {
        MoveTo(Point),
        LineTo(Point),
        QuadTo(Point, Point),
        CubicTo(Point, Point, Point),
        Close,
    }

    pub struct PathStyle {
        pub stroke_color: Color,
        pub stroke_width: f32,
        pub fill_color: Option<Color>,
        pub dash_pattern: Option<(f32, f32)>,
        pub line_cap: tiny_skia::LineCap,
        pub line_join: tiny_skia::LineJoin,
        pub opacity: f32,
    }
    ```

#### Set up snapshot testing

- [ ] Add to root `Cargo.toml`:

    ```toml
    [workspace.dependencies]
    insta = { version = "1.47.2", features = ["binary"] }
    ```

- [ ] Create `starsight-layer-1/tests/snapshot_basic.rs`:

    ```rust
    use starsight_layer_1::backend::skia::raster::SkiaBackend;
    use starsight_layer_1::primitives::{Color, Rect};

    #[test]
    fn blue_rect_on_white() {
        let mut backend = SkiaBackend::new(200, 100).unwrap();
        backend.fill(Color::WHITE);
        backend.fill_rect(Rect::from_xywh(10.0, 10.0, 180.0, 80.0), Color::BLUE).unwrap();
        let bytes = backend.png_bytes().unwrap();
        insta::assert_binary_snapshot!(".png", bytes);
    }
    ```

    Run `cargo insta test`, then `cargo insta review` to accept.

### Layer 1: Implement the SVG backend

- [ ] Create starsight-layer-1/src/backend/svg/mod.rs with an SvgBackend struct. It holds an svg::Document and the dimensions (width: u32, height: u32). The constructor takes dimensions and creates a Document with the viewBox attribute set.

- [ ] Implement DrawBackend for SvgBackend. fill_rect adds a Rectangle element with x, y, width, height, and fill attributes. draw_path converts PathCommands to SVG path data using svg::node::element::path::Data. draw_text adds a Text element with x, y, font-size, text-anchor, and dominant-baseline attributes.

- [ ] Implement save_svg: call svg::save(path, &self.document) and map errors.

- [ ] Implement save_png: this is not directly supported by the SVG backend. Return StarsightError::Export("SVG backend cannot save PNG directly; use the skia backend or resvg").

- [ ] Write a snapshot test that generates SVG output for a simple chart and asserts the SVG string content with assert_snapshot!.

### Layer 2: Linear scale and Wilkinson ticks

- [ ] Create `starsight-layer-2/src/scale.rs`:

    ```rust
    pub trait Scale {
        fn map(&self, value: f64) -> f64;
        fn inverse(&self, normalized: f64) -> f64;
    }

    pub struct LinearScale {
        pub domain_min: f64,
        pub domain_max: f64,
    }

    impl Scale for LinearScale {
        fn map(&self, value: f64) -> f64 {
            if (self.domain_max - self.domain_min).abs() < f64::EPSILON { return 0.5; }
            (value - self.domain_min) / (self.domain_max - self.domain_min)
        }
        fn inverse(&self, normalized: f64) -> f64 {
            normalized * (self.domain_max - self.domain_min) + self.domain_min
        }
    }
    ```

- [ ] Create `starsight-layer-2/src/tick.rs` with the Wilkinson Extended algorithm. See Part 1 "Listen" for the full explanation. See Part 3 "Look up" for the scoring formula.

    ```rust
    pub fn extended_ticks(dmin: f64, dmax: f64, target_count: usize) -> Vec<f64> {
        let q_list = &[1.0, 5.0, 2.0, 2.5, 4.0, 3.0];
        let w = [0.2, 0.25, 0.5, 0.05];
        let mut best_score = -2.0_f64;
        let mut best = (0.0, 0.0, 0.0_f64);
        // Nested loops: j (skip), q (step base), k (tick count), z (power of 10), start
        // At each level, compute upper bound and break if no candidate can beat best_score
        // Full pseudocode in Part 3
        todo!("implement the nested loop with pruning")
    }
    ```

    Tests: `extended_ticks(0.0, 100.0, 5)` returns round numbers, always sorted, always >= 2 elements.

- [ ] Create `starsight-layer-2/src/axis.rs`:

    ```rust
    pub struct Axis {
        pub scale: LinearScale,
        pub label: Option<String>,
        pub tick_positions: Vec<f64>,
        pub tick_labels: Vec<String>,
    }

    impl Axis {
        pub fn auto_from_data(values: &[f64], target_ticks: usize) -> Self {
            let dmin = values.iter().copied().fold(f64::INFINITY, f64::min);
            let dmax = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let ticks = crate::tick::extended_ticks(dmin, dmax, target_ticks);
            let labels: Vec<String> = ticks.iter().map(|t| format!("{t}")).collect();
            Self {
                scale: LinearScale { domain_min: ticks[0], domain_max: *ticks.last().unwrap() },
                label: None, tick_positions: ticks, tick_labels: labels,
            }
        }
    }
    ```

- [ ] Create `starsight-layer-2/src/coord.rs`:

    ```rust
    pub struct CartesianCoord {
        pub x_axis: Axis,
        pub y_axis: Axis,
        pub plot_area: Rect,
    }

    impl CartesianCoord {
        pub fn data_to_pixel(&self, x: f64, y: f64) -> Point {
            let nx = self.x_axis.scale.map(x);
            let ny = self.y_axis.scale.map(y);
            Point::new(
                self.plot_area.left + nx as f32 * self.plot_area.width(),
                self.plot_area.bottom - ny as f32 * self.plot_area.height(),
            )
        }
    }
    ```

### Layer 3: Line mark and point mark

- [ ] Create `starsight-layer-3/src/mark.rs`:

    ```rust
    use starsight_layer_1::backend::DrawBackend;
    use starsight_layer_1::error::Result;
    use starsight_layer_2::coord::CartesianCoord;

    pub trait Mark {
        fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()>;
    }
    ```

- [ ] Create `starsight-layer-3/src/line.rs`. Handle NaN by starting a new `MoveTo` (breaks the line at gaps):

    ```rust
    pub struct LineMark {
        pub x_data: Vec<f64>,
        pub y_data: Vec<f64>,
        pub color: Color,
        pub width: f32,
    }

    impl Mark for LineMark {
        fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
            let mut commands = Vec::new();
            let mut need_move = true;
            for (x, y) in self.x_data.iter().zip(&self.y_data) {
                if x.is_nan() || y.is_nan() { need_move = true; continue; }
                let p = coord.data_to_pixel(*x, *y);
                if need_move { commands.push(PathCommand::MoveTo(p)); need_move = false; }
                else { commands.push(PathCommand::LineTo(p)); }
            }
            let path = Path { commands };
            let style = PathStyle {
                stroke_color: self.color,
                stroke_width: self.width,
                fill_color: None,
                line_cap: tiny_skia::LineCap::Round,
                line_join: tiny_skia::LineJoin::Round,
                ..Default::default()
            };
            backend.draw_path(&path, &style)
        }
    }
    ```

- [ ] Create `starsight-layer-3/src/point.rs`. Batch all circles into one path for performance:

    ```rust
    pub struct PointMark {
        pub x_data: Vec<f64>,
        pub y_data: Vec<f64>,
        pub color: Color,
        pub radius: f32,
    }

    impl Mark for PointMark {
        fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
            // Batch: collect all pixel positions, draw as one filled path
            let mut commands = Vec::new();
            for (x, y) in self.x_data.iter().zip(&self.y_data) {
                if x.is_nan() || y.is_nan() { continue; }
                let p = coord.data_to_pixel(*x, *y);
                // Approximate circle with 4 cubic bezier arcs
                // Or: backend could have a draw_circles batch method
                commands.push(PathCommand::MoveTo(Point::new(p.x + self.radius, p.y)));
                // ... arc approximation commands
            }
            // Alternative: use backend-specific circle batching
            todo!("circle rendering")
        }
    }
    ```

### Layer 5: Figure builder and plot macro

- [ ] Create `starsight-layer-5/src/figure.rs`:

    ```rust
    use starsight_layer_3::mark::Mark;

    pub struct Figure {
        marks: Vec<Box<dyn Mark>>,
        pub x_label: Option<String>,
        pub y_label: Option<String>,
        pub title: Option<String>,
        pub width: u32,
        pub height: u32,
    }

    impl Figure {
        pub fn new() -> Self {
            Self { marks: Vec::new(), x_label: None, y_label: None,
                   title: None, width: 800, height: 600 }
        }
        pub fn title(&mut self, s: impl Into<String>) -> &mut Self { self.title = Some(s.into()); self }
        pub fn x_label(&mut self, s: impl Into<String>) -> &mut Self { self.x_label = Some(s.into()); self }
        pub fn y_label(&mut self, s: impl Into<String>) -> &mut Self { self.y_label = Some(s.into()); self }
        pub fn size(&mut self, w: u32, h: u32) -> &mut Self { self.width = w; self.height = h; self }
        pub fn add(&mut self, mark: impl Mark + 'static) -> &mut Self {
            self.marks.push(Box::new(mark)); self
        }

        pub fn save(&self, path: impl AsRef<std::path::Path>) -> starsight_layer_1::error::Result<()> {
            let mut backend = starsight_layer_1::backend::skia::raster::SkiaBackend::new(self.width, self.height)?;
            backend.fill(Color::WHITE);
            // Compute plot area, create CartesianCoord, render axes, render marks
            todo!("full render pipeline")
        }
    }
    ```

- [ ] Create the `plot!` macro in `starsight-layer-5/src/macros.rs`:

    ```rust
    #[macro_export]
    macro_rules! plot {
        ($x:expr, $y:expr $(,)?) => {{
            let mut fig = $crate::figure::Figure::new();
            fig.add($crate::line::LineMark {
                x_data: $x.into_iter().map(|v| v as f64).collect(),
                y_data: $y.into_iter().map(|v| v as f64).collect(),
                color: starsight_layer_1::primitives::Color::BLUE,
                width: 2.0,
            });
            fig
        }};
    }
    ```

- [ ] Wire the facade. In `starsight/src/lib.rs`:

    ```rust
    pub use starsight_layer_1 as layer1;
    pub use starsight_layer_2 as layer2;
    pub use starsight_layer_3 as layer3;
    pub use starsight_layer_4 as layer4;
    pub use starsight_layer_5 as layer5;
    pub use starsight_layer_6 as layer6;
    pub use starsight_layer_7 as layer7;
    pub mod prelude;
    ```

    In `starsight/src/prelude.rs`:

    ```rust
    pub use starsight_layer_1::primitives::{Color, Point, Vec2, Rect, Size};
    pub use starsight_layer_1::error::{StarsightError, Result};
    pub use starsight_layer_5::figure::Figure;
    pub use starsight_layer_5::plot;
    ```

- [ ] Write the integration test in `starsight/tests/integration.rs`:

    ```rust
    use starsight::prelude::*;

    #[test]
    fn quickstart_produces_png() {
        let fig = plot!([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        let tmp = std::env::temp_dir().join("starsight_test.png");
        fig.save(&tmp).unwrap();
        assert!(tmp.exists());
        assert!(std::fs::metadata(&tmp).unwrap().len() > 0);
        std::fs::remove_file(&tmp).ok();
    }
    ```


## 0.2.0 through 1.0.0 — Remaining milestones

These are abbreviated. Expand each when the previous milestone is complete.

### 0.2.0 — Core chart types part 1

- [ ] BarMark (vertical and horizontal bars, grouped and stacked)
- [ ] AreaMark (filled area between line and baseline)
- [ ] Histogram stat transform (bin data into counts)
- [ ] HeatmapMark (2D color grid from matrix data)
- [ ] Snapshot tests for all four

### 0.3.0 — Core chart types part 2

- [ ] BoxPlotMark (compute quartiles, whiskers, outliers)
- [ ] ViolinMark (KDE mirrored vertically)
- [ ] KDE stat transform (kernel density estimation)
- [ ] PieMark and DonutMark (arc geometry)
- [ ] ContourMark (isolines from 2D scalar field)
- [ ] CandlestickMark (OHLC financial chart)
- [ ] Polars DataFrame integration in layer 5 (accept &DataFrame, reference columns by name)
- [ ] Snapshot tests for all

### 0.4.0 — Layout and composition

- [ ] GridLayout in layer 4 (arrange multiple figures in rows/columns)
- [ ] FacetWrap (one subplot per category value, wrapping to multiple rows)
- [ ] FacetGrid (row and column faceting variables)
- [ ] Legend (map visual encodings back to data labels)
- [ ] Colorbar (continuous color scale display)
- [ ] PairPlot shorthand (scatter matrix)
- [ ] JointPlot shorthand (scatter with marginal distributions)

### 0.5.0 — Scale infrastructure

- [ ] LogScale, SymlogScale (symmetric log for data spanning zero)
- [ ] DateTimeScale (auto tick granularity: year/month/day/hour/minute/second)
- [ ] BandScale, CategoricalScale (discrete axis positions)
- [ ] ColorScale backed by prismatica (Sequential, Diverging, Qualitative)
- [ ] TickLocator and TickFormatter traits for custom tick logic

### 0.6.0 — GPU and interactivity

- [ ] wgpu DrawBackend in starsight-layer-1/src/backend/wgpu/
- [ ] Native window via winit in layer 6
- [ ] Hover tooltips, box zoom, wheel zoom, pan
- [ ] Legend click-to-toggle visibility
- [ ] Streaming data append with rolling window

### 0.7.0 — 3D visualization

- [ ] Scatter3D, Surface3D, Wireframe3D, Line3D
- [ ] Camera orbit/pan with nalgebra transforms
- [ ] Isosurface, VolumeRender

### 0.8.0 — Terminal backend

- [ ] Kitty graphics protocol output
- [ ] Sixel output
- [ ] iTerm2 inline images
- [ ] Half-block and Braille character rendering
- [ ] StarsightWidget implementing ratatui::Widget
- [ ] Automatic protocol detection

### 0.9.0 — All chart types

- [ ] Complete the remaining 40+ mark types from the taxonomy
- [ ] Snapshot test for every one

### 0.10.0 — Export and WASM

- [ ] PDF export via krilla
- [ ] Self-contained interactive HTML export
- [ ] GIF animation export
- [ ] WASM + WebGPU browser target

### 0.11.0 — Polish

- [ ] Recipe proc macro (#[starsight::recipe])
- [ ] ndarray and Arrow RecordBatch data acceptance
- [ ] API audit against Rust API Guidelines checklist

### 0.12.0 — Documentation

- [ ] Rustdoc for every public item
- [ ] 12 example programs
- [ ] Gallery generation via xtask
- [ ] docs.rs configuration

### 1.0.0 — Stable release

- [ ] cargo-semver-checks pass
- [ ] Full CI green on all platforms
- [ ] Announcement


---
---

# Part 3 — Look up

Quick-reference for type signatures, API details, conversion formulas, and dependency specifics. Come here mid-implementation when you need to check something.

---

## tiny-skia 0.12 API reference

### Color types

| Type | Fields | Alpha | Constructor | Returns |
|------|--------|-------|-------------|---------|
| `Color` | f32 × 4 | Straight | `from_rgba(r,g,b,a)` | `Option<Self>` (None if out of 0.0-1.0) |
| `Color` | f32 × 4 | Straight | `from_rgba8(r,g,b,a)` | `Self` (infallible) |
| `ColorU8` | u8 × 4 | Straight | `from_rgba(r,g,b,a)` | `Self` (const, infallible) |
| `PremultipliedColorU8` | u8 × 4 | Premultiplied | `from_rgba(r,g,b,a)` | `Option<Self>` (None if channel > alpha) |

### Drawing methods (all take `Option<&Mask>` as final param)

```rust
pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
pixmap.fill_rect(rect, &paint, Transform::identity(), None);
pixmap.draw_pixmap(x: i32, y: i32, pixmap_ref, &pixmap_paint, Transform::identity(), None);
```

### PathBuilder

```rust
let mut pb = PathBuilder::new();
pb.move_to(x, y);
pb.line_to(x, y);
pb.quad_to(x1, y1, x, y);
pb.cubic_to(x1, y1, x2, y2, x, y);
pb.close();
pb.push_circle(cx, cy, r);          // add circle to existing builder
let path: Option<Path> = pb.finish(); // None if empty
```

Static constructors: `PathBuilder::from_rect(rect) -> Path`, `PathBuilder::from_circle(cx, cy, r) -> Option<Path>`.

### Stroke

```rust
Stroke {
    width: 2.0,
    miter_limit: 4.0,
    line_cap: LineCap::Round,    // Butt | Round | Square
    line_join: LineJoin::Round,  // Miter | MiterClip | Round | Bevel
    dash: StrokeDash::new(vec![10.0, 5.0], 0.0), // returns Option
}
```

### Transform — DEGREES not radians

```rust
Transform::identity()
Transform::from_translate(tx, ty)
Transform::from_scale(sx, sy)
Transform::from_rotate(degrees)              // NOT radians
Transform::from_rotate_at(degrees, cx, cy)
t.pre_translate(tx, ty)
t.post_concat(other)
```

### PNG export

```rust
pixmap.save_png("file.png")?;                          // to file
let bytes: Vec<u8> = pixmap.encode_png()?;             // to memory
// DPI: 300 DPI = 11811 pixels/meter
```

---

## cosmic-text 0.18 API reference

```rust
let mut font_system = FontSystem::new();           // loads system fonts (~1s)
let mut swash_cache = SwashCache::new();           // no params
let metrics = Metrics::new(14.0, 20.0);            // font_size, line_height (f32 px)
let mut buffer = Buffer::new(&mut font_system, metrics);
buffer.set_text(&mut font_system, "text", &Attrs::new(), Shaping::Advanced, None);
buffer.set_size(&mut font_system, Some(width), Some(height));
buffer.shape_until_scroll(&mut font_system, true);
```

### Measure text dimensions

```rust
let (mut w, mut h) = (0.0f32, 0.0f32);
for run in buffer.layout_runs() {
    w = w.max(run.line_w);
    h = run.line_top + run.line_height;
}
```

### Draw onto tiny-skia (NO channel swap for file output)

```rust
buffer.draw(&mut swash_cache, text_color, |x, y, w, h, color| {
    paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
    if let Some(rect) = Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
});
```

### Embed custom font

```rust
font_system.db_mut().load_font_data(include_bytes!("fonts/Inter.ttf").to_vec());
```

---

## prismatica API reference

```rust
// Continuous colormap — sample at t in [0,1]
let color: Color = prismatica::crameri::BATLOW.eval(0.5);
let color: Color = prismatica::crameri::BATLOW.eval_rational(5, 10);

// Reversed (zero allocation)
let rev = prismatica::crameri::BATLOW.reversed();

// Discrete palette — categorical data
let color: Color = prismatica::colorbrewer::SET2_PALETTE.get(0); // wraps around

// Metadata
prismatica::crameri::BATLOW.name()       // "batlow"
prismatica::crameri::BATLOW.kind()       // ColormapKind::Sequential
prismatica::crameri::BATLOW.meta.perceptually_uniform  // true
prismatica::crameri::BATLOW.meta.cvd_friendly          // true

// Runtime lookup
let cm = prismatica::find_by_name("batlow");
let diverging = prismatica::filter_by_kind(ColormapKind::Diverging);
```

### Colormap selection guide

| Data type | Use | Examples |
|-----------|-----|---------|
| Sequential (temperature, elevation) | Sequential | `BATLOW`, `VIRIDIS`, `OSLO` |
| Diverging (anomalies, residuals) | Diverging | `BERLIN`, `VIK`, `SMOOTH_COOL_WARM` |
| Cyclic (phase, direction) | Cyclic | `ROMA_O`, `PHASE` |
| Categorical (labels, classes) | Discrete palette | `SET2_PALETTE`, `TABLEAU10` |

---

## chromata API reference

```rust
// Access theme
let theme: &Theme = &chromata::popular::gruvbox::DARK_HARD;
theme.bg           // Color { r, g, b } — always present
theme.fg           // Color — always present
theme.keyword      // Option<Color>
theme.accent()     // Color — first available (blue > purple > cyan > green > orange > red > fg)
theme.is_dark()    // bool
theme.colors()     // Vec<(&str, Color)> — all defined fields

// Query
chromata::find_by_name("Catppuccin Mocha")       // Option<&'static Theme>
chromata::filter_by_variant(Variant::Dark)        // Vec<&'static Theme>
chromata::collect_all_themes()                     // Vec<&'static Theme>
```

### Theme fields

Always: `name`, `author`, `variant`, `contrast`, `bg`, `fg`.
Optional UI: `cursor`, `selection`, `line_highlight`, `gutter`, `statusbar_bg`, `statusbar_fg`.
Optional syntax: `comment`, `keyword`, `string`, `function`, `variable`, `r#type`, `constant`, `operator`, `tag`.
Optional diagnostics: `error`, `warning`, `info`, `success`.
Optional accents: `red`, `orange`, `yellow`, `green`, `cyan`, `blue`, `purple`, `magenta`.

---

## Wilkinson Extended tick algorithm

```
Score = 0.2 * simplicity + 0.25 * coverage + 0.5 * density + 0.05 * legibility
Q = [1, 5, 2, 2.5, 4, 3]
Step = j * q * 10^z

simplicity = 1 - (i-1)/(|Q|-1) - j + v   (v=1 if zero included)
coverage = 1 - 0.5 * ((dmax-lmax)^2 + (dmin-lmin)^2) / (0.1*(dmax-dmin))^2
density = 2 - max(rho/rho_t, rho_t/rho)
```

Paper: https://vis.stanford.edu/files/2010-TickLabels-InfoVis.pdf
R reference: https://rdrr.io/cran/labeling/src/R/labeling.R

---

## SVG text positioning

```xml
<!-- Centered text -->
<text x="100" y="50" text-anchor="middle" dominant-baseline="central">Label</text>

<!-- Rotated Y-axis label -->
<text transform="translate(15, 200) rotate(-90)" text-anchor="middle" dominant-baseline="central">Y Label</text>
```

Text width estimation: digits ≈ 0.55 × font_size, average char ≈ 0.6 × font_size.

---

## Data-to-pixel conversion

```rust
fn to_px_x(val: f64, dmin: f64, dmax: f64, px_left: f64, px_right: f64) -> f64 {
    (val - dmin) / (dmax - dmin) * (px_right - px_left) + px_left
}
fn to_px_y(val: f64, dmin: f64, dmax: f64, px_top: f64, px_bottom: f64) -> f64 {
    px_bottom - (val - dmin) / (dmax - dmin) * (px_bottom - px_top)  // Y inverted
}
```

---

## Chart layout margins

```
left_margin   = pad + y_label_height + label_pad + max_ytick_width + tick_pad
bottom_margin = pad + x_label_height + label_pad + xtick_height + tick_pad
plot_width    = figure_width  - left_margin - right_margin
plot_height   = figure_height - top_margin  - bottom_margin
max_ytick_width = max(len(format(tick))) * font_size * 0.6
```

---

## Error handling pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum StarsightError {
    #[error("Rendering: {0}")]  Render(String),
    #[error("Data: {0}")]       Data(String),
    #[error("I/O: {0}")]        Io(#[from] std::io::Error),
    #[error("Scale: {0}")]      Scale(String),
    #[error("Export: {0}")]     Export(String),
    #[error("Config: {0}")]     Config(String),
    #[error("Unknown: {0}")]    Unknown(String),
}
pub type Result<T> = std::result::Result<T, StarsightError>;
```

---

## plot! macro pattern

```rust
#[macro_export]
macro_rules! plot {
    ($df:expr, x = $x:expr, y = $y:expr $(, $key:ident = $val:expr)* $(,)?) => {{
        let mut cfg = $crate::DataFramePlotConfig::new($df, $x, $y);
        $( cfg = cfg.$key($val); )*
        cfg.build()
    }};
    ($x:expr, $y:expr $(,)?) => { $crate::PlotBuilder::from_arrays($x, $y).build() };
    ($data:expr $(,)?) => { $crate::PlotBuilder::from_single($data).build() };
}
```

---

## Links

| Crate | docs.rs | GitHub |
|-------|---------|--------|
| tiny-skia | https://docs.rs/tiny-skia | https://github.com/linebender/tiny-skia |
| cosmic-text | https://docs.rs/cosmic-text | https://github.com/pop-os/cosmic-text |
| svg | https://docs.rs/svg | https://github.com/bodoni/svg |
| palette | https://docs.rs/palette | https://github.com/Ogeon/palette |
| image | https://docs.rs/image | https://github.com/image-rs/image |
| thiserror | https://docs.rs/thiserror | https://github.com/dtolnay/thiserror |
| insta | https://docs.rs/insta | https://github.com/mitsuhiko/insta |
| prismatica | https://docs.rs/prismatica | https://github.com/resonant-jovian/prismatica |
| chromata | https://docs.rs/chromata | https://github.com/resonant-jovian/chromata |
| wgpu | https://docs.rs/wgpu | https://github.com/gfx-rs/wgpu |
| ratatui | https://docs.rs/ratatui | https://github.com/ratatui/ratatui |
| polars | https://docs.rs/polars | https://github.com/pola-rs/polars |
| krilla | https://docs.rs/krilla | https://github.com/LaurenzV/krilla |
| winit | https://docs.rs/winit | https://github.com/rust-windowing/winit |
| egui | https://docs.rs/egui | https://github.com/emilk/egui |

### Theory and standards

- Wilkinson Extended ticks: https://vis.stanford.edu/files/2010-TickLabels-InfoVis.pdf
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/checklist.html
- Cargo workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Edition 2024: https://doc.rust-lang.org/edition-guide/rust-2024/index.html
- Kitty protocol: https://sw.kovidgoyal.net/kitty/graphics-protocol/
- Sixel: https://vt100.net/docs/vt3xx-gp/chapter14.html
- Crameri colormaps: https://www.fabiocrameri.ch/colourmaps/


---
---

# Part 4 — Navigate

Tree structures and maps. Come here when you need to know which file to create, which crate a type belongs in, or how the pieces connect.

---

## Crate dependency graph

```
starsight (facade — re-exports everything, the only crate users depend on)
├── starsight-layer-1  (rendering, primitives, error, backends)
├── starsight-layer-2  (scales, axes, coordinates)
│   └── starsight-layer-1
├── starsight-layer-3  (marks, stats, aesthetics)
│   ├── starsight-layer-1
│   └── starsight-layer-2
├── starsight-layer-4  (layout, faceting, legends)
│   ├── starsight-layer-1
│   ├── starsight-layer-2
│   └── starsight-layer-3
├── starsight-layer-5  (Figure, plot!(), data acceptance)
│   ├── starsight-layer-1
│   ├── starsight-layer-2
│   ├── starsight-layer-3
│   └── starsight-layer-4
├── starsight-layer-6  (interactivity, streaming)
│   ├── starsight-layer-1 through 5
├── starsight-layer-7  (animation, export)
│   ├── starsight-layer-1 through 6
└── xtask              (build automation, not published)
```

The rule: each layer can depend on any layer below it, never on a layer above. This is enforced by Cargo.toml, not convention.

---

## File tree — current state and target

Exists means the file is in the repo right now. Target means it needs to be created for 0.1.0.

```
starsight/
├── Cargo.toml                          [exists]  workspace root
├── .spec/STARSIGHT.md                  [exists]  this document
├── LICENSE                             [exists]
├── README.md                           [exists]
├── CONTRIBUTING.md                     [exists]
├── CHANGELOG.md                        [exists]
├── CODE_OF_CONDUCT.md                  [exists]
├── SECURITY.md                         [exists]
├── .clippy.toml                        [exists]
├── .rustfmt.toml                       [exists]
├── deny.toml                           [exists]
│
├── .github/
│   ├── FUNDING.yml                     [exists]
│   ├── PULL_REQUEST_TEMPLATE.md        [exists]
│   ├── ISSUE_TEMPLATE/                 [exists]  bug_report.md, feature_request.md, config.yml
│   └── workflows/
│       ├── ci.yml                      [exists]  fmt, clippy, check, test matrix, deny
│       ├── release.yml                 [exists]  publish, github-release with git-cliff
│       ├── coverage.yml                [exists]  cargo-llvm-cov, codecov
│       ├── snapshots.yml               [exists]  cargo insta test, artifact upload
│       └── gallery.yml                 [exists]  xtask gallery, artifact upload
│
├── starsight/                          FACADE CRATE
│   ├── Cargo.toml                      [exists]  depends on all layers
│   └── src/
│       ├── lib.rs                      [exists]  re-exports
│       └── prelude.rs                  [exists]  pub use of primary types
│
├── starsight-layer-1/                  RENDERING + PRIMITIVES + ERROR
│   ├── Cargo.toml                      [exists]  deps: tiny-skia, thiserror
│   └── src/
│       ├── lib.rs                      [exists]  pub mod backend, error, primitives
│       ├── error.rs                    [exists]  StarsightError enum, Result type
│       ├── primitives.rs               [exists]  Color, Point, Rect, Size + From impls
│       │                                [target] add Vec2, Transform, ColorAlpha
│       │                                [target] add all arithmetic, accessors, conversions
│       ├── scene.rs                    [target]  SceneNode enum, Scene struct
│       └── backend/
│           ├── mod.rs                  [exists]  DrawBackend trait (partial)
│           │                            [target] uncomment all methods
│           ├── skia/
│           │   ├── mod.rs              [exists]  sub-module declarations
│           │   ├── raster/mod.rs       [exists]  empty — [target] SkiaBackend struct + DrawBackend impl
│           │   ├── headless/mod.rs     [exists]  empty — headless rendering (later)
│           │   └── png/mod.rs          [exists]  empty — PNG-specific logic (later)
│           ├── svg/
│           │   └── mod.rs              [exists]  empty — [target] SvgBackend struct + DrawBackend impl
│           ├── pdf/
│           │   └── mod.rs              [exists]  empty — PDF backend (0.10.0)
│           ├── wgpu/
│           │   ├── mod.rs              [exists]  sub-module declarations
│           │   ├── native/mod.rs       [exists]  empty — native GPU window (0.6.0)
│           │   └── web/mod.rs          [exists]  empty — WASM WebGPU (0.10.0)
│           └── terminal/
│               ├── mod.rs              [exists]  sub-module declarations
│               ├── kitty/mod.rs        [exists]  empty — Kitty protocol (0.8.0)
│               ├── sixel/mod.rs        [exists]  empty — Sixel protocol (0.8.0)
│               ├── iterm2/mod.rs       [exists]  empty — iTerm2 protocol (0.8.0)
│               ├── half_block/mod.rs   [exists]  empty — half-block chars (0.8.0)
│               └── braille/mod.rs      [exists]  empty — Braille dots (0.8.0)
│
├── starsight-layer-2/                  SCALES + AXES + COORDINATES
│   ├── Cargo.toml                      [exists]  deps: starsight-layer-1
│   └── src/
│       ├── lib.rs                      [exists]  empty — [target] pub mod scale, tick, axis, coord
│       ├── scale.rs                    [target]  Scale trait, LinearScale, LogScale, etc.
│       ├── tick.rs                     [target]  Wilkinson Extended algorithm
│       ├── axis.rs                     [target]  Axis struct (scale + ticks + labels)
│       └── coord.rs                    [target]  CartesianCoord (data-to-pixel mapping)
│
├── starsight-layer-3/                  MARKS + STATS + AESTHETICS
│   ├── Cargo.toml                      [exists]  deps: layer-1, layer-2
│   └── src/
│       ├── lib.rs                      [exists]  empty — [target] pub mod mark, line, point, bar, ...
│       ├── mark.rs                     [target]  Mark trait
│       ├── line.rs                     [target]  LineMark
│       ├── point.rs                    [target]  PointMark
│       ├── bar.rs                      [target]  BarMark (0.2.0)
│       ├── area.rs                     [target]  AreaMark (0.2.0)
│       ├── aes.rs                      [target]  Aesthetic mapping types
│       ├── position.rs                 [target]  Dodge, Stack, Jitter adjustments
│       └── stat/
│           ├── mod.rs                  [target]  stat module
│           ├── bin.rs                  [target]  Histogram binning (0.2.0)
│           └── kde.rs                  [target]  Kernel density estimation (0.3.0)
│
├── starsight-layer-4/                  LAYOUT + FACETING + LEGENDS
│   ├── Cargo.toml                      [exists]  deps: layer-1, layer-2, layer-3
│   └── src/
│       ├── lib.rs                      [exists]  empty — [target] pub mod grid, facet, legend, colorbar
│       ├── grid.rs                     [target]  GridLayout (0.4.0)
│       ├── facet.rs                    [target]  FacetWrap, FacetGrid (0.4.0)
│       ├── legend.rs                   [target]  Legend (0.4.0)
│       └── colorbar.rs                [target]  Colorbar (0.4.0)
│
├── starsight-layer-5/                  HIGH-LEVEL API
│   ├── Cargo.toml                      [exists]  deps: layer-1 through layer-4
│   └── src/
│       ├── lib.rs                      [exists]  empty — [target] pub mod figure, macro, data
│       ├── figure.rs                   [target]  Figure struct + builder
│       ├── macro.rs                    [target]  plot!() macro
│       ├── auto.rs                     [target]  chart type auto-inference
│       └── data/
│           ├── mod.rs                  [target]  DataSource trait
│           ├── raw.rs                  [target]  Vec/slice acceptance
│           ├── polars.rs               [target]  DataFrame acceptance (0.3.0)
│           ├── ndarray.rs              [target]  ndarray acceptance (0.11.0)
│           └── arrow.rs               [target]  Arrow acceptance (0.11.0)
│
├── starsight-layer-6/                  INTERACTIVITY
│   ├── Cargo.toml                      [exists]  deps: layer-1 through layer-5
│   └── src/
│       ├── lib.rs                      [exists]  empty — all 0.6.0+
│       ├── hover.rs                    [target]  tooltips (0.6.0)
│       ├── zoom.rs                     [target]  box/wheel zoom (0.6.0)
│       ├── pan.rs                      [target]  drag pan (0.6.0)
│       ├── select.rs                   [target]  box/lasso selection (0.6.0)
│       └── stream.rs                   [target]  streaming data (0.6.0)
│
├── starsight-layer-7/                  ANIMATION + EXPORT
│   ├── Cargo.toml                      [exists]  deps: layer-1 through layer-6
│   └── src/
│       ├── lib.rs                      [exists]  empty — all 0.7.0+
│       ├── animation.rs               [target]  frame recording (0.10.0)
│       ├── pdf.rs                      [target]  PDF export (0.10.0)
│       ├── html.rs                     [target]  interactive HTML (0.10.0)
│       └── terminal.rs                [target]  terminal inline output (0.8.0)
│
├── examples/
│   ├── quickstart.rs                   [exists]  empty — [target] plot!(x,y).save(...)
│   ├── scatter.rs                      [exists]  empty
│   ├── statistical.rs                  [exists]  empty
│   ├── surface3d.rs                    [exists]  empty
│   ├── terminal.rs                     [exists]  empty
│   ├── interactive.rs                  [exists]  empty
│   ├── polars_integration.rs           [exists]  empty
│   ├── streaming.rs                    [exists]  empty
│   ├── faceting.rs                     [exists]  empty
│   ├── custom_theme.rs                 [exists]  empty
│   ├── recipe.rs                       [exists]  empty
│   └── gallery.rs                      [exists]  empty
│
└── xtask/
    ├── Cargo.toml                      [exists]
    └── src/main.rs                     [exists]  empty main
```

---

## What belongs where — type ownership

| Type | Lives in | Why |
|------|----------|-----|
| `Point`, `Vec2`, `Rect`, `Size` | `starsight-layer-1::primitives` | Geometry primitives are the foundation everything else builds on |
| `Color`, `ColorAlpha` | `starsight-layer-1::primitives` | Every layer needs colors; layer 1 owns conversion to backend types |
| `Transform` | `starsight-layer-1::primitives` | Wraps tiny_skia::Transform, needed by Scene and backends |
| `StarsightError`, `Result` | `starsight-layer-1::error` | Error types must be in the lowest layer so all layers can return them |
| `DrawBackend` trait | `starsight-layer-1::backend` | Trait that backends implement |
| `SkiaBackend` | `starsight-layer-1::backend::skia::raster` | CPU rendering via tiny-skia |
| `SvgBackend` | `starsight-layer-1::backend::svg` | SVG document generation |
| `Scene`, `SceneNode` | `starsight-layer-1::scene` | Scene is data that backends consume |
| `PathStyle`, `PathCommand` | `starsight-layer-1::backend` | Drawing primitives consumed by DrawBackend |
| `Scale` trait, `LinearScale` | `starsight-layer-2::scale` | Maps data values to normalized positions |
| `extended_ticks()` | `starsight-layer-2::tick` | Tick generation algorithm |
| `Axis` | `starsight-layer-2::axis` | Scale + ticks + labels bundled together |
| `CartesianCoord` | `starsight-layer-2::coord` | Data-to-pixel coordinate mapping |
| `Mark` trait | `starsight-layer-3::mark` | Interface all visual marks implement |
| `LineMark`, `PointMark`, etc. | `starsight-layer-3::line`, etc. | Concrete mark implementations |
| `GridLayout`, `FacetWrap` | `starsight-layer-4` | Multi-chart arrangement |
| `Figure` | `starsight-layer-5::figure` | The main builder users interact with |
| `plot!()` macro | `starsight-layer-5::macro` | Zero-config entry point |
| Interactivity types | `starsight-layer-6` | Hover, zoom, pan, selection |
| Export/animation types | `starsight-layer-7` | PNG/SVG/PDF/HTML/GIF/terminal output |

---

## Hard rules

1. No JavaScript runtime dependencies
2. No C/C++ system library dependencies in default feature set
3. No unsafe in layers 3-7
4. No runtime file I/O for core functionality (colormaps, themes, fonts are compile-time)
5. No println or eprintln in library code (use log crate)
6. No panics except in .show() when no display backend is available
7. No nightly-only features required
8. No async in the public API

---

## MSRV

1.85 (edition 2024). Tracks latest stable minus two.

---

## License

GPL-3.0-only.

