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
use starsight_layer_1::errors::{Result, StarsightError};
use starsight_layer_1::paths::{LineCap, LineJoin, Path, PathCommand, PathStyle};
use starsight_layer_1::primitives::{Color, Point, Rect};
use starsight_layer_2::coords::{CartesianCoord, Coord, PolarCoord};
use std::collections::HashMap;

/// Downcast a `&dyn Coord` to the concrete `CartesianCoord` required by every
/// 0.2.x-era cartesian mark. New polar marks (`ArcMark`, `RadarMark`, etc.)
/// bypass this and downcast to their own coord type instead.
pub(crate) fn require_cartesian(coord: &dyn Coord) -> Result<&CartesianCoord> {
    coord
        .as_any()
        .downcast_ref::<CartesianCoord>()
        .ok_or_else(|| {
            StarsightError::Config("this mark requires a Cartesian coordinate system".to_string())
        })
}

/// Downcast a `&dyn Coord` to the concrete `PolarCoord`. Mirror of
/// [`require_cartesian`] for polar marks ([`crate::marks::ArcMark`],
/// `RadarMark`, etc.).
pub(crate) fn require_polar(coord: &dyn Coord) -> Result<&PolarCoord> {
    coord.as_any().downcast_ref::<PolarCoord>().ok_or_else(|| {
        StarsightError::Config("this mark requires a polar coordinate system".to_string())
    })
}

// ── Submodule marks (0.3.0+) ─────────────────────────────────────────────────────────────────────
//
// Statistical marks added in 0.3.0 live in their own submodule files to keep
// this top-level file readable. Each submodule re-exports its public type(s)
// up to `marks::` so users can write `starsight::marks::BoxPlotMark` without
// caring that the implementation lives in a sibling file.
pub mod arc;
pub mod bar_polar;
pub mod box_plot;
pub mod candlestick;
pub mod contour;
pub mod errorbar;
pub(crate) mod palette;
pub mod pie;
pub mod radar;
pub mod rect_polar;
pub mod rug;
pub mod violin;
pub use arc::ArcMark;
pub use bar_polar::PolarBarMark;
pub use box_plot::{BoxPlotGroup, BoxPlotMark};
pub use candlestick::{CandlestickMark, Ohlc};
pub use contour::{ContourMark, ContourMode};
pub use errorbar::{ErrorBarMark, ErrorBarOrientation};
pub use pie::PieMark;
pub use radar::RadarMark;
pub use rect_polar::PolarRectMark;
pub use rug::{AxisDir, RugMark};
pub use violin::{ViolinGroup, ViolinMark, ViolinScale};
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

// ── LegendGlyph ──────────────────────────────────────────────────────────────────────────────────

/// Visual sample drawn next to a legend entry's label.
///
/// Each [`Mark`] reports the glyph that best represents its appearance via
/// [`Mark::legend_glyph`]. The legend renderer dispatches per variant so
/// [`PointMark`] entries show a filled disk rather than a horizontal line,
/// [`BarMark`] / [`HistogramMark`] entries show a filled rectangle, and
/// [`AreaMark`] entries show a translucent fill — matching what the mark
/// actually draws on the chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum LegendGlyph {
    /// Horizontal stroke. Default for line-shaped marks ([`LineMark`], [`StepMark`]).
    Line,
    /// Filled disk. Used by [`PointMark`].
    Point,
    /// Filled rectangle. Used by [`BarMark`] and [`HistogramMark`].
    Bar,
    /// Filled rectangle with a top stroke. Used by [`AreaMark`].
    Area,
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
    /// `coord` is `&dyn Coord` so the same trait can dispatch to cartesian and
    /// polar marks. Cartesian marks use [`require_cartesian`] to downcast at
    /// the top of their impl; polar marks downcast to their own coord type.
    ///
    /// # Errors
    /// Returns [`StarsightError`] if the coord type is unsupported by this mark
    /// or if the backend errors on a draw call.
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()>;
    /// Render with bar context for grouped/stacked bar rendering.
    ///
    /// Default implementation forwards to [`render`](Self::render); bar marks
    /// override this to handle group offsets and stacked baselines.
    ///
    /// # Errors
    /// Returns the backend's [`Result`] error if drawing the bar fails.
    #[allow(unused_variables)]
    fn render_bar(
        &self,
        coord: &dyn Coord,
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
    /// Returns the color of this mark for legend display.
    fn legend_color(&self) -> Option<Color> {
        None
    }
    /// Returns a label for this mark in the legend.
    fn legend_label(&self) -> Option<&str> {
        None
    }
    /// Returns the glyph the legend should draw for this mark. The default of
    /// [`LegendGlyph::Line`] suits stroked-line marks; concrete marks override
    /// to surface a more honest sample (e.g. [`PointMark`] returns
    /// [`LegendGlyph::Point`], so a scatter legend shows a dot, not a dash).
    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Line
    }
    /// Whether the figure should draw numeric x/y axes around this mark. Most
    /// marks live on a Cartesian axis and want them; angular / decorative
    /// marks like [`PieMark`] override to `false` so the figure suppresses
    /// the axis chrome. The figure honours this only when *every* mark on it
    /// returns `false`; mixed charts keep the axes for the others.
    fn wants_axes(&self) -> bool {
        true
    }

    /// Optional colormap legend description for this mark, used by the
    /// figure to auto-attach a [`Colorbar`](starsight_layer_3::statistics).
    /// Marks that map a continuous value range through a colormap
    /// (`HeatmapMark`, `ContourMark` with a colormap) override this to
    /// expose `(colormap, value_min, value_max, label?, log?)`. Other marks
    /// return `None` (the default), so they don't trigger auto-attach.
    fn colormap_legend(&self) -> Option<ColormapLegend> {
        None
    }
}

// ── ColormapLegend ───────────────────────────────────────────────────────────────────────────────

