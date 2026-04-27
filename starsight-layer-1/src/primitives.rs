//! Primitive types: colors, geometry, and transforms.
//!
//! Every other layer depends on these types. They are intentionally `Copy` where
//! possible (`Point`, `Vec2`, `Color`, `Transform`, ...) so they can be passed
//! by value without lifetime concerns.
//!
//! Distinctions encoded in the type system:
//! - `Point` is a position; `Vec2` is a displacement. `Point - Point = Vec2` and
//!   `Point + Vec2 = Point`, but `Point + Point` does not compile.
//! - `Color` is opaque; [`ColorAlpha`] carries straight (non-premultiplied) alpha.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::float_cmp,
    clippy::many_single_char_names
)]

// ── Color ────────────────────────────────────────────────────────────────────────────────────────

/// 8-bit RGB color.
///
/// Use [`ColorAlpha`] when transparency matters. Conversions to/from `tiny_skia`
/// and the sister crates `chromata`/`prismatica` are provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red channel, 0–255.
    pub r: u8,
    /// Green channel, 0–255.
    pub g: u8,
    /// Blue channel, 0–255.
    pub b: u8,
}

impl Color {
    /// Construct from individual 8-bit channels.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Construct from a packed 24-bit hex literal like `0x9634AD`.
    #[must_use]
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }

    /// Pure black: `(0, 0, 0)`.
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    /// Pure white: `(255, 255, 255)`.
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
    };
    /// Saturated red: `(255, 0, 0)`.
    pub const RED: Self = Self { r: 255, g: 0, b: 0 };
    /// Saturated green: `(0, 255, 0)`.
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    /// Saturated blue: `(0, 0, 255)`.
    pub const BLUE: Self = Self { r: 0, g: 0, b: 255 };

    /// Convert to a normalized `(r, g, b)` triplet in `[0, 1]`.
    #[must_use]
    pub const fn to_f32(self) -> (f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }

    /// Construct from normalized floats. Out-of-range channels are clamped.
    #[must_use]
    pub fn from_f32(r: f32, g: f32, b: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        }
    }

    /// Convert to the `tiny_skia` color type used by the raster backend.
    #[must_use]
    pub fn to_tiny_skia(self) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba8(self.r, self.g, self.b, 255)
    }

    /// Promote to [`ColorAlpha`] with the given alpha channel.
    #[must_use]
    pub fn with_alpha(self, a: u8) -> ColorAlpha {
        ColorAlpha {
            r: self.r,
            g: self.g,
            b: self.b,
            a,
        }
    }

    /// Parse a CSS-style hex string: `"#9634AD"`, `"9634AD"`, `"#abc"`, or `"abc"`.
    #[must_use]
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
                Some(Self {
                    r: r << 4 | r,
                    g: g << 4 | g,
                    b: b << 4 | b,
                })
            }
            _ => None,
        }
    }

    /// Format as a CSS hex string: `#rrggbb`.
    #[must_use]
    pub fn to_css_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// WCAG relative luminance in `[0, 1]`.
    #[must_use]
    pub fn luminance(self) -> f64 {
        fn linearize(c: f64) -> f64 {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        let r = linearize(f64::from(self.r) / 255.0);
        let g = linearize(f64::from(self.g) / 255.0);
        let b = linearize(f64::from(self.b) / 255.0);
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// WCAG contrast ratio against another color, in `[1, 21]`.
    #[must_use]
    pub fn contrast_ratio(self, other: Color) -> f64 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Linear interpolation between `self` and `other` at fraction `t ∈ [0, 1]`.
    #[must_use]
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: (f32::from(self.r) + (f32::from(other.r) - f32::from(self.r)) * t) as u8,
            g: (f32::from(self.g) + (f32::from(other.g) - f32::from(self.g)) * t) as u8,
            b: (f32::from(self.b) + (f32::from(other.b) - f32::from(self.b)) * t) as u8,
        }
    }
}

impl From<chromata::Color> for Color {
    fn from(c: chromata::Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
        }
    }
}

