//!
mod pdf;
mod skia;
mod svg;
mod terminal;
mod wgpu;

use crate::error::Result;

pub trait DrawBackend {
    fn draw_path(&mut self, path: &Path, style: &PathStyle) -> Result<()>;

    //fn draw_text(&mut self, text: &TextBlock, position: Point) -> Result<()>;

    //fn draw_image(&mut self, image: &ImageData, rect: Rect) -> Result<()>;
    //fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()>;

    fn dimensions(&self) -> (u32, u32);

    fn save_png(&self, path: &std::path::Path) -> Result<()>;

    fn save_svg(&self, path: &std::path::Path) -> Result<()>;
}

pub struct PathStyle {
    //stroke_color: Color,
    stroke_width: f32,

    //fill_color: Color,
    dash_pattern: Option<(f32, f32)>,

    //line_cap: LineCap,

    //line_join: LineJoin,
    opacity: f32,
}

pub type Path = PathCommand;

pub enum PathCommand {
    //MoveTo(Point),

    //LineTo(Point),

    //QuadTo(Point, Point),

    //CubicTo(Point, Point, Point),
    Close,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform(pub(crate) tiny_skia::Transform);

impl Transform {
    pub fn identity() -> Self {
        Self(tiny_skia::Transform::identity())
    }
    pub fn translate(dx: f32, dy: f32) -> Self {
        Self(tiny_skia::Transform::from_translate(dx, dy))
    }
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self(tiny_skia::Transform::from_scale(sx, sy))
    }
    /// NOTE: tiny-skia takes DEGREES, not radians.
    pub fn rotate_degrees(angle: f32) -> Self {
        Self(tiny_skia::Transform::from_rotate(angle))
    }

    pub fn then(self, other: Transform) -> Self {
        Self(self.0.post_concat(other.0))
    }
    pub fn pre_translate(self, dx: f32, dy: f32) -> Self {
        Self(self.0.pre_translate(dx, dy))
    }

    pub(crate) fn as_tiny_skia(self) -> tiny_skia::Transform {
        self.0
    }
}