/// Information a mark exposes when it wants the figure to render a
/// colorbar/scale-to-colormap legend on its behalf.
///
/// Returned by [`Mark::colormap_legend`]. Layer-5 inspects every mark on
/// the figure; when at least one returns `Some` and the figure has not
/// opted out via `Figure::colorbar(false)`, a vertical colorbar strip is
/// attached on the right side of the plot area.
#[derive(Clone, Debug)]
pub struct ColormapLegend {
    /// Colormap used to render the mark's continuous value field.
    pub colormap: starsight_layer_1::colormap::Colormap,
    /// Lowest value the mark renders.
    pub value_min: f64,
    /// Highest value the mark renders.
    pub value_max: f64,
    /// Legend label (often the same as the mark's `label`). When `None`,
    /// the colorbar reads as a bare value scale.
    pub label: Option<String>,
    /// Hint that the value range is log-distributed; the colorbar will
    /// place ticks accordingly.
    pub log_scale: bool,
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
    /// Legend label.
    pub label: Option<String>,
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
            label: None,
        }
    }

    /// Builder: set line color.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Builder: set legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
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
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = require_cartesian(coord)?;
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

    fn legend_color(&self) -> Option<Color> {
        Some(self.color)
    }

    fn legend_label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

// ── PointMark ────────────────────────────────────────────────────────────────────────────────────

/// Scatter plot of individual points.
///
/// `colors` and `radii` are parallel to `x`/`y` and follow the same broadcast rule:
/// `None` → default; `Some(vec)` of length 1 → broadcast; length matching the data
/// → per-point. Length-mismatched vectors fall through to defaults rather than panic.
#[derive(Debug, Clone)]
pub struct PointMark {
    /// X data values.
    pub x: Vec<f64>,
    /// Y data values (must be the same length as `x`).
    pub y: Vec<f64>,
    /// Per-point fill colors. See broadcast rule on the struct doc.
    pub colors: Option<Vec<Color>>,
    /// Per-point radii in pixels. See broadcast rule on the struct doc.
    pub radii: Option<Vec<f32>>,
    /// Legend label.
    pub label: Option<String>,
    /// Mark-wide alpha multiplier in `[0, 1]`. Applied uniformly to every point
    /// at draw time via the path's opacity attribute. Default 1.0.
    pub alpha: f32,
}

impl PointMark {
    /// New scatter from x and y data with default color and radius.
    #[must_use]
    pub fn new(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            colors: Some(vec![Color::BLUE]),
            radii: Some(vec![4.0]),
            label: None,
            alpha: 1.0,
        }
    }

    /// Builder: broadcast a single color to every point.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.colors = Some(vec![c]);
        self
    }

    /// Builder: set per-point colors. Length 1 broadcasts; length matching data
    /// gives per-point colors. Used by bubble scatters where each point's color
    /// encodes a continuous variable through a colormap.
    #[must_use]
    pub fn colors(mut self, cs: Vec<Color>) -> Self {
        self.colors = Some(cs);
        self
    }

    /// Builder: broadcast a single radius (pixels) to every point.
    #[must_use]
    pub fn radius(mut self, r: f32) -> Self {
        self.radii = Some(vec![r]);
        self
    }

    /// Builder: set per-point radii (pixels). Length 1 broadcasts; length matching
    /// data gives per-point sizes. Used by bubble scatters.
    #[must_use]
    pub fn radii(mut self, rs: Vec<f32>) -> Self {
        self.radii = Some(rs);
        self
    }

    /// Builder: set the mark-wide alpha multiplier (clamped to `[0, 1]`).
    #[must_use]
    pub fn alpha(mut self, a: f32) -> Self {
        self.alpha = a.clamp(0.0, 1.0);
        self
    }

    /// Builder: set legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Per-point color lookup. Same broadcast rule as `BarMark` — `None`/mismatched
    /// fall back to BLUE; length 1 broadcasts; length matching data is per-point.
    fn color_at(&self, i: usize) -> Color {
        match self.colors.as_deref() {
            None => Color::BLUE,
            Some([c]) => *c,
            Some(cs) if cs.len() == self.x.len() => cs[i],
            Some(cs) => cs.first().copied().unwrap_or(Color::BLUE),
        }
    }

    /// Per-point radius lookup. Default 4.0 px when unset/mismatched.
    fn radius_at(&self, i: usize) -> f32 {
        match self.radii.as_deref() {
            None => 4.0,
            Some([r]) => *r,
            Some(rs) if rs.len() == self.x.len() => rs[i],
            Some(rs) => rs.first().copied().unwrap_or(4.0),
        }
    }
}

impl Mark for PointMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = require_cartesian(coord)?;
        // Per-point colors/radii mean we can't share a single Path across all
        // points the way the original implementation did. Group consecutive points
        // by (color, radius) so each unique combination still maps to one draw_path
        // call — common bubble-scatter cases (a few size buckets) emit at most a
        // dozen draws, single-color/single-radius cases collapse back to one draw.
        let mut current_key: Option<(Color, u32)> = None;
        let mut commands: Vec<PathCommand> = Vec::new();

        let flush = |backend: &mut dyn DrawBackend,
                     commands: &mut Vec<PathCommand>,
                     key: Option<(Color, u32)>,
                     alpha: f32|
         -> Result<()> {
            if commands.is_empty() {
                return Ok(());
            }
            if let Some((color, _)) = key {
                let path = Path {
                    commands: std::mem::take(commands),
                };
                let style = PathStyle {
                    stroke_color: color,
                    stroke_width: 0.0,
                    fill_color: Some(color),
                    opacity: alpha,
                    ..PathStyle::default()
                };
                backend.draw_path(&path, &style)?;
            }
            Ok(())
        };

        for (i, (x, y)) in self.x.iter().zip(&self.y).enumerate() {
            if x.is_nan() || y.is_nan() {
                continue;
            }
            let color = self.color_at(i);
            let radius = self.radius_at(i);
            // Bucket by radius via its bit representation so f32 NaN is not a key
            // (already filtered above) and equal radii hash equal.
            let key = (color, radius.to_bits());
            if current_key != Some(key) {
                flush(backend, &mut commands, current_key, self.alpha)?;
                current_key = Some(key);
            }
            let center = coord.data_to_pixel(*x, *y);
            push_circle(&mut commands, center, radius);
        }
        flush(backend, &mut commands, current_key, self.alpha)?;

        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        extent_from_xy(&self.x, &self.y)
    }

    fn legend_color(&self) -> Option<Color> {
        Some(
            self.colors
                .as_deref()
                .and_then(|cs| cs.first().copied())
                .unwrap_or(Color::BLUE),
        )
    }

    fn legend_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Point
    }
}

