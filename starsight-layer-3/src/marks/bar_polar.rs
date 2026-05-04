//! Polar bar mark — stacked annular bars.
//!
//! [`PolarBarMark`] is the polar counterpart of the cartesian
//! [`BarMark`](crate::marks::BarMark): each entry produces a wedge spanning a
//! configured angular width (`theta ± theta_width/2`) from `r_base` to
//! `r_base + value`. Stacking is opt-in by chaining marks — the second layer
//! sets `r_base` to the first layer's `value`, and so on. This shape backs
//! wind roses (#33: 16 directions × 4 speed bins) and any chart where bars
//! sit on a compass-like angular axis.
//!
//! Pair with `Figure::polar_axes(theta_axis, r_axis)` and a categorical
//! angular axis (`Axis::polar_angular_categorical(n)`) so the `n` bars align
//! to band-center thetas.
//!
//! Status: lands in 0.3.0 (Epic E.1).

#![allow(clippy::cast_precision_loss)]

use std::f64::consts::TAU;

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::PathStyle;
use starsight_layer_1::primitives::Color;
use starsight_layer_2::coords::Coord;

use crate::marks::arc::build_arc_wedge;
use crate::marks::{DataExtent, LegendGlyph, Mark};

// ── PolarBarMark ─────────────────────────────────────────────────────────────────────────────────

/// Annular bar mark plotted on a [`PolarCoord`].
///
/// One entry per `(theta, value)` pair. Each bar spans
/// `[theta - theta_width/2, theta + theta_width/2]` angularly and
/// `[r_base, r_base + value]` radially, where `r_base` defaults to zero.
/// Stack multiple `PolarBarMark` layers (each with its own `r_base`) for
/// stacked wind roses.
///
/// [`PolarCoord`]: starsight_layer_2::coords::PolarCoord
#[derive(Clone, Debug)]
pub struct PolarBarMark {
    /// Angular bin centers in data space (interpreted by `theta_axis`).
    pub thetas: Vec<f64>,
    /// Per-bar radial extents in data space (interpreted by `r_axis`).
    pub values: Vec<f64>,
    /// Per-bar inner radii (data space). When `None`, all bars start at 0.
    /// Used to stack a layer on top of a previous `PolarBarMark`'s outer edge.
    pub r_base: Option<Vec<f64>>,
    /// Per-bar angular widths in data space. When `None`, every bar gets the
    /// minimum neighbor gap as its width — i.e., bars tile the disk without
    /// gaps when `thetas` are evenly spaced.
    pub theta_widths: Option<Vec<f64>>,
    /// Per-bar fill colors. Cycled if shorter than the data; defaults to the
    /// built-in 8-color palette when empty.
    pub colors: Vec<Color>,
    /// Optional outer stroke `(color, width)`. `None` = no border.
    pub stroke: Option<(Color, f32)>,
    /// Legend label.
    pub label: Option<String>,
}

impl PolarBarMark {
    /// New polar bar set with one entry per `(theta, value)` pair. Shorter
    /// input is silently truncated to the smaller length.
    #[must_use]
    pub fn new(mut thetas: Vec<f64>, mut values: Vec<f64>) -> Self {
        let n = thetas.len().min(values.len());
        thetas.truncate(n);
        values.truncate(n);
        Self {
            thetas,
            values,
            r_base: None,
            theta_widths: None,
            colors: Vec::new(),
            stroke: None,
            label: None,
        }
    }

    /// Builder: per-bar inner radii in data space — typically the outer edge
    /// of the layer below in a stacked wind rose.
    #[must_use]
    pub fn r_base(mut self, base: Vec<f64>) -> Self {
        self.r_base = Some(base);
        self
    }

    /// Builder: uniform angular width for every bar, in data-space units.
    #[must_use]
    pub fn theta_width(mut self, width: f64) -> Self {
        let n = self.thetas.len();
        self.theta_widths = Some(vec![width; n]);
        self
    }

    /// Builder: per-bar angular widths in data-space units. Cycled if shorter
    /// than `thetas`.
    #[must_use]
    pub fn theta_widths(mut self, widths: Vec<f64>) -> Self {
        self.theta_widths = Some(widths);
        self
    }