impl From<prismatica::Color> for Color {
    fn from(c: prismatica::Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
        }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

// ── ColorAlpha ───────────────────────────────────────────────────────────────────────────────────

/// 8-bit RGBA color with **straight** (non-premultiplied) alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorAlpha {
    /// Red channel, 0–255.
    pub r: u8,
    /// Green channel, 0–255.
    pub g: u8,
    /// Blue channel, 0–255.
    pub b: u8,
    /// Alpha channel, 0–255 (0 = transparent, 255 = opaque).
    pub a: u8,
}

impl ColorAlpha {
    /// Construct from individual 8-bit channels.
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Construct from a packed 32-bit `0xAARRGGBB` hex literal.
    #[must_use]
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: (hex >> 24) as u8,
        }
    }

    /// Opaque black.
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Opaque white.
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    /// Opaque red.
    pub const RED: Self = Self {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    /// Opaque green.
    pub const GREEN: Self = Self {
        r: 0,
        g: 255,
        b: 0,
        a: 255,
    };
    /// Opaque blue.
    pub const BLUE: Self = Self {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };

    /// Convert RGB to a normalized `(r, g, b)` triplet (alpha is dropped).
    #[must_use]
    pub const fn to_f32(self) -> (f32, f32, f32) {
        (
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }

    /// Construct from normalized floats. Out-of-range channels are clamped.
    #[must_use]
    pub fn from_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            a: (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        }
    }

    /// Convert to the `tiny_skia` color type, preserving alpha.
    #[must_use]
    pub fn to_tiny_skia(self) -> tiny_skia::Color {
        tiny_skia::Color::from_rgba8(self.r, self.g, self.b, self.a)
    }

    /// Strip the alpha channel, returning a plain [`Color`].
    #[must_use]
    pub fn without_alpha(self) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

// ── Point ────────────────────────────────────────────────────────────────────────────────────────

/// 2D position in pixel space.
///
/// `Point` represents a *location*. Use [`Vec2`] for *displacements*. Subtracting
/// two `Point`s yields a `Vec2`; adding a `Vec2` to a `Point` yields a `Point`.
/// `Point + Point` is intentionally not implemented.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    /// X coordinate (rightward).
    pub x: f32,
    /// Y coordinate (downward — pixel coordinates, not math convention).
    pub y: f32,
}

impl Point {
    /// Origin: `(0, 0)`.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    /// Unit X: `(1, 0)`.
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    /// Unit Y: `(0, 1)`.
    pub const Y: Self = Self { x: 0.0, y: 1.0 };

    /// Construct from individual coordinates.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<tiny_skia::Point> for Point {
    fn from(value: tiny_skia::Point) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<[f32; 2]> for Point {
    fn from([x, y]: [f32; 2]) -> Self {
        Self { x, y }
    }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<Point> for [f32; 2] {
    fn from(p: Point) -> Self {
        [p.x, p.y]
    }
}

impl From<Point> for (f32, f32) {
    fn from(p: Point) -> Self {
        (p.x, p.y)
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point({}, {})", self.x, self.y)
    }
}

// ── Vec2 ─────────────────────────────────────────────────────────────────────────────────────────

/// 2D displacement (the difference between two `Point`s).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
}

impl Vec2 {
    /// Zero displacement.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    /// Unit X.
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    /// Unit Y.
    pub const Y: Self = Self { x: 0.0, y: 1.0 };

    /// Construct from individual components.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Euclidean length: `√(x² + y²)`.
    #[must_use]
    pub fn length(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Unit vector in the same direction. Returns [`Vec2::ZERO`] if `self` is zero.
    #[must_use]
    pub fn normalize(self) -> Self {
        let len = self.length();
        if len == 0.0 {
            Self::ZERO
        } else {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        }
    }
}

impl std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vec2({}, {})", self.x, self.y)
    }
}

// ── Point/Vec2 arithmetic ────────────────────────────────────────────────────────────────────────