// ── BarMark ──────────────────────────────────────────────────────────────────────────────────────
/// Bar chart for individual values.
///
/// `colors` and `bases` are parallel arrays to `y`. They follow the same broadcast
/// rule everywhere they're used:
///
/// - `None` → use the default for every bar (blue / 0.0).
/// - `Some(vec)` with `len == 1` → broadcast the single value across all bars.
/// - `Some(vec)` with `len == y.len()` → per-bar value at index `i`.
/// - Anything else → falls through to the default; render is robust, no panic.
#[derive(Debug, Clone)]
pub struct BarMark {
    /// X category labels.
    pub x: Vec<String>,
    /// Y data height (the value rendered as the bar's extent above its base).
    pub y: Vec<f64>,
    /// Per-bar fill colors (see broadcast rules on the struct doc).
    pub colors: Option<Vec<Color>>,
    /// Define the width of each bar as a fraction of the band.
    pub width: Option<f32>,
    /// Set bar origin axis.
    pub orientation: Orientation,
    /// Group name for grouped bars (dodged within band).
    pub group: Option<String>,
    /// Stack name for stacked bars (accumulated baseline).
    pub stack: Option<String>,
    /// Per-bar base values (see broadcast rules on the struct doc). When unset, all
    /// bars start at 0; when set, each bar floats at its own base — used by waterfall
    /// charts where bar `i` sits on the running total of bars `0..i`.
    pub bases: Option<Vec<f64>>,
    /// Legend label.
    pub label: Option<String>,
    /// Draw thin gray connector lines between consecutive bars at `bases[i] + y[i]`.
    /// Only honored for `Orientation::Vertical`. Used by waterfall charts.
    pub connectors: bool,
}
/// Bar/box mark orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Orientation {
    /// Bars rise vertically along the y-axis (categories on x).
    #[default]
    Vertical,
    /// Bars extend horizontally along the x-axis (categories on y).
    Horizontal,
}
impl BarMark {
    /// New bar chart from x and y data with default color and bar width.
    #[must_use]
    pub fn new(x: Vec<String>, y: Vec<f64>) -> Self {
        Self {
            x,
            y,
            colors: Some(vec![Color::BLUE]),
            width: Some(0.8),
            orientation: Orientation::Vertical,
            group: None,
            stack: None,
            bases: None,
            label: None,
            connectors: false,
        }
    }

    /// Builder: switch the bars to a horizontal orientation.
    #[must_use]
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

    /// Builder: broadcast a single base value to every bar (where bars start).
    #[must_use]
    pub fn base(mut self, b: f64) -> Self {
        self.bases = Some(vec![b]);
        self
    }

    /// Builder: set per-bar base values (length should match `y` for per-bar bases;
    /// length 1 broadcasts). Used by waterfall charts.
    #[must_use]
    pub fn bases(mut self, bs: Vec<f64>) -> Self {
        self.bases = Some(bs);
        self
    }

