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

    /// Returns `true` when every segment in this path is axis-aligned: only
    /// `MoveTo`, `Close`, or `LineTo` where each `LineTo` shares either an
    /// x- or y-coordinate with the previous anchor. Any `QuadTo` / `CubicTo`
    /// is an immediate `false`.
    ///
    /// Backends use this to drop antialiasing on grid lines, tick marks,
    /// axis edges, and box outlines so 1-px hairlines stay crisp instead
    /// of fuzzing across two pixel rows. Curves and diagonals always keep
    /// AA because the smoothing genuinely helps there.
    #[must_use]
    pub fn is_axis_aligned(&self) -> bool {
        const EPS: f32 = 1.0e-4;
        let mut prev: Option<Point> = None;
        let mut subpath_start: Option<Point> = None;
        for cmd in &self.commands {
            match *cmd {
                PathCommand::MoveTo(p) => {
                    prev = Some(p);
                    subpath_start = Some(p);
                }
                PathCommand::LineTo(p) => {
                    let Some(prev_p) = prev else {
                        return false;
                    };
                    let dx = (p.x - prev_p.x).abs();
                    let dy = (p.y - prev_p.y).abs();
                    if dx > EPS && dy > EPS {
                        return false;
                    }
                    prev = Some(p);
                }
                PathCommand::Close => {
                    // The implicit segment from prev → subpath_start must
                    // also be axis-aligned for the closed path to qualify.
                    if let (Some(prev_p), Some(start)) = (prev, subpath_start) {
                        let dx = (start.x - prev_p.x).abs();
                        let dy = (start.y - prev_p.y).abs();
                        if dx > EPS && dy > EPS {
                            return false;
                        }
                        prev = Some(start);
                    }
                }
                PathCommand::QuadTo(_, _) | PathCommand::CubicTo(_, _, _) => return false,
            }
        }
        true
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

#[cfg(test)]
mod tests {
    use super::{Color, LineCap, LineJoin, Path, PathCommand, PathStyle, Point};

    #[test]
    fn path_default_is_empty() {
        let p: Path = Path::default();
        assert!(p.commands.is_empty());
    }

    #[test]
    fn path_close_appends_command() {
        let p = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(10.0, 10.0))
            .close();
        assert_eq!(p.commands.len(), 3);
        assert_eq!(p.commands[2], PathCommand::Close);
    }

    #[test]
    fn axis_aligned_empty_path_is_aligned() {
        // Vacuously true — no commands → no diagonal segments.
        assert!(Path::new().is_axis_aligned());
    }

    #[test]
    fn axis_aligned_horizontal_run() {
        let p = Path::new()
            .move_to(Point::new(0.0, 5.0))
            .line_to(Point::new(20.0, 5.0))
            .line_to(Point::new(40.0, 5.0));
        assert!(p.is_axis_aligned());
    }

    #[test]
    fn axis_aligned_vertical_run() {
        let p = Path::new()
            .move_to(Point::new(5.0, 0.0))
            .line_to(Point::new(5.0, 30.0));
        assert!(p.is_axis_aligned());
    }

    #[test]
    fn axis_aligned_orthogonal_box_with_close() {
        let p = Path::new()
            .move_to(Point::new(10.0, 10.0))
            .line_to(Point::new(20.0, 10.0))
            .line_to(Point::new(20.0, 20.0))
            .line_to(Point::new(10.0, 20.0))
            .close();
        assert!(p.is_axis_aligned());
    }

    #[test]
    fn axis_aligned_diagonal_segment_is_rejected() {
        let p = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(10.0, 10.0));
        assert!(!p.is_axis_aligned());
    }

    #[test]
    fn axis_aligned_cubic_present_is_rejected() {
        let mut p = Path::new().move_to(Point::new(0.0, 0.0));
        p.commands.push(PathCommand::CubicTo(
            Point::new(0.0, 5.0),
            Point::new(5.0, 10.0),
            Point::new(10.0, 10.0),
        ));
        assert!(!p.is_axis_aligned());
    }

    #[test]
    fn axis_aligned_close_with_diagonal_back_to_start_is_rejected() {
        // Triangle: two orthogonal sides, but Close would close diagonally
        // back to the starting point — that hidden diagonal disqualifies
        // the path even though every explicit LineTo is axis-aligned.
        let p = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(10.0, 0.0))
            .line_to(Point::new(10.0, 10.0))
            .close();
        assert!(!p.is_axis_aligned());
    }

    #[test]
    fn line_cap_to_tiny_skia_all_variants() {
        assert_eq!(
            LineCap::Butt.to_tiny_skia() as u8,
            tiny_skia::LineCap::Butt as u8
        );
        assert_eq!(
            LineCap::Round.to_tiny_skia() as u8,
            tiny_skia::LineCap::Round as u8
        );
        assert_eq!(
            LineCap::Square.to_tiny_skia() as u8,
            tiny_skia::LineCap::Square as u8
        );
    }

    #[test]
    fn line_join_to_tiny_skia_all_variants() {
        assert_eq!(
            LineJoin::Miter.to_tiny_skia() as u8,
            tiny_skia::LineJoin::Miter as u8
        );
        assert_eq!(
            LineJoin::Round.to_tiny_skia() as u8,
            tiny_skia::LineJoin::Round as u8
        );
        assert_eq!(
            LineJoin::Bevel.to_tiny_skia() as u8,
            tiny_skia::LineJoin::Bevel as u8
        );
    }

    #[test]
    fn path_style_fill_constructor() {
        let s = PathStyle::fill(Color::RED);
        assert_eq!(s.fill_color, Some(Color::RED));
        assert_eq!(s.stroke_width, 0.0);
    }

    #[test]
    fn path_style_stroke_constructor() {
        let s = PathStyle::stroke(Color::BLUE, 3.0);
        assert_eq!(s.stroke_color, Color::BLUE);
        assert_eq!(s.stroke_width, 3.0);
        assert_eq!(s.fill_color, None);
    }
}