impl std::ops::Sub for Point {
    type Output = Vec2;
    fn sub(self, rhs: Point) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add<Vec2> for Point {
    type Output = Point;
    fn add(self, rhs: Vec2) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Vec2> for Point {
    type Output = Point;
    fn sub(self, rhs: Vec2) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;
    fn mul(self, rhs: f32) -> Vec2 {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Mul<Vec2> for f32 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

impl std::ops::Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

// ── Rect ─────────────────────────────────────────────────────────────────────────────────────────

/// Axis-aligned rectangle in left/top/right/bottom form.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Left edge (smallest x).
    pub left: f32,
    /// Top edge (smallest y).
    pub top: f32,
    /// Right edge (largest x).
    pub right: f32,
    /// Bottom edge (largest y).
    pub bottom: f32,
}

impl Rect {
    /// Construct from explicit ltrb edges.
    #[must_use]
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Construct from origin and size.
    #[must_use]
    pub fn from_xywh(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        }
    }

    /// Construct from a center point and a size.
    #[must_use]
    pub fn from_center_size(center: Point, size: Size) -> Self {
        let half_w = size.width * 0.5;
        let half_h = size.height * 0.5;
        Self {
            left: center.x - half_w,
            top: center.y - half_h,
            right: center.x + half_w,
            bottom: center.y + half_h,
        }
    }

    /// Width: `right - left`.
    #[must_use]
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    /// Height: `bottom - top`.
    #[must_use]
    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    /// Width and height as a [`Size`].
    #[must_use]
    pub fn size(&self) -> Size {
        Size::new(self.width(), self.height())
    }

    /// Center point.
    #[must_use]
    pub fn center(&self) -> Point {
        Point::new(
            (self.left + self.right) * 0.5,
            (self.top + self.bottom) * 0.5,
        )
    }

    /// Top-left corner.
    #[must_use]
    pub fn top_left(&self) -> Point {
        Point::new(self.left, self.top)
    }

    /// Bottom-right corner.
    #[must_use]
    pub fn bottom_right(&self) -> Point {
        Point::new(self.right, self.bottom)
    }

    /// Whether `p` is inside `self` (inclusive on all edges).
    #[must_use]
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.left && p.x <= self.right && p.y >= self.top && p.y <= self.bottom
    }

    /// Intersection with another rect, or `None` if they do not overlap.
    #[must_use]
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let r = Rect {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        };
        if r.left < r.right && r.top < r.bottom {
            Some(r)
        } else {
            None
        }
    }

    /// Expand by `amount` pixels on all sides.
    #[must_use]
    pub fn pad(&self, amount: f32) -> Rect {
        Rect {
            left: self.left - amount,
            top: self.top - amount,
            right: self.right + amount,
            bottom: self.bottom + amount,
        }
    }

    /// Convert to a `tiny_skia::Rect`. Returns `None` if `left >= right` or `top >= bottom`.
    #[must_use]
    pub fn to_tiny_skia(&self) -> Option<tiny_skia::Rect> {
        tiny_skia::Rect::from_ltrb(self.left, self.top, self.right, self.bottom)
    }
}

impl From<tiny_skia::Rect> for Rect {
    fn from(value: tiny_skia::Rect) -> Self {
        Self {
            left: value.left(),
            top: value.top(),
            right: value.right(),
            bottom: value.bottom(),
        }
    }
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rect({}, {}, {}, {})",
            self.left, self.top, self.right, self.bottom
        )
    }
}

// ── Size ─────────────────────────────────────────────────────────────────────────────────────────

/// Width-and-height pair (no position).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
}

impl Size {
    /// Construct from width and height.
    #[must_use]
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl From<tiny_skia::Size> for Size {
    fn from(value: tiny_skia::Size) -> Self {
        Self {
            width: value.width(),
            height: value.height(),
        }
    }
}

impl std::fmt::Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Size({}, {})", self.width, self.height)
    }
}

// ── Transform ────────────────────────────────────────────────────────────────────────────────────

/// Affine 2D transform: opaque newtype around `tiny_skia::Transform`.
///
/// Note: tiny-skia takes **degrees**, not radians, for rotations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform(pub(crate) tiny_skia::Transform);

impl Transform {
    /// Identity transform (no translation, no rotation, no scale).
    #[must_use]
    pub fn identity() -> Self {
        Self(tiny_skia::Transform::identity())
    }

    /// Pure translation by `(dx, dy)`.
    #[must_use]
    pub fn translate(dx: f32, dy: f32) -> Self {
        Self(tiny_skia::Transform::from_translate(dx, dy))
    }