    /// Builder: broadcast a single color to every bar.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.colors = Some(vec![c]);
        self
    }

    /// Builder: set per-bar colors (length should match `y` for per-bar colors;
    /// length 1 broadcasts). Used by waterfall charts and any other chart where
    /// individual bars carry semantic color (e.g. above/below threshold).
    #[must_use]
    pub fn colors(mut self, cs: Vec<Color>) -> Self {
        self.colors = Some(cs);
        self
    }

    /// Builder: set bar width as a fraction of the band (0.0..1.0).
    #[must_use]
    pub fn width(mut self, r: f32) -> Self {
        self.width = Some(r);
        self
    }

    /// Builder: set legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Builder: enable thin gray connector lines between consecutive bars (at
    /// `bases[i] + y[i]`). Only honored for vertical orientation. Used by waterfalls.
    #[must_use]
    pub fn connectors(mut self, on: bool) -> Self {
        self.connectors = on;
        self
    }

    /// Per-bar color lookup: `colors[i]` if set and lengths align, else broadcast
    /// the first element, else fall back to BLUE. Robust against length mismatches.
    fn color_at(&self, i: usize) -> Color {
        match self.colors.as_deref() {
            None => Color::BLUE,
            Some([c]) => *c,
            Some(cs) if cs.len() == self.y.len() => cs[i],
            // length mismatch (and not 1) — fall back rather than panic
            Some(cs) => cs.first().copied().unwrap_or(Color::BLUE),
        }
    }

    /// Per-bar base lookup with the same broadcast rules as colors. 0.0 default.
    fn base_at(&self, i: usize) -> f64 {
        match self.bases.as_deref() {
            None => 0.0,
            Some([b]) => *b,
            Some(bs) if bs.len() == self.y.len() => bs[i],
            Some(bs) => bs.first().copied().unwrap_or(0.0),
        }
    }
}
impl Mark for BarMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = require_cartesian(coord)?;
        // Keep the original index alongside each valid bar so per-bar bases/colors
        // line up with self.y even when some entries are filtered out as NaN/empty.
        let valid: Vec<(usize, &str, f64)> = self
            .x
            .iter()
            .zip(&self.y)
            .enumerate()
            .filter(|(_, (x, y))| !x.is_empty() && !y.is_nan())
            .map(|(i, (x, y))| (i, x.as_str(), *y))
            .collect();

        let n = valid.len();
        if n == 0 {
            return Ok(());
        }

        let area = coord.plot_area;
        let width_fraction = self.width.unwrap_or(0.8);

        match self.orientation {
            Orientation::Vertical => {
                let band_width = area.width() / n as f32;
                let bar_width = band_width * width_fraction;

                for (i, (orig_i, _label, value)) in valid.iter().enumerate() {
                    let x_center = area.left + (i as f32 + 0.5) * band_width;
                    let x_left = x_center - bar_width / 2.0;
                    let x_right = x_center + bar_width / 2.0;

                    let base = self.base_at(*orig_i);
                    let y_top =
                        area.bottom - coord.y_axis.scale.map(base + *value) as f32 * area.height();
                    let y_bottom =
                        area.bottom - coord.y_axis.scale.map(base) as f32 * area.height();
                    let rect_top = y_top.min(y_bottom);
                    let rect_bottom = y_top.max(y_bottom);

                    let rect = Rect::new(x_left, rect_top, x_right, rect_bottom);
                    backend.fill_rect(rect, self.color_at(*orig_i))?;
                }
            }
            Orientation::Horizontal => {
                let band_height = area.height() / n as f32;
                let bar_height = band_height * width_fraction;

                for (i, (orig_i, _label, value)) in valid.iter().enumerate() {
                    let y_center = area.top + (i as f32 + 0.5) * band_height;
                    let y_top = y_center - bar_height / 2.0;
                    let y_bottom = y_center + bar_height / 2.0;

                    let base = self.base_at(*orig_i);
                    let x_left = area.left + coord.x_axis.scale.map(base) as f32 * area.width();
                    let x_right =
                        area.left + coord.x_axis.scale.map(base + *value) as f32 * area.width();
                    let rect_left = x_left.min(x_right);
                    let rect_right = x_left.max(x_right);

                    let rect = Rect::new(rect_left, y_top, rect_right, y_bottom);
                    backend.fill_rect(rect, self.color_at(*orig_i))?;
                }
            }
        }

        Ok(())
    }

    // Bar rendering is symmetric between vertical and horizontal orientations;
    // splitting helpers here would duplicate ~30 lines of parameter setup.
    #[allow(clippy::too_many_lines)]
    fn render_bar(
        &self,
        coord: &dyn Coord,
        backend: &mut dyn DrawBackend,
        context: &BarRenderContext,
    ) -> Result<()> {
        let coord = require_cartesian(coord)?;
        let valid: Vec<(usize, &str, f64)> = self
            .x
            .iter()
            .zip(&self.y)
            .enumerate()
            .filter(|(_, (x, y))| !x.is_empty() && !y.is_nan())
            .map(|(i, (x, y))| (i, x.as_str(), *y))
            .collect();

        let n = valid.len();
        if n == 0 {
            return Ok(());
        }

        let area = coord.plot_area;
        let width_fraction = self.width.unwrap_or(0.8);

        // Connector geometry: collected during the bar pass for vertical orientation
        // when self.connectors is on. Each entry is (x_left, x_right, y_running_total_pixel).
        let mut connector_geom: Vec<(f32, f32, f32)> = if self.connectors {
            Vec::with_capacity(n)
        } else {
            Vec::new()
        };

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

                for (i, (orig_i, label, value)) in valid.iter().enumerate() {
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

                    // Stacked > per-bar base > 0.0. Stacked bars float at the running
                    // height for that category; per-bar bases let waterfalls sit at
                    // their running totals.
                    let (y_bottom_val, y_top_val) = if self.stack.is_some() {
                        let baseline = *context.stacked_baselines.get(*label).unwrap_or(&0.0);
                        (baseline, baseline + value)
                    } else {
                        let base = self.base_at(*orig_i);
                        (base, base + value)
                    };

                    let y_top =
                        area.bottom - coord.y_axis.scale.map(y_top_val) as f32 * area.height();
                    let y_bottom =
                        area.bottom - coord.y_axis.scale.map(y_bottom_val) as f32 * area.height();

                    let rect_top = y_top.min(y_bottom);
                    let rect_bottom = y_top.max(y_bottom);

                    let rect = Rect::new(x_left, rect_top, x_right, rect_bottom);
                    backend.fill_rect(rect, self.color_at(*orig_i))?;

                    if self.connectors {
                        // y_top is the pixel-y of (base + value), the running total —
                        // exactly where the connector to the next bar should sit.
                        connector_geom.push((x_left, x_right, y_top));
                    }
                }

                // Connector pass: thin gray segments from bar i's right edge to bar
                // i+1's left edge, both at bar i's running-total y. Vertical only.
                if self.connectors && connector_geom.len() >= 2 {
                    let connector_color = Color::new(0x88, 0x88, 0x88);
                    let style = PathStyle {
                        stroke_color: connector_color,
                        stroke_width: 1.0,
                        fill_color: None,
                        ..PathStyle::default()
                    };
                    for pair in connector_geom.windows(2) {
                        let (_, x_right_i, y_running) = pair[0];
                        let (x_left_next, _, _) = pair[1];
                        let path = Path {
                            commands: vec![
                                PathCommand::MoveTo(Point::new(x_right_i, y_running)),
                                PathCommand::LineTo(Point::new(x_left_next, y_running)),
                            ],
                        };
                        backend.draw_path(&path, &style)?;
                    }
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

                for (i, (orig_i, label, value)) in valid.iter().enumerate() {
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

                    let (x_left_val, x_right_val) = if self.stack.is_some() {
                        let baseline = *context.stacked_baselines.get(*label).unwrap_or(&0.0);
                        (baseline, baseline + value)
                    } else {
                        let base = self.base_at(*orig_i);
                        (base, base + value)
                    };
                    let x_left =
                        area.left + coord.x_axis.scale.map(x_left_val) as f32 * area.width();
                    let x_right =
                        area.left + coord.x_axis.scale.map(x_right_val) as f32 * area.width();
                    let rect_left = x_left.min(x_right);
                    let rect_right = x_left.max(x_right);

                    let rect = Rect::new(rect_left, y_top, rect_right, y_bottom);
                    backend.fill_rect(rect, self.color_at(*orig_i))?;
                }
                // Horizontal connectors are out of scope at 0.3.0 — would need a
                // second pass keyed on x rather than y. Spec only calls for vertical.
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

    fn data_extent(&self) -> Option<DataExtent> {
        let valid_y: Vec<f64> = self.y.iter().copied().filter(|v| !v.is_nan()).collect();
        if valid_y.is_empty() {
            return None;
        }
        let v_min = valid_y
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min)
            .min(0.0);
        let v_max = valid_y
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
            .max(0.0);
        let n = self.x.len() as f64;
        // Orientation determines which axis carries values vs categories. Reporting
        // category count as x for horizontal bars caused Wilkinson tick selection
        // to extend the value range way past actual data, so bars only filled ~70%
        // of the plot area.
        Some(match self.orientation {
            Orientation::Vertical => DataExtent {
                x_min: 0.0,
                x_max: n,
                y_min: v_min,
                y_max: v_max,
            },
            Orientation::Horizontal => DataExtent {
                x_min: v_min,
                x_max: v_max,
                y_min: 0.0,
                y_max: n,
            },
        })
    }

    fn legend_color(&self) -> Option<Color> {
        // Broadcast first color (or BLUE fallback). We always return Some so that
        // bar marks reliably render a legend swatch even when colors aren't set.
        Some(
            self.colors
                .as_deref()
                .and_then(|cs| cs.first().copied())
                .unwrap_or(Color::BLUE),
        )
    }

    fn legend_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Bar
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
/// Where the area mark's lower edge sits.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[non_exhaustive]
pub enum AreaBaseline {
    /// Fill between the data and y = 0.
    #[default]
    Zero,
    /// Fill between the data and a fixed y value.
    Fixed(f64),
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

    /// Builder: set baseline for area fill (default is Zero, i.e., y=0).
    #[must_use]
    pub fn baseline(mut self, b: f64) -> Self {
        self.baseline = AreaBaseline::Fixed(b);
        self
    }
}

fn render_segment(
    coord: &CartesianCoord,
    backend: &mut dyn DrawBackend,
    points: &[(f64, f64)],
    baseline_y: f64,
    fill: Color,
    opacity: f32,
) -> Result<()> {
    let mut commands = Vec::new();

    if let Some((first_x, first_y)) = points.first() {
        commands.push(PathCommand::MoveTo(coord.data_to_pixel(*first_x, *first_y)));
    }

    for (x, y) in points.iter().skip(1) {
        commands.push(PathCommand::LineTo(coord.data_to_pixel(*x, *y)));
    }

    if let Some((last_x, _)) = points.last() {
        commands.push(PathCommand::LineTo(
            coord.data_to_pixel(*last_x, baseline_y),
        ));
    }
    if let Some((first_x, _)) = points.first() {
        commands.push(PathCommand::LineTo(
            coord.data_to_pixel(*first_x, baseline_y),
        ));
    }
    commands.push(PathCommand::Close);

    let path = Path { commands };
    let style = PathStyle {
        fill_color: Some(fill),
        opacity,
        ..PathStyle::default()
    };
    backend.draw_path(&path, &style)
}

impl Mark for AreaMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = require_cartesian(coord)?;
        let baseline_y = match self.baseline {
            AreaBaseline::Zero => 0.0,
            AreaBaseline::Fixed(y) => y,
        };

        let n = self.x.len().min(self.y.len());
        let mut segment_start: Option<usize> = None;
        let mut segment_points: Vec<(f64, f64)> = Vec::new();

        for i in 0..n {
            let xi = self.x[i];
            let yi = self.y[i];

            if xi.is_nan() || yi.is_nan() {
                if let Some(_start) = segment_start {
                    if segment_points.len() >= 2 {
                        render_segment(
                            coord,
                            backend,
                            &segment_points,
                            baseline_y,
                            self.fill,
                            self.opacity,
                        )?;
                    }
                    segment_points.clear();
                    segment_start = None;
                }
                continue;
            }

            if segment_start.is_none() {
                segment_start = Some(i);
            }
            segment_points.push((xi, yi));
        }

        if segment_start.is_some() && segment_points.len() >= 2 {
            render_segment(
                coord,
                backend,
                &segment_points,
                baseline_y,
                self.fill,
                self.opacity,
            )?;
        }

        Ok(())
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

    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Area
    }
}

