# starsight — development reference

For background knowledge and theory, see LEARN.md.

> Authored by Claude Opus 4.6 (Anthropic) with Albin Sjoegren. Last generated: 2026-04-03.

**LEARN.md** — Explanation. Pure prose, TTS-compatible. Teaches every concept in plain sentences.

**Part 1 — Build** — Ordered task list with code. Follow top to bottom.

**Part 2 — Look up** — API signatures, imports, formulas, quick-reference tables.

**Part 3 — Navigate** — Architecture maps, file trees, crate routing.

---

# Part 1 — Build

Every task below has enough detail to implement it without looking anything else up. Items are ordered by dependency. Do not skip ahead. When a task says to create a struct, it tells you the fields, the derives, the trait implementations, and why.

Checked items reflect the current state of the codebase as of 2026-04-03.

---

## Quick start — first chart in 3 lines

```rust
use starsight::prelude::*;

fn main() -> starsight::Result<()> {
    plot!(&[1.0, 2.0, 3.0, 4.0], &[10.0, 20.0, 15.0, 25.0]).save("chart.png")
}
```

### What happens under the hood

```
plot!(x, y)
 │
 ├── Figure::new(800, 600)                              Layer 5
 │
 ├── .add(LineMark::new(x, y))                          Layer 3
 │    ├── LinearScale::fit(&x)                          Layer 2
 │    ├── LinearScale::fit(&y)                          Layer 2
 │    ├── wilkinson_ticks(x_min, x_max, ~5)             Layer 2
 │    └── CartesianCoord { x_axis, y_axis }             Layer 2
 │
 ├── .render(&mut SkiaBackend::new(800, 600)?)          Layer 1
 │    ├── backend.fill(Color::WHITE)
 │    ├── coord.render_axes(&mut backend)
 │    └── line_mark.render(&coord, &mut backend)
 │         ├── coord.data_to_pixel(x, y) per point
 │         └── backend.draw_path(&path, &style)
 │
 └── .save("chart.png")
      └── pixmap.save_png("chart.png")
```

### Coming from another language?

| You used | starsight | Key difference |
|---|---|---|
| `plt.plot(x, y)` | `plot!(x, y)` | No global state |
| `plt.scatter(x, y, c=c)` | `PointMark::new(x,y).color_by(&c)` | Builder pattern |
| `plt.bar(labels, vals)` | `BarMark::new(labels, vals)` | Grammar of graphics |
| `plt.savefig("out.png")` | `.save("out.png")?` | Returns `Result` |
| `plt.show()` | `.show()?` | Feature `interactive` |
| `sns.heatmap(data)` | `HeatmapMark::new(data)` | prismatica colormaps |
| `ggplot + geom_point()` | `Figure::new().add(PointMark)` | Builder, not `+` |
| `px.scatter(df, x="a")` | `plot!(df, x="a", y="b")` | Feature `polars` |

### Milestone progress

| Milestone | Done | Todo | Total |
|---|---|---|---|
| Pre-0.1.0 | 28 | 0 | 28 |
| **0.1.0** | **30** | **51** | **81** |
| 0.2.0 | 0 | 32 | 32 |
| 0.3.0 | 0 | 40 | 40 |
| 0.4.0+ | 0 | 117 | 117 |
| **Total** | **58** | **280** | **338** |

---

## Pre-0.1.0 — Workspace bootstrap

These are done. Listed for audit completeness.

- [x] Create resonant-jovian/starsight GitHub repository

    ```bash
    gh repo create resonant-jovian/starsight --public --license gpl-3.0
    ```
- [x] Add GPL-3.0-only LICENSE

    ```
    SPDX: GPL-3.0-only
    ```
- [x] Create CONTRIBUTING.md, CODE_OF_CONDUCT.md, CHANGELOG.md, SECURITY.md

    ```bash
    touch CONTRIBUTING.md CODE_OF_CONDUCT.md CHANGELOG.md SECURITY.md
    ```
- [x] Create .github/ISSUE_TEMPLATE/ (bug_report.md, feature_request.md, config.yml)

    ```bash
    mkdir -p .github/ISSUE_TEMPLATE
    touch .github/ISSUE_TEMPLATE/{bug_report,feature_request}.md .github/ISSUE_TEMPLATE/config.yml
    ```
- [x] Create .github/PULL_REQUEST_TEMPLATE.md

    ```bash
    touch .github/PULL_REQUEST_TEMPLATE.md
    ```
- [x] Create .github/FUNDING.yml

    ```yaml
    # .github/FUNDING.yml
    github: [resonant-jovian]
    thanks_dev: u/gh/resonant-jovian
    ```
- [x] Initialize workspace Cargo.toml with resolver 3, edition 2024, all workspace members

    ```toml
    [workspace]
    members = ["starsight", "starsight-layer-1", "starsight-layer-2", "starsight-layer-3",
           "starsight-layer-4", "starsight-layer-5", "starsight-layer-6", "starsight-layer-7", "xtask"]
    resolver = "3"
    
    [workspace.package]
    edition = "2024"
    license = "GPL-3.0-only"
    ```
- [x] Create all 8 crate Cargo.toml files (starsight, layer-1 through layer-7) with workspace inheritance

    ```toml
    # Each crate's Cargo.toml:
    [package]
    name = "starsight-layer-1"
    version.workspace = true
    edition.workspace = true
    license.workspace = true
    ```
- [x] Create xtask/Cargo.toml

    ```toml
    [package]
    name = "xtask"
    version = "0.0.0"
    edition.workspace = true
    publish = false
    ```
- [x] Define all feature flags in starsight/Cargo.toml

    ```toml
    [features]
    default = []
    gpu = []
    terminal = []
    polars = []
    3d = []
    pdf = []
    stats = []
    arrow = []
    ndarray = []
    interactive = []
    web = []
    ```
- [x] Configure workspace lints: unsafe_code forbid, clippy pedantic warn

    ```toml
    [workspace.lints.rust]
    unsafe_code = "forbid"
    
    [workspace.lints.clippy]
    pedantic = { level = "warn", priority = -1 }
    ```
- [x] Create .rustfmt.toml and .clippy.toml with full config

    ```toml
    # .rustfmt.toml (key settings)
    max_width = 100
    tab_spaces = 4
    edition = "2024"
    ```
- [x] Create deny.toml for cargo-deny

    ```toml
    # deny.toml
    [licenses]
    allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Zlib", "GPL-3.0"]
    
    [advisories]
    vulnerability = "deny"
    ```
- [x] Configure profile.release (LTO, codegen-units 1) and profile.dev (opt-level 1)

    ```toml
    [profile.release]
    lto = true
    codegen-units = 1
    
    [profile.dev]
    opt-level = 1
    ```
- [x] Create .github/workflows/ci.yml (fmt, clippy, check, test matrix, deny)

    ```yaml
    # .github/workflows/ci.yml
    name: CI
    on: [push, pull_request]
    jobs:
    fmt: { runs-on: ubuntu-latest, steps: [{ uses: actions/checkout@v4 }, { run: cargo fmt --all -- --check }] }
    clippy: { runs-on: ubuntu-latest, steps: [{ run: cargo clippy --workspace --all-targets -- -D warnings }] }
    test: { runs-on: ubuntu-latest, steps: [{ run: cargo test --workspace }] }
    deny: { runs-on: ubuntu-latest, steps: [{ run: cargo deny check }] }
    ```
- [x] Create .github/workflows/release.yml (publish, GitHub release with git-cliff)

    ```yaml
    name: Release
    on: { push: { tags: ["v*"] } }
    jobs:
    publish: { runs-on: ubuntu-latest, steps: [{ run: cargo publish -p starsight-layer-1 }, { run: cargo publish -p starsight }] }
    ```
- [x] Create .github/workflows/coverage.yml (cargo-llvm-cov, Codecov upload)

    ```yaml
    name: Coverage
    on: [push]
    jobs:
    coverage: { runs-on: ubuntu-latest, steps: [{ run: cargo llvm-cov --workspace --lcov --output-path lcov.info }] }
    ```
- [x] Create .github/workflows/snapshots.yml (cargo insta test, artifact upload on failure)

    ```yaml
    name: Snapshots
    on: [pull_request]
    jobs:
    snapshots: { runs-on: ubuntu-latest, steps: [{ run: cargo insta test --workspace }] }
    ```
- [x] Create .github/workflows/gallery.yml (xtask gallery, artifact upload)

    ```yaml
    name: Gallery
    on: [push]
    jobs:
    gallery: { runs-on: ubuntu-latest, steps: [{ run: cargo xtask gallery }] }
    ```
- [x] Create README.md with badges, feature table, roadmap

    ```markdown
    # starsight
    [![crates.io](https://img.shields.io/crates/v/starsight?style=for-the-badge)](https://crates.io/crates/starsight)
    
    A unified scientific visualization crate for Rust.
    ```
- [x] Create starsight-layer-1/src/error.rs with StarsightError enum (7 variants) and Result type alias

    ```rust
    #[derive(thiserror::Error, Debug)]
    #[non_exhaustive]
    pub enum StarsightError {
    #[error("Rendering: {0}")] Render(String),
    #[error("Data: {0}")] Data(String),
    #[error("I/O: {0}")] Io(#[from] std::io::Error),
    #[error("Scale: {0}")] Scale(String),
    #[error("Export: {0}")] Export(String),
    #[error("Config: {0}")] Config(String),
    #[error("Unknown: {0}")] Unknown(String),
    }
    pub type Result<T> = std::result::Result<T, StarsightError>;
    ```
- [x] Create starsight-layer-1/src/backend/mod.rs with DrawBackend trait (partial, some methods commented)

    ```rust
    pub trait DrawBackend {
    fn draw_path(&mut self, path: &Path, style: &PathStyle) -> Result<()>;
    fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()>;
    fn dimensions(&self) -> (u32, u32);
    fn save_png(&self, path: &std::path::Path) -> Result<()>;
    fn save_svg(&self, path: &std::path::Path) -> Result<()>;
    }
    ```
- [x] Create starsight-layer-1/src/primitives.rs with Color (r/g/b u8), Point (x/y f32), Rect (ltrb f32), Size (wh f32)

    ```rust
    pub struct Color { pub r: u8, pub g: u8, pub b: u8 }
    pub struct Point { pub x: f32, pub y: f32 }
    pub struct Rect { pub left: f32, pub top: f32, pub right: f32, pub bottom: f32 }
    pub struct Size { pub width: f32, pub height: f32 }
    ```
- [x] Create From<tiny_skia::Point> for Point, From<tiny_skia::Rect> for Rect, From<tiny_skia::Size> for Size

    ```rust
    impl From<tiny_skia::Point> for Point {
    fn from(p: tiny_skia::Point) -> Self { Self { x: p.x, y: p.y } }
    }
    impl From<tiny_skia::Rect> for Rect {
    fn from(r: tiny_skia::Rect) -> Self { Self { left: r.left(), top: r.top(), right: r.right(), bottom: r.bottom() } }
    }
    ```
- [x] Create all stub module files for every backend (skia/, svg/, pdf/, wgpu/, terminal/)

    ```bash
    mkdir -p starsight-layer-1/src/backend/{skia,svg,pdf,wgpu,terminal/{kitty,sixel,braille,half_block,iterm2}}
    for d in skia svg pdf wgpu terminal terminal/kitty terminal/sixel terminal/braille terminal/half_block terminal/iterm2; do
    echo "//!" > starsight-layer-1/src/backend/$d/mod.rs
    done
    ```
- [x] Create all stub lib.rs files for layers 2-7

    ```bash
    for i in 2 3 4 5 6 7; do
    echo "//!" > starsight-layer-$i/src/lib.rs
    done
    ```
- [x] Verify cargo check --workspace passes

    ```bash
    cargo check --workspace
    # Expected: no errors
    ```
- [x] Verify cargo test --workspace passes (zero tests, zero failures)

    ```bash
    cargo test --workspace
    # Expected: 0 tests, 0 failures
    ```

---

## 0.1.0 — Foundation

Exit criteria: plot!([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]).save("test.png") produces a correct line chart PNG.

### Layer 1: Complete the primitive types

The primitive types are the foundation. Every other layer depends on them. Get these right and the rest of the codebase inherits their correctness.

#### Add Vec2 with semantic arithmetic

