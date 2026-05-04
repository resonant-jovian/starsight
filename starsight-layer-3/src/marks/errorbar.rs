//! Error-bar mark — vertical / horizontal whiskers attached to data points.
//!
//! [`ErrorBarMark`] draws an error bar per `(x, y)` pair: a perpendicular line
//! with optional caps. Errors can be symmetric (single magnitude per point)
//! or asymmetric (lower / upper bounds per point) — typical for confidence
//! intervals computed from non-normal distributions.
//!
//! Status: lands in 0.3.0 (Epic H.1, replaces the long-standing
//! `TODO(0.3.0): pub struct ErrorBarMark` placeholder in `marks.rs`).

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathStyle};
use starsight_layer_1::primitives::{Color, Point};
use starsight_layer_2::coords::Coord;

use crate::marks::{DataExtent, LegendGlyph, Mark, require_cartesian};

// ── ErrorBarOrientation ──────────────────────────────────────────────────────────────────────────

/// Direction the whiskers extend along.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ErrorBarOrientation {
    /// Whiskers extend along the y-axis (errors apply to `ys`). Default.
    #[default]
    Vertical,
    /// Whiskers extend along the x-axis (errors apply to `xs`).
    Horizontal,
}

// ── ErrorBarMark ─────────────────────────────────────────────────────────────────────────────────

/// Whisker mark anchored at `(x, y)` with optional caps.
///
/// `errors_low[i]` and `errors_high[i]` are positive distances from the data
/// point to the lower and upper whisker tips. Construct via
/// [`ErrorBarMark::new`] for symmetric errors or chain
/// [`ErrorBarMark::errors_pair`] for asymmetric.
#[derive(Clone, Debug)]
pub struct ErrorBarMark {
    /// X data values.
    pub xs: Vec<f64>,
    /// Y data values.
    pub ys: Vec<f64>,
    /// Lower-whisker magnitudes (positive distance below the data point on
    /// the active axis).
    pub errors_low: Vec<f64>,
    /// Upper-whisker magnitudes (positive distance above the data point on
    /// the active axis).
    pub errors_high: Vec<f64>,
    /// Whisker direction.
    pub orientation: ErrorBarOrientation,
    /// Perpendicular cap length in pixels (`0.0` disables caps). Default 6.0.
    pub cap_width: f32,
    /// Stroke color.
    pub color: Color,
    /// Stroke width in pixels. Default 1.5.
    pub width: f32,
    /// Legend label.
    pub label: Option<String>,
}

impl ErrorBarMark {
    /// New error bars with symmetric errors (`errors[i]` applies as both
    /// low and high). Vectors are silently truncated to the shortest input.
    #[must_use]
    pub fn new(xs: Vec<f64>, ys: Vec<f64>, errors: Vec<f64>) -> Self {
        let n = xs.len().min(ys.len()).min(errors.len());
        let mut xs = xs;
        let mut ys = ys;
        let errors_low = errors.clone();
        let mut errors_high = errors;
        xs.truncate(n);
        ys.truncate(n);
        let mut errors_low_truncated = errors_low;
        errors_low_truncated.truncate(n);
        errors_high.truncate(n);
        Self {
            xs,
            ys,
            errors_low: errors_low_truncated,
            errors_high,
            orientation: ErrorBarOrientation::default(),
            cap_width: 6.0,
            color: Color::from_hex(0x0040_4040),
            width: 1.5,
            label: None,
        }
    }

    /// Builder: replace the symmetric errors with asymmetric `(low, high)`
    /// pairs. `low` and `high` are positive distances from the data point.
    #[must_use]
    pub fn errors_pair(mut self, pairs: Vec<(f64, f64)>) -> Self {
        let n = self.xs.len().min(pairs.len());
        self.errors_low.clear();
        self.errors_high.clear();
        self.errors_low.reserve(n);
        self.errors_high.reserve(n);
        for (lo, hi) in pairs.into_iter().take(n) {
            self.errors_low.push(lo);
            self.errors_high.push(hi);
        }
        self.xs.truncate(n);
        self.ys.truncate(n);
        self
    }