// ── HeatmapMark ──────────────────────────────────────────────────────────────────────────────────

use starsight_layer_1::colormap::Colormap;

/// How heatmap cell values map to the `[0, 1]` colormap input.
///
/// The default `Linear` is what most heatmaps want (`(v - vmin) / (vmax - vmin)`).
/// `Log` applies `log10` to value and bounds with a small epsilon, useful when the
/// data spans several decades — without it, the brightest cells saturate the
/// colormap and dim cells collapse into the floor color.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[non_exhaustive]
pub enum HeatmapColorScale {
    /// Linear: `t = (v - vmin) / (vmax - vmin)`.
    #[default]
    Linear,
    /// Log10: `t = (log10(v') - log10(vmin')) / (log10(vmax') - log10(vmin'))`,
    /// where `v' = max(v, eps)` to keep `log10` finite. Negative cells clip to `eps`.
    Log,
}

/// Heatmap: a 2D grid of values mapped to colors via a colormap.
#[derive(Debug, Clone)]
pub struct HeatmapMark {
    /// 2D grid of values (row = y, col = x).
    pub data: Vec<Vec<f64>>,
    /// Colormap for mapping values to colors.
    pub colormap: Colormap,
    /// How values map to the `[0, 1]` colormap input. Default linear.
    pub color_scale: HeatmapColorScale,
    /// Optional label for legend.
    pub label: Option<String>,
}

impl HeatmapMark {
    /// Create a new heatmap from a 2D grid of values.
    #[must_use]
    pub fn new(data: Vec<Vec<f64>>) -> Self {
        Self {
            data,
            colormap: Colormap::default(),
            color_scale: HeatmapColorScale::Linear,
            label: None,
        }
    }

    /// Set the colormap for mapping values to colors.
    #[must_use]
    pub fn colormap(mut self, colormap: Colormap) -> Self {
        self.colormap = colormap;
        self
    }

    /// Set the value-to-`[0, 1]` mapping mode (linear or log).
    #[must_use]
    pub fn color_scale(mut self, scale: HeatmapColorScale) -> Self {
        self.color_scale = scale;
        self
    }

    /// Convenience: switch to log-scale color mapping.
    #[must_use]
    pub fn log_scale(self) -> Self {
        self.color_scale(HeatmapColorScale::Log)
    }

    /// Set the legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn value_range(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for row in &self.data {
            for &v in row {
                if !v.is_nan() {
                    min = min.min(v);
                    max = max.max(v);
                }
            }
        }
        (min, max)
    }
}