- [x] Create `Vec2` struct in `starsight-layer-1/src/primitives.rs` with `x: f32, y: f32` fields

    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct Vec2 { pub x: f32, pub y: f32 }
    ```
- [x] Add derives: `Debug, Clone, Copy, PartialEq, Default`

    ```rust
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct Vec2 { pub x: f32, pub y: f32 }
    ```
- [x] Add constants: `ZERO`, `X`, `Y`

    ```rust
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };
    ```
- [x] Add `new(x, y)` constructor

    ```rust
    pub const fn new(x: f32, y: f32) -> Self { Self { x, y } }
    ```
- [x] Add `length()` and `normalize()` methods

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

- [x] Implement `Point - Point = Vec2` (Sub trait)

    ```rust
    impl std::ops::Sub for Point { type Output = Vec2;
    fn sub(self, rhs: Point) -> Vec2 { Vec2 { x: self.x - rhs.x, y: self.y - rhs.y } }
    }
    ```
- [x] Implement `Point + Vec2 = Point` (Add trait)

    ```rust
    impl std::ops::Add<Vec2> for Point { type Output = Point;
    fn add(self, rhs: Vec2) -> Point { Point { x: self.x + rhs.x, y: self.y + rhs.y } }
    }
    ```
- [x] Implement `Point - Vec2 = Point` (Sub trait)

    ```rust
    impl std::ops::Sub<Vec2> for Point { type Output = Point;
    fn sub(self, rhs: Vec2) -> Point { Point { x: self.x - rhs.x, y: self.y - rhs.y } }
    }
    ```
- [x] Implement `Vec2 + Vec2 = Vec2` (Add trait)

    ```rust
    impl std::ops::Add for Vec2 { type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 { Vec2 { x: self.x + rhs.x, y: self.y + rhs.y } }
    }
    ```
- [x] Implement `Vec2 * f32 = Vec2` (Mul trait)

    ```rust
    impl std::ops::Mul<f32> for Vec2 { type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 { Vec2 { x: self.x * rhs, y: self.y * rhs } }
    }
    impl std::ops::Mul<Vec2> for f32 { type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 { Vec2 { x: self * rhs.x, y: self * rhs.y } }
    }
    ```
- [x] Verify `Point + Point` does not compile (no Add<Point> for Point)

    ```rust
    // This should NOT compile — no Add<Point> for Point:
    // let p = Point::new(1.0, 2.0) + Point::new(3.0, 4.0); // ERROR
    // Verify with: cargo check and see E0369
    ```
- [x] Verify `Point * f32` does not compile (no Mul<f32> for Point)

    ```rust
    // This should NOT compile — no Mul<f32> for Point:
    // let p = Point::new(1.0, 2.0) * 3.0; // ERROR
    // Verify with: cargo check and see E0369
    ```
- [x] Write unit tests for all arithmetic operations

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

- [x] Add `From`/`Into` conversions for interop with other libraries:

    ```rust
    impl From<[f32; 2]> for Point { fn from([x, y]: [f32; 2]) -> Self { Self { x, y } } }
    impl From<(f32, f32)> for Point { fn from((x, y): (f32, f32)) -> Self { Self { x, y } } }
    impl From<Point> for [f32; 2] { fn from(p: Point) -> Self { [p.x, p.y] } }
    impl From<Point> for (f32, f32) { fn from(p: Point) -> Self { (p.x, p.y) } }
    // Same four impls for Vec2
    ```

- [x] Write tests:

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

- [x] Add convenience constructors and accessors:

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

- [x] Verify derives on Transform: `Debug, Clone, Copy, PartialEq`

    ```rust
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Transform(pub(crate) tiny_skia::Transform);
    ```
- [x] Add `Display` implementation for Transform:

    ```rust
    impl std::fmt::Display for Rect {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Rect({}, {}, {}, {})", self.left, self.top, self.right, self.bottom)
        }
    }
    ```

#### Complete the Color type

- [x] Add `ColorAlpha` and core Color methods:

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

- [x] Add `from_css_hex` and `to_css_hex`:

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

- [x] Add `luminance`, `contrast_ratio`, `lerp`:

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

- [x] Add sister crate conversions:

    ```rust
    impl From<chromata::Color> for Color {
        fn from(c: chromata::Color) -> Self { Self { r: c.r, g: c.g, b: c.b } }
    }
    impl From<prismatica::Color> for Color {
        fn from(c: prismatica::Color) -> Self { Self { r: c.r, g: c.g, b: c.b } }
    }
    ```

- [x] Write tests: `from_hex` roundtrip, `from_css_hex` with all formats, luminance black ≈ 0, luminance white ≈ 1, contrast black/white ≈ 21, lerp at 0.0 returns self, lerp at 1.0 returns other.

    ```rust
    #[test] fn from_hex_roundtrip() { let c = Color::from_hex(0x9634AD); assert_eq!(c.r, 150); }
    #[test] fn black_luminance() { assert!(Color::BLACK.luminance() < f64::EPSILON); }
    #[test] fn white_luminance() { assert!((Color::WHITE.luminance() - 1.0).abs() < f64::EPSILON); }
    #[test] fn contrast_21() { assert_eq!(Color::BLACK.contrast_ratio(Color::WHITE), 21.0); }
    ```

#### Add the Transform type

- [x] Create a `Transform` newtype wrapping `tiny_skia::Transform`:

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

- [x] Create `starsight-layer-1/src/backend/skia/raster/mod.rs`:

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

- [x] Implement `DrawBackend::dimensions()` for SkiaBackend

    ```rust
    fn dimensions(&self) -> (u32, u32) { (self.pixmap.width(), self.pixmap.height()) }
    ```
- [x] Implement `DrawBackend::save_png()` for SkiaBackend

    ```rust
    fn save_png(&self, path: &std::path::Path) -> Result<()> {
    self.pixmap.save_png(path).map_err(|e| StarsightError::Export(e.to_string()))
    }
    ```
- [x] Implement `DrawBackend::fill_rect()` for SkiaBackend

    ```rust
    fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
    let sk_rect = rect.to_tiny_skia().ok_or_else(|| StarsightError::Render("Invalid rect".into()))?;
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.r, color.g, color.b, 255);
    self.pixmap.fill_rect(sk_rect, &paint, tiny_skia::Transform::identity(), None);
    Ok(())
    }
    ```
- [x] Implement `DrawBackend::draw_path()` for SkiaBackend

    ```rust
    fn draw_path(&mut self, path: &crate::backend::Path, style: &PathStyle) -> Result<()> {
    let mut pb = PathBuilder::new();
    for cmd in path.commands() {
        match cmd {
            PathCommand::MoveTo(p) => pb.move_to(p.x, p.y),
            PathCommand::LineTo(p) => pb.line_to(p.x, p.y),
            PathCommand::QuadTo(c, p) => pb.quad_to(c.x, c.y, p.x, p.y),
            PathCommand::CubicTo(c1, c2, p) => pb.cubic_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y),
            PathCommand::Close => pb.close(),
        }
    }
    let sk_path = pb.finish().ok_or_else(|| StarsightError::Render("Empty path".into()))?;
    // ... stroke with paint and style
    Ok(())
    }
    ```
- [x] Implement `DrawBackend::draw_text()` for SkiaBackend:

    ```rust
    fn draw_text(&mut self, text: &str, position: Point, font_size: f32, color: Color) -> Result<()> {
        let metrics = cosmic_text::Metrics::new(font_size, font_size * 1.2);
        let mut buffer = cosmic_text::Buffer::new(&mut self.font_system, metrics);
        buffer.set_text(
            &mut self.font_system, text,
            &cosmic_text::Attrs::new(), cosmic_text::Shaping::Advanced, None,
        );
        buffer.set_size(&mut self.font_system, Some(self.pixmap.width() as f32), None);
        buffer.shape_until_scroll(&mut self.font_system, true);
    
        let text_color = cosmic_text::Color::rgba(color.r, color.g, color.b, 255);
        let mut paint = Paint::default();
        buffer.draw(&mut self.font_system, &mut self.swash_cache, text_color, |x, y, w, h, c| {
            paint.set_color_rgba8(c.r(), c.g(), c.b(), c.a());
            let px = x as f32 + position.x;
            let py = y as f32 + position.y;
            if let Some(rect) = tiny_skia::Rect::from_xywh(px, py, w as f32, h as f32) {
                self.pixmap.fill_rect(rect, &paint, tiny_skia::Transform::identity(), None);
            }
        });
        Ok(())
    }
    ```

- [x] Implement `DrawBackend::set_clip()` for SkiaBackend:

    ```rust
    fn set_clip(&mut self, rect: Option<Rect>) -> Result<()> {
        match rect {
            Some(r) => {
                let mut mask = tiny_skia::Mask::new(self.pixmap.width(), self.pixmap.height())
                    .ok_or_else(|| StarsightError::Render("Failed to create mask".into()))?;
                let clip_path = PathBuilder::from_rect(
                    r.to_tiny_skia().ok_or_else(|| StarsightError::Render("Invalid clip rect".into()))?
                );
                mask.fill_path(&clip_path, tiny_skia::FillRule::Winding, false, tiny_skia::Transform::identity());
                self.clip_mask = Some(mask);
            }
            None => { self.clip_mask = None; }
        }
        Ok(())
    }
    // Add field to SkiaBackend: clip_mask: Option<tiny_skia::Mask>
    ```
- [x] Key methods reference:

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

- [x] Uncomment the commented-out methods and `PathCommand` variants in `backend/mod.rs`:

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

- [x] Add to root `Cargo.toml`:

    ```toml
    [workspace.dependencies]
    insta = { version = "1.47.2", features = ["binary"] }
    ```

- [x] Create `starsight-layer-1/tests/snapshot_basic.rs`:

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

- [x] Create `SvgBackend` struct and constructor:

    ```rust
    use svg::node::element::{Rectangle, Text as SvgText, Path as SvgPath, Group, ClipPath};
    use svg::Document;
    use crate::backend::DrawBackend;
    use crate::error::{Result, StarsightError};
    use crate::primitives::{color::Color, geom::{Point, Rect}};
    
    pub struct SvgBackend {
        width: u32,
        height: u32,
        elements: Vec<Box<dyn svg::Node>>,
        clip_id: usize,
    }
    
    impl SvgBackend {
        pub fn new(width: u32, height: u32) -> Self {
            Self { width, height, elements: Vec::new(), clip_id: 0 }
        }
    
        fn build_document(&self) -> Document {
            let mut doc = Document::new()
                .set("viewBox", (0, 0, self.width, self.height))
                .set("xmlns", "http://www.w3.org/2000/svg")
                .set("width", self.width)
                .set("height", self.height);
            for el in &self.elements {
                doc = doc.add((*el).clone());
            }
            doc
        }
    
        pub fn svg_string(&self) -> String {
            self.build_document().to_string()
        }
    }
    ```

- [x] Implement `DrawBackend` for SvgBackend:

    ```rust
    impl DrawBackend for SvgBackend {
        fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
            let r = Rectangle::new()
                .set("x", rect.left)
                .set("y", rect.top)
                .set("width", rect.width())
                .set("height", rect.height())
                .set("fill", color.to_css_hex());
            self.elements.push(Box::new(r));
            Ok(())
        }
    
        fn draw_path(&mut self, path: &crate::backend::Path, style: &crate::backend::PathStyle) -> Result<()> {
            let mut data = svg::node::element::path::Data::new();
            for cmd in path.commands() {
                match cmd {
                    PathCommand::MoveTo(p) => { data = data.move_to((p.x, p.y)); }
                    PathCommand::LineTo(p) => { data = data.line_to((p.x, p.y)); }
                    PathCommand::Close => { data = data.close(); }
                    _ => {} // QuadTo, CubicTo — extend later
                }
            }
            let p = SvgPath::new()
                .set("d", data)
                .set("stroke", style.stroke_color.to_css_hex())
                .set("stroke-width", style.stroke_width)
                .set("fill", style.fill_color.map_or("none".to_string(), |c| c.to_css_hex()));
            self.elements.push(Box::new(p));
            Ok(())
        }
    
        fn draw_text(&mut self, text: &str, position: Point, font_size: f32, color: Color) -> Result<()> {
            let t = SvgText::new(text)
                .set("x", position.x)
                .set("y", position.y)
                .set("font-size", font_size)
                .set("fill", color.to_css_hex())
                .set("font-family", "sans-serif");
            self.elements.push(Box::new(t));
            Ok(())
        }
    
        fn dimensions(&self) -> (u32, u32) { (self.width, self.height) }
    
        fn save_png(&self, _path: &std::path::Path) -> Result<()> {
            Err(StarsightError::Export("SVG backend cannot save PNG directly".into()))
        }
    
        fn save_svg(&self, path: &std::path::Path) -> Result<()> {
            svg::save(path, &self.build_document())
                .map_err(|e| StarsightError::Export(e.to_string()))
        }
    }
    ```

- [x] Implement save_svg: call svg::save(path, &self.document) and map errors.

    ```rust
    // Already covered in DrawBackend impl above:
    fn save_svg(&self, path: &std::path::Path) -> Result<()> {
    svg::save(path, &self.build_document()).map_err(|e| StarsightError::Export(e.to_string()))
    }
    ```

- [x] Implement `save_svg()`: serialize document to file

    ```rust
    fn save_svg(&self, path: &std::path::Path) -> Result<()> {
    svg::save(path, &self.build_document()).map_err(|e| StarsightError::Export(e.to_string()))
    }
    ```
- [x] Implement `save_png()`: return `StarsightError::Export` (not supported by SVG backend)

    ```rust
    fn save_png(&self, _path: &std::path::Path) -> Result<()> {
    Err(StarsightError::Export("SVG backend cannot save PNG directly; use SkiaBackend or resvg".into()))
    }
    ```

- [x] Write SVG snapshot test:

    ```rust
    #[test]
    fn svg_blue_rect() {
        let mut backend = SvgBackend::new(200, 100);
        backend.fill_rect(Rect::from_xywh(10.0, 10.0, 180.0, 80.0), Color::BLUE).unwrap();
        let svg = backend.svg_string();
        insta::assert_snapshot!(svg);
    }
    ```

### Layer 2: Linear scale and Wilkinson ticks

- [x] Create `starsight-layer-2/src/scale.rs`:

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

- [x] Create `starsight-layer-2/src/tick.rs` and implement the algorithm:

    ```rust
    const Q: &[f64] = &[1.0, 5.0, 2.0, 2.5, 4.0, 3.0];
    const W: [f64; 4] = [0.2, 0.25, 0.5, 0.05]; // simplicity, coverage, density, legibility
    
    pub fn wilkinson_extended(dmin: f64, dmax: f64, target: usize) -> Vec<f64> {
        if (dmax - dmin).abs() < f64::EPSILON {
            return vec![dmin];
        }
        let mut best_score = -2.0;
        let mut best_ticks = Vec::new();
    
        for i in 0..Q.len() {
            let q = Q[i];
            for j in 1..=20 {  // j = skip factor
                let step_candidates = nice_steps(q, j);
                for &step in &step_candidates {
                    let k_min = (dmin / step).ceil() as i64;
                    let k_max = (dmax / step).floor() as i64;
                    for k in k_min..=k_max {
                        let lmin = k as f64 * step;
                        let lmax = lmin + (target as f64 - 1.0) * step;
                        if lmin > dmin || lmax < dmax { continue; }
    
                        let simp = 1.0 - (i as f64) / (Q.len() as f64 - 1.0);
                        let cov = coverage_score(dmin, dmax, lmin, lmax);
                        let dens = density_score(target, ((lmax - lmin) / step) as usize + 1);
                        let score = W[0]*simp + W[1]*cov + W[2]*dens + W[3];
    
                        if score > best_score {
                            best_score = score;
                            let n = ((lmax - lmin) / step).round() as usize + 1;
                            best_ticks = (0..n).map(|i| lmin + i as f64 * step).collect();
                        }
                    }
                }
            }
        }
        best_ticks
    }
    
    fn coverage_score(dmin: f64, dmax: f64, lmin: f64, lmax: f64) -> f64 {
        let range = dmax - dmin;
        1.0 - 0.5 * ((dmax - lmax).powi(2) + (dmin - lmin).powi(2)) / (0.1 * range).powi(2)
    }
    
    fn density_score(target: usize, actual: usize) -> f64 {
        let r = actual as f64 / target as f64;
        2.0 - r.max(1.0 / r)
    }
    
    fn nice_steps(q: f64, j: usize) -> Vec<f64> {
        (-10..=10).map(|z| j as f64 * q * 10f64.powi(z)).collect()
    }
    ```
- [x] Write tick unit tests and property tests:

    ```rust
    #[cfg(test)]
    mod tests {
        use super::*;
    
        #[test]
        fn ticks_0_to_100() {
            let ticks = wilkinson_extended(0.0, 100.0, 5);
            assert!(!ticks.is_empty());
            assert!(ticks[0] <= 0.0);
            assert!(*ticks.last().unwrap() >= 100.0);
        }
    
        #[test]
        fn ticks_0_to_1() {
            let ticks = wilkinson_extended(0.0, 1.0, 5);
            assert!(!ticks.is_empty());
            // Steps should be 0.2 or 0.25
            let step = ticks[1] - ticks[0];
            assert!(step > 0.0 && step <= 0.5);
        }
    
        #[test]
        fn ticks_negative_range() {
            let ticks = wilkinson_extended(-50.0, 50.0, 5);
            assert!(ticks[0] <= -50.0);
            assert!(*ticks.last().unwrap() >= 50.0);
        }
    
        #[test]
        fn ticks_zero_width() {
            let ticks = wilkinson_extended(42.0, 42.0, 5);
            assert_eq!(ticks, vec![42.0]);
        }
    }
    
    // In a separate proptest module:
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn ticks_monotonic(min in -1e6f64..0.0, max in 0.1f64..1e6) {
            let ticks = wilkinson_extended(min, max, 5);
            for pair in ticks.windows(2) {
                prop_assert!(pair[0] < pair[1], "ticks not monotonic: {:?}", ticks);
            }
        }
    }
    ```

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

- [x] Create `starsight-layer-2/src/axis.rs`:

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

- [x] Create `starsight-layer-2/src/coord.rs`:

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

- [x] Create `starsight-layer-3/src/mark.rs`:

    ```rust
    use starsight_layer_1::backend::DrawBackend;
    use starsight_layer_1::error::Result;
    use starsight_layer_2::coord::CartesianCoord;
    
    pub trait Mark {
        fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()>;
    }
    ```

- [x] Create `LineMark` and implement `Mark` trait:

    ```rust
    use crate::backend::{DrawBackend, PathCommand, PathStyle};
    use crate::primitives::{color::Color, geom::Point};
    
    #[derive(Debug, Clone)]
    pub struct LineMark {
        pub x: Vec<f64>,
        pub y: Vec<f64>,
        pub color: Color,
        pub width: f32,
    }
    
    impl LineMark {
        pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
            Self { x, y, color: Color::BLUE, width: 2.0 }
        }
        pub fn color(mut self, c: Color) -> Self { self.color = c; self }
        pub fn width(mut self, w: f32) -> Self { self.width = w; self }
    }
    
    impl Mark for LineMark {
        fn render(
            &self,
            coord: &CartesianCoord,
            backend: &mut dyn DrawBackend,
        ) -> crate::error::Result<()> {
            let mut commands = Vec::new();
            let mut pen_down = false;
    
            for i in 0..self.x.len() {
                if self.x[i].is_nan() || self.y[i].is_nan() {
                    pen_down = false; // NaN gap: lift pen
                    continue;
                }
                let px = coord.map_x(self.x[i]);
                let py = coord.map_y(self.y[i]);
                let pt = Point::new(px as f32, py as f32);
    
                if pen_down {
                    commands.push(PathCommand::LineTo(pt));
                } else {
                    commands.push(PathCommand::MoveTo(pt));
                    pen_down = true;
                }
            }
            // TODO: convert commands to Path and call backend.draw_path()
            Ok(())
        }
    }
    ```

- [ ] Write snapshot test: basic line chart:

    ```rust
    #[test]
    fn snapshot_line_basic() {
        let fig = Figure::new(800, 600)
            .add(LineMark::new(vec![0.0, 1.0, 2.0, 3.0], vec![0.0, 1.0, 0.5, 2.0]));
        let bytes = fig.render_png().unwrap();
        insta::assert_binary_snapshot!(".png", bytes);
    }
    ```
- [ ] Write snapshot test: line chart with NaN gaps:

    ```rust
    #[test]
    fn snapshot_line_nan_gaps() {
        let fig = Figure::new(800, 600)
            .add(LineMark::new(vec![0.0, 1.0, 2.0, 3.0, 4.0], vec![0.0, 1.0, f64::NAN, 0.5, 2.0]));
        let bytes = fig.render_png().unwrap();
        insta::assert_binary_snapshot!(".png", bytes);
    }
    ```
- [ ] Write snapshot test: multi-series line chart:

    ```rust
    #[test]
    fn snapshot_line_multi() {
        let fig = Figure::new(800, 600)
            .add(LineMark::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0]).color(Color::BLUE))
            .add(LineMark::new(vec![0.0, 1.0, 2.0], vec![2.0, 1.0, 0.0]).color(Color::RED));
        let bytes = fig.render_png().unwrap();
        insta::assert_binary_snapshot!(".png", bytes);
    }
    ```
- [ ] Implementation reference (NaN gap handling):

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

- [ ] Create `PointMark` and implement `Mark` trait:

    ```rust
    #[derive(Debug, Clone)]
    pub struct PointMark {
        pub x: Vec<f64>,
        pub y: Vec<f64>,
        pub color: Color,
        pub radius: f32,
    }
    
    impl PointMark {
        pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
            Self { x, y, color: Color::BLUE, radius: 4.0 }
        }
        pub fn color(mut self, c: Color) -> Self { self.color = c; self }
        pub fn radius(mut self, r: f32) -> Self { self.radius = r; self }
    }
    
    impl Mark for PointMark {
        fn render(
            &self,
            coord: &CartesianCoord,
            backend: &mut dyn DrawBackend,
        ) -> crate::error::Result<()> {
            // Batch all circles into one path for performance
            let mut pb = tiny_skia::PathBuilder::new();
            for i in 0..self.x.len() {
                if self.x[i].is_nan() || self.y[i].is_nan() { continue; }
                let px = coord.map_x(self.x[i]) as f32;
                let py = coord.map_y(self.y[i]) as f32;
                pb.push_circle(px, py, self.radius);
            }
            let path = pb.finish().ok_or_else(|| StarsightError::Render("Empty scatter".into()))?;
    
            let mut paint = Paint::default();
            paint.set_color_rgba8(self.color.r, self.color.g, self.color.b, 255);
            // Fill all circles in one call
            backend.pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
            Ok(())
        }
    }
    ```

- [ ] Write snapshot test: basic scatter plot:

    ```rust
    #[test]
    fn snapshot_scatter_basic() {
        let fig = Figure::new(800, 600)
            .add(PointMark::new(vec![0.5, 1.5, 2.5], vec![1.0, 3.0, 2.0]));
        let bytes = fig.render_png().unwrap();
        insta::assert_binary_snapshot!(".png", bytes);
    }
    ```
- [ ] Write snapshot test: scatter with varying point sizes:

    ```rust
    #[test]
    fn snapshot_scatter_sizes() {
        let fig = Figure::new(800, 600)
            .add(PointMark::new(vec![1.0, 2.0, 3.0], vec![1.0, 2.0, 3.0]).radius(8.0))
            .add(PointMark::new(vec![1.0, 2.0, 3.0], vec![3.0, 2.0, 1.0]).radius(3.0));
        let bytes = fig.render_png().unwrap();
        insta::assert_binary_snapshot!(".png", bytes);
    }
    ```
- [ ] Implementation reference (batched circles):

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

- [ ] Create facade `starsight/src/lib.rs` with re-exports:

    ```rust
    pub use starsight_layer_1::primitives::color::Color;
    pub use starsight_layer_1::primitives::geom::{Point, Rect, Size, Vec2};
    pub use starsight_layer_1::error::{StarsightError, Result};
    pub use starsight_layer_1::backend::skia::raster::SkiaBackend;
    pub use starsight_layer_1::backend::DrawBackend;
    pub use starsight_layer_5::figure::Figure;
    
    pub mod prelude;
    
    #[macro_export]
    macro_rules! plot {
        ($x:expr, $y:expr $(,)?) => {
            $crate::Figure::from_arrays($x, $y)
        };
        ($data:expr $(,)?) => {
            $crate::Figure::from_single($data)
        };
    }
    ```

- [ ] Create `starsight/src/prelude.rs`:

    ```rust
    pub use crate::{Color, Point, Figure, Result, SkiaBackend, plot};
    ```

- [ ] Write integration test `starsight/tests/integration.rs`:

    ```rust
    use starsight::prelude::*;
    
    #[test]
    fn plot_macro_produces_png() {
        let fig = plot!(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]);
        fig.save("test_output.png").unwrap();
        assert!(std::path::Path::new("test_output.png").exists());
        std::fs::remove_file("test_output.png").ok();
    }
    ```

- [ ] Verify `cargo test --workspace` passes with the full pipeline

    ```bash
    cargo test --workspace
    # Expected: all tests pass, integration test produces test.png
    ```
- [ ] Facade wiring reference:

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

## 0.2.0 — Core chart types part 1

Exit criteria: bar charts, area charts, histograms, and heatmaps render correctly with snapshot tests.

### Layer 3: BarMark

- [x] Create `BarMark` in `starsight-layer-3/src/marks/bar.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct BarMark {
        x: Vec<String>,          // category labels
        y: Vec<f64>,             // bar heights
        color: Option<Color>,
        width: Option<f32>,      // bar width as fraction of band (0.0-1.0, default 0.8)
        orientation: Orientation, // Vertical | Horizontal
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[non_exhaustive]
    pub enum Orientation {
        #[default]
        Vertical,
        Horizontal,
    }
    
    impl BarMark {
        pub fn new(x: Vec<String>, y: Vec<f64>) -> Self {
            Self { x, y, color: None, width: None, orientation: Orientation::default() }
        }
        pub fn horizontal(mut self) -> Self { self.orientation = Orientation::Horizontal; self }
        pub fn color(mut self, color: impl Into<Color>) -> Self { self.color = Some(color.into()); self }
        pub fn width(mut self, w: f32) -> Self { self.width = Some(w); self }
    }
    ```

- [x] Implement `Mark` for `BarMark` — vertical variant first:

    ```rust
    // For vertical bars:
    let band_width = band_scale.bandwidth();
    let bar_width = band_width * self.width.unwrap_or(0.8);
    let x_center = band_scale.map(&label);
    let x_left = x_center - bar_width / 2.0;
    let y_top = y_scale.map(value);
    let y_bottom = y_scale.map(0.0);  // bars grow from baseline
    let rect = Rect::from_ltrb(x_left, y_top, x_left + bar_width, y_bottom);
    backend.fill_rect(rect, color)?;
    ```

- [x] Implement horizontal bar variant:

    ```rust
    // In BarMark::render(), when self.orientation == Orientation::Horizontal:
    let y_center = band_scale.map(&label);
    let bar_height = band_scale.bandwidth() * self.width.unwrap_or(0.8);
    let y_top = y_center - bar_height / 2.0;
    let x_left = x_scale.map(0.0);       // bars grow from left
    let x_right = x_scale.map(value);     // to data value
    let rect = Rect::from_ltrb(x_left as f32, y_top as f32, x_right as f32, (y_top + bar_height) as f32);
    backend.fill_rect(rect, color)?;
    ```
- [ ] Add grouped bars: accept a `group` field, subdivide each band into sub-bands per group, offset each sub-group's bar within the band

    ```rust
    // In BarMark:
    pub fn group(mut self, name: &str) -> Self { self.group = Some(name.to_string()); self }
    
    // In render: subdivide band into sub-bands
    let n_groups = groups.len();
    let sub_width = band_width / n_groups as f32;
    let group_idx = groups.iter().position(|g| g == &self.group.as_deref().unwrap_or("")).unwrap_or(0);
    let x_offset = sub_width * group_idx as f32 - band_width / 2.0 + sub_width / 2.0;
    ```
- [ ] Add stacked bars: accept a `stack` field, accumulate y values per category, each bar's baseline is the top of the previous bar

    ```rust
    pub fn stack(mut self, name: &str) -> Self { self.stack = Some(name.to_string()); self }
    
    // In render: accumulate baselines per category
    let mut baselines: HashMap<String, f64> = HashMap::new();
    for (label, value) in self.x.iter().zip(self.y.iter()) {
    let base = baselines.entry(label.clone()).or_insert(0.0);
    let rect = Rect::from_ltrb(x_left, coord.map_y(*base + value), x_right, coord.map_y(*base));
    backend.fill_rect(rect, color)?;
    *base += value;
    }
    ```
- [ ] Write snapshot test: single vertical bar chart:

    ```rust
    #[test]
    fn snapshot_bar_vertical() {
        let fig = Figure::new(800, 600).add(
            BarMark::new(vec!["A".into(), "B".into(), "C".into()], vec![10.0, 25.0, 15.0])
        );
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: horizontal bar chart:

    ```rust
    #[test]
    fn snapshot_bar_horizontal() {
        let fig = Figure::new(800, 600).add(
            BarMark::new(vec!["A".into(), "B".into()], vec![10.0, 25.0]).horizontal()
        );
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: grouped bar chart:

    ```rust
    #[test]
    fn snapshot_bar_grouped() {
        let fig = Figure::new(800, 600)
            .add(BarMark::new(vec!["Q1".into(), "Q2".into()], vec![10.0, 20.0]).group("Sales"))
            .add(BarMark::new(vec!["Q1".into(), "Q2".into()], vec![15.0, 12.0]).group("Costs"));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: stacked bar chart:

    ```rust
    #[test]
    fn snapshot_bar_stacked() {
        let fig = Figure::new(800, 600)
            .add(BarMark::new(vec!["A".into(), "B".into()], vec![10.0, 20.0]).stack("s1"))
            .add(BarMark::new(vec!["A".into(), "B".into()], vec![5.0, 8.0]).stack("s2"));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### Layer 3: AreaMark

- [ ] Create `AreaMark` in `starsight-layer-3/src/marks/area.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct AreaMark {
        x: Vec<f64>,
        y: Vec<f64>,
        baseline: AreaBaseline,
        color: Option<Color>,
        alpha: f32,  // fill opacity, default 0.4
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    #[non_exhaustive]
    pub enum AreaBaseline {
        #[default]
        Zero,                    // fill between y and y=0
        Fixed(f64),              // fill between y and a fixed value
    }
    ```

- [ ] Build and render area path:

    ```rust
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let baseline_px = coord.map_y(match self.baseline {
            AreaBaseline::Zero => 0.0,
            AreaBaseline::Fixed(v) => v,
        }) as f32;
    
        // Closed fill path
        let mut pb = PathBuilder::new();
        pb.move_to(coord.map_x(self.x[0]) as f32, baseline_px);
        for i in 0..self.x.len() {
            pb.line_to(coord.map_x(self.x[i]) as f32, coord.map_y(self.y[i]) as f32);
        }
        pb.line_to(coord.map_x(*self.x.last().unwrap()) as f32, baseline_px);
        pb.close();
        let fill_path = pb.finish().unwrap();
    
        // Fill with semi-transparent color
        let mut paint = Paint::default();
        let ca = self.color.unwrap_or(Color::BLUE).with_alpha((self.alpha * 255.0) as u8);
        paint.set_color_rgba8(ca.r, ca.g, ca.b, ca.a);
        pixmap.fill_path(&fill_path, &paint, FillRule::Winding, Transform::identity(), None);
    
        // Stroke top edge only (full opacity)
        let mut stroke_pb = PathBuilder::new();
        stroke_pb.move_to(coord.map_x(self.x[0]) as f32, coord.map_y(self.y[0]) as f32);
        for i in 1..self.x.len() {
            stroke_pb.line_to(coord.map_x(self.x[i]) as f32, coord.map_y(self.y[i]) as f32);
        }
        let stroke_path = stroke_pb.finish().unwrap();
        let stroke_color = self.color.unwrap_or(Color::BLUE);
        paint.set_color_rgba8(stroke_color.r, stroke_color.g, stroke_color.b, 255);
        pixmap.stroke_path(&stroke_path, &paint, &Stroke { width: 2.0, ..Default::default() },
            Transform::identity(), None);
        Ok(())
    }
    ```

- [ ] Implement stacked area rendering:

    ```rust
    fn render_stacked(areas: &[AreaMark], coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let n_points = areas[0].x.len();
        let mut cumulative = vec![0.0f64; n_points]; // running y-baseline per x position
    
        for area in areas {
            let mut pb = PathBuilder::new();
            // Bottom edge: previous cumulative (right to left)
            for i in (0..n_points).rev() {
                let px = coord.map_x(area.x[i]) as f32;
                let py = coord.map_y(cumulative[i]) as f32;
                if i == n_points - 1 { pb.move_to(px, py); } else { pb.line_to(px, py); }
            }
            // Top edge: current cumulative (left to right)
            for i in 0..n_points {
                let new_y = cumulative[i] + area.y[i];
                let px = coord.map_x(area.x[i]) as f32;
                let py = coord.map_y(new_y) as f32;
                pb.line_to(px, py);
            }
            pb.close();
            // Fill and update cumulative
            // ...
            for i in 0..n_points { cumulative[i] += area.y[i]; }
        }
        Ok(())
    }
    ```
- [ ] Write snapshot test: basic area chart:

    ```rust
    #[test]
    fn snapshot_area_basic() {
        let fig = Figure::new(800, 600).add(
            AreaMark::new(vec![0.0, 1.0, 2.0, 3.0], vec![0.0, 3.0, 1.0, 4.0])
        );
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: stacked area chart:

    ```rust
    #[test]
    fn snapshot_area_stacked() {
        let x = vec![0.0, 1.0, 2.0, 3.0];
        let fig = Figure::new(800, 600)
            .add(AreaMark::new(x.clone(), vec![1.0, 2.0, 1.5, 3.0]).color(Color::BLUE))
            .add(AreaMark::new(x, vec![2.0, 1.0, 2.5, 1.0]).color(Color::RED));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### Layer 3: Histogram stat transform

- [ ] Create `BinTransform` in `starsight-layer-3/src/stats/bin.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct BinTransform {
        bins: BinMethod,
    }
    
    #[derive(Debug, Clone, Copy)]
    #[non_exhaustive]
    pub enum BinMethod {
        Count(usize),           // exact number of bins
        Width(f64),             // exact bin width
        Auto,                   // Sturges' rule: ceil(1 + log2(n))
        Fd,                     // Freedman-Diaconis: 2 * IQR * n^(-1/3)
    }
    
    impl BinTransform {
        pub fn compute(&self, data: &[f64]) -> Vec<Bin> {
            let n = self.resolve_count(data);
            let (min, max) = data_range(data);
            let width = (max - min) / n as f64;
            let mut bins = vec![Bin { left: 0.0, right: 0.0, count: 0 }; n];
            for i in 0..n {
                bins[i].left = min + i as f64 * width;
                bins[i].right = min + (i + 1) as f64 * width;
            }
            for &val in data {
                if val.is_nan() { continue; }
                let idx = ((val - min) / width).floor() as usize;
                let idx = idx.min(n - 1); // clamp last edge
                bins[idx].count += 1;
            }
            bins
        }
    }
    ```

- [ ] Create `HistogramMark`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct HistogramMark {
        data: Vec<f64>,
        bins: BinMethod,
        kde: bool,
        color: Color,
    }
    
    impl HistogramMark {
        pub fn new(data: Vec<f64>) -> Self {
            Self { data, bins: BinMethod::Auto, kde: false, color: Color::from_hex(0x4C72B0) }
        }
        pub fn bins(mut self, method: BinMethod) -> Self { self.bins = method; self }
        pub fn kde(mut self, show: bool) -> Self { self.kde = show; self }
    }
    
    impl Mark for HistogramMark {
        fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
            let bins = BinTransform { bins: self.bins }.compute(&self.data);
            for bin in &bins {
                let rect = Rect::from_ltrb(
                    coord.map_x(bin.left) as f32,
                    coord.map_y(bin.count as f64) as f32,
                    coord.map_x(bin.right) as f32,
                    coord.map_y(0.0) as f32,
                );
                backend.fill_rect(rect, self.color)?;
            }
            if self.kde {
                // Overlay KDE as a LineMark on secondary y-axis (density)
                // ...
            }
            Ok(())
        }
    }
    ```
- [ ] Write snapshot test: basic histogram:

    ```rust
    #[test]
    fn snapshot_histogram_basic() {
        let data: Vec<f64> = (0..1000).map(|i| (i as f64 / 100.0).sin() * 3.0 + 5.0).collect();
        let fig = Figure::new(800, 600).add(HistogramMark::new(data));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: histogram with KDE overlay:

    ```rust
    #[test]
    fn snapshot_histogram_kde() {
        let data: Vec<f64> = (0..500).map(|i| (i as f64 * 0.1).sin() * 2.0).collect();
        let fig = Figure::new(800, 600).add(HistogramMark::new(data).kde(true));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: histogram with Freedman-Diaconis bins:

    ```rust
    #[test]
    fn snapshot_histogram_fd() {
        let data: Vec<f64> = (0..200).map(|i| i as f64 * 0.5).collect();
        let fig = Figure::new(800, 600).add(HistogramMark::new(data).bins(BinMethod::Fd));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### Layer 3: HeatmapMark

- [ ] Create `HeatmapMark` in `starsight-layer-3/src/marks/heatmap.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct HeatmapMark {
        data: Vec<Vec<f64>>,    // row-major 2D matrix
        colormap: ColormapRef,  // reference to a prismatica colormap
        annotate: bool,         // draw value text in each cell
    }
    ```

- [ ] Implement heatmap cell rendering:

    ```rust
    fn render_cell(&self, row: usize, col: usize, val: f64, vmin: f64, vmax: f64,
                   cell_rect: Rect, backend: &mut dyn DrawBackend) -> Result<()> {
        let t = ((val - vmin) / (vmax - vmin)).clamp(0.0, 1.0) as f32;
        let color: Color = self.colormap.eval(t).into();
        backend.fill_rect(cell_rect, color)?;
    
        if self.annotate {
            let text = format!("{:.1}", val);
            // Auto text color: white on dark cells, black on light cells
            let text_color = if color.luminance() > 0.5 { Color::BLACK } else { Color::WHITE };
            backend.draw_text(&text, cell_rect.center(), 12.0, text_color)?;
        }
        Ok(())
    }
    ```

- [ ] Write snapshot test: basic heatmap with sequential colormap:

    ```rust
    #[test]
    fn snapshot_heatmap_basic() {
        let data = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0], vec![7.0, 8.0, 9.0]];
        let fig = Figure::new(600, 600).add(HeatmapMark::new(data));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: annotated heatmap with value text:

    ```rust
    #[test]
    fn snapshot_heatmap_annotated() {
        let data = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let fig = Figure::new(400, 400).add(HeatmapMark::new(data).annotate(true));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: heatmap with diverging colormap:

    ```rust
    #[test]
    fn snapshot_heatmap_diverging() {
        let data = vec![vec![-3.0, -1.0, 0.0], vec![1.0, 0.0, -2.0], vec![2.0, 3.0, -1.0]];
        let fig = Figure::new(600, 600).add(
            HeatmapMark::new(data).colormap(prismatica::crameri::VIK)
        );
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

---

## 0.3.0 — Core chart types part 2

Exit criteria: statistical chart types (box, violin, pie, candlestick) render correctly.

### Layer 3: BoxPlotMark

- [ ] Create `BoxPlotMark` with five-number summary computation:

    ```rust
    #[derive(Debug, Clone)]
    pub struct BoxPlotStats {
        pub min: f64,
        pub q1: f64,
        pub median: f64,
        pub q3: f64,
        pub max: f64,
        pub outliers: Vec<f64>,  // points beyond 1.5 * IQR from Q1/Q3
    }
    
    impl BoxPlotStats {
        pub fn compute(data: &mut [f64]) -> Self {
            data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let data: Vec<f64> = data.iter().filter(|v| !v.is_nan()).copied().collect();
            let n = data.len();
            let q1 = percentile(&data, 0.25);
            let median = percentile(&data, 0.50);
            let q3 = percentile(&data, 0.75);
            let iqr = q3 - q1;
            let lower_fence = q1 - 1.5 * iqr;
            let upper_fence = q3 + 1.5 * iqr;
            let min = data.iter().filter(|&&v| v >= lower_fence).copied().next().unwrap_or(q1);
            let max = data.iter().filter(|&&v| v <= upper_fence).copied().last().unwrap_or(q3);
            let outliers: Vec<f64> = data.iter().filter(|&&v| v < lower_fence || v > upper_fence).copied().collect();
            Self { min, q1, median, q3, max, outliers }
        }
    }
    ```

- [ ] Render box plot elements:

    ```rust
    fn render_box(&self, stats: &BoxPlotStats, x_center: f32, half_width: f32,
                  coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let q1_px = coord.map_y(stats.q1) as f32;
        let q3_px = coord.map_y(stats.q3) as f32;
        let med_px = coord.map_y(stats.median) as f32;
        let min_px = coord.map_y(stats.min) as f32;
        let max_px = coord.map_y(stats.max) as f32;
        let cap = half_width * 0.5;
    
        // Box body (Q1 to Q3)
        backend.fill_rect(Rect::from_ltrb(x_center - half_width, q3_px, x_center + half_width, q1_px),
            Color::from_hex(0x4C72B0).with_alpha(180))?;
    
        // Median line
        draw_hline(backend, med_px, x_center - half_width, x_center + half_width, Color::WHITE, 2.0)?;
    
        // Whiskers (vertical lines + horizontal caps)
        draw_vline(backend, x_center, min_px, q1_px, Color::BLACK, 1.0)?;
        draw_vline(backend, x_center, q3_px, max_px, Color::BLACK, 1.0)?;
        draw_hline(backend, min_px, x_center - cap, x_center + cap, Color::BLACK, 1.0)?;
        draw_hline(backend, max_px, x_center - cap, x_center + cap, Color::BLACK, 1.0)?;
    
        // Outliers
        for &val in &stats.outliers {
            let py = coord.map_y(val) as f32;
            draw_circle(backend, x_center, py, 3.0, Color::BLACK)?;
        }
        Ok(())
    }
    ```
- [ ] Write snapshot test: basic box plot:

    ```rust
    #[test]
    fn snapshot_boxplot_basic() {
        let data = vec![
            ("A".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 20.0]),
            ("B".into(), vec![3.0, 4.0, 5.0, 5.5, 6.0, 6.5, 7.0, 8.0]),
        ];
        let fig = Figure::new(600, 400).add(BoxPlotMark::new(data));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: grouped box plot:

    ```rust
    #[test]
    fn snapshot_boxplot_grouped() {
        // Two groups per category
        let fig = Figure::new(800, 400)
            .add(BoxPlotMark::from_groups(vec![("A", "G1", vec![1.0,2.0,3.0,4.0])]))
            .add(BoxPlotMark::from_groups(vec![("A", "G2", vec![2.0,3.0,4.0,5.0])]));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: box plot with outliers:

    ```rust
    #[test]
    fn snapshot_boxplot_outliers() {
        let data = vec![("X".into(), vec![1.0, 2.0, 3.0, 4.0, 5.0, 100.0, -50.0])];
        let fig = Figure::new(400, 400).add(BoxPlotMark::new(data));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### Layer 3: ViolinMark

- [ ] Create `ViolinMark` in `starsight-layer-3/src/marks/violin.rs`. Depends on KDE stat transform.

    ```rust
    #[derive(Debug, Clone)]
    pub struct ViolinMark {
        groups: Vec<ViolinGroup>,
        bandwidth: Option<f64>,      // KDE bandwidth (None = auto via Silverman's rule)
        show_box: bool,              // overlay inner box plot (default true)
        show_median: bool,           // show median line (default true)
        cut: f64,                    // extend KDE beyond data range by cut*bw (default 2.0)
        scale: ViolinScale,          // area | count | width normalization
    }
    
    #[derive(Debug, Clone)]
    pub struct ViolinGroup {
        pub label: String,
        pub data: Vec<f64>,
    }
    ```

- [ ] Implement violin rendering:

    ```rust
    fn render_violin(&self, group: &ViolinGroup, x_center: f32, max_width: f32,
                     coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        // 1. Compute KDE
        let bw = self.bandwidth.unwrap_or_else(|| silverman_bandwidth(&group.data));
        let (y_min, y_max) = data_range(&group.data);
        let eval_points: Vec<f64> = (0..256).map(|i| {
            y_min - self.cut * bw + (y_max - y_min + 2.0 * self.cut * bw) * i as f64 / 255.0
        }).collect();
        let density: Vec<f64> = eval_points.iter().map(|&y| kde_at(y, &group.data, bw)).collect();
        let d_max = density.iter().cloned().fold(0.0f64, f64::max);
    
        // 2. Build mirrored path
        let mut pb = PathBuilder::new();
        // Right side (top to bottom)
        for i in 0..256 {
            let py = coord.map_y(eval_points[i]) as f32;
            let dx = (density[i] / d_max * max_width as f64 * 0.5) as f32;
            if i == 0 { pb.move_to(x_center + dx, py); } else { pb.line_to(x_center + dx, py); }
        }
        // Left side (bottom to top, mirrored)
        for i in (0..256).rev() {
            let py = coord.map_y(eval_points[i]) as f32;
            let dx = (density[i] / d_max * max_width as f64 * 0.5) as f32;
            pb.line_to(x_center - dx, py);
        }
        pb.close();
        // Fill and optionally overlay box plot
        // ...
        Ok(())
    }
    
    fn silverman_bandwidth(data: &[f64]) -> f64 {
        let n = data.len() as f64;
        let std = std_dev(data);
        let iqr = percentile(data, 0.75) - percentile(data, 0.25);
        0.9 * std.min(iqr / 1.34) * n.powf(-0.2)
    }
    ```

- [ ] Implement split violins: when two groups share a category, render left/right halves from different groups

    ```rust
    // When split=true, render group A density on left, group B on right:
    if self.split && groups.len() == 2 {
    let left_density = kde(&groups[0].data, bw);
    let right_density = kde(&groups[1].data, bw);
    // Left half: x_center - dx for left_density
    // Right half: x_center + dx for right_density
    }
    ```
- [ ] Write snapshot test: basic violin plot:

    ```rust
    #[test]
    fn snapshot_violin_basic() {
        let data = vec![ViolinGroup { label: "A".into(), data: vec![1.0,2.0,2.5,3.0,3.0,3.5,4.0,5.0] }];
        let fig = Figure::new(400, 400).add(ViolinMark::new(data));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: grouped violins

    ```rust
    #[test]
    fn snapshot_violin_grouped() {
    let data = vec![
        ViolinGroup { label: "A".into(), data: vec![1.0,2.0,3.0,4.0,5.0] },
        ViolinGroup { label: "B".into(), data: vec![2.0,3.0,4.0,5.0,6.0] },
    ];
    let fig = Figure::new(600, 400).add(ViolinMark::new(data));
    insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: split violin

    ```rust
    #[test]
    fn snapshot_violin_split() {
    let fig = Figure::new(600, 400).add(
        ViolinMark::new(data).split(true)
    );
    insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: violin with box overlay disabled

    ```rust
    #[test]
    fn snapshot_violin_no_box() {
    let fig = Figure::new(600, 400).add(
        ViolinMark::new(data).show_box(false)
    );
    insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### Layer 3: PieMark and DonutMark

- [ ] Implement PieMark with arc geometry:

    ```rust
    impl Mark for PieMark {
        fn render(&self, _coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
            let (cx, cy) = (self.center_x, self.center_y);
            let total: f64 = self.values.iter().sum();
            let mut start_angle = -std::f64::consts::FRAC_PI_2; // start at top
    
            for (i, &val) in self.values.iter().enumerate() {
                let sweep = val / total * std::f64::consts::TAU;
                let end_angle = start_angle + sweep;
    
                let mut pb = PathBuilder::new();
                if self.inner_radius > 0.0 {
                    // Donut: outer arc then inner arc reversed
                    arc_to(&mut pb, cx, cy, self.radius, start_angle, end_angle);
                    arc_to_reverse(&mut pb, cx, cy, self.inner_radius, end_angle, start_angle);
                } else {
                    pb.move_to(cx as f32, cy as f32);
                    arc_to(&mut pb, cx, cy, self.radius, start_angle, end_angle);
                }
                pb.close();
                // Fill with color from palette
                // ...
                start_angle = end_angle;
            }
            Ok(())
        }
    }
    
    /// Approximate arc with cubic beziers (one per quarter-circle segment)
    fn arc_to(pb: &mut PathBuilder, cx: f64, cy: f64, r: f64, start: f64, end: f64) {
        let segments = ((end - start).abs() / std::f64::consts::FRAC_PI_2).ceil() as usize;
        let step = (end - start) / segments as f64;
        for s in 0..segments {
            let a0 = start + s as f64 * step;
            let a1 = a0 + step;
            let k = (4.0 / 3.0) * ((a1 - a0) / 4.0).tan();
            let p0 = (cx + r * a0.cos(), cy + r * a0.sin());
            let p1 = (cx + r * a1.cos(), cy + r * a1.sin());
            let c0 = (p0.0 - k * r * a0.sin(), p0.1 + k * r * a0.cos());
            let c1 = (p1.0 + k * r * a1.sin(), p1.1 - k * r * a1.cos());
            if s == 0 { pb.move_to(p0.0 as f32, p0.1 as f32); }
            pb.cubic_to(c0.0 as f32, c0.1 as f32, c1.0 as f32, c1.1 as f32, p1.0 as f32, p1.1 as f32);
        }
    }
    ```
- [ ] Write snapshot test: basic pie chart:

    ```rust
    #[test]
    fn snapshot_pie_basic() {
        let fig = Figure::new(400, 400).add(
            PieMark::new(vec![30.0, 20.0, 50.0], vec!["A".into(), "B".into(), "C".into()])
        );
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: donut chart:

    ```rust
    #[test]
    fn snapshot_donut() {
        let fig = Figure::new(400, 400).add(
            PieMark::new(vec![30.0, 70.0], vec!["Yes".into(), "No".into()]).inner_radius(0.5)
        );
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

    ```rust
    fn arc_path(cx: f32, cy: f32, r: f32, start_rad: f32, end_rad: f32) -> Path {
        // Approximate arc with cubic bezier
        // For arcs <= PI/2, one cubic is sufficient
        // For larger arcs, subdivide into segments
        let mut pb = PathBuilder::new();
        let segments = ((end_rad - start_rad) / (std::f32::consts::FRAC_PI_2)).ceil() as usize;
        // ... build arc segments ...
        pb.finish().unwrap()
    }
    ```

### Layer 3: CandlestickMark

- [ ] Create `CandlestickMark` in `starsight-layer-3/src/marks/candlestick.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct CandlestickMark {
        data: Vec<OHLC>,
        up_color: Color,      // default green (#26a69a)
        down_color: Color,    // default red (#ef5350)
        body_width: f32,      // candle body width as fraction of available space (default 0.7)
        wick_width: f32,      // wick line width in pixels (default 1.0)
    }
    
    #[derive(Debug, Clone, Copy)]
    pub struct OHLC {
        pub timestamp: f64,   // x-axis position (epoch seconds or index)
        pub open: f64,
        pub high: f64,
        pub low: f64,
        pub close: f64,
    }
    ```

- [ ] Render candlestick elements:

    ```rust
    fn render_candle(&self, ohlc: &OHLC, x_px: f32, half_w: f32,
                     coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let color = if ohlc.close >= ohlc.open { self.up_color } else { self.down_color };
        let open_px = coord.map_y(ohlc.open) as f32;
        let close_px = coord.map_y(ohlc.close) as f32;
        let high_px = coord.map_y(ohlc.high) as f32;
        let low_px = coord.map_y(ohlc.low) as f32;
    
        // Body: filled rect from open to close
        let top = open_px.min(close_px);
        let bottom = open_px.max(close_px);
        backend.fill_rect(Rect::from_ltrb(x_px - half_w, top, x_px + half_w, bottom), color)?;
    
        // Upper wick: from top of body to high
        draw_vline(backend, x_px, high_px, top, color, self.wick_width)?;
        // Lower wick: from bottom of body to low
        draw_vline(backend, x_px, bottom, low_px, color, self.wick_width)?;
        Ok(())
    }
    ```

- [ ] Write snapshot test: basic candlestick chart:

    ```rust
    #[test]
    fn snapshot_candlestick() {
        let data = vec![
            OHLC { timestamp: 0.0, open: 100.0, high: 110.0, low: 95.0, close: 105.0 },
            OHLC { timestamp: 1.0, open: 105.0, high: 115.0, low: 100.0, close: 98.0 },
            OHLC { timestamp: 2.0, open: 98.0, high: 108.0, low: 90.0, close: 107.0 },
        ];
        let fig = Figure::new(800, 400).add(CandlestickMark::new(data));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: candlestick with custom colors

    ```rust
    #[test]
    fn snapshot_candlestick_colors() {
    let fig = Figure::new(800, 400).add(
        CandlestickMark::new(data).up_color(Color::from_hex(0x00BCD4)).down_color(Color::from_hex(0xFF5722))
    );
    insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### Layer 5: Polars integration

- [ ] Create data acceptance layer in `starsight-layer-5/src/data/polars.rs` (behind `polars` feature flag):

    ```rust
    use polars::prelude::*;
    
    pub fn extract_f64(df: &DataFrame, col: &str) -> Result<Vec<f64>> {
        let series = df.column(col)
            .map_err(|e| StarsightError::Data(format!("Column '{}': {}", col, e)))?;
        let ca = series.f64()
            .or_else(|_| series.cast(&DataType::Float64).and_then(|s| s.f64()))
            .map_err(|e| StarsightError::Data(format!("Cannot convert '{}' to f64: {}", col, e)))?;
        Ok(ca.into_no_null_iter().collect())
    }
    
    pub fn extract_strings(df: &DataFrame, col: &str) -> Result<Vec<String>> {
        let series = df.column(col)?;
        let ca = series.str()?;
        Ok(ca.into_no_null_iter().map(|s| s.to_string()).collect())
    }
    ```

- [ ] Integrate DataFrame path with `plot!` macro

    ```rust
    // In plot! macro, DataFrame arm:
    ($df:expr, x = $x:expr, y = $y:expr $(, $key:ident = $val:expr)* $(,)?) => {{
    let x_data = extract_f64($df, $x)?;
    let y_data = extract_f64($df, $y)?;
    let mut fig = Figure::new(800, 600).add(LineMark::new(x_data, y_data));
    $( fig = fig.$key($val); )*
    fig
    }};
    ```
- [ ] Auto-detect column types: numeric columns → LineMark, categorical x → BarMark

    ```rust
    fn detect_mark(df: &DataFrame, x_col: &str, y_col: &str) -> Box<dyn Mark> {
    let x_dtype = df.column(x_col).unwrap().dtype().clone();
    match x_dtype {
        DataType::String | DataType::Categorical(_, _) => {
            let labels = extract_strings(df, x_col).unwrap();
            let values = extract_f64(df, y_col).unwrap();
            Box::new(BarMark::new(labels, values))
        }
        _ => {
            let x = extract_f64(df, x_col).unwrap();
            let y = extract_f64(df, y_col).unwrap();
            Box::new(LineMark::new(x, y))
        }
    }
    }
    ```
- [ ] Support `color = "column"` for automatic grouping

    ```rust
    // When color = "col_name" is specified:
    let groups = extract_strings(df, color_col)?;
    let unique: Vec<String> = groups.iter().cloned().collect::<HashSet<_>>().into_iter().collect();
    for (i, group_val) in unique.iter().enumerate() {
    let mask: Vec<bool> = groups.iter().map(|g| g == group_val).collect();
    let x_sub: Vec<f64> = x.iter().zip(&mask).filter(|(_, &m)| m).map(|(v, _)| *v).collect();
    let y_sub: Vec<f64> = y.iter().zip(&mask).filter(|(_, &m)| m).map(|(v, _)| *v).collect();
    fig = fig.add(LineMark::new(x_sub, y_sub).color(palette[i % palette.len()]).label(group_val));
    }
    ```

- [ ] Accept eager `DataFrame` directly

    ```rust
    impl From<&DataFrame> for DataSource {
    fn from(df: &DataFrame) -> Self { DataSource::Polars(df.clone()) }
    }
    ```
- [ ] Accept `LazyFrame`: call `.collect()` before extraction, log warning about materialization

    ```rust
    impl From<LazyFrame> for DataSource {
    fn from(lf: LazyFrame) -> Self {
        log::warn!("Collecting LazyFrame — this materializes the entire frame");
        DataSource::Polars(lf.collect().unwrap())
    }
    }
    ```

- [ ] Convert Polars null values to f64 NaN for mark rendering pipeline compatibility

    ```rust
    fn extract_f64_with_nulls(df: &DataFrame, col: &str) -> Result<Vec<f64>> {
    let series = df.column(col)?;
    let ca = series.f64().or_else(|_| series.cast(&DataType::Float64)?.f64())?;
    Ok(ca.iter().map(|opt| opt.unwrap_or(f64::NAN)).collect())
    }
    ```

- [ ] Write snapshot test: line chart from DataFrame:

    ```rust
    #[test]
    fn snapshot_polars_line() {
        use polars::prelude::*;
        let df = df!("x" => &[0.0, 1.0, 2.0, 3.0], "y" => &[0.0, 1.0, 0.5, 2.0]).unwrap();
        let fig = plot!(df, x = "x", y = "y");
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: scatter with color grouping:

    ```rust
    #[test]
    fn snapshot_polars_scatter_grouped() {
        use polars::prelude::*;
        let df = df!(
            "x" => &[0.0, 1.0, 2.0, 0.5, 1.5, 2.5],
            "y" => &[0.0, 1.0, 0.5, 1.0, 0.0, 1.5],
            "group" => &["A", "A", "A", "B", "B", "B"]
        ).unwrap();
        let fig = plot!(df, x = "x", y = "y", color = "group");
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

---

## 0.4.0 — Layout and composition

Exit criteria: faceted charts and multi-panel layouts render correctly.

### Layer 4: GridLayout

- [ ] Create `GridLayout` in `starsight-layer-4/src/grid.rs`:

    ```rust
    #[derive(Debug)]
    pub struct GridLayout {
        figures: Vec<Vec<Option<Figure>>>,  // row-major, None for empty cells
        row_heights: Vec<f32>,              // proportional weights (default: equal)
        col_widths: Vec<f32>,               // proportional weights (default: equal)
        gap: f32,                           // pixels between panels (default: 10.0)
        title: Option<String>,              // overall title above the grid
    }
    
    impl GridLayout {
        pub fn new(rows: usize, cols: usize) -> Self { /* ... */ }
        pub fn set(&mut self, row: usize, col: usize, figure: Figure) { /* ... */ }
        pub fn row_height(mut self, row: usize, weight: f32) -> Self { /* ... */ }
        pub fn col_width(mut self, col: usize, weight: f32) -> Self { /* ... */ }
        pub fn gap(mut self, pixels: f32) -> Self { /* ... */ }
    }
    ```

- [ ] Implement grid cell layout and rendering:

    ```rust
    impl GridLayout {
        pub fn render(&self, backend: &mut dyn DrawBackend) -> Result<()> {
            let (total_w, total_h) = backend.dimensions();
            let rows = self.figures.len();
            let cols = self.figures[0].len();
            let gap = self.gap;
    
            let col_total: f32 = self.col_widths.iter().sum();
            let row_total: f32 = self.row_heights.iter().sum();
            let usable_w = total_w as f32 - gap * (cols as f32 - 1.0);
            let usable_h = total_h as f32 - gap * (rows as f32 - 1.0);
    
            let mut y_offset = 0.0f32;
            for r in 0..rows {
                let cell_h = usable_h * self.row_heights[r] / row_total;
                let mut x_offset = 0.0f32;
                for c in 0..cols {
                    let cell_w = usable_w * self.col_widths[c] / col_total;
                    if let Some(fig) = &self.figures[r][c] {
                        let cell_rect = Rect::from_xywh(x_offset, y_offset, cell_w, cell_h);
                        backend.set_clip(Some(cell_rect))?;
                        fig.render_at(backend, x_offset, y_offset, cell_w as u32, cell_h as u32)?;
                        backend.set_clip(None)?;
                    }
                    x_offset += cell_w + gap;
                }
                y_offset += cell_h + gap;
            }
            Ok(())
        }
    }
    ```

- [ ] Implement convenience constructors:

    ```rust
    impl GridLayout {
        pub fn row(figures: Vec<Figure>) -> Self {
            let n = figures.len();
            Self { figures: vec![figures.into_iter().map(Some).collect()],
                   row_heights: vec![1.0], col_widths: vec![1.0; n], gap: 10.0, title: None }
        }
        pub fn column(figures: Vec<Figure>) -> Self {
            let n = figures.len();
            Self { figures: figures.into_iter().map(|f| vec![Some(f)]).collect(),
                   row_heights: vec![1.0; n], col_widths: vec![1.0], gap: 10.0, title: None }
        }
        pub fn from_figures(figures: Vec<Figure>, ncol: usize) -> Self {
            let chunks: Vec<Vec<Option<Figure>>> = figures.chunks(ncol)
                .map(|chunk| {
                    let mut row: Vec<Option<Figure>> = chunk.iter().cloned().map(Some).collect();
                    row.resize_with(ncol, || None); // pad last row
                    row
                }).collect();
            let nrow = chunks.len();
            Self { figures: chunks, row_heights: vec![1.0; nrow], col_widths: vec![1.0; ncol],
                   gap: 10.0, title: None }
        }
    }
    ```

- [ ] Write snapshot test: 2x2 grid with mixed chart types:

    ```rust
    #[test]
    fn snapshot_grid_2x2() {
        let mut grid = GridLayout::new(2, 2);
        grid.set(0, 0, Figure::new(400, 300).add(LineMark::new(vec![0.,1.,2.], vec![0.,1.,0.5])));
        grid.set(0, 1, Figure::new(400, 300).add(PointMark::new(vec![0.,1.,2.], vec![1.,0.,2.])));
        grid.set(1, 0, Figure::new(400, 300).add(BarMark::new(vec!["A".into(),"B".into()], vec![10.,20.])));
        grid.set(1, 1, Figure::new(400, 300).add(HistogramMark::new(vec![1.,2.,2.,3.,3.,3.,4.])));
        let bytes = grid.render_png(800, 600).unwrap();
        insta::assert_binary_snapshot!(".png", bytes);
    }
    ```
- [ ] Write snapshot test: 1x3 row layout:

    ```rust
    #[test]
    fn snapshot_grid_row() {
        let grid = GridLayout::row(vec![
            Figure::new(200, 200).add(LineMark::new(vec![0.,1.], vec![0.,1.])),
            Figure::new(200, 200).add(LineMark::new(vec![0.,1.], vec![1.,0.])),
            Figure::new(200, 200).add(PointMark::new(vec![0.5], vec![0.5])),
        ]);
        insta::assert_binary_snapshot!(".png", grid.render_png(600, 200).unwrap());
    }
    ```
- [ ] Write snapshot test: grid with unequal column widths:

    ```rust
    #[test]
    fn snapshot_grid_unequal() {
        let mut grid = GridLayout::new(1, 2);
        grid = grid.col_width(0, 2.0).col_width(1, 1.0); // left panel twice as wide
        grid.set(0, 0, Figure::new(400, 300).add(LineMark::new(vec![0.,1.,2.], vec![0.,2.,1.])));
        grid.set(0, 1, Figure::new(200, 300).add(PointMark::new(vec![0.5], vec![1.0])));
        insta::assert_binary_snapshot!(".png", grid.render_png(600, 300).unwrap());
    }
    ```

### Layer 4: FacetWrap and FacetGrid

- [ ] Create `FacetWrap` in `starsight-layer-4/src/facet.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct FacetWrap {
        column: String,             // column name to split by
        ncol: Option<usize>,        // number of columns (None = auto sqrt)
        scales: FacetScales,        // Free | FreeX | FreeY | Fixed
        label_position: FacetLabelPosition,  // Top (default) | Bottom
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[non_exhaustive]
    pub enum FacetScales {
        #[default]
        Fixed,       // all panels share same axes
        FreeX,       // each panel has independent x scale
        FreeY,       // each panel has independent y scale
        Free,        // fully independent scales
    }
    ```

- [ ] Implement FacetWrap:

    ```rust
    impl FacetWrap {
        pub fn split_data(&self, data: &DataFrame) -> Vec<(String, DataFrame)> {
            let col = data.column(&self.column).unwrap();
            let unique_vals: Vec<String> = col.unique().unwrap().str().unwrap()
                .into_no_null_iter().map(|s| s.to_string()).collect();
            unique_vals.iter().map(|val| {
                let mask = col.equal(val).unwrap();
                (val.clone(), data.filter(&mask).unwrap())
            }).collect()
        }
    
        pub fn layout(&self, n_panels: usize) -> (usize, usize) {
            let ncol = self.ncol.unwrap_or_else(|| (n_panels as f64).sqrt().ceil() as usize);
            let nrow = (n_panels + ncol - 1) / ncol;
            (nrow, ncol)
        }
    
        pub fn render(&self, figure: &Figure, backend: &mut dyn DrawBackend) -> Result<()> {
            let panels = self.split_data(figure.data());
            let (nrow, ncol) = self.layout(panels.len());
            let (w, h) = backend.dimensions();
            let cell_w = w as f32 / ncol as f32;
            let cell_h = h as f32 / nrow as f32;
            let title_h = 20.0; // pixels for facet label strip
    
            for (idx, (label, panel_data)) in panels.iter().enumerate() {
                let row = idx / ncol;
                let col = idx % ncol;
                let x = col as f32 * cell_w;
                let y = row as f32 * cell_h;
    
                // Render facet title strip
                backend.draw_text(&label, Point::new(x + cell_w / 2.0, y + 10.0), 11.0, Color::BLACK)?;
    
                // Render chart in remaining space
                let chart_rect = Rect::from_xywh(x, y + title_h, cell_w, cell_h - title_h);
                backend.set_clip(Some(chart_rect))?;
                figure.render_with_data(backend, &panel_data, chart_rect)?;
                backend.set_clip(None)?;
            }
            Ok(())
        }
    }
    ```

- [ ] Implement FacetGrid:

    ```rust
    impl FacetGrid {
        pub fn render(&self, figure: &Figure, backend: &mut dyn DrawBackend) -> Result<()> {
            let data = figure.data();
            let row_vals: Vec<String> = data.column(&self.row).unwrap().unique().unwrap()
                .str().unwrap().into_no_null_iter().map(|s| s.to_string()).collect();
            let col_vals: Vec<String> = data.column(&self.col).unwrap().unique().unwrap()
                .str().unwrap().into_no_null_iter().map(|s| s.to_string()).collect();
    
            let (w, h) = backend.dimensions();
            let cell_w = w as f32 / col_vals.len() as f32;
            let cell_h = h as f32 / row_vals.len() as f32;
    
            for (ri, rv) in row_vals.iter().enumerate() {
                for (ci, cv) in col_vals.iter().enumerate() {
                    let mask = data.column(&self.row).unwrap().equal(rv).unwrap()
                        & data.column(&self.col).unwrap().equal(cv).unwrap();
                    let panel_data = data.filter(&mask).unwrap();
                    let rect = Rect::from_xywh(ci as f32 * cell_w, ri as f32 * cell_h, cell_w, cell_h);
                    backend.set_clip(Some(rect))?;
                    figure.render_with_data(backend, &panel_data, rect)?;
                    backend.set_clip(None)?;
                }
            }
            Ok(())
        }
    }
    ```
- [ ] Create subplot matrix: row var determines rows, col var determines columns

    ```rust
    #[derive(Debug, Clone)]
    pub struct FacetGrid {
        row: String,                // row facet variable
        col: String,                // column facet variable
        scales: FacetScales,
        margin_titles: bool,        // row titles on right margin, col titles on top
    }
    ```

- [ ] Implement shared axes logic:

    ```rust
    fn render_shared_axes(&self, nrow: usize, ncol: usize, coord: &CartesianCoord,
                          backend: &mut dyn DrawBackend) -> Result<()> {
        match self.scales {
            FacetScales::Fixed => {
                // Compute single scale across all panels
                // Render x tick labels only on bottom row (row == nrow - 1)
                // Render y tick labels only on left column (col == 0)
                for row in 0..nrow {
                    for col in 0..ncol {
                        let show_x_ticks = row == nrow - 1;
                        let show_y_ticks = col == 0;
                        coord.render_axes(backend, show_x_ticks, show_y_ticks)?;
                    }
                }
            }
            FacetScales::FreeY => {
                // Independent y scale per panel, shared x
            }
            FacetScales::FreeX => {
                // Independent x scale per panel, shared y
            }
            FacetScales::Free => {
                // Fully independent scales
            }
        }
        Ok(())
    }
    ```

- [ ] Write snapshot test: facet wrap with 6 panels:

    ```rust
    #[test]
    fn snapshot_facet_wrap_6() {
        let fig = Figure::new(900, 600)
            .add(PointMark::new(x.clone(), y.clone()).color_by(&groups))
            .facet_wrap("group", Some(3)); // 3 columns
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: facet grid 2x3:

    ```rust
    #[test]
    fn snapshot_facet_grid() {
        let fig = Figure::new(900, 600)
            .add(PointMark::new(x.clone(), y.clone()))
            .facet_grid("row_var", "col_var");
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: facet wrap with free y:

    ```rust
    #[test]
    fn snapshot_facet_free_y() {
        let fig = Figure::new(900, 600)
            .add(LineMark::new(x.clone(), y.clone()))
            .facet_wrap("group", Some(2))
            .facet_scales(FacetScales::FreeY);
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### Layer 4: Legend

- [ ] Create `Legend` in `starsight-layer-4/src/legend.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct Legend {
        entries: Vec<LegendEntry>,
        position: LegendPosition,
        title: Option<String>,
    }
    
    #[derive(Debug, Clone)]
    pub struct LegendEntry {
        pub label: String,
        pub swatch: LegendSwatch,   // colored square, line segment, or circle
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    #[non_exhaustive]
    pub enum LegendPosition {
        #[default]
        TopRight,
        TopLeft,
        BottomRight,
        BottomLeft,
        OutsideRight,    // outside the plot area
        OutsideBottom,
    }
    ```

- [ ] Implement legend rendering:

    ```rust
    impl Legend {
        pub fn render(&self, backend: &mut dyn DrawBackend, plot_area: Rect) -> Result<()> {
            let swatch_size = 12.0f32;
            let padding = 8.0f32;
            let line_height = 18.0f32;
    
            // Measure widest label (approximate: chars * 0.6 * font_size)
            let max_label_w = self.entries.iter()
                .map(|e| e.label.len() as f32 * 7.0)
                .fold(0.0f32, f32::max);
            let box_w = padding * 2.0 + swatch_size + 6.0 + max_label_w;
            let box_h = padding * 2.0 + self.entries.len() as f32 * line_height;
    
            // Position
            let (bx, by) = match self.position {
                LegendPosition::TopRight => (plot_area.right - box_w - 8.0, plot_area.top + 8.0),
                LegendPosition::TopLeft => (plot_area.left + 8.0, plot_area.top + 8.0),
                LegendPosition::OutsideRight => (plot_area.right + 8.0, plot_area.top),
                _ => (plot_area.right - box_w - 8.0, plot_area.bottom - box_h - 8.0),
            };
    
            // Semi-transparent background
            let bg = Rect::from_xywh(bx, by, box_w, box_h);
            backend.fill_rect(bg, Color::WHITE.with_alpha(220))?;
    
            // Entries
            for (i, entry) in self.entries.iter().enumerate() {
                let y = by + padding + i as f32 * line_height;
                let sx = bx + padding;
                // Swatch (colored square)
                backend.fill_rect(Rect::from_xywh(sx, y, swatch_size, swatch_size), entry.color)?;
                // Label
                backend.draw_text(&entry.label, Point::new(sx + swatch_size + 6.0, y + 10.0), 12.0, Color::BLACK)?;
            }
            Ok(())
        }
    }
    ```

- [ ] Implement auto-generation: Figure creates Legend from marks' color/label mappings when more than one series is present

    ```rust
    fn auto_legend(&self) -> Option<Legend> {
    let entries: Vec<LegendEntry> = self.marks.iter()
        .filter_map(|m| m.label().map(|l| LegendEntry {
            label: l.to_string(), color: m.primary_color(), swatch: m.legend_swatch(),
        })).collect();
    (entries.len() > 1).then(|| Legend { entries, position: LegendPosition::TopRight, title: None })
    }
    ```
- [ ] Allow user to override position, hide legend, or customize entries

    ```rust
    impl Figure {
    pub fn legend(mut self, pos: LegendPosition) -> Self { self.legend_position = Some(pos); self }
    pub fn hide_legend(mut self) -> Self { self.show_legend = false; self }
    pub fn legend_entries(mut self, entries: Vec<LegendEntry>) -> Self { self.custom_legend = Some(entries); self }
    }
    ```
- [ ] Write snapshot test: legend inside top-right:

    ```rust
    #[test]
    fn snapshot_legend_inside() {
        let fig = Figure::new(800, 600)
            .add(LineMark::new(vec![0.,1.,2.], vec![0.,1.,2.]).color(Color::BLUE).label("Series A"))
            .add(LineMark::new(vec![0.,1.,2.], vec![2.,1.,0.]).color(Color::RED).label("Series B"))
            .legend(LegendPosition::TopRight);
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: legend outside right:

    ```rust
    #[test]
    fn snapshot_legend_outside() {
        let fig = Figure::new(800, 600)
            .add(LineMark::new(vec![0.,1.], vec![0.,1.]).label("A"))
            .add(LineMark::new(vec![0.,1.], vec![1.,0.]).label("B"))
            .legend(LegendPosition::OutsideRight);
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: legend with mixed entries:

    ```rust
    #[test]
    fn snapshot_legend_mixed() {
        let fig = Figure::new(800, 600)
            .add(LineMark::new(vec![0.,1.,2.], vec![0.,1.,2.]).label("Line"))
            .add(PointMark::new(vec![0.5, 1.5], vec![0.8, 1.2]).label("Scatter"))
            .legend(LegendPosition::TopRight);
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### Layer 4: Colorbar

- [ ] Create `Colorbar` in `starsight-layer-4/src/colorbar.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct Colorbar {
        colormap: ColormapRef,       // prismatica colormap reference
        domain: (f64, f64),          // data range
        label: Option<String>,
        orientation: Orientation,     // Vertical (default) | Horizontal
        width: f32,                  // bar width in pixels (default 20)
        tick_count: usize,           // target number of ticks (default 5)
    }
    ```

- [ ] Implement colorbar rendering:

    ```rust
    impl Colorbar {
        pub fn render(&self, backend: &mut dyn DrawBackend, plot_area: Rect) -> Result<()> {
            let n_rects = 256;
            let bar_x = plot_area.right + 10.0;
            let bar_y = plot_area.top;
            let bar_h = plot_area.height();
            let rect_h = bar_h / n_rects as f32;
    
            // Gradient strip
            for i in 0..n_rects {
                let t = i as f32 / (n_rects - 1) as f32;
                let color: Color = self.colormap.eval(1.0 - t).into(); // top = max
                let rect = Rect::from_xywh(bar_x, bar_y + i as f32 * rect_h, self.width, rect_h);
                backend.fill_rect(rect, color)?;
            }
    
            // Tick marks and labels
            let ticks = wilkinson_extended(self.domain.0, self.domain.1, self.tick_count);
            for &tick_val in &ticks {
                let t = ((tick_val - self.domain.0) / (self.domain.1 - self.domain.0)) as f32;
                let y = bar_y + (1.0 - t) * bar_h;
                let label = format!("{:.1}", tick_val);
                draw_hline(backend, y, bar_x, bar_x + self.width, Color::BLACK, 1.0)?;
                backend.draw_text(&label, Point::new(bar_x + self.width + 4.0, y + 4.0), 10.0, Color::BLACK)?;
            }
            Ok(())
        }
    }
    ```

- [ ] Write snapshot test: vertical colorbar:

    ```rust
    #[test]
    fn snapshot_colorbar_vertical() {
        let fig = Figure::new(700, 500)
            .add(HeatmapMark::new(vec![vec![0.,1.,2.], vec![3.,4.,5.]]))
            .colorbar(Colorbar::new(prismatica::crameri::BATLOW, (0.0, 5.0)));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: horizontal colorbar:

    ```rust
    #[test]
    fn snapshot_colorbar_horizontal() {
        let fig = Figure::new(700, 500)
            .add(HeatmapMark::new(vec![vec![-2., 0., 2.], vec![1., -1., 0.]]))
            .colorbar(Colorbar::new(prismatica::crameri::VIK, (-2.0, 2.0)).horizontal());
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

---

## 0.5.0 — Scale infrastructure

Exit criteria: all scale types render correctly. Tick locator and formatter traits enable custom tick logic. Log and datetime scales produce correct tick positions.

### LogScale and SymlogScale

- [ ] Implement `LogScale`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct LogScale {
        domain: (f64, f64),
        range: (f64, f64),
    }
    
    impl LogScale {
        pub fn new(domain: (f64, f64), range: (f64, f64)) -> Result<Self> {
            if domain.0 <= 0.0 || domain.1 <= 0.0 {
                return Err(StarsightError::Scale(
                    format!("LogScale domain must be strictly positive, got ({}, {})", domain.0, domain.1)
                ));
            }
            Ok(Self { domain, range })
        }
    }
    
    impl Scale for LogScale {
        fn map(&self, val: f64) -> f64 {
            let log_val = val.max(f64::MIN_POSITIVE).log10();
            let log_min = self.domain.0.log10();
            let log_max = self.domain.1.log10();
            let t = (log_val - log_min) / (log_max - log_min);
            self.range.0 + t * (self.range.1 - self.range.0)
        }
    
        fn inverse(&self, px: f64) -> f64 {
            let log_min = self.domain.0.log10();
            let log_max = self.domain.1.log10();
            let t = (px - self.range.0) / (self.range.1 - self.range.0);
            10f64.powf(log_min + t * (log_max - log_min))
        }
    }
    ```
- [ ] Implement `LogLocator`: ticks at powers of 10 and sub-ticks at 2 and 5

    ```rust
    impl Scale for LogScale {
        fn map(&self, val: f64) -> f64 {
            let log_val = val.log10();
            let log_min = self.domain.0.log10();
            let log_max = self.domain.1.log10();
            (log_val - log_min) / (log_max - log_min) * (self.range.1 - self.range.0) + self.range.0
        }
    }
    ```

- [ ] Implement `SymlogScale`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct SymlogScale {
        domain: (f64, f64),
        range: (f64, f64),
        threshold: f64,  // default 1.0
    }
    
    impl Scale for SymlogScale {
        fn map(&self, val: f64) -> f64 {
            let t = symlog_transform(val, self.threshold);
            let t_min = symlog_transform(self.domain.0, self.threshold);
            let t_max = symlog_transform(self.domain.1, self.threshold);
            self.range.0 + (t - t_min) / (t_max - t_min) * (self.range.1 - self.range.0)
        }
    }
    
    fn symlog_transform(x: f64, threshold: f64) -> f64 {
        x.signum() * (1.0 + x.abs() / threshold).log10()
    }
    ```

- [ ] `SymlogScale` uses `sign(x) * log10(1 + |x| / threshold)` with a configurable linear threshold near zero.

    ```rust
    // Already implemented above in the SymlogScale section.
    // This checkbox is a duplicate — verify the implementation matches.
    ```

### DateTimeScale

- [ ] Create `DateTimeScale` in `starsight-layer-2/src/scale/datetime.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct DateTimeScale {
        domain: (f64, f64),          // epoch seconds
        range: (f64, f64),           // pixel range
        granularity: Option<TimeGranularity>,  // None = auto-detect
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[non_exhaustive]
    pub enum TimeGranularity {
        Second, Minute, Hour, Day, Week, Month, Quarter, Year,
    }
    ```

- [ ] Implement auto-detection: compute span in seconds, select granularity by threshold

    ```rust
    fn auto_granularity(span_seconds: f64) -> TimeGranularity {
    if span_seconds > 4.0 * 365.25 * 86400.0 { TimeGranularity::Year }
    else if span_seconds > 180.0 * 86400.0 { TimeGranularity::Month }
    else if span_seconds > 14.0 * 86400.0 { TimeGranularity::Week }
    else if span_seconds > 2.0 * 86400.0 { TimeGranularity::Day }
    else if span_seconds > 4.0 * 3600.0 { TimeGranularity::Hour }
    else if span_seconds > 4.0 * 60.0 { TimeGranularity::Minute }
    else { TimeGranularity::Second }
    }
    ```

- [ ] Implement tick generation per granularity (Year → Jan 1, Month → 1st, Day → midnight, etc.)

    ```rust
    fn ticks_for_granularity(domain: (f64, f64), gran: TimeGranularity) -> Vec<f64> {
    match gran {
        TimeGranularity::Year => {
            let start_year = epoch_to_year(domain.0);
            let end_year = epoch_to_year(domain.1);
            (start_year..=end_year).map(|y| epoch_from_year(y)).filter(|&t| t >= domain.0 && t <= domain.1).collect()
        }
        TimeGranularity::Month => { /* ticks at 1st of each month */ todo!() }
        TimeGranularity::Day => { /* ticks at midnight */ todo!() }
        _ => { /* round to nearest hour/minute/second */ todo!() }
    }
    }
    ```
- [ ] Skip ticks that fall outside the domain

    ```rust
    // Applied in ticks_for_granularity via:
    .filter(|&t| t >= domain.0 && t <= domain.1).collect()
    ```

- [ ] Implement label formatting per granularity (Year → "2024", Month → "Jan", Day → "Jan 15", etc.)

    ```rust
    fn format_tick(epoch: f64, gran: TimeGranularity) -> String {
    let (y, m, d, h, min, s) = epoch_to_components(epoch);
    match gran {
        TimeGranularity::Year => format!("{y}"),
        TimeGranularity::Month => format!("{}", MONTH_ABBR[m as usize - 1]),
        TimeGranularity::Day => format!("{} {d}", MONTH_ABBR[m as usize - 1]),
        TimeGranularity::Hour => format!("{h:02}:00"),
        TimeGranularity::Minute => format!("{h:02}:{min:02}"),
        TimeGranularity::Second => format!("{h:02}:{min:02}:{s:02}"),
        _ => format!("{y}-{m:02}-{d:02}"),
    }
    }
    const MONTH_ABBR: &[&str] = &["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    ```

- [ ] Write snapshot test: 10-year time series:

    ```rust
    #[test]
    fn snapshot_datetime_years() {
        let x: Vec<f64> = (2015..2025).map(|y| epoch_from_year(y)).collect();
        let y: Vec<f64> = x.iter().enumerate().map(|(i, _)| (i as f64).sin()).collect();
        let fig = Figure::new(800, 400)
            .add(LineMark::new(x, y))
            .x_scale(DateTimeScale::auto());
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: 6-month time series:

    ```rust
    #[test]
    fn snapshot_datetime_months() {
        let start = epoch_from_date(2024, 1, 1);
        let x: Vec<f64> = (0..180).map(|d| start + d as f64 * 86400.0).collect();
        let y: Vec<f64> = x.iter().map(|t| (t / 1e6).sin()).collect();
        let fig = Figure::new(800, 400).add(LineMark::new(x, y)).x_scale(DateTimeScale::auto());
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: 48-hour time series:

    ```rust
    #[test]
    fn snapshot_datetime_hours() {
        let start = epoch_from_date(2024, 6, 15);
        let x: Vec<f64> = (0..48).map(|h| start + h as f64 * 3600.0).collect();
        let y: Vec<f64> = x.iter().map(|t| (t / 3600.0).sin()).collect();
        let fig = Figure::new(800, 400).add(LineMark::new(x, y)).x_scale(DateTimeScale::auto());
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### BandScale and CategoricalScale

- [ ] Create `BandScale` in `starsight-layer-2/src/scale/band.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct BandScale {
        domain: Vec<String>,          // category labels in order
        range: (f64, f64),            // pixel range
        inner_padding: f64,           // gap between bands as fraction of step (default 0.1)
        outer_padding: f64,           // gap at edges as fraction of step (default 0.05)
    }
    
    impl BandScale {
        pub fn bandwidth(&self) -> f64 {
            let n = self.domain.len() as f64;
            let total = self.range.1 - self.range.0;
            total / (n + (n - 1.0) * self.inner_padding + 2.0 * self.outer_padding)
        }
    
        pub fn map(&self, label: &str) -> Option<f64> {
            let idx = self.domain.iter().position(|l| l == label)?;
            let bw = self.bandwidth();
            let step = bw * (1.0 + self.inner_padding);
            Some(self.range.0 + self.outer_padding * bw + idx as f64 * step + bw / 2.0)
        }
    }
    ```

- [ ] Implement `CategoricalScale`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct CategoricalScale {
        domain: Vec<String>,
        range: (f64, f64),
    }
    
    impl CategoricalScale {
        pub fn new(categories: Vec<String>, range: (f64, f64)) -> Self {
            Self { domain: categories, range }
        }
        pub fn map(&self, label: &str) -> Option<f64> {
            let idx = self.domain.iter().position(|l| l == label)?;
            let n = self.domain.len();
            if n == 1 { return Some((self.range.0 + self.range.1) / 2.0); }
            let step = (self.range.1 - self.range.0) / (n - 1) as f64;
            Some(self.range.0 + idx as f64 * step)
        }
    }
    ```

- [ ] Write property tests for BandScale:

    ```rust
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn band_positions_in_range(n in 1usize..20) {
            let cats: Vec<String> = (0..n).map(|i| format!("cat_{i}")).collect();
            let scale = BandScale::new(cats.clone(), (0.0, 800.0));
            for cat in &cats {
                let pos = scale.map(cat).unwrap();
                prop_assert!(pos >= 0.0 && pos <= 800.0, "pos {} out of range", pos);
            }
        }
        #[test]
        fn bandwidth_fits(n in 1usize..50) {
            let cats: Vec<String> = (0..n).map(|i| format!("c{i}")).collect();
            let scale = BandScale::new(cats.clone(), (0.0, 1000.0));
            let bw = scale.bandwidth();
            prop_assert!(bw * n as f64 <= 1000.0 + 1e-10);
        }
    }
    #[test]
    fn unknown_label_returns_none() {
        let scale = BandScale::new(vec!["A".into(), "B".into()], (0.0, 100.0));
        assert!(scale.map("Z").is_none());
    }
    ```

- [ ] Write snapshot test: bar chart using BandScale:

    ```rust
    #[test]
    fn snapshot_band_scale_bar() {
        let fig = Figure::new(600, 400).add(
            BarMark::new(vec!["Mon".into(),"Tue".into(),"Wed".into(),"Thu".into(),"Fri".into()],
                         vec![5.0, 8.0, 3.0, 9.0, 6.0])
        );
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: scatter with categorical x:

    ```rust
    #[test]
    fn snapshot_categorical_scatter() {
        let fig = Figure::new(600, 400).add(
            PointMark::new_categorical(
                vec!["A".into(), "A".into(), "B".into(), "B".into(), "C".into()],
                vec![1.0, 2.0, 1.5, 3.0, 2.5]
            )
        );
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### ColorScale

- [ ] Create `ColorScale` in `starsight-layer-2/src/scale/color.rs`:

    ```rust
    #[derive(Debug, Clone)]
    pub struct ColorScale {
        domain: (f64, f64),
        colormap: ColormapRef,       // reference to a prismatica colormap
        midpoint: Option<f64>,       // for diverging: the neutral center value
    }
    
    impl ColorScale {
        pub fn sequential(domain: (f64, f64), colormap: ColormapRef) -> Self { /* ... */ }
        pub fn diverging(domain: (f64, f64), colormap: ColormapRef, midpoint: f64) -> Self { /* ... */ }
    
        pub fn map(&self, val: f64) -> Color {
            let t = match self.midpoint {
                None => (val - self.domain.0) / (self.domain.1 - self.domain.0),
                Some(mid) => {
                    if val < mid { 0.5 * (val - self.domain.0) / (mid - self.domain.0) }
                    else { 0.5 + 0.5 * (val - mid) / (self.domain.1 - mid) }
                }
            };
            self.colormap.eval(t.clamp(0.0, 1.0) as f32)
        }
    }
    ```

- [ ] Implement diverging midpoint mapping:

    ```rust
    impl ColorScale {
        pub fn map(&self, val: f64) -> Color {
            let t = match self.midpoint {
                None => {
                    // Sequential: linear 0→1
                    ((val - self.domain.0) / (self.domain.1 - self.domain.0)).clamp(0.0, 1.0)
                }
                Some(mid) => {
                    // Diverging: below midpoint → 0.0-0.5, above → 0.5-1.0
                    if val <= mid {
                        0.5 * (val - self.domain.0) / (mid - self.domain.0)
                    } else {
                        0.5 + 0.5 * (val - mid) / (self.domain.1 - mid)
                    }
                }
            };
            self.colormap.eval(t.clamp(0.0, 1.0) as f32).into()
        }
    }
    ```

- [ ] Write snapshot test: heatmap with sequential ColorScale:

    ```rust
    #[test]
    fn snapshot_colorscale_seq() {
        let data = vec![vec![0.0, 0.25, 0.5], vec![0.75, 1.0, 0.5]];
        let fig = Figure::new(400, 300).add(
            HeatmapMark::new(data).color_scale(ColorScale::sequential((0.0, 1.0), prismatica::crameri::BATLOW))
        );
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: heatmap with diverging ColorScale:

    ```rust
    #[test]
    fn snapshot_colorscale_div() {
        let data = vec![vec![-3.0, -1.0, 0.0], vec![1.0, 3.0, -2.0]];
        let fig = Figure::new(400, 300).add(
            HeatmapMark::new(data).color_scale(ColorScale::diverging((-3.0, 3.0), prismatica::crameri::VIK, 0.0))
        );
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### TickLocator and TickFormatter traits

- [ ] Define traits in `starsight-layer-2/src/tick/mod.rs`:

    ```rust
    pub trait TickLocator {
        fn locate(&self, domain: (f64, f64), target_count: usize) -> Vec<f64>;
    }
    
    pub trait TickFormatter {
        fn format(&self, value: f64) -> String;
    }
    ```

- [ ] Implement `WilkinsonLocator` and `AutoFormatter`:

    ```rust
    pub struct WilkinsonLocator;
    
    impl TickLocator for WilkinsonLocator {
        fn locate(&self, domain: (f64, f64), target_count: usize) -> Vec<f64> {
            wilkinson_extended(domain.0, domain.1, target_count)
        }
    }
    
    pub struct AutoFormatter {
        precision: Option<usize>,
    }
    
    impl TickFormatter for AutoFormatter {
        fn format(&self, value: f64) -> String {
            let p = self.precision.unwrap_or_else(|| {
                // Auto-detect: use enough decimals to distinguish adjacent ticks
                if value == 0.0 { return 0; }
                let abs = value.abs();
                if abs >= 1.0 { 0 } else { (-abs.log10()).ceil() as usize + 1 }
            });
            format!("{:.prec$}", value, prec = p)
        }
    }
    ```

- [ ] Built-in locators: `LogLocator` (ticks at powers of 10 and optionally at 2 and 5 sub-ticks), `DateLocator` (ticks at time boundaries based on granularity), `FixedLocator` (user-specified positions).

    ```rust
    pub struct LogLocator { pub sub_ticks: bool }
    impl TickLocator for LogLocator {
    fn locate(&self, domain: (f64, f64), _target: usize) -> Vec<f64> {
        let start = domain.0.max(f64::MIN_POSITIVE).log10().floor() as i32;
        let end = domain.1.log10().ceil() as i32;
        let mut ticks = Vec::new();
        for exp in start..=end {
            let base = 10f64.powi(exp);
            ticks.push(base);
            if self.sub_ticks {
                for &sub in &[2.0, 5.0] { ticks.push(base * sub); }
            }
        }
        ticks.into_iter().filter(|&t| t >= domain.0 && t <= domain.1).collect()
    }
    }
    
    pub struct DateLocator;
    impl TickLocator for DateLocator {
    fn locate(&self, domain: (f64, f64), target: usize) -> Vec<f64> {
        let gran = auto_granularity(domain.1 - domain.0);
        ticks_for_granularity(domain, gran)
    }
    }
    
    pub struct FixedLocator(pub Vec<f64>);
    impl TickLocator for FixedLocator {
    fn locate(&self, _domain: (f64, f64), _target: usize) -> Vec<f64> { self.0.clone() }
    }
    ```

- [ ] Implement built-in formatters:

    ```rust
    pub struct PercentFormatter;
    impl TickFormatter for PercentFormatter {
        fn format(&self, value: f64) -> String { format!("{:.0}%", value * 100.0) }
    }
    
    pub struct SIFormatter;
    impl TickFormatter for SIFormatter {
        fn format(&self, value: f64) -> String {
            let abs = value.abs();
            let (scaled, suffix) = if abs >= 1e12 { (value / 1e12, "T") }
                else if abs >= 1e9 { (value / 1e9, "G") }
                else if abs >= 1e6 { (value / 1e6, "M") }
                else if abs >= 1e3 { (value / 1e3, "k") }
                else if abs >= 1.0 { (value, "") }
                else if abs >= 1e-3 { (value * 1e3, "m") }
                else if abs >= 1e-6 { (value * 1e6, "μ") }
                else { (value * 1e9, "n") };
            format!("{:.1}{}", scaled, suffix)
        }
    }
    
    pub struct DateFormatter;
    impl TickFormatter for DateFormatter {
        fn format(&self, epoch_secs: f64) -> String {
            // Convert epoch to human-readable — use chrono or manual calculation
            todo!("date formatting")
        }
    }
    
    pub struct FixedFormatter(pub Vec<String>);
    impl TickFormatter for FixedFormatter {
        fn format(&self, _value: f64) -> String {
            // Called with index; return self.0[index] — needs tick index context
            todo!("fixed formatter needs tick index")
        }
    }
    ```

- [ ] Ensure traits are object-safe: Axis holds `Box<dyn TickLocator>` and `Box<dyn TickFormatter>`

    ```rust
    pub struct Axis {
    pub locator: Box<dyn TickLocator>,
    pub formatter: Box<dyn TickFormatter>,
    pub label: Option<String>,
    pub visible: bool,
    }
    
    impl Default for Axis {
    fn default() -> Self {
        Self {
            locator: Box::new(WilkinsonLocator),
            formatter: Box::new(AutoFormatter { precision: None }),
            label: None,
            visible: true,
        }
    }
    }
    ```

- [ ] Write snapshot test: default Wilkinson ticks:

    ```rust
    #[test]
    fn snapshot_ticks_default() {
        let fig = Figure::new(800, 200)
            .add(LineMark::new(vec![0.0, 100.0], vec![0.0, 100.0]))
            .x_tick_locator(WilkinsonLocator::new());
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: log ticks:

    ```rust
    #[test]
    fn snapshot_ticks_log() {
        let x: Vec<f64> = (0..100).map(|i| 10f64.powf(i as f64 * 0.04)).collect();
        let y: Vec<f64> = x.iter().map(|v| v.log10()).collect();
        let fig = Figure::new(800, 400).add(LineMark::new(x, y)).x_scale(LogScale::new((1.0, 10000.0)));
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: percentage ticks:

    ```rust
    #[test]
    fn snapshot_ticks_percent() {
        let fig = Figure::new(600, 400)
            .add(BarMark::new(vec!["A".into(),"B".into()], vec![0.35, 0.65]))
            .y_tick_formatter(PercentFormatter);
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```
- [ ] Write snapshot test: SI-formatted ticks:

    ```rust
    #[test]
    fn snapshot_ticks_si() {
        let fig = Figure::new(600, 400)
            .add(LineMark::new(vec![0., 1., 2., 3.], vec![0., 1e3, 1e6, 1e9]))
            .y_tick_formatter(SIFormatter);
        insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
    }
    ```

### 0.6.0 — GPU and interactivity

Exit criteria: charts render in a native window with hover tooltips. GPU backend produces identical output to CPU backend.

- [ ] Create `WgpuBackend` struct in `starsight-layer-1/src/backend/wgpu/mod.rs`

    ```rust
    pub struct WgpuBackend {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: Option<wgpu::Surface<'static>>,
    texture: wgpu::Texture,
    pipeline: wgpu::RenderPipeline,
    clip_mask: Option<Rect>,
    }
    ```
- [ ] Initialize wgpu device, queue, and render pipeline

    ```rust
    impl WgpuBackend {
    pub async fn new(width: u32, height: u32) -> Result<Self> {
        let instance = wgpu::Instance::default();
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await
            .ok_or_else(|| StarsightError::Render("No GPU adapter".into()))?;
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await
            .map_err(|e| StarsightError::Render(e.to_string()))?;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            ..Default::default()
        });
        // Build pipeline...
        todo!()
    }
    }
    ```
- [ ] Write vertex shader: transform data coordinates to clip space

    ```wgsl
    // shader.wgsl
    struct VertexInput { @location(0) position: vec2<f32>, @location(1) color: vec4<f32> }
    struct VertexOutput { @builtin(position) clip_position: vec4<f32>, @location(0) color: vec4<f32> }
    
    @vertex fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(in.position * 2.0 - 1.0, 0.0, 1.0); // NDC
    out.color = in.color;
    return out;
    }
    ```
- [ ] Write fragment shader: handle solid colors, gradients, and colormaps

    ```wgsl
    @fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
    }
    ```
- [ ] Implement Path-to-triangle tessellation (via lyon or manual ear-clipping)

    ```rust
    use lyon::tessellation::{VertexBuffers, FillTessellator, FillOptions};
    use lyon::path::Path as LyonPath;
    
    fn tessellate_path(commands: &[PathCommand]) -> VertexBuffers<[f32; 2], u16> {
    let mut buffers = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    let mut builder = LyonPath::builder();
    for cmd in commands {
        match cmd {
            PathCommand::MoveTo(p) => { builder.begin(lyon::math::point(p.x, p.y)); }
            PathCommand::LineTo(p) => { builder.line_to(lyon::math::point(p.x, p.y)); }
            PathCommand::Close => { builder.close(); }
            _ => {}
        }
    }
    let path = builder.build();
    tessellator.tessellate_path(&path, &FillOptions::default(), &mut buffers).unwrap();
    buffers
    }
    ```
- [ ] Implement `DrawBackend` for `WgpuBackend`

    ```rust
    impl DrawBackend for WgpuBackend {
    fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
        let vertices = rect_to_triangles(rect, color);
        self.upload_and_draw(&vertices)
    }
    fn draw_path(&mut self, path: &Path, style: &PathStyle) -> Result<()> {
        let tris = tessellate_path(&path.commands());
        self.upload_and_draw(&tris)
    }
    fn dimensions(&self) -> (u32, u32) { (self.texture.width(), self.texture.height()) }
    fn save_png(&self, path: &std::path::Path) -> Result<()> { self.readback_and_encode(path) }
    fn save_svg(&self, _: &std::path::Path) -> Result<()> { Err(StarsightError::Export("GPU cannot SVG".into())) }
    }
    ```
- [ ] Verify GPU output is visually identical to CPU backend

    ```rust
    pub struct WgpuBackend {
        device: wgpu::Device,
        queue: wgpu::Queue,
        surface: Option<wgpu::Surface<'static>>,  // None for headless
        texture: wgpu::Texture,                     // render target
        pipeline: wgpu::RenderPipeline,
    }
    ```

- [ ] Implement GPU texture readback: copy texture to CPU buffer

    ```rust
    fn readback(&self) -> Result<Vec<u8>> {
    let (w, h) = self.dimensions();
    let buf_size = (w * h * 4) as u64;
    let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
        size: buf_size, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false, label: None,
    });
    let mut encoder = self.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        self.texture.as_image_copy(),
        wgpu::ImageCopyBuffer { buffer: &buffer, layout: wgpu::ImageDataLayout {
            offset: 0, bytes_per_row: Some(w * 4), rows_per_image: Some(h),
        }},
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    self.queue.submit([encoder.finish()]);
    // Map and read buffer...
    todo!()
    }
    ```
- [ ] Implement `save_png()` for WgpuBackend using readback buffer

    ```rust
    fn readback_and_encode(&self, path: &std::path::Path) -> Result<()> {
    let pixels = self.readback()?;
    let (w, h) = self.dimensions();
    let pixmap = Pixmap::from_vec(pixels, tiny_skia::IntSize::from_wh(w, h).unwrap())
        .ok_or_else(|| StarsightError::Export("Invalid pixel data".into()))?;
    pixmap.save_png(path).map_err(|e| StarsightError::Export(e.to_string()))
    }
    ```

- [ ] Create `InteractiveWindow` struct in `starsight-layer-6/src/window.rs`

    ```rust
    pub struct InteractiveWindow {
        event_loop: EventLoop<()>,
        window: Window,
        backend: WgpuBackend,
        figure: Figure,
        hover_state: Option<HoverInfo>,
        zoom: ZoomState,
    }
    ```

- [ ] Implement winit event loop: create window, initialize WgpuBackend, enter render loop

    ```rust
    pub fn show(figure: Figure) -> Result<()> {
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let window = winit::window::WindowBuilder::new()
        .with_title("starsight").with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop).unwrap();
    let backend = pollster::block_on(WgpuBackend::new_with_surface(&window))?;
    let mut state = InteractiveState { figure, backend, hover: None, zoom: ZoomState::default() };
    event_loop.run(move |event, target| {
        match event {
            winit::event::Event::WindowEvent { event, .. } => state.handle_event(event, target),
            _ => {}
        }
    }).unwrap();
    Ok(())
    }
    ```
- [ ] Implement frame lifecycle: check input events → update state → re-render if dirty

    ```rust
    fn handle_event(&mut self, event: WindowEvent, target: &EventLoopWindowTarget<()>) {
    match event {
        WindowEvent::RedrawRequested => { self.render(); }
        WindowEvent::CursorMoved { position, .. } => { self.update_hover(position); self.request_redraw(); }
        WindowEvent::MouseWheel { delta, .. } => { self.handle_zoom(delta); self.request_redraw(); }
        WindowEvent::CloseRequested => { target.exit(); }
        _ => {}
    }
    }
    ```
- [ ] Implement hit-testing: spatial index for mark bounding boxes (grid for scatter, sequential for lines)

    ```rust
    fn hit_test(&self, px: f64, py: f64) -> Option<HoverInfo> {
    let point = Point::new(px as f32, py as f32);
    for mark in &self.figure.marks {
        if let Some(info) = mark.hit_test(point, &self.coord) {
            return Some(info);
        }
    }
    None
    }
    
    pub struct HoverInfo { pub label: String, pub x_val: f64, pub y_val: f64, pub position: Point }
    ```
- [ ] Implement hover tooltips: on mouse move, hit-test marks, render tooltip box near cursor with data values

    ```rust
    fn render_tooltip(&self, info: &HoverInfo, backend: &mut dyn DrawBackend) -> Result<()> {
    let text = format!("{}: ({:.2}, {:.2})", info.label, info.x_val, info.y_val);
    let w = text.len() as f32 * 7.0 + 16.0;
    let h = 24.0;
    let x = info.position.x + 12.0;
    let y = info.position.y - h - 4.0;
    backend.fill_rect(Rect::from_xywh(x, y, w, h), Color::from_hex(0x333333).with_alpha(230))?;
    backend.draw_text(&text, Point::new(x + 8.0, y + 16.0), 12.0, Color::WHITE)?;
    Ok(())
    }
    ```
- [ ] Implement box zoom: click-drag draws selection rect, on release update scale domains, double-click resets

    ```rust
    fn handle_box_zoom(&mut self, start: Point, end: Point) {
    let x_min = self.coord.inverse_x(start.x.min(end.x) as f64);
    let x_max = self.coord.inverse_x(start.x.max(end.x) as f64);
    let y_min = self.coord.inverse_y(start.y.max(end.y) as f64); // y inverted
    let y_max = self.coord.inverse_y(start.y.min(end.y) as f64);
    self.figure.x_domain(x_min, x_max).y_domain(y_min, y_max);
    }
    fn handle_double_click(&mut self) { self.figure.reset_domains(); }
    ```
- [ ] Implement wheel zoom: scroll scales both axes around cursor, shift = x only, ctrl = y only

    ```rust
    fn handle_zoom(&mut self, delta: MouseScrollDelta) {
    let factor = match delta {
        MouseScrollDelta::LineDelta(_, y) => 1.0 - y as f64 * 0.1,
        MouseScrollDelta::PixelDelta(p) => 1.0 - p.y * 0.001,
    };
    let cx = self.coord.inverse_x(self.cursor.x as f64);
    let cy = self.coord.inverse_y(self.cursor.y as f64);
    self.figure.zoom_around(cx, cy, factor, factor);
    }
    ```
- [ ] Implement pan: middle-click-drag or shift-click-drag translates both axes

    ```rust
    fn handle_pan(&mut self, dx: f64, dy: f64) {
    let data_dx = self.coord.inverse_dx(dx);
    let data_dy = self.coord.inverse_dy(dy);
    self.figure.pan(-data_dx, -data_dy); // negative because dragging moves viewport
    }
    ```
- [ ] Implement legend toggle: click legend entry to hide/show corresponding mark, dim hidden entries

    ```rust
    fn handle_legend_click(&mut self, click_pos: Point) {
    if let Some(idx) = self.legend_hit_test(click_pos) {
        self.figure.marks[idx].visible = !self.figure.marks[idx].visible;
    }
    }
    ```
- [ ] Implement `Figure::push_data(series_id, new_points)` for streaming data

    ```rust
    impl Figure {
    pub fn push_data(&mut self, series_id: usize, new_x: &[f64], new_y: &[f64]) {
        if let Some(mark) = self.marks.get_mut(series_id) {
            mark.extend_data(new_x, new_y);
            if let Some(window) = self.rolling_window {
                mark.trim_to_last(window);
            }
        }
    }
    }
    ```
- [ ] Implement rolling window: ring buffer for O(1) append and trim, re-render on each push

    ```rust
    pub struct RingBuffer<T> {
    data: Vec<T>,
    start: usize,
    len: usize,
    }
    impl<T: Clone> RingBuffer<T> {
    pub fn push(&mut self, item: T) {
        let idx = (self.start + self.len) % self.data.len();
        self.data[idx] = item;
        if self.len < self.data.len() { self.len += 1; }
        else { self.start = (self.start + 1) % self.data.len(); }
    }
    pub fn as_slice(&self) -> (&[T], &[T]) {
        let end = self.start + self.len;
        if end <= self.data.len() { (&self.data[self.start..end], &[]) }
        else { (&self.data[self.start..], &self.data[..end % self.data.len()]) }
    }
    }
    ```
- [ ] Write snapshot test: static render through WgpuBackend matches SkiaBackend output

    ```rust
    #[test]
    fn gpu_matches_cpu() {
    let fig = Figure::new(400, 300).add(LineMark::new(vec![0.,1.,2.], vec![0.,1.,0.5]));
    let cpu_bytes = fig.render_png_with(SkiaBackend::new(400, 300).unwrap()).unwrap();
    let gpu_bytes = pollster::block_on(async {
        fig.render_png_with(WgpuBackend::new(400, 300).await.unwrap()).unwrap()
    });
    assert_eq!(cpu_bytes.len(), gpu_bytes.len(), "PNG sizes differ");
    }
    ```
- [ ] Write integration test: window opens, renders chart, closes without panic

    ```rust
    #[test]
    #[ignore] // requires display server
    fn window_opens_and_closes() {
    let fig = Figure::new(400, 300).add(LineMark::new(vec![0.,1.], vec![0.,1.]));
    // Spawn window in a thread, close after 100ms
    std::thread::spawn(|| { std::thread::sleep(std::time::Duration::from_millis(100)); std::process::exit(0); });
    fig.show().unwrap();
    }
    ```

### 0.7.0 — 3D visualization

Exit criteria: 3D surface, scatter, wireframe, and line charts render with camera orbit.

- [ ] Create `Scene3D` in `starsight-layer-3/src/marks3d/mod.rs`. Uses nalgebra for camera transforms (projection matrix, view matrix, model matrix).

    ```rust
    pub struct Camera3D {
        position: nalgebra::Point3<f64>,
        target: nalgebra::Point3<f64>,
        up: nalgebra::Vector3<f64>,
        fov: f64,            // field of view in degrees
        near: f64,
        far: f64,
    }
    
    impl Camera3D {
        pub fn view_projection(&self, aspect: f64) -> nalgebra::Matrix4<f64> {
            let view = nalgebra::Isometry3::look_at_rh(&self.position, &self.target, &self.up);
            let proj = nalgebra::Perspective3::new(aspect, self.fov.to_radians(), self.near, self.far);
            proj.as_matrix() * view.to_homogeneous()
        }
    }
    ```

- [ ] Implement `Surface3DMark`: accept 2D grid of z-values

    ```rust
    #[derive(Debug, Clone)]
    pub struct Surface3DMark {
    pub z_grid: Vec<Vec<f64>>,  // row-major
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
    pub colormap: ColormapRef,
    }
    ```
- [ ] Tessellate grid into triangles

    ```rust
    fn tessellate_grid(z: &[Vec<f64>], rows: usize, cols: usize) -> Vec<Triangle3D> {
    let mut tris = Vec::new();
    for r in 0..rows-1 {
        for c in 0..cols-1 {
            let p00 = point3(c, r, z[r][c]);
            let p10 = point3(c+1, r, z[r][c+1]);
            let p01 = point3(c, r+1, z[r+1][c]);
            let p11 = point3(c+1, r+1, z[r+1][c+1]);
            tris.push(Triangle3D(p00, p10, p11));
            tris.push(Triangle3D(p00, p11, p01));
        }
    }
    tris
    }
    ```
- [ ] Project each vertex through camera view-projection matrix

    ```rust
    fn project(p: nalgebra::Point3<f64>, vp: &nalgebra::Matrix4<f64>, w: f64, h: f64) -> Point {
    let clip = vp * nalgebra::Vector4::new(p.x, p.y, p.z, 1.0);
    let ndc_x = clip.x / clip.w;
    let ndc_y = clip.y / clip.w;
    Point::new(((ndc_x + 1.0) * 0.5 * w) as f32, ((1.0 - ndc_y) * 0.5 * h) as f32)
    }
    ```
- [ ] Sort triangles by depth (painter's algorithm) for CPU backend

    ```rust
    fn sort_by_depth(tris: &mut [Triangle3D], camera_pos: &nalgebra::Point3<f64>) {
    tris.sort_by(|a, b| {
        let da = (a.centroid() - camera_pos).norm();
        let db = (b.centroid() - camera_pos).norm();
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal) // far first
    });
    }
    ```
- [ ] Color each face by z-value using prismatica colormap

    ```rust
    fn face_color(z_avg: f64, z_min: f64, z_max: f64, cmap: &dyn Colormap) -> Color {
    let t = ((z_avg - z_min) / (z_max - z_min)).clamp(0.0, 1.0) as f32;
    cmap.eval(t).into()
    }
    ```

- [ ] Implement `Scatter3DMark`: project 3D points to 2D screen

    ```rust
    impl Mark for Scatter3DMark {
    fn render(&self, coord: &dyn CoordSystem, backend: &mut dyn DrawBackend) -> Result<()> {
        let (w, h) = backend.dimensions();
        let vp = self.camera.view_projection(w as f64 / h as f64);
        let mut points: Vec<(Point, f64, Color)> = self.data.iter().map(|p| {
            let projected = project(p.pos, &vp, w as f64, h as f64);
            let depth = (p.pos - self.camera.position).norm();
            (projected, depth, p.color)
        }).collect();
        points.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap()); // back to front
        for (pt, depth, color) in &points {
            let r = (self.base_radius / depth.sqrt() * 10.0) as f32;
            draw_circle(backend, pt.x, pt.y, r, *color)?;
        }
        Ok(())
    }
    }
    ```
- [ ] Attenuate circle size by depth

    ```rust
    let radius = (self.base_radius / depth.sqrt() * 10.0).max(1.0) as f32;
    ```
- [ ] Sort back-to-front for correct overlap

    ```rust
    points.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));
    ```

- [ ] `Wireframe3DMark`: same as Surface3D but renders edges only (no filled faces).

    ```rust
    impl Mark for Wireframe3DMark {
    fn render(&self, coord: &dyn CoordSystem, backend: &mut dyn DrawBackend) -> Result<()> {
        // Same tessellation as Surface3D, but draw only edges (no fill)
        for tri in &self.triangles {
            let p0 = project(tri.0, &vp, w, h);
            let p1 = project(tri.1, &vp, w, h);
            let p2 = project(tri.2, &vp, w, h);
            draw_line(backend, p0, p1, self.color, 1.0)?;
            draw_line(backend, p1, p2, self.color, 1.0)?;
            draw_line(backend, p2, p0, self.color, 1.0)?;
        }
        Ok(())
    }
    }
    ```

- [ ] Implement `Line3DMark`: project 3D polyline to 2D

    ```rust
    impl Mark for Line3DMark {
    fn render(&self, coord: &dyn CoordSystem, backend: &mut dyn DrawBackend) -> Result<()> {
        let (w, h) = backend.dimensions();
        let vp = self.camera.view_projection(w as f64 / h as f64);
        let projected: Vec<Point> = self.points.iter().map(|p| project(*p, &vp, w as f64, h as f64)).collect();
        for pair in projected.windows(2) {
            draw_line(backend, pair[0], pair[1], self.color, self.width)?;
        }
        Ok(())
    }
    }
    ```

- [ ] Implement camera orbit: click-drag rotates via spherical coordinates

    ```rust
    fn orbit(&mut self, dx: f64, dy: f64) {
    self.theta += dx * 0.01;  // azimuth
    self.phi = (self.phi + dy * 0.01).clamp(0.1, std::f64::consts::PI - 0.1); // elevation
    let r = (self.camera.position - self.camera.target).norm();
    self.camera.position = self.camera.target + nalgebra::Vector3::new(
        r * self.phi.sin() * self.theta.cos(),
        r * self.phi.cos(),
        r * self.phi.sin() * self.theta.sin(),
    );
    }
    ```
- [ ] Implement camera zoom: scroll moves camera toward/away from target

    ```rust
    fn zoom_3d(&mut self, factor: f64) {
    let dir = self.camera.position - self.camera.target;
    self.camera.position = self.camera.target + dir * factor;
    }
    ```

### 0.8.0 — Terminal backend

Exit criteria: `figure.show_terminal()` renders a chart inline in a terminal emulator.

- [ ] Protocol detection cascade in `starsight-layer-7/src/terminal/mod.rs`:

    ```rust
    pub fn detect_protocol() -> TerminalProtocol {
        if kitty_supported() { return TerminalProtocol::Kitty; }
        if sixel_supported() { return TerminalProtocol::Sixel; }
        if iterm2_supported() { return TerminalProtocol::ITerm2; }
        TerminalProtocol::Braille  // universal fallback
    }
    ```

    Detection: query the terminal with escape sequences and parse the response. Kitty sends `\x1b_Gi=31;OK\x1b\\` in response to a query. Sixel support is indicated by Device Attributes response containing `4`. iTerm2 responds to `\x1b[>0q` with version info.

- [ ] Implement Kitty backend: encode PNG bytes with Kitty graphics protocol

    ```rust
    pub fn kitty_display(png_bytes: &[u8], width: u32, height: u32) {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(png_bytes);
    // Send in chunks of 4096
    for (i, chunk) in b64.as_bytes().chunks(4096).enumerate() {
        let m = if i == 0 { "1" } else { "0" };
        let cont = if chunk.len() == 4096 { ",m=1" } else { "" };
        print!("\x1b_Gf=100,a=T,t=d,s={width},v={height}{cont};{}\x1b\\", std::str::from_utf8(chunk).unwrap());
    }
    println!();
    }
    ```
- [ ] Send `\x1b_Gf=100,a=T,t=d,s=W,v=H;BASE64DATA\x1b\\` escape sequence

    ```rust
    // Covered in kitty_display() above — the escape sequence format:
    // \x1b_G = start Kitty graphics command
    // f=100 = PNG format
    // a=T = transmit and display
    // t=d = direct data transmission
    // s=W,v=H = image dimensions
    // ;BASE64 = the payload
    // \x1b\\ = end command
    ```

- [ ] Implement Sixel backend: convert Pixmap to Sixel using `icy_sixel`

    ```rust
    pub fn sixel_display(pixmap: &Pixmap) {
    let rgba = pixmap.data(); // premultiplied RGBA
    let (w, h) = (pixmap.width(), pixmap.height());
    let output = icy_sixel::sixel_string(
        rgba, w as i32, h as i32, icy_sixel::PixelFormat::RGBA8888,
        icy_sixel::DiffusionMethod::Stucki,
    ).unwrap();
    print!("{output}");
    }
    ```

- [ ] iTerm2 backend: encode the PNG bytes as base64, send `\x1b]1337;File=inline=1;size=N:BASE64DATA\x07`.

    ```rust
    pub fn iterm2_display(png_bytes: &[u8]) {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(png_bytes);
    print!("\x1b]1337;File=inline=1;size={};preserveAspectRatio=1:{b64}\x07", png_bytes.len());
    println!();
    }
    ```

- [ ] Implement Braille backend: map pixel brightness to Braille dot patterns (U+2800-U+28FF)

    ```rust
    pub fn braille_render(pixmap: &Pixmap) -> String {
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);
    let mut output = String::new();
    for row in (0..h).step_by(4) {
        for col in (0..w).step_by(2) {
            let mut dots = 0u8;
            // Braille pattern: 2 columns x 4 rows = 8 dots
            for dy in 0..4 { for dx in 0..2 {
                let y = row + dy; let x = col + dx;
                if y < h && x < w {
                    let px = pixmap.pixel(x as u32, y as u32).unwrap();
                    let bright = (px.red() as u16 + px.green() as u16 + px.blue() as u16) / 3;
                    if bright < 128 { dots |= 1 << (dy + dx * 4); } // dot mapping
                }
            }}
            output.push(char::from_u32(0x2800 + dots as u32).unwrap());
        }
        output.push('\n');
    }
    output
    }
    ```
- [ ] Each character represents 2x4 pixel grid for sub-character resolution

    ```rust
    // Braille dot mapping: each char U+2800-U+28FF encodes 8 dots in a 2x4 grid:
    // Col 0: bits 0,1,2,6  (top to bottom)
    // Col 1: bits 3,4,5,7  (top to bottom)
    // dot_index = dy + dx * 4 for dy in 0..4, dx in 0..2 (except row 3 uses bits 6,7)
    ```

- [ ] Implement half-block backend: use U+2580/U+2584 with ANSI 24-bit color

    ```rust
    pub fn halfblock_render(pixmap: &Pixmap) -> String {
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);
    let mut output = String::new();
    for row in (0..h).step_by(2) {
        for col in 0..w {
            let top = pixel_color(pixmap, col, row);
            let bot = if row + 1 < h { pixel_color(pixmap, col, row + 1) } else { top };
            // Upper half block with top color as fg, bottom color as bg
            output.push_str(&format!("\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m\u{2580}",
                top.0, top.1, top.2, bot.0, bot.1, bot.2));
        }
        output.push_str("\x1b[0m\n");
    }
    output
    }
    ```
- [ ] Each cell represents two vertically stacked pixels

    ```rust
    // U+2580 (upper half block) fills the top half of the character cell.
    // Set foreground = top pixel color, background = bottom pixel color.
    // Effective resolution: width * (height * 2) pixels.
    ```

- [ ] Implement `ratatui::Widget` for `StarsightWidget`

    ```rust
    pub struct StarsightWidget { pub figure: Figure, pub protocol: TerminalProtocol }
    
    impl ratatui::widgets::Widget for StarsightWidget {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let w = area.width as u32 * 2; // half-block doubles vertical resolution
        let h = area.height as u32 * 4; // braille quadruples it
        let mut backend = SkiaBackend::new(w, h).unwrap();
        self.figure.render_to(&mut backend).unwrap();
        match self.protocol {
            TerminalProtocol::Braille => {
                let text = braille_render(backend.pixmap());
                // Write to ratatui buffer...
            }
            _ => { /* Kitty/Sixel use escape sequences outside ratatui */ }
        }
    }
    }
    ```
- [ ] Render Figure into ratatui buffer area using detected protocol

    ```rust
    // Covered in StarsightWidget::render() above.
    // The protocol cascade: Kitty > Sixel > iTerm2 > HalfBlock > Braille
    ```
- [ ] Write integration test: StarsightWidget renders in ratatui app

    ```rust
    #[test]
    fn starsight_widget_renders() {
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(80, 24)).unwrap();
    let fig = Figure::new(160, 96).add(LineMark::new(vec![0.,1.,2.], vec![0.,1.,0.5]));
    terminal.draw(|frame| {
        frame.render_widget(StarsightWidget { figure: fig, protocol: TerminalProtocol::Braille }, frame.area());
    }).unwrap();
    }
    ```

### 0.9.0 — All chart types

Exit criteria: every chart type from the gallery reference (70 types) has an implementation and a snapshot test.

Each mark follows this template:

```rust
// Example: ErrorBarMark
#[derive(Debug, Clone)]
pub struct ErrorBarMark {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub y_err: Vec<f64>,       // mark-specific fields
    pub color: Color,
}

impl ErrorBarMark {
    pub fn new(x: Vec<f64>, y: Vec<f64>, y_err: Vec<f64>) -> Self {
        Self { x, y, y_err, color: Color::BLACK }
    }
    pub fn color(mut self, c: Color) -> Self { self.color = c; self }
}

impl Mark for ErrorBarMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        for i in 0..self.x.len() {
            let px = coord.map_x(self.x[i]) as f32;
            let py = coord.map_y(self.y[i]) as f32;
            let err_px = (coord.map_y(self.y[i] - self.y_err[i]) - coord.map_y(self.y[i] + self.y_err[i])).abs() as f32 / 2.0;
            draw_vline(backend, px, py - err_px, py + err_px, self.color, 1.5)?;
            draw_hline(backend, py - err_px, px - 3.0, px + 3.0, self.color, 1.5)?;
            draw_hline(backend, py + err_px, px - 3.0, px + 3.0, self.color, 1.5)?;
        }
        Ok(())
    }
}

#[test]
fn snapshot_error_bar() {
    let fig = Figure::new(600, 400)
        .add(PointMark::new(vec![1.,2.,3.], vec![10.,20.,15.]))
        .add(ErrorBarMark::new(vec![1.,2.,3.], vec![10.,20.,15.], vec![2.,3.,1.5]));
    insta::assert_binary_snapshot!(".png", fig.render_png().unwrap());
}
```

- [ ] ErrorBarMark: vertical and horizontal error bars (struct + Mark impl + snapshot test)

    ```rust
    pub struct ErrorBarMark { pub x: Vec<f64>, pub y: Vec<f64>, pub y_err: Vec<f64>, pub color: Color }
    // Render: vertical lines from y-err to y+err with horizontal caps
    ```
- [ ] StepMark: staircase interpolation between points (struct + Mark impl + snapshot test)

    ```rust
    pub struct StepMark { pub x: Vec<f64>, pub y: Vec<f64>, pub color: Color, pub where_: StepWhere }
    pub enum StepWhere { Pre, Post, Mid } // step before, after, or at midpoint
    // Render: horizontal line to next x, then vertical line to next y
    ```
- [ ] RidgelineMark: overlapping KDE distributions per category (struct + Mark impl + snapshot test)

    ```rust
    pub struct RidgelineMark { pub groups: Vec<(String, Vec<f64>)>, pub overlap: f64, pub color_cycle: Vec<Color> }
    // Render: stacked KDE curves with vertical offset per group
    ```
- [ ] StripMark: categorical scatter with random jitter (struct + Mark impl + snapshot test)

    ```rust
    pub struct StripMark { pub x: Vec<String>, pub y: Vec<f64>, pub jitter: f32, pub color: Color }
    // Render: for each point, add random horizontal offset within [-jitter, jitter]
    ```
- [ ] SwarmMark: beeswarm layout for categorical scatter (struct + Mark impl + snapshot test)

    ```rust
    pub struct SwarmMark { pub x: Vec<String>, pub y: Vec<f64>, pub radius: f32, pub color: Color }
    // Render: position dots to avoid overlap using simple collision detection
    ```
- [ ] RugMark: tick marks along an axis edge (struct + Mark impl + snapshot test)

    ```rust
    pub struct RugMark { pub values: Vec<f64>, pub axis: RugAxis, pub length: f32, pub color: Color }
    pub enum RugAxis { X, Y }
    // Render: short perpendicular lines at each value position along the axis
    ```
- [ ] LollipopMark: stem + dot (struct + Mark impl + snapshot test)

    ```rust
    pub struct LollipopMark { pub x: Vec<String>, pub y: Vec<f64>, pub color: Color, pub radius: f32 }
    // Render: vertical line from baseline to y, circle at (x, y)
    ```
- [ ] DumbbellMark: two dots connected by a line (struct + Mark impl + snapshot test)

    ```rust
    pub struct DumbbellMark { pub x: Vec<String>, pub y1: Vec<f64>, pub y2: Vec<f64>, pub color: Color }
    // Render: circle at y1, circle at y2, line connecting them
    ```
- [ ] WaterfallMark: cumulative bar chart with color for up/down (struct + Mark impl + snapshot test)

    ```rust
    pub struct WaterfallMark { pub labels: Vec<String>, pub values: Vec<f64>, pub up_color: Color, pub down_color: Color }
    // Render: bars where each starts at the previous cumulative sum, green for + red for -
    ```
- [ ] PolarMark: data on polar coordinates (struct + Mark impl + snapshot test)

    ```rust
    pub struct PolarMark { pub r: Vec<f64>, pub theta: Vec<f64>, pub color: Color }
    // Render: convert (r, theta) to Cartesian, draw on circular grid
    ```
- [ ] RadarMark: multi-variable radar/spider chart (struct + Mark impl + snapshot test)

    ```rust
    pub struct RadarMark { pub axes: Vec<String>, pub values: Vec<Vec<f64>>, pub colors: Vec<Color> }
    // Render: n axes radiating from center, polygon connecting values on each axis
    ```
- [ ] TreemapMark: nested rectangles via squarified algorithm (struct + Mark impl + snapshot test)

    ```rust
    pub struct TreemapMark { pub labels: Vec<String>, pub values: Vec<f64>, pub colors: Vec<Color> }
    // Render: squarified treemap layout — subdivide rectangle proportional to values
    ```
- [ ] SunburstMark: nested arcs for hierarchical data (struct + Mark impl + snapshot test)

    ```rust
    pub struct SunburstMark { pub root: TreeNode, pub colormap: ColormapRef }
    pub struct TreeNode { pub label: String, pub value: f64, pub children: Vec<TreeNode> }
    // Render: concentric rings of arcs, each ring = one tree depth level
    ```
- [ ] SankeyMark: flow diagram with node-link layout (struct + Mark impl + snapshot test)

    ```rust
    pub struct SankeyMark { pub nodes: Vec<String>, pub links: Vec<(usize, usize, f64)>, pub colors: Vec<Color> }
    // Render: nodes as vertical bars in columns, curved bands connecting them
    ```
- [ ] ChordMark: circular flow between groups (struct + Mark impl + snapshot test)

    ```rust
    pub struct ChordMark { pub matrix: Vec<Vec<f64>>, pub labels: Vec<String>, pub colors: Vec<Color> }
    // Render: circular layout with arcs per group, ribbons connecting groups proportional to flow
    ```
- [ ] NetworkMark: force-directed graph layout (struct + Mark impl + snapshot test)

    ```rust
    pub struct NetworkMark { pub nodes: Vec<String>, pub edges: Vec<(usize, usize)>, pub iterations: usize }
    // Render: force-directed placement (repulsion between nodes, attraction along edges)
    ```
- [ ] ParallelCoordsMark: multi-axis parallel lines (struct + Mark impl + snapshot test)

    ```rust
    pub struct ParallelCoordsMark { pub axes: Vec<String>, pub data: Vec<Vec<f64>>, pub colors: Vec<Color> }
    // Render: n vertical axes side by side, one polyline per data row crossing all axes
    ```
- [ ] StreamgraphMark: stacked area with baseline centering (struct + Mark impl + snapshot test)

    ```rust
    pub struct StreamgraphMark { pub x: Vec<f64>, pub layers: Vec<Vec<f64>>, pub colors: Vec<Color> }
    // Render: stacked areas centered around y=0 (wiggle baseline algorithm)
    ```
- [ ] SlopeMark: paired line segments between two positions (struct + Mark impl + snapshot test)

    ```rust
    pub struct SlopeMark { pub labels: Vec<String>, pub left: Vec<f64>, pub right: Vec<f64>, pub colors: Vec<Color> }
    // Render: two vertical axes (left, right), line segment from left value to right value per item
    ```
- [ ] FunnelMark: horizontally centered decreasing bars (struct + Mark impl + snapshot test)

    ```rust
    pub struct FunnelMark { pub labels: Vec<String>, pub values: Vec<f64>, pub colors: Vec<Color> }
    // Render: stacked horizontal bars centered on x-axis, widths proportional to values
    ```
- [ ] GaugeMark: arc-based indicator (struct + Mark impl + snapshot test)

    ```rust
    pub struct GaugeMark { pub value: f64, pub min: f64, pub max: f64, pub color: Color }
    // Render: semicircular arc from min to max, filled arc from min to value, needle at value
    ```
- [ ] Run `cargo xtask gallery` to generate all reference images

    ```bash
    cargo xtask gallery
    # Expected: gallery/*.png files generated for every example
    ls gallery/*.png | wc -l
    # Should match the number of examples
    ```
- [ ] Compare gallery output against GALLERY_REFERENCE.md entries

    ```bash
    cargo xtask gallery --compare
    # Compares gallery/*.png against gallery/reference/*.png
    # Reports visual differences
    ```

### 0.10.0 — Export and WASM

Exit criteria: charts export to PDF and render in a web browser via WASM.

- [ ] Create `PdfBackend` struct wrapping krilla's Document and Page

    ```rust
    pub struct PdfBackend {
    document: krilla::Document,
    page: krilla::Page,
    width: u32,
    height: u32,
    }
    ```
- [ ] Implement `DrawBackend::fill_rect()` for `PdfBackend`

    ```rust
    fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
    self.page.push();
    self.page.set_fill_color(krilla::color::rgb(color.r, color.g, color.b));
    self.page.fill_rect(rect.left, rect.top, rect.width(), rect.height());
    self.page.pop();
    Ok(())
    }
    ```
- [ ] Implement `DrawBackend::draw_path()` for `PdfBackend`

    ```rust
    fn draw_path(&mut self, path: &Path, style: &PathStyle) -> Result<()> {
    self.page.push();
    self.page.set_stroke_color(krilla::color::rgb(style.stroke_color.r, style.stroke_color.g, style.stroke_color.b));
    self.page.set_line_width(style.stroke_width);
    // Convert PathCommands to krilla path...
    self.page.stroke();
    self.page.pop();
    Ok(())
    }
    ```
- [ ] Implement `DrawBackend::draw_text()` for `PdfBackend`

    ```rust
    fn draw_text(&mut self, text: &str, pos: Point, font_size: f32, color: Color) -> Result<()> {
    self.page.push();
    self.page.set_fill_color(krilla::color::rgb(color.r, color.g, color.b));
    self.page.set_font(&self.font, font_size);
    self.page.fill_text(pos.x, pos.y, text);
    self.page.pop();
    Ok(())
    }
    ```
- [ ] Embed fonts as subsets in PDF output

    ```rust
    // krilla handles font subsetting automatically when you provide a font:
    let font_data = include_bytes!("fonts/Inter.ttf");
    let font = krilla::Font::new(font_data.to_vec(), 0).unwrap();
    // Only glyphs actually used in the document are embedded.
    ```
- [ ] Write snapshot test: PDF output opens correctly

    ```rust
    pub struct PdfBackend {
        document: krilla::Document,
        page: krilla::Page,
        font: krilla::Font,
    }
    
    impl DrawBackend for PdfBackend {
        fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
            self.page.fill_rect(rect.to_krilla(), color.to_krilla());
            Ok(())
        }
        // ...
    }
    ```

- [ ] Generate self-contained HTML file with embedded SVG chart

    ```rust
    pub fn export_html(figure: &Figure, path: &std::path::Path) -> Result<()> {
    let mut svg_backend = SvgBackend::new(800, 600);
    figure.render_to(&mut svg_backend)?;
    let svg = svg_backend.svg_string();
    let html = format!(r#"<!DOCTYPE html><html><head><meta charset="utf-8"><style>body{{margin:0;display:flex;justify-content:center;align-items:center;height:100vh}}</style></head><body>{svg}<script>{JS_RUNTIME}</script></body></html>"#);
    std::fs::write(path, html)?;
    Ok(())
    }
    ```
- [ ] Write JS runtime (< 5KB) for pan, zoom, and tooltip interactions

    ```javascript
    // Inline JS runtime (~3KB minified):
    const svg = document.querySelector("svg");
    let zoom = 1, panX = 0, panY = 0;
    svg.addEventListener("wheel", e => {
    e.preventDefault();
    zoom *= e.deltaY > 0 ? 0.9 : 1.1;
    svg.style.transform = `scale(${zoom}) translate(${panX}px, ${panY}px)`;
    });
    let dragging = false, lastX, lastY;
    svg.addEventListener("mousedown", e => { dragging = true; lastX = e.clientX; lastY = e.clientY; });
    svg.addEventListener("mousemove", e => {
    if (!dragging) return;
    panX += (e.clientX - lastX) / zoom; panY += (e.clientY - lastY) / zoom;
    lastX = e.clientX; lastY = e.clientY;
    svg.style.transform = `scale(${zoom}) translate(${panX}px, ${panY}px)`;
    });
    svg.addEventListener("mouseup", () => dragging = false);
    ```
- [ ] Ensure HTML file has zero external dependencies

    ```rust
    // The HTML export is fully self-contained:
    // - SVG is inline (not a linked file)
    // - JS is inline (not a CDN script)
    // - CSS is inline (not a stylesheet link)
    // - No fonts loaded from external URLs
    assert!(!html.contains("src=\"http"), "HTML must not reference external resources");
    ```

- [ ] Implement GIF frame rendering: render each frame as Pixmap

    ```rust
    pub fn export_gif<F>(path: &std::path::Path, frames: usize, fps: u16, mut render_frame: F) -> Result<()>
    where F: FnMut(usize) -> Figure {
    let mut encoder = gif::Encoder::new(std::fs::File::create(path)?, 800, 600, &[]).unwrap();
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();
    for i in 0..frames {
        let fig = render_frame(i);
        let mut backend = SkiaBackend::new(800, 600)?;
        fig.render_to(&mut backend)?;
        let rgba = backend.pixmap().data();
        let quantized = median_cut_quantize(rgba, 256);
        let mut frame = gif::Frame::from_rgba(800, 600, &mut quantized.pixels);
        frame.delay = 100 / fps;
        encoder.write_frame(&frame).unwrap();
    }
    Ok(())
    }
    ```
- [ ] Implement color quantization: median-cut algorithm to 256 colors

    ```rust
    fn median_cut_quantize(rgba: &[u8], max_colors: usize) -> QuantizedImage {
    // 1. Build a list of all unique colors
    // 2. Find the channel (R/G/B) with the widest range
    // 3. Sort by that channel and split at the median
    // 4. Recurse until max_colors buckets
    // 5. Map each pixel to the nearest bucket center
    todo!("median cut quantization")
    }
    ```
- [ ] Encode frames with `gif` crate

    ```rust
    use gif::{Encoder, Frame, Repeat};
    let mut encoder = Encoder::new(file, width, height, &global_palette)?;
    encoder.set_repeat(Repeat::Infinite)?;
    for frame_data in &frames {
    let frame = Frame::from_palette_pixels(width, height, frame_data, &palette, None);
    encoder.write_frame(&frame)?;
    }
    ```
- [ ] Accept frame iterator or per-frame closure API

    ```rust
    // Closure API (shown above):
    export_gif("anim.gif", 60, 30, |frame_idx| {
    let t = frame_idx as f64 / 60.0;
    Figure::new(800, 600).add(LineMark::new(x.clone(), y.iter().map(|v| v + t.sin()).collect()))
    });
    
    // Iterator API:
    export_gif_from_iter("anim.gif", 30, figures.into_iter());
    ```

- [ ] Compile starsight to `wasm32-unknown-unknown`

    ```bash
    cargo build --target wasm32-unknown-unknown -p starsight --no-default-features --features web
    ```
- [ ] Verify wgpu backend works via WebGPU in browsers

    ```rust
    // In wasm entry point:
    #[wasm_bindgen(start)]
    pub async fn main() {
    let canvas = web_sys::window().unwrap().document().unwrap()
        .get_element_by_id("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let backend = WgpuBackend::new_from_canvas(&canvas).await.unwrap();
    let fig = Figure::new(800, 600).add(LineMark::new(vec![0.,1.,2.], vec![0.,1.,0.5]));
    fig.render_to(&mut backend).unwrap();
    }
    ```
- [ ] Implement SVG fallback for browsers without WebGPU (via web-sys)

    ```rust
    fn render_to_svg_dom(figure: &Figure, container: &web_sys::Element) {
    let mut svg_backend = SvgBackend::new(800, 600);
    figure.render_to(&mut svg_backend).unwrap();
    container.set_inner_html(&svg_backend.svg_string());
    }
    ```
- [ ] Bundle font data for cosmic-text (no system font access in WASM)

    ```rust
    #[cfg(target_arch = "wasm32")]
    static BUNDLED_FONT: &[u8] = include_bytes!("../fonts/Inter-Regular.ttf");
    
    #[cfg(target_arch = "wasm32")]
    fn init_font_system() -> cosmic_text::FontSystem {
    let mut fs = cosmic_text::FontSystem::new_with_locale_and_db("en", cosmic_text::fontdb::Database::new());
    fs.db_mut().load_font_data(BUNDLED_FONT.to_vec());
    fs
    }
    ```
- [ ] Write test: chart renders correctly in WASM target

    ```rust
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test]
    async fn wasm_renders() {
    let fig = Figure::new(400, 300).add(LineMark::new(vec![0.,1.], vec![0.,1.]));
    let svg = fig.render_svg_string().unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("line"));
    }
    ```

### 0.11.0 — Polish

Exit criteria: the API is clean, all major input formats are supported, and the recipe system works.

- [ ] Design recipe proc macro: `#[starsight::recipe]` on a function

    ```rust
    // The proc macro reads the function signature and generates:
    // 1. A struct with the same name + "Recipe" suffix
    // 2. A builder with setter methods for each parameter
    // 3. A Mark trait impl that calls the original function
    
    // Input:
    #[starsight::recipe]
    fn sparkline(data: &[f64], width: f32, color: Color) -> Figure { ... }
    
    // Generated:
    pub struct SparklineRecipe { data: Vec<f64>, width: f32, color: Color }
    impl SparklineRecipe {
    pub fn new(data: &[f64]) -> Self { ... }
    pub fn width(mut self, w: f32) -> Self { self.width = w; self }
    pub fn color(mut self, c: Color) -> Self { self.color = c; self }
    }
    impl Mark for SparklineRecipe { ... }
    ```
- [ ] Generate struct from function parameters

    ```rust
    // In the proc macro (starsight-macros/src/lib.rs):
    use syn::{parse_macro_input, ItemFn, FnArg};
    use quote::quote;
    
    #[proc_macro_attribute]
    pub fn recipe(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let name = &func.sig.ident;
    let struct_name = format_ident!("{}Recipe", name.to_string().to_pascal_case());
    let fields: Vec<_> = func.sig.inputs.iter().filter_map(|arg| {
        if let FnArg::Typed(pat_type) = arg { Some((&pat_type.pat, &pat_type.ty)) } else { None }
    }).collect();
    // Generate struct, builder, and Mark impl...
    quote! { /* ... */ }.into()
    }
    ```
- [ ] Generate builder-style setters for each parameter

    ```rust
    // Generated for each non-first parameter:
    impl SparklineRecipe {
    pub fn width(mut self, val: f32) -> Self { self.width = val; self }
    pub fn color(mut self, val: impl Into<Color>) -> Self { self.color = val.into(); self }
    }
    ```
- [ ] Generate `Mark` trait implementation

    ```rust
    impl Mark for SparklineRecipe {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        // Call the original function with the stored parameters
        let fig = sparkline(&self.data, self.width, self.color);
        fig.render_to_coord(coord, backend)
    }
    }
    ```
- [ ] Write the proc macro in a separate `starsight-macros` crate

    ```toml
    # starsight-macros/Cargo.toml
    [package]
    name = "starsight-macros"
    version.workspace = true
    edition.workspace = true
    
    [lib]
    proc-macro = true
    
    [dependencies]
    syn = { version = "2", features = ["full"] }
    quote = "1"
    proc-macro2 = "1"
    ```
- [ ] Write test: recipe macro compiles and produces a usable mark type

    ```rust
    #[starsight::recipe]
    fn sparkline(data: &[f64], width: f32, height: f32, color: Color) -> Figure {
        // The macro generates SparklineRecipe struct with builder methods
        // and registers it as a mark type
    }
    // Usage: figure.add(SparklineRecipe::new(&data).width(100.0).color(Color::BLUE));
    ```

- [ ] Implement `From<Array1<f64>>` for DataSource (ndarray integration)

    ```rust
    #[cfg(feature = "ndarray")]
    impl From<ndarray::Array1<f64>> for DataSource {
    fn from(arr: ndarray::Array1<f64>) -> Self {
        DataSource::Vec(arr.to_vec())
    }
    }
    ```
- [ ] Implement `From<Array2<f64>>` for MatrixSource (ndarray integration)

    ```rust
    #[cfg(feature = "ndarray")]
    impl From<ndarray::Array2<f64>> for MatrixSource {
    fn from(arr: ndarray::Array2<f64>) -> Self {
        let rows = arr.nrows();
        let data: Vec<Vec<f64>> = (0..rows).map(|r| arr.row(r).to_vec()).collect();
        MatrixSource::NestedVec(data)
    }
    }
    ```

- [ ] Accept `arrow::RecordBatch` as data source

    ```rust
    #[cfg(feature = "arrow")]
    impl DataSource {
    pub fn from_record_batch(batch: &arrow::record_batch::RecordBatch, col: &str) -> Result<Self> {
        let col_idx = batch.schema().index_of(col)
            .map_err(|e| StarsightError::Data(format!("Column '{col}': {e}")))?;
        let array = batch.column(col_idx);
        // Convert to f64...
        Ok(DataSource::Vec(values))
    }
    }
    ```
- [ ] Extract columns by name from RecordBatch

    ```rust
    fn extract_f64_arrow(batch: &RecordBatch, col: &str) -> Result<Vec<f64>> {
    let idx = batch.schema().index_of(col)?;
    let array = batch.column(idx);
    let float_arr = arrow::compute::cast(array, &arrow::datatypes::DataType::Float64)?;
    let f64_arr = float_arr.as_any().downcast_ref::<arrow::array::Float64Array>().unwrap();
    Ok(f64_arr.iter().map(|v| v.unwrap_or(f64::NAN)).collect())
    }
    ```
- [ ] Use zero-copy conversion from Arrow arrays where possible

    ```rust
    // Arrow Float64Array can be zero-copy sliced:
    let values: &[f64] = f64_arr.values().as_ref();
    // Pass as slice reference instead of collecting into Vec
    ```

- [ ] Walk through every public type against Rust API Guidelines checklist

    ```bash
    # Checklist items to verify for each public type:
    # [ ] Debug implemented
    # [ ] Display implemented where meaningful
    # [ ] Clone implemented where reasonable
    # [ ] Default implemented where there is a sensible default
    # [ ] From/Into for obvious conversions
    # [ ] #[non_exhaustive] on enums and config structs
    # [ ] #[must_use] on builder methods
    cargo doc --workspace --all-features --open
    # Walk through every type in the docs
    ```
- [ ] Fix naming inconsistencies

    ```bash
    # Check: all types use CamelCase, all functions use snake_case
    # Check: getter methods have no get_ prefix (width() not get_width())
    # Check: conversion methods follow as_/to_/into_ conventions
    # Check: builder methods consume self and return Self
    grep -rn "pub fn get_" starsight-*/src/
    grep -rn "pub fn set_" starsight-*/src/
    ```
- [ ] Add missing trait implementations (Debug, Clone, Display, Default)

    ```bash
    # Find public types missing standard derives:
    cargo doc --workspace 2>&1 | grep "missing"
    # Add #[derive(Debug, Clone)] to every pub struct/enum
    # Add Default where there is a sensible zero/empty state
    ```
- [ ] Ensure all builders follow the same pattern consistently

    ```rust
    // All builders should follow this pattern:
    impl FooBuilder {
    pub fn new(required: Type) -> Self { /* ... */ }
    pub fn optional_field(mut self, val: Type) -> Self { self.field = val; self }
    pub fn build(self) -> Result<Foo> { /* validate and construct */ }
    }
    // Verify: no builder uses &mut self (should consume self)
    // Verify: no builder has a set_ prefix (use field name directly)
    ```

### 0.12.0 — Documentation

Exit criteria: every public item has documentation. The gallery is complete.

- [ ] Run `RUSTDOCFLAGS="-D missing_docs" cargo doc --workspace`

    ```bash
    RUSTDOCFLAGS="-D missing_docs" cargo doc --workspace --all-features
    # Expected: zero warnings. Every public item has a doc comment.
    ```
- [ ] Fix every missing_docs warning

    ```bash
    # Each warning looks like:
    # warning: missing documentation for a function
    #  --> starsight-layer-2/src/scale.rs:42:5
    # Fix by adding /// doc comment above the item
    ```
- [ ] Ensure each doc comment has: one-line summary, parameter descriptions, example, links to related types

    ```rust
    /// Creates a linear scale mapping data values to pixel coordinates.
    ///
    /// # Arguments
    /// * `domain` - The data range (min, max)
    /// * `range` - The pixel range (min, max)
    ///
    /// # Example
    /// ```
    /// let scale = LinearScale::new((0.0, 100.0), (0.0, 800.0));
    /// assert_eq!(scale.map(50.0), 400.0);
    /// ```
    ///
    /// See also: [`LogScale`], [`SymlogScale`]
    pub fn new(domain: (f64, f64), range: (f64, f64)) -> Self { ... }
    ```

- [ ] Write `examples/quickstart.rs` — the simplest possible chart (3 lines)

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    plot!(&[0.0, 1.0, 2.0, 3.0], &[0.0, 1.0, 0.5, 2.0]).save("quickstart.png")
    }
    ```
- [ ] Write `examples/line_chart.rs` — basic line chart with title and labels

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    let x: Vec<f64> = (0..100).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    Figure::new(800, 600)
        .add(LineMark::new(x, y).color(Color::BLUE).width(2.0))
        .title("Sine Wave")
        .x_label("x").y_label("sin(x)")
        .save("line_chart.png")
    }
    ```
- [ ] Write `examples/scatter.rs` — scatter plot with colored groups

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    let x = vec![1.0, 2.0, 3.0, 1.5, 2.5, 3.5];
    let y = vec![2.0, 3.0, 1.0, 3.5, 1.5, 2.5];
    let groups = vec!["A", "A", "A", "B", "B", "B"];
    Figure::new(600, 400)
        .add(PointMark::new(x, y).color_by(&groups))
        .title("Grouped Scatter").save("scatter.png")
    }
    ```
- [ ] Write `examples/bar_chart.rs` — grouped bar chart

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    Figure::new(600, 400)
        .add(BarMark::new(vec!["Q1".into(),"Q2".into(),"Q3".into(),"Q4".into()],
                         vec![120.0, 150.0, 90.0, 200.0]).color(Color::from_hex(0x4C72B0)))
        .title("Quarterly Revenue").y_label("USD (thousands)").save("bar_chart.png")
    }
    ```
- [ ] Write `examples/histogram.rs` — histogram with KDE overlay

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    let data: Vec<f64> = (0..1000).map(|_| rand_normal() * 15.0 + 50.0).collect();
    Figure::new(800, 500)
        .add(HistogramMark::new(data).bins(BinMethod::Fd).kde(true))
        .title("Distribution").x_label("Value").save("histogram.png")
    }
    ```
- [ ] Write `examples/heatmap.rs` — annotated heatmap with diverging colormap

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    let data: Vec<Vec<f64>> = (0..10).map(|r| (0..10).map(|c| ((r+c) as f64 - 9.0)).collect()).collect();
    Figure::new(600, 500)
        .add(HeatmapMark::new(data).colormap(prismatica::crameri::VIK).annotate(true))
        .title("Correlation Matrix").save("heatmap.png")
    }
    ```
- [ ] Write `examples/statistical.rs` — box plot + violin side by side

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    let groups = vec![
        ("A".into(), vec![1.,2.,3.,4.,5.,6.,7.,8.,9.,20.]),
        ("B".into(), vec![3.,4.,5.,5.5,6.,6.5,7.,8.]),
    ];
    let grid = GridLayout::row(vec![
        Figure::new(400, 300).add(BoxPlotMark::new(groups.clone())).title("Box Plot"),
        Figure::new(400, 300).add(ViolinMark::from_groups(groups)).title("Violin Plot"),
    ]);
    grid.save("statistical.png")
    }
    ```
- [ ] Write `examples/faceting.rs` — faceted scatter by category

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    // Generate scatter data with 4 categories
    Figure::new(900, 600)
        .add(PointMark::new(x, y).color_by(&groups))
        .facet_wrap("category", Some(2))
        .title("Faceted Scatter").save("faceting.png")
    }
    ```
- [ ] Write `examples/custom_theme.rs` — applying a chromata theme

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    let theme = chromata::popular::gruvbox::DARK_HARD;
    Figure::new(800, 600)
        .add(LineMark::new(vec![0.,1.,2.,3.], vec![0.,2.,1.,3.]))
        .theme(theme.into())
        .save("custom_theme.png")
    }
    ```
- [ ] Write `examples/terminal.rs` — inline terminal rendering

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    let x: Vec<f64> = (0..50).map(|i| i as f64 * 0.2).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    Figure::new(160, 48) // small for terminal
        .add(LineMark::new(x, y))
        .show_terminal() // auto-detect protocol: Kitty > Sixel > Braille
    }
    ```
- [ ] Write `examples/interactive.rs` — windowed chart with hover and zoom

    ```rust
    use starsight::prelude::*;
    
    fn main() -> starsight::Result<()> {
    let x: Vec<f64> = (0..1000).map(|i| i as f64 * 0.01).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin() * v.cos()).collect();
    Figure::new(800, 600)
        .add(LineMark::new(x, y).color(Color::BLUE))
        .title("Interactive — scroll to zoom, drag to pan")
        .show() // opens native window with hover, zoom, pan
    }
    ```
- [ ] Write `examples/polars_integration.rs` — chart from a Polars DataFrame

    ```rust
    use starsight::prelude::*;
    use polars::prelude::*;
    
    fn main() -> starsight::Result<()> {
    let df = CsvReader::from_path("data.csv")?.finish()?;
    let fig = plot!(df, x = "date", y = "price", color = "symbol");
    fig.title("Stock Prices").save("polars_integration.png")
    }
    ```

- [ ] Verify `cargo xtask gallery` runs all examples and saves PNGs to `gallery/`

    ```bash
    cargo xtask gallery
    ls -la gallery/*.png
    # Verify one PNG per example file
    for ex in examples/*.rs; do
    name=$(basename "$ex" .rs)
    test -f "gallery/$name.png" && echo "OK: $name" || echo "MISSING: $name"
    done
    ```
- [ ] Update GALLERY_REFERENCE.md with generated images

    ```bash
    # Uncomment image placeholders in GALLERY_REFERENCE.md:
    # Change: <!-- ![starsight](gallery/line_chart.png) -->
    # To:     ![starsight](gallery/line_chart.png)
    sed -i 's/<!-- !\[starsight\]/![starsight]/g; s/\.png) -->/\.png)/g' .spec/GALLERY_REFERENCE.md
    ```

- [ ] docs.rs configuration in workspace Cargo.toml:

    ```toml
    [package.metadata.docs.rs]
    all-features = true
    rustdoc-args = ["--cfg", "docsrs"]
    ```

    Use `#[cfg_attr(docsrs, doc(cfg(feature = "gpu")))]` to annotate feature-gated items so docs.rs shows which features enable which types.

### 1.0.0 — Stable release

Exit criteria: the library is production-ready. The public API is stable and will not change without a major version bump.

- [ ] cargo-semver-checks clean pass: no public API changes that violate semver since the last pre-release.

    ```bash
    cargo semver-checks check-release
    # Expected: no breaking changes detected
    ```

- [ ] CI green on stable Rust (Linux, macOS, Windows)

    ```bash
    # Verified via GitHub Actions matrix:
    # runs-on: [ubuntu-latest, macos-latest, windows-latest]
    # toolchain: [stable]
    cargo test --workspace
    ```
- [ ] CI green on MSRV 1.85 (Linux, macOS, Windows)

    ```bash
    cargo +1.85.0 check --workspace
    cargo +1.85.0 test --workspace
    ```
- [ ] All feature combinations compile (`cargo hack check --each-feature`)

    ```bash
    cargo hack check --each-feature --workspace
    # Tests every feature in isolation
    ```
- [ ] All tests pass (`cargo test --workspace --all-features`)

    ```bash
    cargo test --workspace --all-features
    ```
- [ ] All snapshot baselines match (`cargo insta test`)

    ```bash
    cargo insta test --workspace
    # Expected: all snapshots match, zero pending reviews
    ```

- [ ] Create benchmark suite with criterion

    ```rust
    // benches/render.rs
    use criterion::{criterion_group, criterion_main, Criterion};
    use starsight::prelude::*;
    
    fn bench_line_1000(c: &mut Criterion) {
    let x: Vec<f64> = (0..1000).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin()).collect();
    c.bench_function("line_1000pt_800x600", |b| {
        b.iter(|| {
            let fig = Figure::new(800, 600).add(LineMark::new(x.clone(), y.clone()));
            fig.render_png().unwrap();
        })
    });
    }
    criterion_group!(benches, bench_line_1000);
    criterion_main!(benches);
    ```
- [ ] Benchmark: 1000-point line chart at 800x600 renders under 50ms

    ```bash
    cargo bench --bench render -- line_1000
    # Check that mean time is < 50ms
    ```
- [ ] Benchmark: 100,000-point scatter plot renders under 500ms

    ```bash
    cargo bench --bench render -- scatter_100k
    # Check that mean time is < 500ms
    ```

- [ ] Security audit: `cargo audit` and `cargo deny check advisories` report no known vulnerabilities in the dependency tree.

    ```bash
    cargo audit
    cargo deny check advisories
    # Expected: no known vulnerabilities
    ```

- [ ] Generate complete changelog with git-cliff from all pre-release versions

    ```bash
    git cliff --output CHANGELOG.md
    # Generates from all conventional commits
    ```
- [ ] Review every changelog entry

    ```bash
    # Manual review: read CHANGELOG.md, verify:
    # - All breaking changes documented
    # - All new features listed
    # - No internal/CI-only changes polluting the public changelog
    ```
- [ ] Publish changelog on GitHub releases

    ```bash
    gh release create v1.0.0 --title "starsight 1.0.0" --notes-file CHANGELOG.md
    ```

- [ ] Write announcement blog post with example charts and comparisons

    ```markdown
    # starsight 1.0.0: A Unified Visualization Library for Rust
    
    ## What is starsight?
    [screenshot of gallery]
    
    ## How it compares
    [benchmark table vs Plotters]
    
    ## Quick start
    ```rust
use starsight::prelude::*;
plot!(&[1,2,3], &[4,5,6]).save("chart.png")?;
```

## What's next
[roadmap beyond 1.0]
```
- [ ] Post to Reddit r/rust

    ```
    Title: starsight 1.0.0 — unified scientific visualization for Rust
    URL: https://github.com/resonant-jovian/starsight
    Subreddit: r/rust
    ```
- [ ] Post to Hacker News

    ```
    Title: Show HN: starsight — scientific visualization library for Rust
    URL: https://github.com/resonant-jovian/starsight
    ```
- [ ] Post to Rust users forum

    ```
    https://users.rust-lang.org/
    Category: announcements
    Title: starsight 1.0.0 released — unified visualization from one-liners to GPU 3D
    ```
- [ ] Post to Twitter/Mastodon

    ```
    starsight 1.0.0 is out! Unified scientific visualization for Rust.
    From `plot!([1,2,3]).save("chart.png")` to GPU-accelerated 3D.
    70 chart types, terminal rendering, chromata themes.
    https://github.com/resonant-jovian/starsight
    #rustlang
    ```

---

# Part 2 — Look up

Quick-reference for API details, imports, formulas.

---

## Import quick reference

### By task

| I want to... | Write this |
|---|---|
| One-liner chart | `use starsight::prelude::*;` |
| Build figure manually | `use starsight::{Figure, Color, Point};` |
| Line chart | `use starsight::marks::LineMark;` |
| Scatter plot | `use starsight::marks::PointMark;` |
| Bar chart | `use starsight::marks::BarMark;` |
| Heatmap | `use starsight::marks::HeatmapMark;` |
| Apply colormap | `use prismatica::crameri::BATLOW;` |
| Apply theme | `use chromata::popular::gruvbox;` |
| SVG output | `use starsight::backends::SvgBackend;` |
| Terminal output | `use starsight::backends::TerminalBackend;` |
| Multi-chart layout | `use starsight::layout::{GridLayout, FacetWrap};` |

### By layer (internal crate code)

| Layer | Typical imports |
|---|---|
| **1** (primitives) | `use crate::primitives::{Color, Point, Rect, Vec2};` |
| | `use crate::error::{Result, StarsightError};` |
| | `use tiny_skia::{Pixmap, Paint, PathBuilder, Stroke};` |
| | `use cosmic_text::{FontSystem, SwashCache, Buffer, Metrics};` |
| **2** (scales) | `use starsight_layer_1::primitives::{Point, Rect, Color};` |
| | `use starsight_layer_1::error::Result;` |
| **3** (marks) | `use starsight_layer_1::backend::DrawBackend;` |
| | `use starsight_layer_2::{scale::Scale, coord::CartesianCoord};` |
| **5** (API) | `use starsight_layer_3::mark::Mark;` |
| | `use starsight_layer_1::backend::skia::raster::SkiaBackend;` |

---

## Data-to-pixel pipeline

```
 Vec<f64>
 DataFrame       DATA
 &[f64]            │
 Array1            ▼
             ┌───────────┐
             │  Figure   │  Layer 5: accept data, configure
             │  .add()   │
             └─────┬─────┘
                   │
          ┌────────┼────────┐
          │        │        │
          ▼        ▼        ▼
      ┌──────┐ ┌──────┐ ┌───────┐
      │ Line │ │ Bar  │ │ Point │  Layer 3: marks
      │ Mark │ │ Mark │ │ Mark  │
      └───┬──┘ └───┬──┘ └───┬───┘
          │        │        │
          └────────┼────────┘
                   │
                   ▼
          ┌────────────────┐
          │ CartesianCoord │  Layer 2
          │                │
          │  Scale::map()  │  data value ──▶ [0, 1]
          │  Axis::ticks   │  Wilkinson algorithm
          │  data_to_px()  │  [0, 1] ──▶ pixel coord
          └───────┬────────┘
                  │
                  ▼
          ┌────────────────┐
          │    Backend     │  Layer 1
          │                │
          │  fill_rect()   │  SkiaBackend  (CPU raster)
          │  draw_path()   │  SvgBackend   (vector)
          │  draw_text()   │  WgpuBackend  (GPU)
          └───────┬────────┘
                  │
                  ▼
          ┌────────────────┐
          │     OUTPUT     │  .png  .svg  .pdf  terminal
          └────────────────┘
```

---

## Coordinate cheat sheet

```
 Screen coordinates (pixel space)

   (0,0) ─────────────────────────────▶ x increases rightward
     │
     │    margin_top
     │    ┌────────────────────────────┐
     │    │                            │
     │    │   (margin_left, margin_top)│
     │    │   ┌────────────────────┐   │
     │    │   │                    │   │
     │    │   │     PLOT AREA      │   │
     │    │   │                    │   │
     │    │   │          ● (x_px,  │   │
     │    │   │            y_px)   │   │
     │    │   │                    │   │
     │    │   └────────────────────┘   │
     │    │                            │
     │    └────────────────────────────┘
     ▼
   y increases DOWNWARD  (opposite of math convention!)

 Data-to-pixel formulas:

   x_px = margin_left + ((x - x_min) / (x_max - x_min)) * plot_width

   y_px = margin_top  + (1.0 - (y - y_min) / (y_max - y_min)) * plot_height
                         ─────────────────────────────────────
                           ↑  subtracting from 1.0 inverts Y
```

---

## Common recipes

### Minimal line chart

```rust
use starsight::prelude::*;

fn main() -> starsight::Result<()> {
    let fig = Figure::new(800, 600)
        .add(LineMark::new(vec![0., 1., 2., 3.], vec![0., 1., 0.5, 2.]))
        .title("My Chart")
        .x_label("x")
        .y_label("y");
    fig.save("chart.png")
}
```

### Two series with legend

```rust
let fig = Figure::new(800, 600)
    .add(LineMark::new(x1, y1).color(Color::BLUE).label("Series A"))
    .add(LineMark::new(x2, y2).color(Color::RED).label("Series B"))
    .legend(LegendPosition::TopRight);
fig.save("two_series.png")?;
```

### Scatter with color grouping

```rust
let fig = Figure::new(600, 400)
    .add(PointMark::new(x, y).color_by(&groups).radius(5.0));
fig.save("scatter.png")?;
```

### Save as SVG

```rust
fig.save("chart.svg")?;
```

### Apply a theme

```rust
use chromata::popular::gruvbox;
fig.theme(gruvbox::DARK_HARD.into());
```

### Apply a scientific colormap

```rust
use prismatica::crameri::BATLOW;
Figure::new(600, 600)
    .add(HeatmapMark::new(data).colormap(BATLOW));
```

---

## tiny-skia 0.12 API reference

### Color types

tiny-skia has four color types. Use `from_rgba8` for starsight (infallible, takes u8 values). Use `from_rgba` only when accepting user-supplied floats (returns None if out of range). The premultiplied type is internal to tiny-skia — you should never need to construct one directly.

| Type | Fields | Alpha | Constructor | Returns |
|------|--------|-------|-------------|---------|
| `Color` | f32 × 4 | Straight | `from_rgba(r,g,b,a)` | `Option<Self>` (None if out of 0.0-1.0) |
| `Color` | f32 × 4 | Straight | `from_rgba8(r,g,b,a)` | `Self` (infallible) |
| `ColorU8` | u8 × 4 | Straight | `from_rgba(r,g,b,a)` | `Self` (const, infallible) |
| `PremultipliedColorU8` | u8 × 4 | Premultiplied | `from_rgba(r,g,b,a)` | `Option<Self>` (None if channel > alpha) |

### Drawing methods (all take `Option<&Mask>` as final param)

These are the four drawing methods on `Pixmap`. Every method takes a `Transform` (use `identity()` for no transformation) and an optional `&Mask` for clipping (pass `None` to draw everywhere, or `Some(&mask)` to restrict drawing to the plot area).

```rust
pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
pixmap.fill_rect(rect, &paint, Transform::identity(), None);
pixmap.draw_pixmap(x: i32, y: i32, pixmap_ref, &pixmap_paint, Transform::identity(), None);
```

### PathBuilder

Build paths incrementally using move/line/curve commands. Always start with `move_to`. `finish()` returns `Option<Path>` — returns `None` if no segments were added. Static constructors `from_rect` and `from_circle` are shortcuts for common shapes.

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

All fields are public. Create with struct literal syntax.

```rust
Stroke {
    width: 2.0,                  // line thickness in pixels
    miter_limit: 4.0,            // prevents spikes at acute angles (default 4.0)
    line_cap: LineCap::Round,    // Butt | Round | Square — endpoint shape
    line_join: LineJoin::Round,  // Miter | MiterClip | Round | Bevel — corner shape
    dash: StrokeDash::new(vec![10.0, 5.0], 0.0), // returns Option — [visible, gap] pattern
}
```

LineCap variants: `Butt` (flat cut at endpoint, default), `Round` (semicircle extending beyond endpoint), `Square` (rectangle extending by half stroke width). For chart axis lines, use `Butt`. For data lines that end at the plot boundary, use `Butt` to avoid visual overshoot.

LineJoin variants: `Miter` (sharp point, constrained by miter_limit), `MiterClip` (mitered but clipped at miter_limit distance), `Round` (circular arc at corner), `Bevel` (flat cut at corner). For chart lines with sharp turns, `Round` prevents spikes. For rectangular elements like bars, any join works since corners are 90 degrees.

StrokeDash: the array alternates between visible and gap lengths. `[10.0, 5.0]` means 10px visible, 5px gap. `[5.0, 3.0, 1.0, 3.0]` means 5px dash, 3px gap, 1px dot, 3px gap. The offset shifts the starting position along the pattern. Returns `None` if the array is empty.

### Transform — DEGREES not radians

Six fields: sx, kx, ky, sy, tx, ty. All f32. All public. Represents an affine transformation matrix.

```rust
Transform::identity()                        // no transformation
Transform::from_translate(tx, ty)            // shift position
Transform::from_scale(sx, sy)               // stretch/shrink
Transform::from_rotate(degrees)              // NOT radians — this is the #1 gotcha
Transform::from_rotate_at(degrees, cx, cy)  // rotate around a point
Transform::from_row(sx, ky, kx, sy, tx, ty) // manual matrix (note: ky before kx)
```

Composition methods (all return a new Transform, do not mutate):

```rust
t.pre_translate(tx, ty)   // apply translate BEFORE existing transform (to input points)
t.pre_scale(sx, sy)       // apply scale BEFORE existing
t.pre_rotate(degrees)     // apply rotate BEFORE existing
t.post_translate(tx, ty)  // apply translate AFTER existing transform (to output points)
t.post_scale(sx, sy)      // apply scale AFTER existing
t.post_concat(other)      // combine: other applied AFTER self
```

For chart rendering: start with `identity()`, then `pre_translate` to position the element, then `pre_scale` for DPI scaling. The composition order matters: `translate then rotate` produces different results than `rotate then translate`.

### PNG export

```rust
// Save directly to file
pixmap.save_png("file.png")?;

// Encode to memory (for snapshot tests, HTTP responses, embedding)
let bytes: Vec<u8> = pixmap.encode_png()?;

// DPI metadata: PNG stores pixels-per-meter in the pHYs chunk
// 72 DPI  = 2835 pixels/meter
// 96 DPI  = 3780 pixels/meter (standard screen)
// 150 DPI = 5906 pixels/meter
// 300 DPI = 11811 pixels/meter (print quality)
```

The `encode_png` method automatically converts from premultiplied alpha (tiny-skia's internal format) to straight alpha (PNG's format). You do not need to manually demultiply.

For snapshot testing, use `encode_png()` to get bytes and pass them to `insta::assert_binary_snapshot!(".png", bytes)`. The PNG encoding is deterministic: the same pixel data always produces the same bytes, making byte-for-byte comparison reliable.

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

Call this after `shape_until_scroll` to get the bounding box of the laid-out text. The width is the widest line. The height is the bottom of the last line. Use these values for margin calculation (measuring y-axis tick label widths and title heights).

```rust
let (mut w, mut h) = (0.0f32, 0.0f32);
for run in buffer.layout_runs() {
    w = w.max(run.line_w);
    h = run.line_top + run.line_height;
}
```

### Draw onto tiny-skia (NO channel swap for file output)

The `draw` callback fires once per glyph rectangle. Each rectangle has position, size, and a color with alpha representing pixel coverage. Paint each rectangle onto the pixmap. Do NOT swap red and blue channels — the channel swap in cosmic-text's example code is only needed for softbuffer display targets, not for PNG or SVG file output.

```rust
buffer.draw(&mut swash_cache, text_color, |x, y, w, h, color| {
    paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
    if let Some(rect) = Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
});
```

### Embed custom font

Call this before creating any `Buffer` to ensure the font is available for text layout. Use `include_bytes!` to bundle the font into the binary at compile time. For snapshot test determinism, always embed a specific font rather than relying on system fonts (which differ across OS).

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

Choose the colormap family based on the data semantics. Sequential maps encode magnitude (low to high). Diverging maps encode deviation from a central value (negative through zero to positive). Cyclic maps wrap around (useful for angles and phases). Discrete palettes assign distinct colors to categories.

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

Computing margins is a two-pass process. First pass: create scales from data ranges, run ticks, format labels, measure text widths using cosmic-text. This gives the actual margin sizes. Second pass: recompute the plot area with the real margins and re-run scales if needed. The formulas below describe the first pass.

```
left_margin   = pad + y_label_height + label_pad + max_ytick_width + tick_pad
bottom_margin = pad + x_label_height + label_pad + xtick_height + tick_pad
plot_width    = figure_width  - left_margin - right_margin
plot_height   = figure_height - top_margin  - bottom_margin
max_ytick_width = max(len(format(tick))) * font_size * 0.6
```

---

## Error handling pattern

The StarsightError enum has seven variants covering all failure modes. All variants are non-exhaustive so new error kinds can be added without breaking downstream code. The Result type alias saves typing throughout the codebase. Use `#[from]` for automatic conversion from std io Error. All other errors wrap a descriptive String.

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

The plot! macro has three arms: DataFrame input with named columns, two-array input (x and y slices), and single-array input (y values only, x auto-generated as 0..n). The DataFrame arm accepts arbitrary key-value pairs that forward to builder methods. All arms return a Figure ready for `.save()` or `.show()`.

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


## Feature flags

| Feature | Enables | Default |
|---|---|---|
| *(none)* | CPU rendering, PNG/SVG export | yes |
| `gpu` | GPU rendering via wgpu | no |
| `terminal` | Kitty / Sixel / Braille rendering | no |
| `interactive` | Windowed charts with hover/zoom/pan | no |
| `polars` | Polars DataFrame input | no |
| `ndarray` | ndarray input | no |
| `arrow` | Arrow RecordBatch input | no |
| `3d` | 3D surface/scatter/wireframe | no |
| `pdf` | PDF export via krilla | no |
| `stats` | KDE, regression, advanced stats | no |
| `web` | WASM target support | no |
| `full` | All of the above | no |

```bash
cargo add starsight --features gpu,terminal
cargo add starsight --features full
```

---

## Troubleshooting

| Error | Cause | Fix |
|---|---|---|
| `Render("Failed to create 0x0 pixmap")` | Width or height is zero | Ensure both dimensions > 0 |
| `Scale("domain is empty: min equals max")` | All data values identical | Set domain manually or add distinct values |
| `Export("unknown extension .xyz")` | Unsupported output format | Use `.png`, `.svg`, or `.pdf` |
| `Data("column 'x' not found")` | Wrong column name | Check `df.schema()` for exact names |
| clippy: `unwrap_used` | `.unwrap()` in library code | Replace with `?` or `.ok_or_else()` |
| Snapshot mismatch after dep update | Anti-aliasing algorithm changed | Run `cargo insta review` and update baselines |
| Colors look wrong in terminal | Premultiplied alpha issue | Only affects softbuffer display, not PNG/SVG |
| `from_rotate` gives wrong angle | tiny-skia uses DEGREES | Convert: `degrees = radians * 180.0 / PI` |

---

## Links and references

Primary dependencies and their documentation.

| Crate | docs.rs | GitHub |
|-------|---------|--------|
| tiny-skia | https://docs.rs/tiny-skia | https://github.com/linebender/tiny-skia |
| cosmic-text | https://docs.rs/cosmic-text | https://github.com/pop-os/cosmic-text |
| svg | https://docs.rs/svg | https://github.com/bodoni/svg |
| palette | https://docs.rs/palette | https://github.com/Ogeon/palette |
| image | https://docs.rs/image | https://github.com/image-rs/image |
| thiserror | https://docs.rs/thiserror | https://github.com/dtolnay/thiserror |
| insta | https://docs.rs/insta | https://github.com/mitsuhiko/insta |
| prismatica | — | https://github.com/resonant-jovian/prismatica |
| chromata | — | https://github.com/resonant-jovian/chromata |
| wgpu | https://docs.rs/wgpu | https://github.com/gfx-rs/wgpu |
| ratatui | https://docs.rs/ratatui | https://github.com/ratatui/ratatui |
| polars | https://docs.rs/polars | https://github.com/pola-rs/polars |
| krilla | https://docs.rs/krilla | https://github.com/LaurenzV/krilla |
| winit | https://docs.rs/winit | https://github.com/rust-windowing/winit |

### Theory and standards

- Wilkinson Extended ticks: https://vis.stanford.edu/files/2010-TickLabels-InfoVis.pdf
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/checklist.html
- Cargo SemVer compatibility: https://doc.rust-lang.org/cargo/reference/semver.html
- Cargo features: https://doc.rust-lang.org/cargo/reference/features.html
- Cargo workspaces: https://doc.rust-lang.org/cargo/reference/workspaces.html
- Edition 2024: https://doc.rust-lang.org/edition-guide/rust-2024/index.html
- Kitty graphics protocol: https://sw.kovidgoyal.net/kitty/graphics-protocol/
- Sixel: https://vt100.net/docs/vt3xx-gp/chapter14.html
- Crameri colormaps: https://www.fabiocrameri.ch/colourmaps/
- WCAG contrast: https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html

### API design

- https://rust-lang.github.io/api-guidelines/ — Rust API Guidelines (canonical)
- https://rust-lang.github.io/api-guidelines/type-safety.html — Type Safety (newtypes, builders)
- https://rust-lang.github.io/api-guidelines/interoperability.html — Common traits to implement
- https://rust-lang.github.io/api-guidelines/future-proofing.html — non_exhaustive, sealed traits
- https://deterministic.space/elegant-apis-in-rust.html — Elegant Library APIs (Pascal Hertleif)
- https://www.lpalmieri.com/posts/error-handling-rust/ — Error Handling Deep Dive (Luca Palmieri)
- https://burntsushi.net/rust-error-handling/ — Error Handling (Andrew Gallant)
- https://cliffle.com/blog/rust-typestate/ — Typestate Pattern (Cliff Biffle)
- https://rust-unofficial.github.io/patterns/patterns/creational/builder.html — Builder Pattern
- https://predr.ag/blog/definitive-guide-to-sealed-traits-in-rust/ — Sealed Traits
- https://microsoft.github.io/rust-guidelines/ — Microsoft Pragmatic Rust Guidelines
- https://predr.ag/blog/semver-in-rust-tooling-breakage-and-edge-cases/ — SemVer edge cases
- https://www.lurklurk.org/effective-rust/ — Effective Rust (free online)
- https://mmapped.blog/posts/12-rust-error-handling — Designing Error Types
- https://www.philipdaniels.com/blog/2019/rust-api-design/ — API Design with AsRef, Into, Cow

### Architecture

- https://matklad.github.io/2021/08/22/large-rust-workspaces.html — Large Rust Workspaces
- https://matklad.github.io/2021/09/04/fast-rust-builds.html — Fast Rust Builds
- https://matklad.github.io/2021/02/06/ARCHITECTURE.md.html — ARCHITECTURE.md advocacy
- https://corrode.dev/blog/tips-for-faster-rust-compile-times/ — Faster Compile Times
- https://nnethercote.github.io/perf-book/ — The Rust Performance Book
- https://github.com/johnthagen/min-sized-rust — Minimizing binary size
- https://rust-analyzer.github.io/book/contributing/architecture.html — rust-analyzer Architecture
- https://doc.rust-lang.org/cargo/reference/build-scripts.html — Build Scripts

### Patterns

- https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html — Newtype Pattern
- https://www.lurklurk.org/effective-rust/newtype.html — Newtype (Effective Rust)
- https://rust-unofficial.github.io/patterns/anti_patterns/deref.html — Deref Polymorphism anti-pattern
- https://doc.rust-lang.org/std/convert/trait.From.html — From trait (when to implement)
- https://rust-lang.github.io/rfcs/0445-extension-trait-conventions.html — Extension Traits (RFC 445)
- https://corrode.dev/blog/dont-use-preludes-and-globs/ — Don't Use Preludes And Globs

### Testing and safety

- https://docs.rs/insta — insta (snapshot testing)
- https://docs.rs/proptest — proptest (property testing)
- https://doc.rust-lang.org/nomicon/ — The Rustonomicon (unsafe code)
- https://github.com/rust-lang/miri — Miri (undefined behavior detection)
- https://burntsushi.net/unwrap/ — Using unwrap() in Rust is Okay

### Books

- https://rust-for-rustaceans.com/ — Rust for Rustaceans (Jon Gjengset)
- https://marabos.nl/atomics/ — Rust Atomics and Locks (Mara Bos, free)
- https://www.lurklurk.org/effective-rust/ — Effective Rust (David Drysdale, free)
- https://www.zero2prod.com/ — Zero to Production in Rust (Luca Palmieri)
- https://doc.rust-lang.org/book/ — The Rust Book (official)
- https://doc.rust-lang.org/rust-by-example/ — Rust By Example

### Community

- https://this-week-in-rust.org/ — This Week in Rust
- https://blessed.rs — Curated crate recommendations
- https://lib.rs/ — Alternative crates.io frontend
- https://users.rust-lang.org/ — Rust Users Forum
- https://play.rust-lang.org/ — Rust Playground

### Key RFCs

| RFC | Topic |
|-----|-------|
| 344 | Method naming conventions |
| 430 | Naming conventions (CamelCase, snake_case) |
| 505 | API documentation conventions |
| 1105 | API evolution / SemVer policy |
| 1270 | Deprecation attribute |
| 2495 | Minimum Supported Rust Version |

---

## Scale math formulas

### Linear scale

Maps a value from the data domain to the pixel range using linear interpolation. The inverse formula maps back from pixels to data coordinates (used for hover tooltips).

```
output = (input - domain_min) / (domain_max - domain_min) * (range_max - range_min) + range_min
inverse = (output - range_min) / (range_max - range_min) * (domain_max - domain_min) + domain_min
```

### Log scale

Maps using the logarithm of the input value. Spreads out small values and compresses large values. Essential for data spanning multiple orders of magnitude (e.g., frequency spectra, population sizes).

```
output = (log10(input) - log10(domain_min)) / (log10(domain_max) - log10(domain_min)) * range_extent + range_min
```

Domain must be strictly positive. Values <= 0 must be clipped or masked before mapping. Attempting to take log10 of zero or a negative number produces NaN or -infinity.

### Symlog scale

Symmetric log: behaves linearly near zero (within the threshold) and logarithmically beyond it. Handles data that crosses zero, unlike log scale which requires strictly positive values. The threshold parameter controls how wide the linear region is.

```
T(x) = sign(x) * log10(1 + |x| / threshold)
output = linear_map(T(input), T(domain_min), T(domain_max), range_min, range_max)
```

Default threshold = 1.0. Linear region width equals threshold on each side of zero.

### Band scale

Maps n categories to evenly spaced bands with configurable gaps. inner_padding is the gap between bands as a fraction of the step size. outer_padding is the gap at the edges. The bandwidth is the width of each band in pixels.

```
bandwidth = range_extent / (n + (n - 1) * inner_padding + 2 * outer_padding)
step = bandwidth * (1 + inner_padding)
position(i) = range_min + outer_padding * bandwidth + i * step + bandwidth / 2
```

---

## Color conversion formulas

### sRGB to linear

Input: sRGB channel value in [0, 1]. Output: linear light intensity in [0, 1]. The threshold 0.04045 prevents the power function from producing a discontinuity near zero. Use this when computing luminance or doing color blending (blend in linear space, convert back to sRGB for display).

```
if srgb <= 0.04045:
    linear = srgb / 12.92
else:
    linear = ((srgb + 0.055) / 1.055) ^ 2.4
```

### Linear to sRGB

Input: linear light intensity in [0, 1]. Output: sRGB channel value in [0, 1]. This is the inverse of the sRGB-to-linear conversion. Apply after blending or luminance computation to get back to display-ready values.

```
if linear <= 0.0031308:
    srgb = linear * 12.92
else:
    srgb = 1.055 * linear ^ (1/2.4) - 0.055
```

### WCAG relative luminance

Relative luminance measures perceived brightness on a 0-to-1 scale, where 0 is black and 1 is white. The coefficients reflect human eye sensitivity (most sensitive to green, least to blue). R, G, B must be linearized first (see sRGB-to-linear above). Use this for WCAG contrast ratio calculation and for choosing black vs white text over a colored background.

```
L = 0.2126 * R_linear + 0.7152 * G_linear + 0.0722 * B_linear
```

### WCAG contrast ratio

The contrast ratio ranges from 1:1 (identical colors) to 21:1 (black on white). Always put the lighter luminance in the numerator. The 0.05 term prevents division by zero and accounts for ambient light. starsight uses this to auto-select annotation text color in heatmaps: if the cell luminance is below 0.5, use white text; otherwise use black.

```
ratio = (L_lighter + 0.05) / (L_darker + 0.05)
AA normal text: >= 4.5
AA large text:  >= 3.0
AAA normal:     >= 7.0
```

### Premultiplied alpha

tiny-skia stores pixels in premultiplied alpha format internally. In premultiplied form, each RGB channel is already multiplied by the alpha value. This makes compositing faster (one multiply instead of three) but means raw pixel values look wrong if you interpret them as straight alpha. For fully opaque pixels (alpha=255), premultiplication is a no-op. The `encode_png` method automatically converts back to straight alpha.

```
premul_r = straight_r * alpha / 255
premul_g = straight_g * alpha / 255
premul_b = straight_b * alpha / 255

Source-over compositing (premultiplied):
result = source + destination * (1 - source_alpha / 255)
```

---

## Testing patterns

### Snapshot test template

Every chart type needs a snapshot test. Create the backend at a fixed size, render the chart with deterministic data, encode to PNG, and pass to insta. On first run, `cargo insta review` shows the image and you accept it. On subsequent runs, any visual change fails the test.

```rust
#[test]
fn test_line_chart_basic() {
    let mut backend = SkiaBackend::new(800, 600).unwrap();
    backend.fill(Color::WHITE);
    
    // ... build and render chart ...
    
    let bytes = backend.png_bytes().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}
```

### Property test template

Property tests generate thousands of random inputs and check that invariants hold. Use these for mathematical code: scale mapping/inverse roundtrips, tick monotonicity, color clamping, rectangle intersection. proptest automatically shrinks failing inputs to the minimal case.

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn scale_roundtrip(val in -1e6f64..1e6f64) {
        let scale = LinearScale::new((0.0, 100.0), (0.0, 800.0));
        let px = scale.map(val);
        let back = scale.inverse(px);
        prop_assert!((val - back).abs() < 1e-10);
    }
    
    #[test]
    fn ticks_are_monotonic(min in -1e6f64..0.0f64, max in 0.1f64..1e6f64) {
        let ticks = wilkinson_extended(min, max, 5);
        for pair in ticks.windows(2) {
            prop_assert!(pair[0] < pair[1]);
        }
    }
}
```

### Approximate float comparison

Never use `==` to compare floating point values in tests. Use a tolerance-based comparison. The formula below uses both absolute and relative tolerance: it handles values near zero (where relative tolerance fails) and large values (where absolute tolerance is too tight). A typical tolerance for scale roundtrip tests is 1e-10.

```rust
fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
    (a - b).abs() < tol || (a - b).abs() < tol * a.abs().max(b.abs())
}
```

---

## Common tiny-skia patterns

### Fill background

Set the entire pixmap to a solid color. Call this first before drawing anything. Without it, the pixmap is transparent black (all zeros), which produces a black background in PNG output.

```rust
pixmap.fill(tiny_skia::Color::from_rgba8(255, 255, 255, 255));
```

### Draw a horizontal line

Used for axis lines, tick marks, and grid lines. Set `anti_alias = false` for perfectly axis-aligned lines to keep them crisp (a single pixel wide instead of blurred across two).

```rust
let mut pb = PathBuilder::new();
pb.move_to(x1, y);
pb.line_to(x2, y);
let path = pb.finish().unwrap();
let mut paint = Paint::default();
paint.set_color_rgba8(0, 0, 0, 255);
paint.anti_alias = false; // crisp for axis-aligned lines
pixmap.stroke_path(&path, &paint, &Stroke::default(), Transform::identity(), None);
```

### Draw a filled rectangle

Used for bar chart bars, heatmap cells, legend swatches, and background fills. `from_xywh` returns `None` if width or height is zero or negative — always handle the Option. For bars growing downward from a value, use `from_ltrb(left, top, right, bottom)` instead.

```rust
let rect = tiny_skia::Rect::from_xywh(x, y, w, h).unwrap();
pixmap.fill_rect(rect, &paint, Transform::identity(), None);
```

### Create a clipping mask

Clipping restricts all subsequent drawing to the mask region. Use this for the plot area: create a mask matching the plot rectangle, then pass `Some(&mask)` to all mark rendering calls. This prevents data lines from overflowing into the axis/margin area. Create the mask once per render, not per draw call.

```rust
let mut mask = Mask::new(width, height).unwrap();
let clip_rect = PathBuilder::from_rect(
    tiny_skia::Rect::from_ltrb(left, top, right, bottom).unwrap()
);
mask.fill_path(&clip_rect, FillRule::Winding, false, Transform::identity());
// Pass Some(&mask) to drawing methods
```

### Render text

```rust
let mut font_system = FontSystem::new();
let mut cache = SwashCache::new();
let metrics = Metrics::new(font_size, line_height);
let mut buffer = Buffer::new(&mut font_system, metrics);
buffer.set_text(&mut font_system, text, &Attrs::new(), Shaping::Advanced, None);
buffer.set_size(&mut font_system, Some(max_width), Some(max_height));
buffer.shape_until_scroll(&mut font_system, true);

let mut paint = Paint::default();
buffer.draw(&mut font_system, &mut cache, cosmic_text::Color::rgba(0, 0, 0, 255), |x, y, w, h, color| {
    paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
    if let Some(rect) = tiny_skia::Rect::from_xywh(x as f32, y as f32, w as f32, h as f32) {
        pixmap.fill_rect(rect, &paint, Transform::identity(), None);
    }
});
```

---

## Margin computation algorithm

```
1. Create scales from data ranges
2. Run tick algorithm on each scale
3. Format tick labels as strings
4. Measure widest y-tick label: max_ytick_w = max(label_width(tick)) 
5. Measure x-tick label height: xtick_h = font_line_height
6. Compute margins:
   left   = padding + y_label_height + label_gap + max_ytick_w + tick_gap
   bottom = padding + x_label_height + label_gap + xtick_h + tick_gap
   top    = padding + title_height + title_gap (if title exists)
   right  = padding
7. Plot area = figure area minus margins
8. Update scale ranges to match plot area pixel dimensions
9. Re-run tick algorithm if scale ranges changed significantly
```
---

# Part 3 — Navigate

Tree structures and maps. Which file to create, which crate owns a type, how pieces connect.

---

## Layer architecture

```
 ┌─────────────────────────────────────────────────────────────────────────┐
 │                          starsight (facade)                             │
 │  Re-exports everything. The only crate users add to Cargo.toml.         │
 │  pub use starsight_layer_1 through starsight_layer_7                    │
 ├─────────────────────────────────────────────────────────────────────────┤
 │                                                                         │
 │  ┌──────────────────────────┐   ┌───────────────────────────────────┐   │
 │  │  Layer 7                 │   │  Layer 6                          │   │
 │  │  Export & Animation      │   │  Interactivity                    │   │
 │  │                          │   │                                   │   │
 │  │  GIF encoder             │   │  Hover tooltips                   │   │
 │  │  PDF writer (krilla)     │   │  Box zoom, wheel zoom             │   │
 │  │  Interactive HTML        │   │  Click-and-drag pan               │   │
 │  │  Terminal inline         │   │  Lasso & box selection            │   │
 │  │  WASM bridge             │   │  winit event loop                 │   │
 │  └────────────┬─────────────┘   └──────────────────┬────────────────┘   │
 │               │                                    │                    │
 │  ┌────────────┴────────────────────────────────────┴─────────────────┐  │
 │  │  Layer 5: High-level API                                          │  │
 │  │                                                                   │  │
 │  │  Figure builder       plot!() macro       Theme application       │  │
 │  │  Data acceptance: Vec<f64>, &[f64], Polars, ndarray, Arrow        │  │
 │  └───────────────────────────────┬───────────────────────────────────┘  │
 │                                  │                                      │
 │  ┌───────────────────────────────┴───────────────────────────────────┐  │
 │  │  Layer 4: Layout & Composition                                    │  │
 │  │                                                                   │  │
 │  │  GridLayout     FacetWrap     FacetGrid     Legend     Colorbar   │  │
 │  └───────────────────────────────┬───────────────────────────────────┘  │
 │                                  │                                      │
 │  ┌───────────────────────────────┴───────────────────────────────────┐  │
 │  │  Layer 3: Marks & Stats                                           │  │
 │  │                                                                   │  │
 │  │  Marks: Line, Point, Bar, Area, Heatmap, Box, Violin, Pie, ...    │  │
 │  │  Stats: Bin, KDE, Boxplot, Regression, Aggregate                  │  │
 │  └───────────────────────────────┬───────────────────────────────────┘  │
 │                                  │                                      │
 │  ┌───────────────────────────────┴───────────────────────────────────┐  │
 │  │  Layer 2: Scales, Axes, Coordinates                               │  │
 │  │                                                                   │  │
 │  │  Scales: Linear, Log, Symlog, Band, DateTime                      │  │
 │  │  Wilkinson Extended tick algorithm (novel Rust impl)              │  │
 │  │  Axis (scale + ticks + label)                                     │  │
 │  │  CartesianCoord, PolarCoord                                       │  │
 │  └───────────────────────────────┬───────────────────────────────────┘  │
 │                                  │                                      │
 │  ┌───────────────────────────────┴───────────────────────────────────┐  │
 │  │  Layer 1: Primitives, Rendering, Backends                         │  │
 │  │                                                                   │  │
 │  │  Types: Point, Vec2, Rect, Size, Color, Transform                 │  │
 │  │  Error: StarsightError, Result<T>                                 │  │
 │  │  Scene: SceneNode enum (Path, Text, Group, Clip)                  │  │
 │  │                                                                   │  │
 │  │  DrawBackend trait                                                │  │
 │  │  ┌─────────┬─────────┬──────────┬─────────┬────────┬───────────┐  │  │
 │  │  │  Skia   │   SVG   │   wgpu   │   PDF   │ Kitty  │  Braille  │  │  │
 │  │  │  (CPU)  │ (vector)│  (GPU)   │ (krilla)│ (term) │  (term)   │  │  │
 │  │  └─────────┴─────────┴──────────┴─────────┴────────┴───────────┘  │  │
 │  └───────────────────────────────────────────────────────────────────┘  │
 │                                                                         │
 ├─────────────────────────────────────────────────────────────────────────┤
 │  xtask (dev-only, not published)                                        │
 │  Gallery generation, benchmarks, snapshot management                    │
 └─────────────────────────────────────────────────────────────────────────┘

 Rule: each layer depends only on layers below it. Enforced by Cargo.toml.
```

---

## Type flow between layers

```
 User-facing types (accepted by Layer 5)          Internal types (Layer 1)
 ──────────────────────────────────────────────────────────────────────────────

   Vec<f64>   ────┐                               Point  { x: f32, y: f32 }
   &[f64]         │                               Vec2   { x: f32, y: f32 }
   DataFrame      ├── Layer 5 converts ──▶        Rect   { l, t, r, b }
   Array1<f64>    │                               Color  { r, g, b: u8 }
   RecordBatch ───┘                          
							    Transform(tiny_skia::Transform)
                                    PathCommand::MoveTo(Point)
                                    PathCommand::LineTo(Point)
                                    PathCommand::Close
                                    PathStyle { stroke_color, fill, width }
                                    StarsightError { Render | Data | Io | ... }

 Internal types (Layer 2 mapping)
 ─────────────────────────────────────

   LinearScale    { domain: (f64, f64), range: (f64, f64) }
   LogScale       { domain: (f64, f64), range: (f64, f64), base: f64 }
   BandScale      { categories: Vec<String>, range: (f64, f64), padding: f64 }
   Axis           { scale, ticks: Vec<f64>, tick_labels: Vec<String>, label: String }
   CartesianCoord { x_axis: Axis, y_axis: Axis, plot_area: Rect }
```

---

## Which crate do I edit?

| Task | Crate | File |
|---|---|---|
| Primitive types | `starsight-layer-1` | `src/primitives/geom.rs` |
| Color types | `starsight-layer-1` | `src/primitives/color.rs` |
| Error enum | `starsight-layer-1` | `src/error.rs` |
| Skia backend | `starsight-layer-1` | `src/backend/skia/raster.rs` |
| SVG backend | `starsight-layer-1` | `src/backend/svg/mod.rs` |
| New backend | `starsight-layer-1` | `src/backend/<new>/mod.rs` |
| Scale types | `starsight-layer-2` | `src/scale.rs` |
| Tick algorithm | `starsight-layer-2` | `src/tick.rs` |
| Axis rendering | `starsight-layer-2` | `src/axis.rs` |
| Coordinate mapping | `starsight-layer-2` | `src/coord.rs` |
| Existing mark | `starsight-layer-3` | `src/marks/<n>.rs` |
| New chart type | `starsight-layer-3` | `src/marks/<n>.rs` (new file) |
| Stat transform | `starsight-layer-3` | `src/stats/<n>.rs` |
| Layout / faceting | `starsight-layer-4` | `src/grid.rs` or `src/facet.rs` |
| Legend / colorbar | `starsight-layer-4` | `src/legend.rs` or `src/colorbar.rs` |
| Figure builder | `starsight-layer-5` | `src/figure.rs` |
| plot! macro | `starsight-layer-5` | `src/macros.rs` |
| Polars integration | `starsight-layer-5` | `src/data/polars.rs` |
| Interactivity | `starsight-layer-6` | `src/window.rs` |
| Terminal display | `starsight-layer-7` | `src/terminal.rs` |
| Example program | root workspace | `examples/<n>.rs` |
| CI workflow | root workspace | `.github/workflows/ci.yml` |
| Gallery generation | `xtask` | `src/main.rs` |

---

## Examples directory

```
 examples/
 ├── quickstart.rs            3-line plot!() chart
 ├── line_chart.rs            line with title and axis labels
 ├── scatter.rs               scatter with color grouping
 ├── bar_chart.rs             grouped bar chart
 ├── histogram.rs             histogram with KDE overlay
 ├── heatmap.rs               annotated heatmap with colorbar
 ├── statistical.rs           box plot + violin side by side
 ├── faceting.rs              FacetWrap small multiples
 ├── custom_theme.rs          chromata theme application
 ├── custom_colormap.rs       prismatica colormap on heatmap
 ├── terminal.rs              inline terminal rendering
 ├── interactive.rs           windowed chart with hover/zoom/pan
 ├── polars_integration.rs    chart from a Polars DataFrame
 └── surface3d.rs             3D surface plot (feature "3d")
```

Each example is self-contained: `cargo run --example scatter` produces a PNG.

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
├── Cargo.toml  [exists]  workspace root
├── .spec/STARSIGHT.md  [exists]  this document
├── LICENSE  [exists]
├── README.md  [exists]
├── CONTRIBUTING.md  [exists]
├── CHANGELOG.md  [exists]
├── CODE_OF_CONDUCT.md  [exists]
├── SECURITY.md  [exists]
├── .clippy.toml  [exists]
├── .rustfmt.toml  [exists]
├── deny.toml  [exists]
│
├── .github/
│   ├── FUNDING.yml  [exists]
│   ├── PULL_REQUEST_TEMPLATE.md  [exists]
│   ├── ISSUE_TEMPLATE/  [exists]
│   └── workflows/
│       ├── ci.yml  [exists]
│       ├── release.yml  [exists]
│       ├── coverage.yml  [exists]  cargo-llvm-cov, codecov
│       ├── snapshots.yml  [exists]
│       └── gallery.yml  [exists]
│
├── starsight/                          FACADE CRATE
│   ├── Cargo.toml  [exists]  depends on all layers
│   └── src/
│       ├── lib.rs  [exists]  re-exports
│       └── prelude.rs  [exists]  pub use of primary types
│
├── starsight-layer-1/                  RENDERING + PRIMITIVES + ERROR
│   ├── Cargo.toml  [exists]  deps: tiny-skia, thiserror
│   └── src/
│       ├── lib.rs  [exists]
│       ├── error.rs  [exists]
│       ├── primitives.rs  [exists]
│       │  [target]
│       │  [target]
│       ├── scene.rs  [target]  SceneNode enum, Scene struct
│       └── backend/
│           ├── mod.rs  [exists]  DrawBackend trait (partial)
│           │  [target]  uncomment all methods
│           ├── skia/
│           │   ├── mod.rs  [exists]  sub-module declarations
│           │   ├── raster/mod.rs  [exists]
│           │   ├── headless/mod.rs  [exists]
│           │   └── png/mod.rs  [exists]
│           ├── svg/
│           │   └── mod.rs  [exists]
│           ├── pdf/
│           │   └── mod.rs  [exists]  empty — PDF backend (0.10.0)
│           ├── wgpu/
│           │   ├── mod.rs  [exists]  sub-module declarations
│           │   ├── native/mod.rs  [exists]
│           │   └── web/mod.rs  [exists]  empty — WASM WebGPU (0.10.0)
│           └── terminal/
│               ├── mod.rs  [exists]  sub-module declarations
│               ├── kitty/mod.rs  [exists]
│               ├── sixel/mod.rs  [exists]
│               ├── iterm2/mod.rs  [exists]
│               ├── half_block/mod.rs  [exists]
│               └── braille/mod.rs  [exists]  empty — Braille dots (0.8.0)
│
├── starsight-layer-2/                  SCALES + AXES + COORDINATES
│   ├── Cargo.toml  [exists]  deps: starsight-layer-1
│   └── src/
│       ├── lib.rs  [exists]
│       ├── scale.rs  [target]
│       ├── tick.rs  [target]  Wilkinson Extended algorithm
│       ├── axis.rs  [target]
│       └── coord.rs  [target]
│
├── starsight-layer-3/                  MARKS + STATS + AESTHETICS
│   ├── Cargo.toml  [exists]  deps: layer-1, layer-2
│   └── src/
│       ├── lib.rs  [exists]
│       ├── mark.rs  [target]  Mark trait
│       ├── line.rs  [target]  LineMark
│       ├── point.rs  [target]  PointMark
│       ├── bar.rs  [target]  BarMark (0.2.0)
│       ├── area.rs  [target]  AreaMark (0.2.0)
│       ├── aes.rs  [target]  Aesthetic mapping types
│       ├── position.rs  [target]
│       └── stat/
│           ├── mod.rs  [target]  stat module
│           ├── bin.rs  [target]  Histogram binning (0.2.0)
│           └── kde.rs  [target]
│
├── starsight-layer-4/                  LAYOUT + FACETING + LEGENDS
│   ├── Cargo.toml  [exists]
│   └── src/
│       ├── lib.rs  [exists]
│       ├── grid.rs  [target]  GridLayout (0.4.0)
│       ├── facet.rs  [target]  FacetWrap, FacetGrid (0.4.0)
│       ├── legend.rs  [target]  Legend (0.4.0)
│       └── colorbar.rs  [target]  Colorbar (0.4.0)
│
├── starsight-layer-5/                  HIGH-LEVEL API
│   ├── Cargo.toml  [exists]  deps: layer-1 through layer-4
│   └── src/
│       ├── lib.rs  [exists]
│       ├── figure.rs  [target]  Figure struct + builder
│       ├── macro.rs  [target]  plot!() macro
│       ├── auto.rs  [target]  chart type auto-inference
│       └── data/
│           ├── mod.rs  [target]  DataSource trait
│           ├── raw.rs  [target]  Vec/slice acceptance
│           ├── polars.rs  [target]  DataFrame acceptance (0.3.0)
│           ├── ndarray.rs  [target]  ndarray acceptance (0.11.0)
│           └── arrow.rs  [target]  Arrow acceptance (0.11.0)
│
├── starsight-layer-6/                  INTERACTIVITY
│   ├── Cargo.toml  [exists]  deps: layer-1 through layer-5
│   └── src/
│       ├── lib.rs  [exists]  empty — all 0.6.0+
│       ├── hover.rs  [target]  tooltips (0.6.0)
│       ├── zoom.rs  [target]  box/wheel zoom (0.6.0)
│       ├── pan.rs  [target]  drag pan (0.6.0)
│       ├── select.rs  [target]  box/lasso selection (0.6.0)
│       └── stream.rs  [target]  streaming data (0.6.0)
│
├── starsight-layer-7/                  ANIMATION + EXPORT
│   ├── Cargo.toml  [exists]  deps: layer-1 through layer-6
│   └── src/
│       ├── lib.rs  [exists]  empty — all 0.7.0+
│       ├── animation.rs  [target]  frame recording (0.10.0)
│       ├── pdf.rs  [target]  PDF export (0.10.0)
│       ├── html.rs  [target]  interactive HTML (0.10.0)
│       └── terminal.rs  [target]
│
├── examples/
│   ├── quickstart.rs  [exists]
│   ├── scatter.rs  [exists]  empty
│   ├── statistical.rs  [exists]  empty
│   ├── surface3d.rs  [exists]  empty
│   ├── terminal.rs  [exists]  empty
│   ├── interactive.rs  [exists]  empty
│   ├── polars_integration.rs  [exists]  empty
│   ├── streaming.rs  [exists]  empty
│   ├── faceting.rs  [exists]  empty
│   ├── custom_theme.rs  [exists]  empty
│   ├── recipe.rs  [exists]  empty
│   └── gallery.rs  [exists]  empty
│
└── xtask/
    ├── Cargo.toml  [exists]
    └── src/main.rs  [exists]  empty main
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

## Common operations

| Operation | Command |
|---|---|
| Check everything compiles | `cargo check --workspace` |
| Run all tests | `cargo test --workspace` |
| Run tests for one crate | `cargo test -p starsight-layer-2` |
| Run with all features | `cargo test --workspace --all-features` |
| Format code | `cargo fmt --all` |
| Lint | `cargo clippy --workspace --all-targets` |
| License check | `cargo deny check` |
| Review snapshot changes | `cargo insta review` |
| Generate gallery | `cargo xtask gallery` |
| Check MSRV | `cargo +1.85.0 check --workspace` |

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

