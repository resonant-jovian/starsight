use starsight_layer_1::backend::{DrawBackend, LineCap, LineJoin, Path, PathCommand, PathStyle};
use starsight_layer_1::error::Result;
use starsight_layer_1::primitives::color::Color;
use starsight_layer_1::primitives::geom::Point;
use starsight_layer_2::coord::CartesianCoord;

/// Axis-aligned bounding box of a mark's data.
pub struct DataExtent {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

pub trait Mark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()>;
    fn data_extent(&self) -> Option<DataExtent>;
}

// ── LineMark ─────────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LineMark {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub color: Color,
    pub width: f32,
}

impl LineMark {
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            color: Color::BLUE,
            width: 2.0,
        }
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }
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

#[derive(Debug, Clone)]
pub struct PointMark {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub color: Color,
    pub radius: f32,
}

impl PointMark {
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            color: Color::BLUE,
            radius: 4.0,
        }
    }
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }
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

// ── helpers ──────────────────────────────────────────────────────────────────────────────────────

/// Approximate a circle with 4 cubic bezier arcs.
fn push_circle(cmds: &mut Vec<PathCommand>, c: Point, r: f32) {
    // Magic constant: 4/3 * (sqrt(2) - 1)
    const K: f32 = 0.552_284_75;
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
