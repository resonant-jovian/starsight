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

pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self {
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
