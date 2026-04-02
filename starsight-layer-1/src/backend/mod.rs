//!
pub mod pdf;
pub mod skia;
pub mod svg;
pub mod terminal;
pub mod wgpu;

use crate::error::Result;
use crate::primitives::color::Color;
use crate::primitives::geom::{Point, Rect};

pub trait DrawBackend {
    fn draw_path(&mut self, path: &Path, style: &PathStyle) -> Result<()>;
    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        font_size: f32,
        color: Color,
    ) -> Result<()>;
    //fn draw_image(&mut self, image: &ImageData, rect: Rect) -> Result<()>;
    fn set_clip(&mut self, rect: Option<Rect>) -> Result<()>;
    fn dimensions(&self) -> (u32, u32);
    fn save_png(&self, path: &std::path::Path) -> Result<()>;
    fn save_svg(&self, path: &std::path::Path) -> Result<()>;
    fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()>;
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathStyle {
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub fill_color: Option<Color>,
    pub dash_pattern: Option<(f32, f32)>,
    pub line_cap: tiny_skia::LineCap,
    pub line_join: tiny_skia::LineJoin,
    pub opacity: f32,
}
pub type Path = PathCommand;
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    CubicTo(Point, Point, Point),
    Close,
}
impl PathCommand {
    pub fn commands(self) -> Vec<Path> {
        todo!()
    }
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
impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transform({}, {}, {}, {}, {}, {})",
            self.0.sx, self.0.sy, self.0.kx, self.0.ky, self.0.tx, self.0.ty
        )
    }
}
