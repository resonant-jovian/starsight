//! Rug mark — 1-D ticks along an axis margin.
//!
//! [`RugMark`] draws short perpendicular tick marks along one edge of the plot
//! rect, one per data value. Pair it with a histogram to expose the
//! underlying observations beneath the binned distribution, or with a violin
//! plot to mark sample positions on the density curve.
//!
//! Status: lands in 0.3.0 (Epic H.2).

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathStyle};
use starsight_layer_1::primitives::{Color, Point};
use starsight_layer_2::coords::Coord;

use crate::marks::{DataExtent, LegendGlyph, Mark, require_cartesian};

// ── AxisDir ──────────────────────────────────────────────────────────────────────────────────────

/// Axis selector for marks that anchor to a specific edge of the plot rect.
///
/// Used by [`RugMark`] to pick whether ticks ride along the bottom (x) or
/// left (y) margin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AxisDir {
    /// Bottom edge — ticks ride along the x-axis pointing upward into the plot.
    X,
    /// Left edge — ticks ride along the y-axis pointing rightward into the plot.
    Y,
}

// ── RugMark ──────────────────────────────────────────────────────────────────────────────────────

/// Tick-along-axis mark. One short perpendicular tick per value.
#[derive(Clone, Debug)]
pub struct RugMark {
    /// Data values projected to the chosen axis.
    pub values: Vec<f64>,
    /// Which axis edge the ticks anchor to.
    pub axis: AxisDir,
    /// Tick length in pixels, perpendicular to the axis. Default 8.0.
    pub length: f32,
    /// Stroke color.
    pub color: Color,
    /// Stroke width in pixels. Default 1.0.
    pub width: f32,
    /// Legend label.
    pub label: Option<String>,
}

impl RugMark {
    /// New rug along the chosen axis.
    #[must_use]
    pub fn new(values: Vec<f64>, axis: AxisDir) -> Self {
        Self {
            values,
            axis,
            length: 8.0,
            color: Color::from_hex(0x004C_72B0),
            width: 1.0,
            label: None,
        }
    }

    /// Builder: perpendicular tick length in pixels.
    #[must_use]
    pub fn length(mut self, length: f32) -> Self {
        self.length = length;
        self
    }

    /// Builder: stroke color.
    #[must_use]
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Builder: stroke width in pixels.
    #[must_use]
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Builder: legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Mark for RugMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = require_cartesian(coord)?;
        if self.values.is_empty() {
            return Ok(());
        }
        let area = coord.plot_area;
        let style = PathStyle::stroke(self.color, self.width);

        match self.axis {
            AxisDir::X => {
                let y0 = area.bottom;
                let y1 = area.bottom - self.length;
                for &v in &self.values {
                    let px = coord.map_x(v) as f32;
                    if px < area.left || px > area.right {
                        continue;
                    }
                    let path = Path::new()
                        .move_to(Point::new(px, y0))
                        .line_to(Point::new(px, y1));
                    backend.draw_path(&path, &style)?;
                }
            }
            AxisDir::Y => {
                let x0 = area.left;
                let x1 = area.left + self.length;
                for &v in &self.values {
                    let py = coord.map_y(v) as f32;
                    if py < area.top || py > area.bottom {
                        continue;
                    }
                    let path = Path::new()
                        .move_to(Point::new(x0, py))
                        .line_to(Point::new(x1, py));
                    backend.draw_path(&path, &style)?;
                }
            }
        }
        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        if self.values.is_empty() {
            return None;
        }
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for &v in &self.values {
            if !v.is_finite() {
                continue;
            }
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        if !min.is_finite() || !max.is_finite() {
            return None;
        }
        // Rug only carries data on its chosen axis; report a degenerate extent
        // on the orthogonal axis so the figure's shared-axis layout still gets
        // the values it needs without forcing the orthogonal scale to widen.
        let extent = match self.axis {
            AxisDir::X => DataExtent {
                x_min: min,
                x_max: max,
                y_min: 0.0,
                y_max: 0.0,
            },
            AxisDir::Y => DataExtent {
                x_min: 0.0,
                x_max: 0.0,
                y_min: min,
                y_max: max,
            },
        };
        Some(extent)
    }

    fn legend_color(&self) -> Option<Color> {
        self.label.as_ref()?;
        Some(self.color)
    }

    fn legend_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Line
    }
}

#[cfg(test)]
mod tests {
    use super::{AxisDir, RugMark};
    use crate::marks::{LegendGlyph, Mark};
    use starsight_layer_1::primitives::Color;

    #[test]
    fn defaults_are_sane() {
        let mark = RugMark::new(vec![0.0, 0.5, 1.0], AxisDir::X);
        assert!((mark.length - 8.0).abs() < f32::EPSILON);
        assert!((mark.width - 1.0).abs() < f32::EPSILON);
        assert_eq!(mark.axis, AxisDir::X);
    }

    #[test]
    fn builders_set_fields() {
        let mark = RugMark::new(vec![1.0], AxisDir::Y)
            .length(12.0)
            .color(Color::RED)
            .width(2.0)
            .label("samples");
        assert!((mark.length - 12.0).abs() < f32::EPSILON);
        assert!((mark.width - 2.0).abs() < f32::EPSILON);
        assert_eq!(mark.color, Color::RED);
        assert_eq!(mark.label.as_deref(), Some("samples"));
    }

    #[test]
    fn data_extent_x_only_for_x_axis() {
        let mark = RugMark::new(vec![1.0, 2.0, 3.0], AxisDir::X);
        let extent = mark.data_extent().expect("non-empty");
        assert!((extent.x_min - 1.0).abs() < f64::EPSILON);
        assert!((extent.x_max - 3.0).abs() < f64::EPSILON);
        assert_eq!(extent.y_min, 0.0);
        assert_eq!(extent.y_max, 0.0);
    }

    #[test]
    fn data_extent_y_only_for_y_axis() {
        let mark = RugMark::new(vec![10.0, 20.0], AxisDir::Y);
        let extent = mark.data_extent().expect("non-empty");
        assert_eq!(extent.x_min, 0.0);
        assert_eq!(extent.x_max, 0.0);
        assert!((extent.y_min - 10.0).abs() < f64::EPSILON);
        assert!((extent.y_max - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_extent_empty_when_no_values() {
        let mark = RugMark::new(vec![], AxisDir::X);
        assert!(mark.data_extent().is_none());
    }

    #[test]
    fn data_extent_skips_nonfinite() {
        let mark = RugMark::new(vec![f64::NAN, 1.0, 2.0, f64::INFINITY], AxisDir::X);
        let extent = mark.data_extent().expect("two finite values");
        assert!((extent.x_min - 1.0).abs() < f64::EPSILON);
        assert!((extent.x_max - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn legend_glyph_is_line() {
        let mark = RugMark::new(vec![0.0], AxisDir::X).label("x");
        assert_eq!(mark.legend_glyph(), LegendGlyph::Line);
    }

    #[test]
    fn legend_color_only_when_labeled() {
        let labeled = RugMark::new(vec![0.0], AxisDir::X)
            .color(Color::GREEN)
            .label("ok");
        assert_eq!(labeled.legend_color(), Some(Color::GREEN));
        let unlabeled = RugMark::new(vec![0.0], AxisDir::X);
        assert!(unlabeled.legend_color().is_none());
    }

    #[test]
    fn wants_axes_default_true() {
        let mark = RugMark::new(vec![0.0], AxisDir::X);
        assert!(mark.wants_axes());
    }
}
