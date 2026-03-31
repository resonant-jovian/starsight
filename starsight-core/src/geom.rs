//!

use std::fmt::Display;

///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    ///
    x: f32,
    ///
    y: f32,
}

///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    ///
    x: f32,
    ///
    y: f32,
}

///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {}

///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextBlock {}

///
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    ///
    x: f32,
    ///
    y: f32,
}

///
impl Display for Point {
    ///
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}
///
impl Display for Rect {
    ///
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

///
impl Vec2 {
    ///
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    ///
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }
    ///
    pub fn from_points(a: Point, b: Point) -> Self {
        panic!(); // Fix this shit
        let c: Vec2 = Vec2 {
            x: (a.x - b.x),
            y: (a.y - b.y),
        };
        c
    }
}