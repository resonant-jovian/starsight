//! Polar rect mark — annular tiles for spiral heatmaps and polar calendars.
//!
//! [`PolarRectMark`] renders one annular tile per `(theta_min, theta_max,
//! r_min, r_max, color)` tuple — the polar equivalent of a rectangular cell
//! in a heatmap. This shape backs the polar calendar (#8: 21 yr × 52 wk
//! spiral) and any chart where two axes carve a polar disk into a 2-D grid.
//!
//! Pair with `Figure::polar_axes(theta_axis, r_axis)`. Both axes must be
//! configured so that `data_to_pixel` covers the desired tile envelope —
//! typically `Axis::polar_angular_categorical(weeks_per_year)` for theta and
//! `Axis::polar_radial(year_min, year_max)` for r.
//!
//! Status: lands in 0.3.0 (Epic E.2).

#![allow(clippy::cast_precision_loss)]

use std::f64::consts::TAU;

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::PathStyle;
use starsight_layer_1::primitives::Color;
use starsight_layer_2::coords::Coord;

use crate::marks::arc::build_arc_wedge;
use crate::marks::{DataExtent, LegendGlyph, Mark};

// ── PolarRectMark ────────────────────────────────────────────────────────────────────────────────

/// Annular tile mark plotted on a [`PolarCoord`].
///
/// Each tile spans `[theta_min, theta_max]` angularly and `[r_min, r_max]`
/// radially in data space. Tile boundaries flow through the figure's polar
/// axis scales, so log / sqrt radial mappings work without per-mark logic.
///
/// [`PolarCoord`]: starsight_layer_2::coords::PolarCoord
#[derive(Clone, Debug)]
pub struct PolarRectMark {
    /// Angular start of each tile in data space.
    pub theta_min: Vec<f64>,
    /// Angular end of each tile in data space.
    pub theta_max: Vec<f64>,
    /// Radial start of each tile in data space.
    pub r_min: Vec<f64>,
    /// Radial end of each tile in data space.
    pub r_max: Vec<f64>,
    /// Per-tile fill colors (parallel to the tile vectors).
    pub colors: Vec<Color>,
    /// Optional outer stroke `(color, width)`. `None` = no border.
    pub stroke: Option<(Color, f32)>,
    /// Legend label.
    pub label: Option<String>,
}

impl PolarRectMark {
    /// New polar rect set. Vectors are truncated to the shortest length so
    /// length mismatches degrade silently rather than panicking at the
    /// rendering layer.
    #[must_use]
    pub fn new(theta_min: Vec<f64>, theta_max: Vec<f64>, r_min: Vec<f64>, r_max: Vec<f64>) -> Self {
        let n = theta_min
            .len()
            .min(theta_max.len())
            .min(r_min.len())
            .min(r_max.len());
        let mut tmin = theta_min;
        let mut tmax = theta_max;
        let mut rmin = r_min;
        let mut rmax = r_max;
        tmin.truncate(n);
        tmax.truncate(n);
        rmin.truncate(n);
        rmax.truncate(n);
        Self {
            theta_min: tmin,
            theta_max: tmax,
            r_min: rmin,
            r_max: rmax,
            colors: Vec::new(),
            stroke: None,
            label: None,
        }
    }

    /// Builder: broadcast a single fill color to every tile.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.colors = vec![c];
        self
    }

    /// Builder: per-tile fill palette. Cycled when shorter than the tile
    /// count.
    #[must_use]
    pub fn colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self
    }

    /// Builder: outer stroke around each tile.
    #[must_use]
    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        self.stroke = Some((color, width));
        self
    }

    /// Builder: legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn color_at(&self, i: usize) -> Color {
        if self.colors.is_empty() {
            // Single muted default — palette cycling would fight a colormap-
            // driven heatmap call site. Caller almost always sets `.colors()`.
            Color::from_hex(0x004C_72B0)
        } else {
            self.colors[i % self.colors.len()]
        }
    }
}