    /// Pure scale by `(sx, sy)`.
    #[must_use]
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self(tiny_skia::Transform::from_scale(sx, sy))
    }

    /// Pure rotation. Note: tiny-skia takes **degrees**, not radians.
    #[must_use]
    pub fn rotate_degrees(angle: f32) -> Self {
        Self(tiny_skia::Transform::from_rotate(angle))
    }

    /// Compose: apply `other` after `self`.
    #[must_use]
    pub fn then(self, other: Transform) -> Self {
        Self(self.0.post_concat(other.0))
    }

    /// Translate before applying the existing transform.
    #[must_use]
    pub fn pre_translate(self, dx: f32, dy: f32) -> Self {
        Self(self.0.pre_translate(dx, dy))
    }
}

impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transform({}, {}, {}, {}, {}, {})",
            self.0.sx, self.0.sy, self.0.kx, self.0.ky, self.0.tx, self.0.ty
        )
    }
}

// ── tests ────────────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{Color, ColorAlpha, Point, Rect, Size, Transform, Vec2};

    #[test]
    fn from_hex_roundtrip() {
        let hex = 9_843_885_u32;
        let color = Color::from_hex(hex);
        assert_eq!(150, color.r);
        assert_eq!(52, color.g);
        assert_eq!(173, color.b);
    }

    #[test]
    fn from_css_hex_roundtrip() {
        let hex = "#9634AD";
        let color = Color::from_css_hex(hex).unwrap();
        assert_eq!(150, color.r);
        assert_eq!(52, color.g);
        assert_eq!(173, color.b);
    }

    #[test]
    fn from_css_hex_short() {
        let hex = "#abc";
        let color = Color::from_css_hex(hex).unwrap();
        assert_eq!(0xAA, color.r);
    }

    #[test]
    fn from_css_hex_invalid() {
        assert!(Color::from_css_hex("").is_none());
        assert!(Color::from_css_hex("#gggggg").is_none());
    }

    #[test]
    fn color_to_f32() {
        let color = Color::new(128, 128, 128);
        let (r, _g, _b) = color.to_f32();
        assert!((r - 0.502).abs() < 0.01);
    }

    #[test]
    fn color_from_f32() {
        let color = Color::from_f32(0.5, 0.5, 0.5);
        assert!(color.r >= 127 && color.r <= 129);
    }

    #[test]
    fn color_from_f32_clamp() {
        let color = Color::from_f32(1.5, -0.5, 0.5);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
    }

    #[test]
    fn color_to_tiny_skia() {
        let color = Color::new(255, 0, 0);
        let ts_color = color.to_tiny_skia();
        assert_eq!(ts_color.red(), 1.0);
    }

    #[test]
    fn color_with_alpha() {
        let color = Color::new(255, 0, 0);
        let alpha = color.with_alpha(128);
        assert_eq!(alpha.r, 255);
        assert_eq!(alpha.a, 128);
    }

    #[test]
    fn color_to_css_hex() {
        let color = Color::new(150, 52, 173);
        assert_eq!(color.to_css_hex(), "#9634ad");
    }

    #[test]
    fn contrast_ratio_same() {
        let c = Color::new(128, 128, 128);
        assert!((c.contrast_ratio(c) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn color_lerp_mid() {
        let (b, w) = (Color::BLACK, Color::WHITE);
        let gray = b.lerp(w, 0.5);
        assert!(gray.r >= 125 && gray.r <= 130);
    }

    #[test]
    fn color_red() {
        let c = Color::RED;
        assert_eq!(c.r, 255);
    }

    #[test]
    fn color_green() {
        let c = Color::GREEN;
        assert_eq!(c.g, 255);
    }

    #[test]
    fn color_blue() {
        let c = Color::BLUE;
        assert_eq!(c.b, 255);
    }

    #[test]
    fn color_display() {
        let color = Color::new(150, 52, 173);
        let s = format!("{}", color);
        assert!(s.starts_with('#'));
    }

    #[test]
    fn from_chromata_color() {
        let c = chromata::Color { r: 255, g: 0, b: 0 };
        let color = Color::from(c);
        assert_eq!(color.r, 255);
    }

    #[test]
    fn from_prismatica_color() {
        let c = prismatica::Color { r: 255, g: 0, b: 0 };
        let color = Color::from(c);
        assert_eq!(color.r, 255);
    }

    #[test]
    fn black_luminance() {
        assert!(Color::BLACK.luminance() < f64::EPSILON);
    }

    #[test]
    fn white_luminance() {
        assert!((Color::WHITE.luminance() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn black_white_contrast_ratio() {
        assert_eq!(Color::BLACK.contrast_ratio(Color::WHITE), 21.0);
    }

    #[test]
    fn lerp_roundtrip() {
        let (b, w) = (Color::BLACK, Color::WHITE);
        assert_eq!(b.lerp(w, 0.0), b);
        assert_eq!(b.lerp(w, 1.0), w);
    }

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

    // ── ColorAlpha ───────────────────────────────────────────────────────────────────────────────

    #[test]
    fn color_alpha_new() {
        let c = ColorAlpha::new(10, 20, 30, 40);
        assert_eq!(c.r, 10);
        assert_eq!(c.g, 20);
        assert_eq!(c.b, 30);
        assert_eq!(c.a, 40);
    }

    #[test]
    fn color_alpha_from_hex() {
        let c = ColorAlpha::from_hex(0x80_FF_00_00);
        assert_eq!(c.a, 0x80);
        assert_eq!(c.r, 0xFF);
        assert_eq!(c.g, 0x00);
        assert_eq!(c.b, 0x00);
    }

    #[test]
    fn color_alpha_constants() {
        assert_eq!(ColorAlpha::BLACK.r, 0);
        assert_eq!(ColorAlpha::BLACK.a, 255);
        assert_eq!(ColorAlpha::WHITE.r, 255);
        assert_eq!(ColorAlpha::WHITE.a, 255);
        assert_eq!(ColorAlpha::RED.r, 255);
        assert_eq!(ColorAlpha::RED.g, 0);
        assert_eq!(ColorAlpha::GREEN.g, 255);
        assert_eq!(ColorAlpha::BLUE.b, 255);
    }

    #[test]
    fn color_alpha_to_f32() {
        let c = ColorAlpha::new(255, 0, 0, 128);
        let (r, g, b) = c.to_f32();
        assert!((r - 1.0).abs() < f32::EPSILON);
        assert!(g.abs() < f32::EPSILON);
        assert!(b.abs() < f32::EPSILON);
    }

    #[test]
    fn color_alpha_from_f32() {
        let c = ColorAlpha::from_f32(1.0, 0.5, 0.0, 0.5);
        assert_eq!(c.r, 255);
        assert!(c.g >= 127 && c.g <= 129);
        assert_eq!(c.b, 0);
        assert!(c.a >= 127 && c.a <= 129);
    }

    #[test]
    fn color_alpha_from_f32_clamp() {
        let c = ColorAlpha::from_f32(2.0, -1.0, 0.5, 1.5);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn color_alpha_to_tiny_skia() {
        let c = ColorAlpha::new(255, 0, 0, 128);
        let ts = c.to_tiny_skia();
        assert_eq!(ts.red(), 1.0);
    }

    #[test]
    fn color_alpha_without_alpha() {
        let c = ColorAlpha::new(10, 20, 30, 128);
        let plain = c.without_alpha();
        assert_eq!(plain.r, 10);
        assert_eq!(plain.g, 20);
        assert_eq!(plain.b, 30);
    }

    // ── Point ────────────────────────────────────────────────────────────────────────────────────

    #[test]
    fn point_constants() {
        assert_eq!(Point::ZERO, Point::new(0.0, 0.0));
        assert_eq!(Point::X, Point::new(1.0, 0.0));
        assert_eq!(Point::Y, Point::new(0.0, 1.0));
    }

    #[test]
    fn point_from_array() {
        let p: Point = [3.0_f32, 4.0_f32].into();
        assert_eq!(p, Point::new(3.0, 4.0));
    }

    #[test]
    fn point_from_tuple() {
        let p: Point = (3.0_f32, 4.0_f32).into();
        assert_eq!(p, Point::new(3.0, 4.0));
    }

    #[test]
    fn point_into_array() {
        let arr: [f32; 2] = Point::new(3.0, 4.0).into();
        assert_eq!(arr, [3.0, 4.0]);
    }

    #[test]
    fn point_into_tuple() {
        let tup: (f32, f32) = Point::new(3.0, 4.0).into();
        assert_eq!(tup, (3.0, 4.0));
    }

    #[test]
    fn point_from_tiny_skia() {
        let ts = tiny_skia::Point::from_xy(3.0, 4.0);
        let p: Point = ts.into();
        assert_eq!(p, Point::new(3.0, 4.0));
    }

    #[test]
    fn point_display() {
        let s = format!("{}", Point::new(3.0, 4.0));
        assert!(s.contains('3'));
        assert!(s.contains('4'));
    }

    // ── Vec2 ─────────────────────────────────────────────────────────────────────────────────────

    #[test]
    fn vec2_constants() {
        assert_eq!(Vec2::ZERO, Vec2::new(0.0, 0.0));
        assert_eq!(Vec2::X, Vec2::new(1.0, 0.0));
        assert_eq!(Vec2::Y, Vec2::new(0.0, 1.0));
    }

    #[test]
    fn vec2_normalize_unit() {
        let v = Vec2::new(3.0, 4.0).normalize();
        assert!((v.length() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn vec2_normalize_zero() {
        assert_eq!(Vec2::ZERO.normalize(), Vec2::ZERO);
    }

    #[test]
    fn vec2_display() {
        let s = format!("{}", Vec2::new(3.0, 4.0));
        assert!(s.contains('3'));
    }

    #[test]
    fn vec2_add() {
        assert_eq!(
            Vec2::new(1.0, 2.0) + Vec2::new(3.0, 4.0),
            Vec2::new(4.0, 6.0)
        );
    }

    #[test]
    fn vec2_sub() {
        assert_eq!(
            Vec2::new(5.0, 7.0) - Vec2::new(3.0, 4.0),
            Vec2::new(2.0, 3.0)
        );
    }

    #[test]
    fn vec2_neg() {
        assert_eq!(-Vec2::new(3.0, 4.0), Vec2::new(-3.0, -4.0));
    }

    #[test]
    fn point_minus_vec2_is_point() {
        let p = Point::new(10.0, 20.0);
        let v = Vec2::new(3.0, 4.0);
        let result: Point = p - v;
        assert_eq!(result, Point::new(7.0, 16.0));
    }

    // ── Rect ─────────────────────────────────────────────────────────────────────────────────────

    #[test]
    fn rect_new() {
        let r = Rect::new(1.0, 2.0, 5.0, 6.0);
        assert_eq!(r.left, 1.0);
        assert_eq!(r.top, 2.0);
        assert_eq!(r.right, 5.0);
        assert_eq!(r.bottom, 6.0);
    }

    #[test]
    fn rect_from_xywh() {
        let r = Rect::from_xywh(1.0, 2.0, 4.0, 6.0);
        assert_eq!(r.left, 1.0);
        assert_eq!(r.top, 2.0);
        assert_eq!(r.right, 5.0);
        assert_eq!(r.bottom, 8.0);
    }

    #[test]
    fn rect_from_center_size() {
        let r = Rect::from_center_size(Point::new(10.0, 10.0), Size::new(4.0, 6.0));
        assert_eq!(r.left, 8.0);
        assert_eq!(r.top, 7.0);
        assert_eq!(r.right, 12.0);
        assert_eq!(r.bottom, 13.0);
    }

    #[test]
    fn rect_dimensions() {
        let r = Rect::new(1.0, 2.0, 5.0, 6.0);
        assert_eq!(r.width(), 4.0);
        assert_eq!(r.height(), 4.0);
        assert_eq!(r.size(), Size::new(4.0, 4.0));
        assert_eq!(r.center(), Point::new(3.0, 4.0));
        assert_eq!(r.top_left(), Point::new(1.0, 2.0));
        assert_eq!(r.bottom_right(), Point::new(5.0, 6.0));
    }

    #[test]
    fn rect_contains() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        assert!(r.contains(Point::new(5.0, 5.0)));
        assert!(r.contains(Point::new(0.0, 0.0)));
        assert!(r.contains(Point::new(10.0, 10.0)));
        assert!(!r.contains(Point::new(-1.0, 5.0)));
        assert!(!r.contains(Point::new(11.0, 5.0)));
        assert!(!r.contains(Point::new(5.0, -1.0)));
        assert!(!r.contains(Point::new(5.0, 11.0)));
    }

    #[test]
    fn rect_intersection_overlap() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(5.0, 5.0, 15.0, 15.0);
        let inter = a.intersection(&b).unwrap();
        assert_eq!(inter, Rect::new(5.0, 5.0, 10.0, 10.0));
    }

    #[test]
    fn rect_intersection_disjoint() {
        let a = Rect::new(0.0, 0.0, 5.0, 5.0);
        let b = Rect::new(10.0, 10.0, 20.0, 20.0);
        assert!(a.intersection(&b).is_none());
    }

    #[test]
    fn rect_pad() {
        let r = Rect::new(5.0, 5.0, 10.0, 10.0).pad(2.0);
        assert_eq!(r, Rect::new(3.0, 3.0, 12.0, 12.0));
    }

    #[test]
    fn rect_to_tiny_skia() {
        let r = Rect::new(1.0, 2.0, 5.0, 6.0);
        let ts = r.to_tiny_skia().unwrap();
        assert_eq!(ts.left(), 1.0);
        assert_eq!(ts.right(), 5.0);
    }

    #[test]
    fn rect_to_tiny_skia_invalid() {
        let r = Rect::new(5.0, 5.0, 1.0, 1.0);
        assert!(r.to_tiny_skia().is_none());
    }

    #[test]
    fn rect_from_tiny_skia() {
        let ts = tiny_skia::Rect::from_ltrb(1.0, 2.0, 5.0, 6.0).unwrap();
        let r: Rect = ts.into();
        assert_eq!(r, Rect::new(1.0, 2.0, 5.0, 6.0));
    }

    #[test]
    fn rect_display() {
        let r = Rect::new(1.0, 2.0, 5.0, 6.0);
        let s = format!("{}", r);
        assert!(s.starts_with("Rect("));
    }

    // ── Size ─────────────────────────────────────────────────────────────────────────────────────

    #[test]
    fn size_new() {
        let s = Size::new(10.0, 20.0);
        assert_eq!(s.width, 10.0);
        assert_eq!(s.height, 20.0);
    }

    #[test]
    fn size_from_tiny_skia() {
        let ts = tiny_skia::Size::from_wh(10.0, 20.0).unwrap();
        let s: Size = ts.into();
        assert_eq!(s, Size::new(10.0, 20.0));
    }

    #[test]
    fn size_display() {
        let s = format!("{}", Size::new(10.0, 20.0));
        assert!(s.contains("10"));
        assert!(s.contains("20"));
    }

    // ── Transform ────────────────────────────────────────────────────────────────────────────────

    #[test]
    fn transform_identity() {
        let t = Transform::identity();
        assert_eq!(t.0.sx, 1.0);
        assert_eq!(t.0.sy, 1.0);
    }

    #[test]
    fn transform_translate() {
        let t = Transform::translate(10.0, 20.0);
        assert_eq!(t.0.tx, 10.0);
        assert_eq!(t.0.ty, 20.0);
    }

    #[test]
    fn transform_scale() {
        let t = Transform::scale(2.0, 3.0);
        assert_eq!(t.0.sx, 2.0);
        assert_eq!(t.0.sy, 3.0);
    }

    #[test]
    fn transform_rotate_degrees() {
        let t = Transform::rotate_degrees(90.0);
        // 90° rotation: cos→0, sin→1. sx and sy go near zero; kx/ky absolute go near 1.
        assert!(t.0.sx.abs() < 1e-6);
        assert!(t.0.sy.abs() < 1e-6);
        assert!((t.0.kx.abs() - 1.0).abs() < 1e-6 || (t.0.ky.abs() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn transform_then() {
        let a = Transform::translate(10.0, 0.0);
        let b = Transform::translate(0.0, 20.0);
        let c = a.then(b);
        assert_eq!(c.0.tx, 10.0);
        assert_eq!(c.0.ty, 20.0);
    }

    #[test]
    fn transform_pre_translate() {
        let t = Transform::scale(2.0, 2.0).pre_translate(5.0, 5.0);
        assert_eq!(t.0.tx, 10.0);
        assert_eq!(t.0.ty, 10.0);
    }

    #[test]
    fn transform_display() {
        let t = Transform::identity();
        let s = format!("{}", t);
        assert!(s.starts_with("Transform("));
    }
}