    /// Builder: flip whiskers to extend along the x-axis.
    #[must_use]
    pub fn horizontal(mut self) -> Self {
        self.orientation = ErrorBarOrientation::Horizontal;
        self
    }

    /// Builder: perpendicular cap length in pixels. `0.0` removes caps.
    #[must_use]
    pub fn cap_width(mut self, w: f32) -> Self {
        self.cap_width = w;
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

impl Mark for ErrorBarMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = require_cartesian(coord)?;
        if self.xs.is_empty() {
            return Ok(());
        }
        let style = PathStyle::stroke(self.color, self.width);
        let cap = self.cap_width;

        for i in 0..self.xs.len() {
            let x = self.xs[i];
            let y = self.ys[i];
            let lo = self.errors_low.get(i).copied().unwrap_or(0.0);
            let hi = self.errors_high.get(i).copied().unwrap_or(0.0);
            if !x.is_finite() || !y.is_finite() || !lo.is_finite() || !hi.is_finite() {
                continue;
            }

            match self.orientation {
                ErrorBarOrientation::Vertical => {
                    let p_low = coord.data_to_pixel(x, y - lo);
                    let p_high = coord.data_to_pixel(x, y + hi);
                    backend.draw_path(&Path::new().move_to(p_low).line_to(p_high), &style)?;
                    if cap > 0.0 {
                        let half = cap * 0.5;
                        backend.draw_path(
                            &Path::new()
                                .move_to(Point::new(p_low.x - half, p_low.y))
                                .line_to(Point::new(p_low.x + half, p_low.y)),
                            &style,
                        )?;
                        backend.draw_path(
                            &Path::new()
                                .move_to(Point::new(p_high.x - half, p_high.y))
                                .line_to(Point::new(p_high.x + half, p_high.y)),
                            &style,
                        )?;
                    }
                }
                ErrorBarOrientation::Horizontal => {
                    let p_low = coord.data_to_pixel(x - lo, y);
                    let p_high = coord.data_to_pixel(x + hi, y);
                    backend.draw_path(&Path::new().move_to(p_low).line_to(p_high), &style)?;
                    if cap > 0.0 {
                        let half = cap * 0.5;
                        backend.draw_path(
                            &Path::new()
                                .move_to(Point::new(p_low.x, p_low.y - half))
                                .line_to(Point::new(p_low.x, p_low.y + half)),
                            &style,
                        )?;
                        backend.draw_path(
                            &Path::new()
                                .move_to(Point::new(p_high.x, p_high.y - half))
                                .line_to(Point::new(p_high.x, p_high.y + half)),
                            &style,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        if self.xs.is_empty() {
            return None;
        }
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;

        for i in 0..self.xs.len() {
            let x = self.xs[i];
            let y = self.ys[i];
            let lo = self.errors_low.get(i).copied().unwrap_or(0.0);
            let hi = self.errors_high.get(i).copied().unwrap_or(0.0);
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            match self.orientation {
                ErrorBarOrientation::Vertical => {
                    x_min = x_min.min(x);
                    x_max = x_max.max(x);
                    y_min = y_min.min(y - lo);
                    y_max = y_max.max(y + hi);
                }
                ErrorBarOrientation::Horizontal => {
                    x_min = x_min.min(x - lo);
                    x_max = x_max.max(x + hi);
                    y_min = y_min.min(y);
                    y_max = y_max.max(y);
                }
            }
        }
        if !x_min.is_finite() || !y_min.is_finite() {
            return None;
        }
        Some(DataExtent {
            x_min,
            x_max,
            y_min,
            y_max,
        })
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
    use super::{ErrorBarMark, ErrorBarOrientation};
    use crate::marks::{LegendGlyph, Mark};
    use starsight_layer_1::primitives::Color;

    #[test]
    fn new_truncates_to_shortest_input() {
        let mark = ErrorBarMark::new(vec![0.0, 1.0, 2.0], vec![1.0, 2.0], vec![0.1, 0.2, 0.3]);
        assert_eq!(mark.xs.len(), 2);
        assert_eq!(mark.ys.len(), 2);
        assert_eq!(mark.errors_low.len(), 2);
        assert_eq!(mark.errors_high.len(), 2);
    }

    #[test]
    fn defaults_match_spec() {
        let mark = ErrorBarMark::new(vec![0.0], vec![1.0], vec![0.1]);
        assert!((mark.cap_width - 6.0).abs() < f32::EPSILON);
        assert!((mark.width - 1.5).abs() < f32::EPSILON);
        assert_eq!(mark.orientation, ErrorBarOrientation::Vertical);
    }

    #[test]
    fn symmetric_errors_assigned_low_and_high() {
        let mark = ErrorBarMark::new(vec![0.0, 1.0], vec![10.0, 20.0], vec![0.5, 1.0]);
        assert_eq!(mark.errors_low, vec![0.5, 1.0]);
        assert_eq!(mark.errors_high, vec![0.5, 1.0]);
    }

    #[test]
    fn asymmetric_errors_overwrite_pairs() {
        let mark = ErrorBarMark::new(vec![0.0, 1.0], vec![10.0, 20.0], vec![0.5, 0.5])
            .errors_pair(vec![(0.2, 0.8), (0.4, 0.6)]);
        assert_eq!(mark.errors_low, vec![0.2, 0.4]);
        assert_eq!(mark.errors_high, vec![0.8, 0.6]);
    }

    #[test]
    fn horizontal_builder_flips_orientation() {
        let mark = ErrorBarMark::new(vec![0.0], vec![1.0], vec![0.1]).horizontal();
        assert_eq!(mark.orientation, ErrorBarOrientation::Horizontal);
    }

    #[test]
    fn cap_width_zero_disables_caps() {
        let mark = ErrorBarMark::new(vec![0.0], vec![1.0], vec![0.1]).cap_width(0.0);
        assert_eq!(mark.cap_width, 0.0);
    }

    #[test]
    fn data_extent_vertical_widens_y_only() {
        let mark = ErrorBarMark::new(vec![0.0, 5.0], vec![10.0, 20.0], vec![1.0, 2.0]);
        let extent = mark.data_extent().expect("non-empty");
        assert!((extent.x_min - 0.0).abs() < f64::EPSILON);
        assert!((extent.x_max - 5.0).abs() < f64::EPSILON);
        assert!((extent.y_min - 9.0).abs() < f64::EPSILON);
        assert!((extent.y_max - 22.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_extent_horizontal_widens_x_only() {
        let mark = ErrorBarMark::new(vec![10.0, 20.0], vec![0.0, 5.0], vec![1.0, 2.0]).horizontal();
        let extent = mark.data_extent().expect("non-empty");
        assert!((extent.x_min - 9.0).abs() < f64::EPSILON);
        assert!((extent.x_max - 22.0).abs() < f64::EPSILON);
        assert!((extent.y_min - 0.0).abs() < f64::EPSILON);
        assert!((extent.y_max - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_extent_asymmetric_uses_separate_low_high() {
        let mark =
            ErrorBarMark::new(vec![0.0], vec![10.0], vec![1.0]).errors_pair(vec![(2.0, 5.0)]);
        let extent = mark.data_extent().expect("non-empty");
        assert!((extent.y_min - 8.0).abs() < f64::EPSILON);
        assert!((extent.y_max - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn data_extent_skips_nonfinite() {
        let mark = ErrorBarMark::new(
            vec![f64::NAN, 1.0, 2.0],
            vec![10.0, 20.0, 30.0],
            vec![0.5; 3],
        );
        let extent = mark.data_extent().expect("two finite points");
        assert!((extent.x_min - 1.0).abs() < f64::EPSILON);
        assert!((extent.x_max - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn legend_glyph_is_line() {
        let mark = ErrorBarMark::new(vec![0.0], vec![1.0], vec![0.1]).label("ci");
        assert_eq!(mark.legend_glyph(), LegendGlyph::Line);
    }

    #[test]
    fn legend_color_only_when_labeled() {
        let labeled = ErrorBarMark::new(vec![0.0], vec![1.0], vec![0.1])
            .color(Color::RED)
            .label("ci");
        assert_eq!(labeled.legend_color(), Some(Color::RED));
        let unlabeled = ErrorBarMark::new(vec![0.0], vec![1.0], vec![0.1]);
        assert!(unlabeled.legend_color().is_none());
    }
}
