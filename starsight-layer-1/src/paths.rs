//! Path drawing primitives consumed by every backend.
//!
//! `Path` is a sequence of [`PathCommand`]s. `PathStyle` describes how to stroke
//! and/or fill it. [`LineCap`] and [`LineJoin`] describe how stroke endpoints and
//! corners are rendered.
//!
//! Backends translate `Path` into their native representation: tiny-skia builds
//! a `tiny_skia::Path`, the SVG backend writes `<path d="...">`, the PDF backend
//! emits PDF path operators.

use crate::primitives::{Color, Point};

// ── PathCommand ──────────────────────────────────────────────────────────────────────────────────

/// One step of a path: a move, a line, a curve, or a close.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathCommand {
    /// Move the cursor to a new position without drawing.
    MoveTo(Point),
    /// Straight line from the current position.
    LineTo(Point),
    /// Quadratic Bézier curve: control point + endpoint.
    QuadTo(Point, Point),
    /// Cubic Bézier curve: two control points + endpoint.
    CubicTo(Point, Point, Point),
    /// Close the current sub-path with a straight line back to the start.
    Close,
}

// ── Path ─────────────────────────────────────────────────────────────────────────────────────────

/// A sequence of [`PathCommand`]s.
///
/// Build incrementally with the chainable methods, or push commands directly to
/// the `commands` field. Empty paths are valid but most backends will treat them
/// as a no-op.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    /// The raw command list.
    pub commands: Vec<PathCommand>,
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

impl Path {
    /// Empty path.
    #[must_use]
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Append a `MoveTo` and return `self` for chaining.
    #[must_use]
    pub fn move_to(mut self, p: Point) -> Self {
        self.commands.push(PathCommand::MoveTo(p));
        self
    }

    /// Append a `LineTo` and return `self` for chaining.
    #[must_use]
    pub fn line_to(mut self, p: Point) -> Self {
        self.commands.push(PathCommand::LineTo(p));
        self
    }

    /// Append a `Close` and return `self` for chaining.
    #[must_use]
    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::Close);
        self
    }
}

// ── LineCap ──────────────────────────────────────────────────────────────────────────────────────

/// How a stroke ends at unclosed path endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineCap {
    /// Flat cut at the endpoint (default).
    #[default]
    Butt,
    /// Semicircle extending beyond the endpoint by half the stroke width.
    Round,
    /// Square extending beyond the endpoint by half the stroke width.
    Square,
}

impl LineCap {
    /// Convert to the corresponding `tiny_skia::LineCap`.
    pub(crate) fn to_tiny_skia(self) -> tiny_skia::LineCap {
        match self {
            Self::Butt => tiny_skia::LineCap::Butt,
            Self::Round => tiny_skia::LineCap::Round,
            Self::Square => tiny_skia::LineCap::Square,
        }
    }
}

// ── LineJoin ─────────────────────────────────────────────────────────────────────────────────────

/// How a stroke joins two adjacent path segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineJoin {
    /// Sharp point (default), constrained by the miter limit.
    #[default]
    Miter,
    /// Circular arc.
    Round,
    /// Flat cut at the corner.
    Bevel,
}

impl LineJoin {
    /// Convert to the corresponding `tiny_skia::LineJoin`.
    pub(crate) fn to_tiny_skia(self) -> tiny_skia::LineJoin {
        match self {
            Self::Miter => tiny_skia::LineJoin::Miter,
            Self::Round => tiny_skia::LineJoin::Round,
            Self::Bevel => tiny_skia::LineJoin::Bevel,
        }
    }
}

// ── PathStyle ────────────────────────────────────────────────────────────────────────────────────

/// How a `Path` is stroked and/or filled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathStyle {
    /// Stroke color.
    pub stroke_color: Color,
    /// Stroke width in pixels. Set to 0 to disable stroking.
    pub stroke_width: f32,
    /// Fill color, or `None` to disable filling.
    pub fill_color: Option<Color>,
    /// Optional dash pattern: `(visible_len, gap_len)`.
    pub dash_pattern: Option<(f32, f32)>,
    /// Stroke endpoint shape.
    pub line_cap: LineCap,
    /// Stroke corner shape.
    pub line_join: LineJoin,
    /// Overall opacity in `[0, 1]`.
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
    /// Stroke-only style with the given color and width.
    #[must_use]
    pub fn stroke(color: Color, width: f32) -> Self {
        Self {
            stroke_color: color,
            stroke_width: width,
            ..Self::default()
        }
    }

    /// Fill-only style with the given color (no stroke).
    #[must_use]
    pub fn fill(color: Color) -> Self {
        Self {
            fill_color: Some(color),
            stroke_width: 0.0,
            ..Self::default()
        }
    }
}