impl Mark for PolarRectMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = crate::marks::require_polar(coord)?;
        if self.theta_min.is_empty() {
            return Ok(());
        }
        let center = coord.center;
        let radius = coord.radius;

        for i in 0..self.theta_min.len() {
            let theta_a = coord.theta_axis.scale.map(self.theta_min[i]);
            let theta_b = coord.theta_axis.scale.map(self.theta_max[i]);
            let r_in_norm = coord.r_axis.scale.map(self.r_min[i]);
            let r_out_norm = coord.r_axis.scale.map(self.r_max[i]);

            let a_rad = theta_a * TAU;
            let b_rad = theta_b * TAU;
            let r_in_px = (r_in_norm * f64::from(radius)) as f32;
            let r_out_px = (r_out_norm * f64::from(radius)) as f32;

            let path = build_arc_wedge(center, r_in_px, r_out_px, a_rad, b_rad);
            backend.draw_path(&path, &PathStyle::fill(self.color_at(i)))?;
            if let Some((stroke_color, stroke_width)) = self.stroke {
                backend.draw_path(&path, &PathStyle::stroke(stroke_color, stroke_width))?;
            }
        }
        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        None
    }

    fn legend_color(&self) -> Option<Color> {
        self.label.as_ref()?;
        Some(self.color_at(0))
    }

    fn legend_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Bar
    }

    fn wants_axes(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::PolarRectMark;
    use crate::marks::{LegendGlyph, Mark};
    use starsight_layer_1::primitives::Color;

    #[test]
    fn new_truncates_to_shortest_input() {
        let mark = PolarRectMark::new(
            vec![0.0, 0.5, 1.0],
            vec![0.5, 1.0],
            vec![0.0, 0.0, 0.0, 0.0],
            vec![1.0, 1.0, 1.0],
        );
        assert_eq!(mark.theta_min.len(), 2);
        assert_eq!(mark.theta_max.len(), 2);
        assert_eq!(mark.r_min.len(), 2);
        assert_eq!(mark.r_max.len(), 2);
    }

    #[test]
    fn default_color_when_unset() {
        let mark = PolarRectMark::new(vec![0.0], vec![1.0], vec![0.0], vec![1.0]);
        assert_eq!(mark.color_at(0), Color::from_hex(0x004C_72B0));
    }

    #[test]
    fn user_colors_cycle() {
        let mark = PolarRectMark::new(
            vec![0.0, 0.5, 1.0],
            vec![0.5, 1.0, 1.5],
            vec![0.0; 3],
            vec![1.0; 3],
        )
        .colors(vec![Color::RED, Color::GREEN]);
        assert_eq!(mark.color_at(0), Color::RED);
        assert_eq!(mark.color_at(1), Color::GREEN);
        assert_eq!(mark.color_at(2), Color::RED);
    }

    #[test]
    fn legend_glyph_is_bar() {
        let mark = PolarRectMark::new(vec![0.0], vec![1.0], vec![0.0], vec![1.0])
            .color(Color::BLUE)
            .label("series");
        assert_eq!(mark.legend_glyph(), LegendGlyph::Bar);
        assert_eq!(mark.legend_color(), Some(Color::BLUE));
    }

    #[test]
    fn no_legend_when_unlabeled() {
        let mark = PolarRectMark::new(vec![0.0], vec![1.0], vec![0.0], vec![1.0]);
        assert!(mark.legend_color().is_none());
    }

    #[test]
    fn does_not_want_axes() {
        let mark = PolarRectMark::new(vec![0.0], vec![1.0], vec![0.0], vec![1.0]);
        assert!(!mark.wants_axes());
    }

    #[test]
    fn data_extent_is_none() {
        let mark = PolarRectMark::new(vec![0.0], vec![1.0], vec![0.0], vec![1.0]);
        assert!(mark.data_extent().is_none());
    }
}
