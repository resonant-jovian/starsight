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

use crate::statistics::{Bin, BinMethod, BinTransform};
use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{LineCap, LineJoin, Path, PathCommand, PathStyle};
use starsight_layer_1::primitives::{Color, Point, Rect};
use starsight_layer_2::coords::CartesianCoord;
use starsight_layer_2::scales::Scale;
use std::collections::HashMap;
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

/// Context for bar rendering that enables grouped/stacked modes.
#[derive(Debug, Default)]
pub struct BarRenderContext {
    /// Cumulative baselines for stacked bars: category -> baseline value.
    pub stacked_baselines: HashMap<String, f64>,
    /// Group offsets: `group_name` -> (`group_index`, `total_groups`).
    pub group_offsets: HashMap<String, (i32, i32)>,
    /// Whether this is the first render pass (for computing baselines).
    pub first_pass: bool,
}

// ── Mark ─────────────────────────────────────────────────────────────────────────────────────────

/// Object-safe trait every visual mark implements.
///
/// `render` draws the mark using `coord` to map data values to pixel space and
/// `backend` to issue draw calls. `data_extent` reports the mark's data range so
/// the figure can compute appropriate scales.
pub trait Mark {
    /// Render the mark via the given coordinate system and backend.
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()>;
    /// Render with bar context for grouped/stacked bar rendering.
    #[allow(unused_variables)]
    fn render_bar(
        &self,
        coord: &CartesianCoord,
        backend: &mut dyn DrawBackend,
        _context: &BarRenderContext,
    ) -> Result<()> {
        self.render(coord, backend)
    }
    /// Check if this is a bar mark with group/stack info.
    fn as_bar_info(&self) -> Option<(Option<&str>, Option<&str>, Orientation)> {
        None
    }
    /// Get bar data for stacking calculations. Returns (labels, values).
    fn as_bar_data(&self) -> Option<(&[String], &[f64])> {
        None
    }
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
    /// X category labels.
    pub x: Vec<String>,
    /// Y data height
    pub y: Vec<f64>,
    /// Bar color
    pub color: Option<Color>,
    /// Define the width of each bar
    pub width: Option<f32>,
    /// Set bar origin axis
    pub orientation: Orientation,
    /// Group name for grouped bars (dodged within band)
    pub group: Option<String>,
    /// Stack name for stacked bars (accumulated baseline)
    pub stack: Option<String>,
    /// Base value for waterfall chart (defaults to 0)
    pub base: Option<f64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Orientation {
    #[default]
    Vertical,
    Horizontal,
}
impl BarMark {
    /// New bar chart from x and y data with default color and bar width.
    #[must_use]
    pub fn new(x: Vec<String>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            color: Some(Color::BLUE),
            width: Some(0.8),
            orientation: Orientation::Vertical,
            group: None,
            stack: None,
            base: None,
        }
    }

    pub fn horizontal(mut self) -> Self {
        self.orientation = Orientation::Horizontal;
        self
    }

    /// Builder: set group name for grouped bars (bars are dodged within each band).
    #[must_use]
    pub fn group(mut self, name: impl Into<String>) -> Self {
        self.group = Some(name.into());
        self
    }

    /// Builder: set stack name for stacked bars (bars are accumulated).
    #[must_use]
    pub fn stack(mut self, name: impl Into<String>) -> Self {
        self.stack = Some(name.into());
        self
    }

    /// Builder: set base value for waterfall chart (where bar starts).
    #[must_use]
    pub fn base(mut self, b: f64) -> Self {
        self.base = Some(b);
        self
    }

    /// Builder: set bar color.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }

    /// Builder: set bar width in pixels.
    #[must_use]
    pub fn width(mut self, r: f32) -> Self {
        self.width = Some(r);
        self
    }
}
impl Mark for BarMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let valid: Vec<(&str, f64)> = self
            .x
            .iter()
            .zip(&self.y)
            .filter(|(x, y)| !x.is_empty() && !y.is_nan())
            .map(|(x, y)| (x.as_str(), *y))
            .collect();

        let n = valid.len();
        if n == 0 {
            return Ok(());
        }

        let area = coord.plot_area;
        let fill = self.color.unwrap_or(Color::BLUE);
        let width_fraction = self.width.unwrap_or(0.8);

        match self.orientation {
            Orientation::Vertical => {
                let band_width = area.width() / n as f32;
                let bar_width = band_width * width_fraction;

                for (i, (_label, value)) in valid.iter().enumerate() {
                    let x_center = area.left + (i as f32 + 0.5) * band_width;
                    let x_left = x_center - bar_width / 2.0;
                    let x_right = x_center + bar_width / 2.0;

                    let y_top = area.bottom - coord.y_axis.scale.map(*value) as f32 * area.height();
                    let y_bottom = area.bottom - coord.y_axis.scale.map(0.0) as f32 * area.height();

                    let rect = Rect::new(x_left, y_top, x_right, y_bottom);
                    backend.fill_rect(rect, fill)?;
                }
            }
            Orientation::Horizontal => {
                let band_height = area.height() / n as f32;
                let bar_height = band_height * width_fraction;

                for (i, (_label, value)) in valid.iter().enumerate() {
                    let y_center = area.top + (i as f32 + 0.5) * band_height;
                    let y_top = y_center - bar_height / 2.0;
                    let y_bottom = y_center + bar_height / 2.0;

                    let x_left = area.left + coord.x_axis.scale.map(0.0) as f32 * area.width();
                    let x_right = area.left + coord.x_axis.scale.map(*value) as f32 * area.width();

                    let rect = Rect::new(x_left, y_top, x_right, y_bottom);
                    backend.fill_rect(rect, fill)?;
                }
            }
        }

        Ok(())
    }

    fn render_bar(
        &self,
        coord: &CartesianCoord,
        backend: &mut dyn DrawBackend,
        context: &BarRenderContext,
    ) -> Result<()> {
        let valid: Vec<(&str, f64)> = self
            .x
            .iter()
            .zip(&self.y)
            .filter(|(x, y)| !x.is_empty() && !y.is_nan())
            .map(|(x, y)| (x.as_str(), *y))
            .collect();

        let n = valid.len();
        if n == 0 {
            return Ok(());
        }

        let area = coord.plot_area;
        let fill = self.color.unwrap_or(Color::BLUE);
        let width_fraction = self.width.unwrap_or(0.8);

        match self.orientation {
            Orientation::Vertical => {
                let band_width = area.width() / n as f32;
                let total_groups = context
                    .group_offsets
                    .values()
                    .map(|(_, t)| *t)
                    .max()
                    .unwrap_or(1);
                let group_gap = 0.15;
                let bar_width = if total_groups > 1 {
                    band_width * width_fraction * (1.0 - group_gap) / total_groups as f32
                } else {
                    band_width * width_fraction
                };

                for (i, (label, value)) in valid.iter().enumerate() {
                    let base_x_center = area.left + (i as f32 + 0.5) * band_width;
                    let x_offset = if let Some(group_name) = &self.group {
                        if let Some(&(idx, total)) = context.group_offsets.get(group_name) {
                            let sub_band = band_width * (1.0 - group_gap) / total as f32;
                            (idx as f32 - (total - 1) as f32 / 2.0) * sub_band
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    let x_center = base_x_center + x_offset;
                    let x_left = x_center - bar_width / 2.0;
                    let x_right = x_center + bar_width / 2.0;

                    // For stacked or floating bars: y_bottom is baseline, y_top is baseline + value
                    let (y_bottom_val, y_top_val) = if self.stack.is_some() {
                        let baseline = *context.stacked_baselines.get(*label).unwrap_or(&0.0);
                        (baseline, baseline + value)
                    } else if let Some(base) = self.base {
                        (base, base + value)
                    } else {
                        (0.0, *value)
                    };

                    let y_top =
                        area.bottom - coord.y_axis.scale.map(y_top_val) as f32 * area.height();
                    let y_bottom =
                        area.bottom - coord.y_axis.scale.map(y_bottom_val) as f32 * area.height();

                    // Ensure valid rect
                    let rect_top = y_top.min(y_bottom);
                    let rect_bottom = y_top.max(y_bottom);

                    let rect = Rect::new(x_left, rect_top, x_right, rect_bottom);
                    backend.fill_rect(rect, fill)?;
                }
            }
            Orientation::Horizontal => {
                let band_height = area.height() / n as f32;
                let total_groups = context
                    .group_offsets
                    .values()
                    .map(|(_, t)| *t)
                    .max()
                    .unwrap_or(1);
                let group_gap = 0.15;
                let bar_height = if total_groups > 1 {
                    band_height * width_fraction * (1.0 - group_gap) / total_groups as f32
                } else {
                    band_height * width_fraction
                };

                for (i, (label, value)) in valid.iter().enumerate() {
                    let base_y_center = area.top + (i as f32 + 0.5) * band_height;
                    let y_offset = if let Some(group_name) = &self.group {
                        if let Some(&(idx, total)) = context.group_offsets.get(group_name) {
                            let sub_band = band_height * (1.0 - group_gap) / total as f32;
                            (idx as f32 - (total - 1) as f32 / 2.0) * sub_band
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    let y_center = base_y_center + y_offset;
                    let y_top = y_center - bar_height / 2.0;
                    let y_bottom = y_center + bar_height / 2.0;

                    // For stacked or floating horizontal bars: x_left is baseline, x_right is baseline + value
                    let (x_left_val, x_right_val) = if self.stack.is_some() {
                        let baseline = *context.stacked_baselines.get(*label).unwrap_or(&0.0);
                        (baseline, baseline + value)
                    } else if let Some(base) = self.base {
                        (base, base + value)
                    } else {
                        (0.0, *value)
                    };
                    let x_left =
                        area.left + coord.x_axis.scale.map(x_left_val) as f32 * area.width();
                    let x_right =
                        area.left + coord.x_axis.scale.map(x_right_val) as f32 * area.width();

                    let rect = Rect::new(x_left, y_top, x_right, y_bottom);
                    backend.fill_rect(rect, fill)?;
                }
            }
        }

        Ok(())
    }

    fn as_bar_info(&self) -> Option<(Option<&str>, Option<&str>, Orientation)> {
        Some((
            self.group.as_deref(),
            self.stack.as_deref(),
            self.orientation,
        ))
    }

    fn as_bar_data(&self) -> Option<(&[String], &[f64])> {
        Some((&self.x, &self.y))
    }

    // No clue if this works
    fn data_extent(&self) -> Option<DataExtent> {
        let valid_y: Vec<f64> = self.y.iter().copied().filter(|v| !v.is_nan()).collect();
        if valid_y.is_empty() {
            return None;
        }
        let y_min = valid_y
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min)
            .min(0.0);
        let y_max = valid_y
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
            .max(0.0);
        Some(DataExtent {
            x_min: 0.0,
            x_max: self.x.len() as f64,
            y_min,
            y_max,
        })
    }
}

// ── AreaMark ─────────────────────────────────────────────────────────────────────────────────────
/// Area chart for stacked values
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AreaMark {
    /// X data values.
    x: Vec<f64>,
    /// Y data values (must be the same length as `x`)
    y: Vec<f64>,
    baseline: AreaBaseline,
    /// Area fill color
    fill: Color,
    /// Area opacity, 1 to 0
    opacity: f32,
}
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[non_exhaustive]
pub enum AreaBaseline {
    #[default]
    Zero, // fill between y and y=0
    Fixed(f64), // fill between y and a fixed value
}
impl AreaMark {
    /// New area chart from x and y data with default color and full opacity.
    #[must_use]
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            baseline: AreaBaseline::Zero,
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

        let baseline_y = match self.baseline {
            AreaBaseline::Zero => 0.0,
            AreaBaseline::Fixed(y) => y,
        };
        let _baseline = coord.data_to_pixel(0.0, baseline_y);

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

        if !commands.is_empty() && commands.len() > 1 {
            // Close path back to baseline
            let first_x = self.x.first().map_or(0.0, |v| *v);
            let last_x = self.x.last().map_or(0.0, |v| *v);
            commands.push(PathCommand::LineTo(coord.data_to_pixel(last_x, baseline_y)));
            commands.push(PathCommand::LineTo(
                coord.data_to_pixel(first_x, baseline_y),
            ));
            commands.push(PathCommand::Close);
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

    fn data_extent(&self) -> Option<DataExtent> {
        let baseline_y = match self.baseline {
            AreaBaseline::Zero => 0.0,
            AreaBaseline::Fixed(y) => y,
        };
        let y_min = self
            .y
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min)
            .min(baseline_y);
        let y_max = self
            .y
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
            .max(baseline_y);
        Some(DataExtent {
            x_min: self.x.iter().copied().fold(f64::INFINITY, f64::min),
            x_max: self.x.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            y_min,
            y_max,
        })
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
/// Step chart: constant between points with vertical/horizontal transitions.
#[derive(Debug, Clone)]
pub struct StepMark {
    /// X data values.
    pub x: Vec<f64>,
    /// Y data values.
    pub y: Vec<f64>,
    /// Step position: when the vertical transition happens.
    pub where_: StepPosition,
    /// Line color.
    pub color: Color,
    /// Stroke width in pixels.
    pub width: f32,
}

/// Where the vertical step transition happens relative to data points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum StepPosition {
    #[default]
    /// Vertical transition happens *before* the data point (`curveStepBefore`).
    Pre,
    /// Vertical transition at midpoint between points (`curveStep`).
    Mid,
    /// Vertical transition happens *after* the data point (`curveStepAfter`).
    Post,
}

impl StepMark {
    /// New step chart from x and y data with default color.
    #[must_use]
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            where_: StepPosition::default(),
            color: Color::BLUE,
            width: 2.0,
        }
    }

    /// Builder: set step position.
    #[must_use]
    pub fn position(mut self, p: StepPosition) -> Self {
        self.where_ = p;
        self
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

impl Mark for StepMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let mut commands = Vec::new();
        let mut need_move = true;

        let n = self.x.len().min(self.y.len());
        if n < 2 {
            return Ok(());
        }

        // Collect valid points (skip NaN)
        let valid: Vec<(f64, f64)> = (0..n)
            .filter_map(|i| {
                let x = self.x[i];
                let y = self.y[i];
                if x.is_nan() || y.is_nan() {
                    None
                } else {
                    Some((x, y))
                }
            })
            .collect();

        if valid.len() < 2 {
            return Ok(());
        }

        for i in 0..valid.len() {
            let (x, y) = valid[i];
            let curr = coord.data_to_pixel(x, y);

            if need_move {
                commands.push(PathCommand::MoveTo(curr));
                need_move = false;
                continue;
            }

            let (prev_x, prev_y) = valid[i - 1];
            let prev = coord.data_to_pixel(prev_x, prev_y);

            match self.where_ {
                StepPosition::Pre => {
                    commands.push(PathCommand::LineTo(Point::new(curr.x, prev.y)));
                    commands.push(PathCommand::LineTo(curr));
                }
                StepPosition::Post => {
                    commands.push(PathCommand::LineTo(Point::new(prev.x, curr.y)));
                    commands.push(PathCommand::LineTo(curr));
                }
                StepPosition::Mid => {
                    let mid_x = (prev.x + curr.x) / 2.0;
                    commands.push(PathCommand::LineTo(Point::new(mid_x, prev.y)));
                    commands.push(PathCommand::LineTo(Point::new(mid_x, curr.y)));
                    commands.push(PathCommand::LineTo(curr));
                }
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

// ── HistogramMark ───────────────────────────────────────────────────────────────────────

/// Histogram: bar chart with bins computed from data.
#[derive(Debug, Clone)]
pub struct HistogramMark {
    /// Raw data values.
    pub data: Vec<f64>,
    /// Binning method.
    pub method: BinMethod,
    /// Bar color.
    pub color: Color,
}

impl HistogramMark {
    /// New histogram from raw data.
    #[must_use]
    pub fn new(data: Vec<f64>) -> Self {
        Self {
            data,
            method: BinMethod::default(),
            color: Color::from_hex(0x00_4C_72B0),
        }
    }

    /// Builder: set binning method.
    #[must_use]
    pub fn method(mut self, m: BinMethod) -> Self {
        self.method = m;
        self
    }

    /// Builder: set bar color.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }
}

impl Mark for HistogramMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let transform = BinTransform::new(self.method);
        let bins: Vec<Bin> = transform.compute(&self.data);

        if bins.is_empty() {
            return Ok(());
        }

        let area = coord.plot_area;
        let n = bins.len();
        let band_width = area.width() / n as f32;
        let bar_width = band_width * 0.8;

        let y_max = bins.iter().map(|b| b.count as f64).fold(0.0f64, f64::max);

        for (i, bin) in bins.iter().enumerate() {
            let x_center = area.left + (i as f32 + 0.5) * band_width;
            let x_left = x_center - bar_width / 2.0;
            let x_right = x_center + bar_width / 2.0;

            let height_ratio = if y_max > 0.0 {
                (bin.count as f64 / y_max) as f32
            } else {
                0.0
            };
            let bar_height = height_ratio * area.height();

            let y_top = area.bottom - bar_height;
            let y_bottom = area.bottom;

            let rect = Rect::new(x_left, y_top, x_right, y_bottom);
            backend.fill_rect(rect, self.color)?;
        }

        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        let data: Vec<f64> = self.data.iter().copied().filter(|v| !v.is_nan()).collect();
        if data.is_empty() {
            return None;
        }
        let transform = BinTransform::new(self.method);
        let bins = transform.compute(&data);
        let count_max = bins.iter().map(|b| b.count).max().unwrap_or(0) as f64;
        Some(DataExtent {
            x_min: bins.first().map_or(0.0, |b| b.left),
            x_max: bins.last().map_or(1.0, |b| b.right),
            y_min: 0.0,
            y_max: count_max,
        })
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_mark_new() {
        let mark = LineMark::new(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]);
        assert_eq!(mark.x.len(), 3);
        assert_eq!(mark.y.len(), 3);
    }

    #[test]
    fn line_mark_data_extent() {
        let mark = LineMark::new(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]);
        let extent = mark.data_extent();
        assert!(extent.is_some());
        let e = extent.unwrap();
        assert_eq!(e.x_min, 1.0);
        assert_eq!(e.x_max, 3.0);
    }

    #[test]
    fn line_mark_data_extent_empty() {
        let mark = LineMark::new(vec![], vec![]);
        let extent = mark.data_extent();
        assert!(extent.is_none());
    }

    #[test]
    fn line_mark_nan_gaps() {
        let mark = LineMark::new(vec![1.0, 2.0, f64::NAN, 4.0], vec![1.0, 2.0, 3.0, 4.0]);
        let extent = mark.data_extent();
        assert!(extent.is_some());
    }

    #[test]
    fn point_mark_new() {
        let mark = PointMark::new(vec![1.0, 2.0], vec![3.0, 4.0]);
        assert_eq!(mark.x.len(), 2);
    }

    #[test]
    fn point_mark_data_extent() {
        let mark = PointMark::new(vec![1.0, 2.0], vec![3.0, 4.0]);
        let extent = mark.data_extent();
        assert!(extent.is_some());
    }

    #[test]
    fn area_mark_new() {
        let mark = AreaMark::new(vec![1.0, 2.0], vec![3.0, 4.0]);
        assert_eq!(mark.x.len(), 2);
    }

    #[test]
    fn area_mark_data_extent() {
        let mark = AreaMark::new(vec![1.0, 2.0], vec![3.0, 4.0]);
        let extent = mark.data_extent();
        assert!(extent.is_some());
    }

    #[test]
    fn step_mark_new() {
        let mark = StepMark::new(vec![1.0, 2.0], vec![3.0, 4.0]);
        assert_eq!(mark.x.len(), 2);
    }

    #[test]
    fn histogram_mark_new() {
        let mark = HistogramMark::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(mark.data.len(), 5);
    }

    #[test]
    fn data_extent_new() {
        let extent = DataExtent {
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
        };
        assert_eq!(extent.x_min, 0.0);
    }
}
