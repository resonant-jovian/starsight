//! Radar / spider mark — polyline along evenly spaced polar axes.
//!
//! `RadarMark` plots one or more "spokes" of values, with each value lying on
//! its own evenly-distributed angular axis (every spoke at index `i` sits at
//! `2π · i / n`). The values per spoke connect into a closed polyline,
//! optionally filled, that reads as a profile across the named dimensions —
//! the canonical "competence radar" / multi-attribute scorecard.
//!
//! Status: lands in 0.3.0 as part of the polar showcase sweep. Pair with
//! `Figure::polar_axes(theta_axis, r_axis)` and an angular axis whose
//! domain matches the spoke count (typically `polar_angular_categorical(n)`
//! with `n = thetas.len()`).

#![allow(clippy::cast_precision_loss)]

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathStyle};
use starsight_layer_1::primitives::Color;
use starsight_layer_2::coords::Coord;

use crate::marks::{DataExtent, LegendGlyph, Mark};

// ── RadarMark ────────────────────────────────────────────────────────────────────────────────────

/// Single radar polyline.
///
/// `thetas` are data-space angular positions interpreted by the figure's
/// theta axis (typically `0..n` indices for an `n`-spoke
/// [`polar_angular_categorical`](starsight_layer_2::axes::Axis::polar_angular_categorical)
/// axis). `values` are the radial extents in data space.
#[derive(Clone, Debug)]
pub struct RadarMark {
    /// Spoke positions (data-space theta).
    pub thetas: Vec<f64>,
    /// Radial values per spoke.
    pub values: Vec<f64>,
    /// Stroke color for the polyline.
    pub color: Color,
    /// Stroke width in pixels.
    pub width: f32,
    /// Optional fill alpha (`0` = no fill, `255` = opaque). Defaults to
    /// `25` so multiple overlapping radar series stay distinguishable in
    /// raster output (`starsight-61l`); 40 was too opaque and the central
    /// overlap of 3+ series desaturated to a near-uniform tone in PNG.
    pub fill_alpha: u8,
    /// Legend label.
    pub label: Option<String>,
}

impl RadarMark {
    /// New radar polyline. Lengths beyond the shorter input are silently
    /// truncated.
    #[must_use]
    pub fn new(mut thetas: Vec<f64>, mut values: Vec<f64>) -> Self {
        let n = thetas.len().min(values.len());
        thetas.truncate(n);
        values.truncate(n);
        Self {
            thetas,
            values,
            color: Color::from_hex(0x004C_72B0),
            width: 2.0,
            fill_alpha: 25,
            label: None,
        }
    }

    /// Builder: stroke color (also tints the fill).
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self
    }

    /// Builder: stroke width in pixels.
    #[must_use]
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Builder: fill-area alpha (0 disables fill, 255 fully opaque).
    #[must_use]
    pub fn fill_alpha(mut self, alpha: u8) -> Self {
        self.fill_alpha = alpha;
        self
    }

    /// Builder: skip the area fill entirely.
    #[must_use]
    pub fn no_fill(self) -> Self {
        self.fill_alpha(0)
    }

    /// Builder: legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl Mark for RadarMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = crate::marks::require_polar(coord)?;
        if self.thetas.is_empty() {
            return Ok(());
        }
        let mut path = Path::new();
        let mut first = true;
        for (&theta, &value) in self.thetas.iter().zip(&self.values) {
            let p = coord.data_to_pixel(theta, value);
            if first {
                path = path.move_to(p);
                first = false;
            } else {
                path = path.line_to(p);
            }
        }
        // Close the polyline back to the first vertex so the radar reads as
        // a closed profile rather than an open ribbon.
        if !self.thetas.is_empty() {
            let first_theta = self.thetas[0];
            let first_value = self.values[0];
            path = path.line_to(coord.data_to_pixel(first_theta, first_value));
        }

        if self.fill_alpha > 0 {
            let mut fill_style = PathStyle::fill(self.color);
            fill_style.opacity = f32::from(self.fill_alpha) / 255.0;
            backend.draw_path(&path, &fill_style)?;
        }
        backend.draw_path(&path, &PathStyle::stroke(self.color, self.width))?;
        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        // Polar marks let the figure's polar axes drive scaling — defer.
        None
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

    fn wants_axes(&self) -> bool {
        // Polar — suppress cartesian axes.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::RadarMark;
    use crate::marks::{LegendGlyph, Mark};
    use starsight_layer_1::primitives::Color;

    #[test]
    fn new_truncates_to_shorter_input() {
        let mark = RadarMark::new(vec![0.0, 1.0, 2.0], vec![10.0, 20.0]);
        assert_eq!(mark.thetas.len(), 2);
        assert_eq!(mark.values.len(), 2);
    }

    #[test]
    fn no_fill_zeros_alpha() {
        let mark = RadarMark::new(vec![0.0, 1.0], vec![1.0, 2.0]).no_fill();
        assert_eq!(mark.fill_alpha, 0);
    }

    #[test]
    fn fill_alpha_setter() {
        let mark = RadarMark::new(vec![0.0], vec![1.0]).fill_alpha(120);
        assert_eq!(mark.fill_alpha, 120);
    }

    #[test]
    fn legend_glyph_is_line() {
        let mark = RadarMark::new(vec![0.0, 1.0], vec![1.0, 2.0]).label("metric");
        assert_eq!(mark.legend_glyph(), LegendGlyph::Line);
    }

    #[test]
    fn legend_color_only_when_labeled() {
        let labeled = RadarMark::new(vec![0.0], vec![1.0])
            .label("X")
            .color(Color::RED);
        assert_eq!(labeled.legend_color(), Some(Color::RED));
        let unlabeled = RadarMark::new(vec![0.0], vec![1.0]);
        assert!(unlabeled.legend_color().is_none());
    }

    #[test]
    fn does_not_want_axes() {
        let mark = RadarMark::new(vec![0.0], vec![1.0]);
        assert!(!mark.wants_axes());
    }

    #[test]
    fn data_extent_is_none() {
        let mark = RadarMark::new(vec![0.0, 1.0, 2.0], vec![1.0, 2.0, 3.0]);
        assert!(mark.data_extent().is_none());
    }
}