    /// Builder: broadcast a single fill color to every bar.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.colors = vec![c];
        self
    }

    /// Builder: per-bar fill palette. Cycled when shorter than the data.
    #[must_use]
    pub fn colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self
    }

    /// Builder: outer stroke around each bar.
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

    /// Resolve the angular half-width for bar `i`. Falls back to half the
    /// minimum neighbor gap when no widths are configured (matches `ArcMark`).
    fn half_width_at(&self, i: usize) -> f64 {
        if let Some(widths) = &self.theta_widths
            && !widths.is_empty()
        {
            return widths[i % widths.len()] * 0.5;
        }
        if self.thetas.len() < 2 {
            return 0.5;
        }
        let mut sorted = self.thetas.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut min_gap = f64::INFINITY;
        for w in sorted.windows(2) {
            let d = (w[1] - w[0]).abs();
            if d > 0.0 && d < min_gap {
                min_gap = d;
            }
        }
        if min_gap.is_finite() {
            min_gap * 0.5
        } else {
            0.5
        }
    }

    fn r_base_at(&self, i: usize) -> f64 {
        match &self.r_base {
            Some(v) if !v.is_empty() => v.get(i).copied().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    fn color_at(&self, i: usize) -> Color {
        if self.colors.is_empty() {
            crate::marks::palette::POLAR_DEFAULT[i % crate::marks::palette::POLAR_DEFAULT.len()]
        } else {
            self.colors[i % self.colors.len()]
        }
    }
}

impl Mark for PolarBarMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = crate::marks::require_polar(coord)?;
        if self.thetas.is_empty() {
            return Ok(());
        }
        let center = coord.center;
        let radius = coord.radius;

        for (i, (&theta, &value)) in self.thetas.iter().zip(&self.values).enumerate() {
            let half = self.half_width_at(i);
            let r_in_data = self.r_base_at(i);
            let r_out_data = r_in_data + value;

            let theta_a = coord.theta_axis.scale.map(theta - half);
            let theta_b = coord.theta_axis.scale.map(theta + half);
            let a_rad = theta_a * TAU;
            let b_rad = theta_b * TAU;

            let r_in_norm = coord.r_axis.scale.map(r_in_data);
            let r_out_norm = coord.r_axis.scale.map(r_out_data);
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
        // Polar — figure drives axis scaling via `Figure::polar_axes`.
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
    use super::PolarBarMark;
    use crate::marks::{LegendGlyph, Mark};
    use starsight_layer_1::primitives::Color;

    #[test]
    fn new_truncates_to_shorter_input() {
        let mark = PolarBarMark::new(vec![0.0, 1.0, 2.0], vec![10.0, 20.0]);
        assert_eq!(mark.thetas.len(), 2);
        assert_eq!(mark.values.len(), 2);
    }

    #[test]
    fn default_palette_cycles() {
        let mark = PolarBarMark::new(vec![0.0, 1.0], vec![10.0, 20.0]);
        assert_ne!(mark.color_at(0), mark.color_at(1));
        assert_eq!(
            mark.color_at(0),
            mark.color_at(crate::marks::palette::POLAR_DEFAULT.len())
        );
    }

    #[test]
    fn user_palette_cycles() {
        let mark = PolarBarMark::new(vec![0.0, 1.0, 2.0], vec![10.0, 20.0, 30.0])
            .colors(vec![Color::RED, Color::BLUE]);
        assert_eq!(mark.color_at(0), Color::RED);
        assert_eq!(mark.color_at(1), Color::BLUE);
        assert_eq!(mark.color_at(2), Color::RED);
    }

    #[test]
    fn r_base_default_is_zero() {
        let mark = PolarBarMark::new(vec![0.0], vec![10.0]);
        assert_eq!(mark.r_base_at(0), 0.0);
    }

    #[test]
    fn r_base_uses_provided_when_set() {
        let mark = PolarBarMark::new(vec![0.0, 1.0], vec![10.0, 20.0]).r_base(vec![3.0, 5.0]);
        assert_eq!(mark.r_base_at(0), 3.0);
        assert_eq!(mark.r_base_at(1), 5.0);
    }

    #[test]
    fn theta_width_uniform_builder_sets_all() {
        let mark = PolarBarMark::new(vec![0.0, 1.0, 2.0], vec![1.0, 2.0, 3.0]).theta_width(0.4);
        for i in 0..3 {
            assert!((mark.half_width_at(i) - 0.2).abs() < 1e-9);
        }
    }

    #[test]
    fn half_width_default_uses_min_neighbor_gap() {
        let mark = PolarBarMark::new(vec![0.0, 1.0, 3.0], vec![10.0, 20.0, 30.0]);
        for i in 0..3 {
            assert!((mark.half_width_at(i) - 0.5).abs() < 1e-9);
        }
    }

    #[test]
    fn legend_glyph_is_bar() {
        let mark = PolarBarMark::new(vec![0.0], vec![10.0])
            .color(Color::GREEN)
            .label("series");
        assert_eq!(mark.legend_glyph(), LegendGlyph::Bar);
        assert_eq!(mark.legend_color(), Some(Color::GREEN));
    }

    #[test]
    fn no_legend_when_unlabeled() {
        let mark = PolarBarMark::new(vec![0.0], vec![10.0]);
        assert!(mark.legend_color().is_none());
    }

    #[test]
    fn does_not_want_axes() {
        let mark = PolarBarMark::new(vec![0.0], vec![10.0]);
        assert!(!mark.wants_axes());
    }

    #[test]
    fn data_extent_is_none() {
        let mark = PolarBarMark::new(vec![0.0, 1.0], vec![10.0, 20.0]);
        assert!(mark.data_extent().is_none());
    }
}
