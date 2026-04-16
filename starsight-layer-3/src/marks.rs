//! Marks: visual elements that read data and render onto a backend.
//!
//! Every mark implements the [`Mark`] trait. Concrete mark types share the same
//! `*Mark` suffix convention. Adding a new chart type means adding a new struct
//! to this file with its own `// ── ItemName ─────` section.
//!
//! Status:
//! - 0.1.0: `Mark` trait, `LineMark`, `PointMark`.
//! - 0.2.0: `BarMark`, `AreaMark`.
//! - 0.3.0+: `HeatmapMark`, `BoxMark`, `ViolinMark`, `PieMark`, `ContourMark`,
//!   `RidgeMark`, `StepMark`, `ErrorBarMark`, `RugMark`.

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{LineCap, LineJoin, Path, PathCommand, PathStyle};
use starsight_layer_1::primitives::{Color, Point, Rect};
use starsight_layer_2::coords::CartesianCoord;

// ── DataExtent ───────────────────────────────────────────────────────────────────────────────────

/// Axis-aligned bounding box of a mark's data, in data coordinates.
pub struct DataExtent {
    /// Minimum x value across the mark's data.
    pub x_min: f64,
    /// Maximum x value across the mark's data.
    pub x_max: f64,
    /// Minimum y value across the mark's data.
    pub y_min: f64,
    /// Maximum y value across the mark's data.
    pub y_max: f64,
}

// ── Mark ─────────────────────────────────────────────────────────────────────────────────────────

/// Object-safe trait every visual mark implements.
///
/// `render` draws the mark using `coord` to map data values to pixel space and
/// `backend` to issue draw calls. `data_extent` reports the mark's data range so
/// the figure can compute appropriate scales.
pub trait Mark {
    /// Render the mark via the given coordinate system and backend.
    ///
    /// # Errors
    /// Forwards any error returned by the backend's drawing methods. Marks
    /// themselves do not produce errors — they only propagate them.
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()>;
    /// Bounding box of this mark's data, or `None` if it is empty.
    fn data_extent(&self) -> Option<DataExtent>;
}

// ── LineMark ─────────────────────────────────────────────────────────────────────────────────────

/// Connected line series with optional NaN gaps.
#[derive(Debug, Clone)]
pub struct LineMark {
    /// X data values.
    pub x: Vec<f64>,
    /// Y data values (must be the same length as `x`).
    pub y: Vec<f64>,
    /// Line color.
    pub color: Color,
    /// Stroke width in pixels.
    pub width: f32,
}

impl LineMark {
    /// New line series from x and y data with default color and width.
    #[must_use]
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            color: Color::BLUE,
            width: 2.0,
        }
    }

    /// Builder: set line color.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Builder: set stroke width in pixels.
    #[must_use]
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl Mark for LineMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let mut commands = Vec::new();
        let mut need_move = true;

        for (x, y) in self.x.iter().zip(&self.y) {
            if x.is_nan() || y.is_nan() {
                need_move = true;
                continue;
            }
            let p = coord.data_to_pixel(*x, *y);
            if need_move {
                commands.push(PathCommand::MoveTo(p));
                need_move = false;
            } else {
                commands.push(PathCommand::LineTo(p));
            }
        }

        if commands.is_empty() {
            return Ok(());
        }

        let path = Path { commands };
        let style = PathStyle {
            stroke_color: self.color,
            stroke_width: self.width,
            fill_color: None,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..PathStyle::default()
        };
        backend.draw_path(&path, &style)
    }

    fn data_extent(&self) -> Option<DataExtent> {
        extent_from_xy(&self.x, &self.y)
    }
}

// ── PointMark ────────────────────────────────────────────────────────────────────────────────────

/// Scatter plot of individual points.
#[derive(Debug, Clone)]
pub struct PointMark {
    /// X data values.
    pub x: Vec<f64>,
    /// Y data values (must be the same length as `x`).
    pub y: Vec<f64>,
    /// Point color.
    pub color: Color,
    /// Point radius in pixels.
    pub radius: f32,
}

