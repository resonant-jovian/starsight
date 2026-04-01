//!
pub type NonZeroF32 = std::num::NonZero<f32>; //Maybe? or https://docs.rs/strict-num/latest/strict_num/ or skia
///
pub struct Color {
    ///
    r: u8,
    ///
    g: u8,
    ///
    b: u8,
}

///
impl Color {
    ///
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    ///
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0 };
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}
impl From<tiny_skia::Point> for Point {
    fn from(value: tiny_skia::Point) -> Self {
        Self { x: value.x, y: value.y }
    }
}
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl Rect {
    pub fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self { left, top, right, bottom }
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
pub struct Size {
    width: f32,
    height: f32,
}
impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
impl From<tiny_skia::Size> for Size {
    fn from(value: tiny_skia::Size) -> Self {
        Self { width: value.width(), height: value.height() }
    }
}

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
        if len == 0.0 { Self::ZERO } else { Self { x: self.x / len, y: self.y / len} }
    }
}

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

#[cfg(test)]
mod tests {
    use crate::primitives::{Point, Vec2};

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
}
