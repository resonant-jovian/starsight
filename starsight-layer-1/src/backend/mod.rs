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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineCap {
    #[default]
    Butt,
    Round,
    Square,
}

impl LineCap {
    pub(crate) fn to_tiny_skia(self) -> tiny_skia::LineCap {
        match self {
            Self::Butt => tiny_skia::LineCap::Butt,
            Self::Round => tiny_skia::LineCap::Round,
            Self::Square => tiny_skia::LineCap::Square,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

impl LineJoin {
    pub(crate) fn to_tiny_skia(self) -> tiny_skia::LineJoin {
        match self {
            Self::Miter => tiny_skia::LineJoin::Miter,
            Self::Round => tiny_skia::LineJoin::Round,
            Self::Bevel => tiny_skia::LineJoin::Bevel,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathStyle {
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub fill_color: Option<Color>,
    pub dash_pattern: Option<(f32, f32)>,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub opacity: f32,
}

impl Default for PathStyle {
    fn default() -> Self {
        Self {
            stroke_color: Color::BLACK,
            stroke_width: 1.0,
            fill_color: None,
            dash_pattern: None,
            line_cap: LineCap::default(),
            line_join: LineJoin::default(),
            opacity: 1.0,
        }
    }
}

impl PathStyle {
    pub fn stroke(color: Color, width: f32) -> Self {
        Self {
            stroke_color: color,
            stroke_width: width,
            ..Self::default()
        }
    }
    pub fn fill(color: Color) -> Self {
        Self {
            fill_color: Some(color),
            stroke_width: 0.0,
            ..Self::default()
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub commands: Vec<PathCommand>,
}

impl Path {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn move_to(mut self, p: Point) -> Self {
        self.commands.push(PathCommand::MoveTo(p));
        self
    }

    pub fn line_to(mut self, p: Point) -> Self {
        self.commands.push(PathCommand::LineTo(p));
        self
    }

    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::Close);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    CubicTo(Point, Point, Point),
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
impl std::fmt::Display for Transform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Transform({}, {}, {}, {}, {}, {})",
            self.0.sx, self.0.sy, self.0.kx, self.0.ky, self.0.tx, self.0.ty
        )
    }
}