impl Mark for HeatmapMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = require_cartesian(coord)?;
        if self.data.is_empty() || self.data[0].is_empty() {
            return Ok(());
        }

        let (data_min, data_max) = self.value_range();
        let range = if (data_max - data_min).abs() < f64::EPSILON {
            1.0
        } else {
            data_max - data_min
        };

        // Log-scale prep: pre-compute the log10 bounds and a small epsilon to keep
        // log10 finite when data hits zero or goes negative. eps scales with vmax so
        // the floor is a few decades below the top regardless of the data's units.
        let (log_min, log_range, log_eps) = match self.color_scale {
            HeatmapColorScale::Log => {
                let eps = (data_max.abs() * 1e-12).max(f64::MIN_POSITIVE);
                let lmin = data_min.max(eps).log10();
                let lmax = data_max.max(eps).log10();
                let lrange = if (lmax - lmin).abs() < f64::EPSILON {
                    1.0
                } else {
                    lmax - lmin
                };
                (lmin, lrange, eps)
            }
            HeatmapColorScale::Linear => (0.0, 1.0, 0.0),
        };

        let n_rows = self.data.len();
        let n_cols = self.data[0].len();

        let left = coord.plot_area.left;
        let top = coord.plot_area.top;
        let w = coord.plot_area.width();
        let h = coord.plot_area.height();

        for (row_idx, row) in self.data.iter().enumerate() {
            for (col_idx, &value) in row.iter().enumerate() {
                if value.is_nan() {
                    continue;
                }

                let normalized = match self.color_scale {
                    HeatmapColorScale::Linear => (value - data_min) / range,
                    HeatmapColorScale::Log => (value.max(log_eps).log10() - log_min) / log_range,
                };
                let color = self.colormap.sample(normalized);

                // Integer-snapped boundaries: cell N's right == cell N+1's left exactly,
                // so adjacent cells share a pixel edge and anti-aliasing can't leak through.
                let x0 = (left + w * (col_idx as f32 / n_cols as f32)).round();
                let x1 = (left + w * ((col_idx + 1) as f32 / n_cols as f32)).round();
                let y0 = (top + h * (row_idx as f32 / n_rows as f32)).round();
                let y1 = (top + h * ((row_idx + 1) as f32 / n_rows as f32)).round();
                backend.fill_rect(Rect::new(x0, y0, x1, y1), color)?;
            }
        }

        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        if self.data.is_empty() || self.data[0].is_empty() {
            return None;
        }
        let n_cols = self.data[0].len();
        Some(DataExtent {
            x_min: 0.0,
            x_max: n_cols as f64,
            y_min: 0.0,
            y_max: self.data.len() as f64,
        })
    }

    fn legend_color(&self) -> Option<Color> {
        Some(self.colormap.sample(0.5))
    }

    fn legend_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Bar
    }

    fn colormap_legend(&self) -> Option<ColormapLegend> {
        let (vmin, vmax) = self.value_range();
        if !vmin.is_finite() || !vmax.is_finite() || vmax <= vmin {
            return None;
        }
        Some(ColormapLegend {
            colormap: self.colormap,
            value_min: vmin,
            value_max: vmax,
            label: self.label.clone(),
            log_scale: matches!(self.color_scale, HeatmapColorScale::Log),
        })
    }
}

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
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = require_cartesian(coord)?;
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
                    let mid_x = f32::midpoint(prev.x, curr.x);
                    commands.push(PathCommand::LineTo(Point::new(mid_x, prev.y)));
                    commands.push(PathCommand::LineTo(Point::new(mid_x, curr.y)));
                    commands.push(PathCommand::LineTo(curr));
                }
            }
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
            color: Color::from_hex(0x4C_72B0),
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
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = require_cartesian(coord)?;
        let transform = BinTransform::new(self.method);
        let bins: Vec<Bin> = transform.compute(&self.data);

        let area = coord.plot_area;
        let n = bins.len();
        let band_width = area.width() / n as f32;
        let bar_width = band_width;

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

    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Bar
    }
}

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
    use starsight_layer_1::backends::vectors::SvgBackend;
    use starsight_layer_2::axes::Axis;
    use starsight_layer_2::scales::LinearScale;

    fn coord_for(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> CartesianCoord {
        CartesianCoord {
            x_axis: Axis {
                scale: Box::new(LinearScale {
                    domain_min: x_min,
                    domain_max: x_max,
                }),
                label: None,
                tick_positions: vec![],
                tick_labels: vec![],
            },
            y_axis: Axis {
                scale: Box::new(LinearScale {
                    domain_min: y_min,
                    domain_max: y_max,
                }),
                label: None,
                tick_positions: vec![],
                tick_labels: vec![],
            },
            plot_area: Rect::new(0.0, 0.0, 100.0, 100.0),
        }
    }

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
    fn line_mark_builders() {
        let m = LineMark::new(vec![0.0], vec![0.0])
            .color(Color::RED)
            .width(3.0)
            .label("series");
        assert_eq!(m.color, Color::RED);
        assert_eq!(m.width, 3.0);
        assert_eq!(m.label.as_deref(), Some("series"));
        assert_eq!(m.legend_color(), Some(Color::RED));
        assert_eq!(m.legend_label(), Some("series"));
    }

    #[test]
    fn line_mark_render_all_nan_returns_ok() {
        let mark = LineMark::new(vec![f64::NAN], vec![f64::NAN]);
        let coord = coord_for(0.0, 1.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn line_mark_render_with_data() {
        let mark = LineMark::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0]);
        let coord = coord_for(0.0, 2.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
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
    fn point_mark_builders() {
        let m = PointMark::new(vec![0.0], vec![0.0])
            .color(Color::GREEN)
            .radius(8.0)
            .label("dots");
        assert_eq!(m.colors, Some(vec![Color::GREEN]));
        assert_eq!(m.radii, Some(vec![8.0]));
        assert_eq!(m.legend_color(), Some(Color::GREEN));
        assert_eq!(m.legend_label(), Some("dots"));
    }

    #[test]
    fn point_mark_per_point_colors_and_radii() {
        let m = PointMark::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0])
            .colors(vec![Color::RED, Color::GREEN, Color::BLUE])
            .radii(vec![3.0, 5.0, 7.0])
            .alpha(0.5);
        assert_eq!(m.colors.as_ref().map(Vec::len), Some(3));
        assert_eq!(m.radii.as_ref().map(Vec::len), Some(3));
        assert!((m.alpha - 0.5).abs() < 1e-6);
        // legend_color picks the first color
        assert_eq!(m.legend_color(), Some(Color::RED));
    }

    #[test]
    fn point_mark_render_all_nan_returns_ok() {
        let mark = PointMark::new(vec![f64::NAN], vec![f64::NAN]);
        let coord = coord_for(0.0, 1.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn point_mark_render_with_data() {
        let mark = PointMark::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0]);
        let coord = coord_for(0.0, 2.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
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
    fn area_mark_builders() {
        let m = AreaMark::new(vec![0.0], vec![0.0])
            .color(Color::GREEN)
            .opacity(0.5)
            .baseline(10.0);
        assert_eq!(m.fill, Color::GREEN);
        assert_eq!(m.opacity, 0.5);
        assert!(matches!(m.baseline, AreaBaseline::Fixed(_)));
    }

    #[test]
    fn step_mark_new() {
        let mark = StepMark::new(vec![1.0, 2.0], vec![3.0, 4.0]);
        assert_eq!(mark.x.len(), 2);
    }

    #[test]
    fn step_mark_builders() {
        let m = StepMark::new(vec![0.0], vec![0.0])
            .color(Color::RED)
            .width(5.0)
            .position(StepPosition::Mid);
        assert_eq!(m.color, Color::RED);
        assert_eq!(m.width, 5.0);
        assert_eq!(m.where_, StepPosition::Mid);
    }

    #[test]
    fn step_mark_render_short_returns_ok() {
        let mark = StepMark::new(vec![1.0], vec![1.0]);
        let coord = coord_for(0.0, 1.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn step_mark_render_all_nan_returns_ok() {
        let mark = StepMark::new(vec![f64::NAN, f64::NAN], vec![f64::NAN, f64::NAN]);
        let coord = coord_for(0.0, 1.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn histogram_mark_new() {
        let mark = HistogramMark::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(mark.data.len(), 5);
    }

    #[test]
    fn histogram_mark_builders() {
        let m = HistogramMark::new(vec![1.0, 2.0])
            .color(Color::RED)
            .method(BinMethod::default());
        assert_eq!(m.color, Color::RED);
    }

    #[test]
    fn histogram_mark_render_empty_returns_ok() {
        let mark = HistogramMark::new(vec![]);
        let coord = coord_for(0.0, 1.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn histogram_mark_render_with_data() {
        let mark = HistogramMark::new((0..50).map(f64::from).collect());
        let coord = coord_for(0.0, 50.0, 0.0, 10.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn histogram_mark_data_extent_empty_is_none() {
        let mark = HistogramMark::new(vec![]);
        assert!(mark.data_extent().is_none());
    }

    #[test]
    fn histogram_mark_data_extent_only_nan_is_none() {
        let mark = HistogramMark::new(vec![f64::NAN]);
        assert!(mark.data_extent().is_none());
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

    // ── BarMark ────────────────────────────────────────────────────────────────────────────────

    #[test]
    fn bar_mark_builders() {
        let m = BarMark::new(vec!["a".to_string()], vec![1.0])
            .color(Color::RED)
            .width(0.5)
            .group("g")
            .stack("s")
            .base(2.0)
            .label("series");
        assert_eq!(m.colors, Some(vec![Color::RED]));
        assert_eq!(m.width, Some(0.5));
        assert_eq!(m.group.as_deref(), Some("g"));
        assert_eq!(m.stack.as_deref(), Some("s"));
        assert_eq!(m.bases, Some(vec![2.0]));
        assert!(!m.connectors);
        assert_eq!(m.legend_label(), Some("series"));
        assert_eq!(m.legend_color(), Some(Color::RED));
    }

    #[test]
    fn bar_mark_per_bar_bases_and_colors() {
        let m = BarMark::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![1.0, -2.0, 3.0],
        )
        .bases(vec![0.0, 1.0, -1.0])
        .colors(vec![Color::RED, Color::GREEN, Color::BLUE])
        .connectors(true);
        assert_eq!(m.bases.as_ref().map(Vec::len), Some(3));
        assert_eq!(m.colors.as_ref().map(Vec::len), Some(3));
        assert!(m.connectors);
        // legend_color picks the first color
        assert_eq!(m.legend_color(), Some(Color::RED));
    }

    #[test]
    fn bar_mark_render_bar_vertical_with_connectors() {
        let mark = BarMark::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![1.0, -0.5, 1.5],
        )
        .bases(vec![0.0, 1.0, 0.5])
        .connectors(true);
        let coord = coord_for(0.0, 3.0, -1.0, 3.0);
        let mut backend = SvgBackend::new(120, 120);
        let ctx = BarRenderContext::default();
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn bar_mark_horizontal() {
        let m = BarMark::new(vec!["a".to_string()], vec![1.0]).horizontal();
        assert_eq!(m.orientation, Orientation::Horizontal);
    }

    #[test]
    fn bar_mark_render_vertical_simple() {
        let mark = BarMark::new(vec!["a".to_string(), "b".to_string()], vec![1.0, 2.0]);
        let coord = coord_for(0.0, 2.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn bar_mark_render_horizontal_simple() {
        let mark =
            BarMark::new(vec!["a".to_string(), "b".to_string()], vec![1.0, 2.0]).horizontal();
        let coord = coord_for(0.0, 2.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn bar_mark_render_empty_returns_ok() {
        let mark = BarMark::new(vec![], vec![]);
        let coord = coord_for(0.0, 1.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn bar_mark_render_bar_empty_returns_ok() {
        let mark = BarMark::new(vec![], vec![]);
        let coord = coord_for(0.0, 1.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        let ctx = BarRenderContext::default();
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn bar_mark_render_bar_vertical_grouped() {
        let mark = BarMark::new(vec!["a".to_string(), "b".to_string()], vec![1.0, 2.0]).group("g");
        let coord = coord_for(0.0, 2.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        let mut ctx = BarRenderContext::default();
        ctx.group_offsets.insert("g".into(), (0, 2));
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn bar_mark_render_bar_vertical_grouped_offset_missing() {
        // Group set on the mark but the context has no entry for that group
        // exercises the `else` branch where x_offset falls back to 0.0.
        let mark = BarMark::new(vec!["a".to_string()], vec![1.0]).group("missing");
        let coord = coord_for(0.0, 1.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        let ctx = BarRenderContext::default();
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn bar_mark_render_bar_vertical_stacked() {
        let mark = BarMark::new(vec!["a".to_string(), "b".to_string()], vec![1.0, 2.0]).stack("s");
        let coord = coord_for(0.0, 2.0, 0.0, 4.0);
        let mut backend = SvgBackend::new(100, 100);
        let mut ctx = BarRenderContext::default();
        ctx.stacked_baselines.insert("a".into(), 1.0);
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn bar_mark_render_bar_vertical_with_base() {
        let mark = BarMark::new(vec!["a".to_string(), "b".to_string()], vec![1.0, 2.0]).base(0.5);
        let coord = coord_for(0.0, 2.0, 0.0, 4.0);
        let mut backend = SvgBackend::new(100, 100);
        let ctx = BarRenderContext::default();
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn bar_mark_render_bar_horizontal_grouped() {
        let mark = BarMark::new(vec!["a".to_string(), "b".to_string()], vec![1.0, 2.0])
            .horizontal()
            .group("g");
        let coord = coord_for(0.0, 2.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        let mut ctx = BarRenderContext::default();
        ctx.group_offsets.insert("g".into(), (0, 2));
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn bar_mark_render_bar_horizontal_grouped_offset_missing() {
        let mark = BarMark::new(vec!["a".to_string()], vec![1.0])
            .horizontal()
            .group("missing");
        let coord = coord_for(0.0, 2.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        let ctx = BarRenderContext::default();
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn bar_mark_render_bar_horizontal_stacked() {
        let mark = BarMark::new(vec!["a".to_string(), "b".to_string()], vec![1.0, 2.0])
            .horizontal()
            .stack("s");
        let coord = coord_for(0.0, 4.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        let mut ctx = BarRenderContext::default();
        ctx.stacked_baselines.insert("a".into(), 1.0);
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn bar_mark_render_bar_horizontal_with_base() {
        let mark = BarMark::new(vec!["a".to_string(), "b".to_string()], vec![1.0, 2.0])
            .horizontal()
            .base(0.5);
        let coord = coord_for(0.0, 4.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        let ctx = BarRenderContext::default();
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn bar_mark_data_extent_horizontal() {
        let m = BarMark::new(vec!["a".into(), "b".into()], vec![1.0, 2.0]).horizontal();
        let e = m.data_extent().unwrap();
        assert_eq!(e.x_min, 0.0);
        assert_eq!(e.x_max, 2.0);
        assert_eq!(e.y_min, 0.0);
        assert_eq!(e.y_max, 2.0);
    }

    #[test]
    fn bar_mark_data_extent_empty() {
        let m = BarMark::new(vec![], vec![]);
        assert!(m.data_extent().is_none());
    }

    #[test]
    fn bar_mark_as_bar_info_and_data() {
        let m = BarMark::new(vec!["a".into()], vec![1.0])
            .group("g")
            .stack("s");
        let info = m.as_bar_info().unwrap();
        assert_eq!(info.0, Some("g"));
        assert_eq!(info.1, Some("s"));
        let data = m.as_bar_data().unwrap();
        assert_eq!(data.0.len(), 1);
        assert_eq!(data.1.len(), 1);
    }

    // ── HeatmapMark ────────────────────────────────────────────────────────────────────────────

    #[test]
    fn heatmap_mark_builders() {
        let m = HeatmapMark::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]])
            .colormap(starsight_layer_1::colormap::PLASMA)
            .label("h");
        assert_eq!(m.label.as_deref(), Some("h"));
        assert_eq!(m.legend_label(), Some("h"));
    }

    #[test]
    fn heatmap_mark_render_empty_returns_ok() {
        let mark = HeatmapMark::new(vec![]);
        let coord = coord_for(0.0, 1.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn heatmap_mark_render_inner_empty_row_returns_ok() {
        let mark = HeatmapMark::new(vec![vec![]]);
        let coord = coord_for(0.0, 1.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn heatmap_mark_render_constant_values() {
        // All same value, so range collapses to zero (uses fallback `1.0`)
        let mark = HeatmapMark::new(vec![vec![5.0, 5.0], vec![5.0, 5.0]]);
        let coord = coord_for(0.0, 2.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn heatmap_mark_render_skips_nan() {
        let mark = HeatmapMark::new(vec![vec![1.0, f64::NAN], vec![2.0, 3.0]]);
        let coord = coord_for(0.0, 2.0, 0.0, 2.0);
        let mut backend = SvgBackend::new(100, 100);
        mark.render(&coord, &mut backend).unwrap();
    }

    #[test]
    fn heatmap_mark_data_extent_empty() {
        let m = HeatmapMark::new(vec![]);
        assert!(m.data_extent().is_none());
        let m2 = HeatmapMark::new(vec![vec![]]);
        assert!(m2.data_extent().is_none());
    }

    #[test]
    fn heatmap_mark_legend_color() {
        let m = HeatmapMark::new(vec![vec![1.0]]);
        assert!(m.legend_color().is_some());
    }

    // ── Mark trait default impls ──────────────────────────────────────────────────────────────

    #[test]
    fn mark_default_render_bar_falls_back_to_render() {
        let mark = LineMark::new(vec![0.0, 1.0], vec![0.0, 1.0]);
        let coord = coord_for(0.0, 1.0, 0.0, 1.0);
        let mut backend = SvgBackend::new(100, 100);
        let ctx = BarRenderContext::default();
        // LineMark doesn't override render_bar, so default impl forwards to render
        mark.render_bar(&coord, &mut backend, &ctx).unwrap();
    }

    #[test]
    fn mark_default_legend_returns_none() {
        let mark = StepMark::new(vec![0.0, 1.0], vec![0.0, 1.0]);
        assert!(mark.legend_color().is_none());
        assert!(mark.legend_label().is_none());
    }

    #[test]
    fn mark_default_as_bar_info_is_none() {
        let mark = LineMark::new(vec![0.0], vec![0.0]);
        assert!(mark.as_bar_info().is_none());
        assert!(mark.as_bar_data().is_none());
    }
}