impl PointMark {
    /// New scatter from x and y data with default color and radius.
    #[must_use]
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            color: Color::BLUE,
            radius: 4.0,
        }
    }

    /// Builder: set point color.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Builder: set point radius in pixels.
    #[must_use]
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r;
        self
    }
}

impl Mark for PointMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let mut commands = Vec::new();

        for (x, y) in self.x.iter().zip(&self.y) {
            if x.is_nan() || y.is_nan() {
                continue;
            }
            let center = coord.data_to_pixel(*x, *y);
            push_circle(&mut commands, center, self.radius);
        }

        if commands.is_empty() {
            return Ok(());
        }

        let path = Path { commands };
        let style = PathStyle {
            stroke_color: self.color,
            stroke_width: 0.0,
            fill_color: Some(self.color),
            ..PathStyle::default()
        };
        backend.draw_path(&path, &style)
    }

    fn data_extent(&self) -> Option<DataExtent> {
        extent_from_xy(&self.x, &self.y)
    }
}

// ── BarMark ──────────────────────────────────────────────────────────────────────────────────────
/// Bar chart for individual values
#[derive(Debug, Clone)]
pub struct BarMark {
    /// X data values.
    x: Vec<f64>,
    /// Y data values (must be the same length as `x`)
    y: Vec<f64>,
    /// Bar color
    color: Color,
    /// Define the width of each bar
    bar_width: f32
}
impl BarMark {
    /// New bar chart from x and y data with default color and bar width.
    #[must_use]
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            color: Color::BLUE,
            bar_width: 2.0,
        }
    }

    /// Builder: set bar color.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Builder: set bar width in pixels.
    #[must_use]
    pub fn bar_width(mut self, r: f32) -> Self {
        self.bar_width = r;
        self
    }
}
impl Mark for BarMark {
    // coord unused for now?
    fn render(&self, _coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        Ok(for (x, y) in self.x.iter().zip(&self.y) {
            if x.is_nan() || y.is_nan() {
                continue;
            }
            let left = *x as f32 - (self.bar_width / 2.);
            let top;
            let bottom;
            if y > &0. {
                top = *y as f32;
                bottom = 0.;
            } else {
                top = 0.;
                bottom = *y as f32;
            }
            let right = *x as f32 + (self.bar_width / 2.);
            let rect = Rect::new(left, top, right, bottom);

            backend.fill_rect(rect, self.color)?
        })
    }
    // No clue if this works
    fn data_extent(&self) -> Option<DataExtent> {
        let y_min = self.y.iter().cloned().fold(f64::NAN, f64::min);
        if y_min == y_min.min(0.0) {
            extent_from_xy(&self.x, &self.y)
        }
        else { None }
    }
}

// ── AreaMark ─────────────────────────────────────────────────────────────────────────────────────
/// Area chart for stacked values
pub struct AreaMark {
    /// X data values.
    x: Vec<f64>,
    /// Y data values (must be the same length as `x`)
    y: Vec<f64>,
    /// Area fill color
    fill: Color,
    /// Area opacity, 1 to 0
    opacity: f32,
}
impl AreaMark {
    /// New area chart from x and y data with default color and full opacity.
    #[must_use]
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            fill: Color::RED,
            opacity: 1.,
        }
    }

    /// Builder: set area color.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.fill = c;
        self
    }

    /// Builder: set area opacity 1 to 0.
    #[must_use]
    pub fn opacity(mut self, o: f32) -> Self {
        self.opacity = o;
        self
    }
}
impl Mark for AreaMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let mut commands = Vec::new();
        let mut need_move = true;

        for (x, y) in self.x.iter().zip(&self.y) {
            if x.is_nan() || y.is_nan() {
                need_move = true;
                continue;
            }
            let p = coord.data_to_pixel(*x, *y);
            if need_move {
                commands.push(PathCommand::MoveTo(p));
                need_move = false;
            } else {
                commands.push(PathCommand::LineTo(p));
                commands.push(PathCommand::LineTo(Point::new(p.x, 0.)));
                commands.push(PathCommand::LineTo(Point::new(0., 0.)));
                commands.push(PathCommand::Close);
            }
        }

        if commands.is_empty() {
            return Ok(());
        }

        let path = Path { commands };
        let style = PathStyle {
            fill_color: Some(self.fill),
            opacity: self.opacity,
            ..PathStyle::default()
        };
        backend.draw_path(&path, &style)
    }
    // No clue if this works either
    fn data_extent(&self) -> Option<DataExtent> {
        let y_min = self.y.iter().cloned().fold(f64::NAN, f64::min);
        if y_min == y_min.min(0.0) {
            extent_from_xy(&self.x, &[0.])
        }
        else { None }
    }
}
// ── HeatmapMark ──────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct HeatmapMark { data: Vec<Vec<f64>>, colormap: Colormap, ... }

// ── BoxMark ──────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct BoxMark { groups: Vec<Vec<f64>>, color: Color, ... }

// ── ViolinMark ───────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct ViolinMark { groups: Vec<Vec<f64>>, ... }

// ── PieMark ──────────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct PieMark { values: Vec<f64>, labels: Vec<String>, ... }

// ── ContourMark ──────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct ContourMark { z: Vec<Vec<f64>>, levels: Vec<f64>, ... }

// ── RidgeMark ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.5.0): pub struct RidgeMark { densities: Vec<Vec<f64>>, offset: f64 }

// ── StepMark ─────────────────────────────────────────────────────────────────────────────────────
// TODO(0.2.0): pub struct StepMark { x: Vec<f64>, y: Vec<f64>, where_: Step, color: Color }

// ── ErrorBarMark ─────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct ErrorBarMark { x: Vec<f64>, y: Vec<f64>, err: Vec<f64>, color: Color }

// ── RugMark ──────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct RugMark { values: Vec<f64>, side: Side, color: Color }

// ── helpers ──────────────────────────────────────────────────────────────────────────────────────

/// Approximate a circle with four cubic Bézier arcs (magic constant `4/3 * (√2 - 1)`).
fn push_circle(cmds: &mut Vec<PathCommand>, c: Point, r: f32) {
    const K: f32 = 0.552_284_8;
    let kr = K * r;

    cmds.push(PathCommand::MoveTo(Point::new(c.x + r, c.y)));
    cmds.push(PathCommand::CubicTo(
        Point::new(c.x + r, c.y + kr),
        Point::new(c.x + kr, c.y + r),
        Point::new(c.x, c.y + r),
    ));
    cmds.push(PathCommand::CubicTo(
        Point::new(c.x - kr, c.y + r),
        Point::new(c.x - r, c.y + kr),
        Point::new(c.x - r, c.y),
    ));
    cmds.push(PathCommand::CubicTo(
        Point::new(c.x - r, c.y - kr),
        Point::new(c.x - kr, c.y - r),
        Point::new(c.x, c.y - r),
    ));
    cmds.push(PathCommand::CubicTo(
        Point::new(c.x + kr, c.y - r),
        Point::new(c.x + r, c.y - kr),
        Point::new(c.x + r, c.y),
    ));
    cmds.push(PathCommand::Close);
}

/// Compute the axis-aligned bounding box of paired x/y data, skipping NaN entries.
fn extent_from_xy(x: &[f64], y: &[f64]) -> Option<DataExtent> {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    let mut any = false;

    for (&xv, &yv) in x.iter().zip(y) {
        if xv.is_nan() || yv.is_nan() {
            continue;
        }
        x_min = x_min.min(xv);
        x_max = x_max.max(xv);
        y_min = y_min.min(yv);
        y_max = y_max.max(yv);
        any = true;
    }

    if any {
        Some(DataExtent {
            x_min,
            x_max,
            y_min,
            y_max,
        })
    } else {
        None
    }
}
