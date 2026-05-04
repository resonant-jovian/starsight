//! Arc mark — polar-data-mapped wedges.
//!
//! `ArcMark` is the polar counterpart of [`crate::marks::PieMark`]. Where
//! `PieMark` carves the plot area into wedges proportional to a flat list of
//! values (independent of any coord system), `ArcMark` maps each wedge's
//! center angle through the figure's [`PolarCoord::theta_axis`] and its outer
//! radius through [`PolarCoord::r_axis`]. The result composes with the polar
//! grid renderer and supports:
//!
//! - **Nightingale coxcomb** (`#34`): 12-month band-center thetas, sqrt
//!   radial axis, equal-width wedges, value-driven outer radius.
//! - **Gauge** (`#41`): single wedge with a configured [`start_offset`] and
//!   `theta_half_width` carving a partial sweep (e.g., 270°).
//! - **Sunburst** (`#39C`): nested wedges with progressively larger
//!   [`r_inner`] fractions for each ring.
//!
//! Status: lands in 0.3.0.
//!
//! [`start_offset`]: ArcMark::start_offset
//! [`r_inner`]: ArcMark::r_inner
//! [`PolarCoord::theta_axis`]: starsight_layer_2::coords::PolarCoord::theta_axis
//! [`PolarCoord::r_axis`]: starsight_layer_2::coords::PolarCoord::r_axis

#![allow(clippy::cast_precision_loss)]

use std::f64::consts::{FRAC_PI_2, TAU};

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathCommand, PathStyle};
use starsight_layer_1::primitives::{Color, Point};
use starsight_layer_2::coords::Coord;

use crate::marks::{DataExtent, LegendGlyph, Mark};

// ── ArcMark ──────────────────────────────────────────────────────────────────────────────────────

/// Wedge mark plotted on a [`PolarCoord`].
///
/// Each entry produces one wedge spanning `[theta - half_width, theta + half_width]`
/// in the angular axis, from radius `r_inner_i` (inner) to radius `r_i` (outer)
/// in the radial axis. Both axes go through [`PolarCoord`]'s scales — pair
/// `Axis::polar_radial_sqrt` with this mark for Nightingale's value-as-area
/// invariant, or `Axis::polar_angular_categorical` with `theta_half_widths`
/// equal to half-band-width for an equal-sweep coxcomb.
#[derive(Clone, Debug)]
pub struct ArcMark {
    /// Wedge center angles in data space (interpreted by the figure's
    /// `theta_axis`).
    pub thetas: Vec<f64>,
    /// Wedge outer radii in data space (interpreted by the figure's
    /// `r_axis`).
    pub rs: Vec<f64>,
    /// Per-wedge angular half-widths in data space. When `None`, every wedge
    /// gets `(theta_axis_span / n) / 2` as its half-width — i.e., the wedges
    /// tile the disk without gaps.
    pub theta_half_widths: Option<Vec<f64>>,
    /// Per-wedge inner radii in data space. When `None`, all entries inner-
    /// radius at 0 (full pie slices).
    pub r_inner: Option<Vec<f64>>,
    /// Per-wedge fill colors. Cycled if shorter than the data; defaults to
    /// the figure's prelude palette via index when empty.
    pub colors: Vec<Color>,
    /// Optional outer stroke (color, width). `None` = no border.
    pub stroke: Option<(Color, f32)>,
    /// Angular start offset in radians — added to every wedge's pixel angle
    /// after axis mapping. Use for partial-sweep gauges (e.g. `-3π/4` so the
    /// 0° tick lands at the bottom-left of a 270° gauge).
    pub start_offset: f64,
    /// Legend label.
    pub label: Option<String>,
}

impl ArcMark {
    /// New arc set with one wedge per `(theta, r)` pair. Shorter input is
    /// silently truncated to the smaller length.
    #[must_use]
    pub fn new(mut thetas: Vec<f64>, mut rs: Vec<f64>) -> Self {
        let n = thetas.len().min(rs.len());
        thetas.truncate(n);
        rs.truncate(n);
        Self {
            thetas,
            rs,
            theta_half_widths: None,
            r_inner: None,
            colors: Vec::new(),
            stroke: None,
            start_offset: 0.0,
            label: None,
        }
    }

    /// Builder: per-wedge angular half-widths in data-space units.
    #[must_use]
    pub fn theta_half_widths(mut self, hw: Vec<f64>) -> Self {
        self.theta_half_widths = Some(hw);
        self
    }

    /// Builder: uniform half-width for every wedge.
    #[must_use]
    pub fn theta_half_width(mut self, hw: f64) -> Self {
        let n = self.thetas.len();
        self.theta_half_widths = Some(vec![hw; n]);
        self
    }

    /// Builder: per-wedge inner radii (data space). Pair with `r_outer`-style
    /// values for sunburst rings.
    #[must_use]
    pub fn r_inner(mut self, r_inner: Vec<f64>) -> Self {
        self.r_inner = Some(r_inner);
        self
    }

    /// Builder: per-wedge fill palette. Cycled when shorter than the data.
    #[must_use]
    pub fn colors(mut self, colors: Vec<Color>) -> Self {
        self.colors = colors;
        self
    }

    /// Builder: outer stroke around each wedge.
    #[must_use]
    pub fn stroke(mut self, color: Color, width: f32) -> Self {
        self.stroke = Some((color, width));
        self
    }

    /// Builder: angular start offset in radians applied after axis mapping.
    /// Defaults to zero (compass: theta = 0 lands at top).
    #[must_use]
    pub fn start_offset(mut self, offset: f64) -> Self {
        self.start_offset = offset;
        self
    }

    /// Builder: legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Resolve the half-width for wedge `i`. Falls back to a uniform default
    /// when no per-wedge widths are configured.
    fn half_width_at(&self, i: usize) -> f64 {
        if let Some(hw) = &self.theta_half_widths
            && let Some(w) = hw.get(i)
        {
            return *w;
        }
        // Default: spread `thetas` evenly through the angular axis. Inferring
        // the "natural" data-space half-width without the axis's domain isn't
        // possible here, so we use a half-band based on neighboring entries.
        if self.thetas.len() < 2 {
            return 0.5; // single wedge → ±0.5 data units (sensible for indices).
        }
        let mut sorted: Vec<f64> = self.thetas.clone();
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

    fn r_inner_at(&self, i: usize) -> f64 {
        match &self.r_inner {
            Some(v) => v.get(i).copied().unwrap_or(0.0),
            None => 0.0,
        }
    }

    fn color_at(&self, i: usize) -> Color {
        if self.colors.is_empty() {
            // Mirror PieMark's default palette logic at minimum: cycle a
            // small built-in palette so unconfigured charts still distinguish
            // wedges.
            DEFAULT_PALETTE[i % DEFAULT_PALETTE.len()]
        } else {
            self.colors[i % self.colors.len()]
        }
    }
}

/// Default palette mirrors `PieMark`'s; chosen to match perceptually distinct
/// hues at common counts (12 for Nightingale, 8 for radar, etc.).
const DEFAULT_PALETTE: [Color; 8] = [
    Color::from_hex(0x004C_72B0),
    Color::from_hex(0x00DD_8452),
    Color::from_hex(0x0055_A868),
    Color::from_hex(0x00C4_4E52),
    Color::from_hex(0x008B_8B6B),
    Color::from_hex(0x0093_75A0),
    Color::from_hex(0x008C_5C3D),
    Color::from_hex(0x00DA_8BC3),
];

impl Mark for ArcMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = crate::marks::require_polar(coord)?;
        if self.thetas.is_empty() {
            return Ok(());
        }
        let center = coord.center;
        let radius = coord.radius;

        for (i, (&theta, &r_outer)) in self.thetas.iter().zip(&self.rs).enumerate() {
            let r_inner_data = self.r_inner_at(i);
            let half_w_data = self.half_width_at(i);
            // Angular endpoints in pixel-angle space (radians, compass: 0 = up,
            // increasing clockwise) — apply the axis scale, multiply by 2π,
            // then add start_offset.
            let theta_a = coord.theta_axis.scale.map(theta - half_w_data);
            let theta_b = coord.theta_axis.scale.map(theta + half_w_data);
            let a_rad = theta_a * TAU + self.start_offset;
            let b_rad = theta_b * TAU + self.start_offset;
            // Radial extents in pixel space.
            let r_outer_norm = coord.r_axis.scale.map(r_outer);
            let r_inner_norm = coord.r_axis.scale.map(r_inner_data);
            let r_out_px = (r_outer_norm * f64::from(radius)) as f32;
            let r_in_px = (r_inner_norm * f64::from(radius)) as f32;

            let path = build_arc_wedge(center, r_in_px, r_out_px, a_rad, b_rad);
            let color = self.color_at(i);
            backend.draw_path(&path, &PathStyle::fill(color))?;
            if let Some((stroke_color, stroke_width)) = self.stroke {
                backend.draw_path(&path, &PathStyle::stroke(stroke_color, stroke_width))?;
            }
        }
        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        // ArcMark contributes data-space ranges through its theta/r vectors,
        // but the figure that hosts it sets the axes manually — typically
        // `Axis::polar_angular_categorical(n)` and `Axis::polar_radial(0, max)`.
        // Returning `None` keeps the figure's own axis configuration in
        // charge instead of overriding via auto-extent.
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
        // Same as PieMark — a filled rect approximates a wedge well enough.
        LegendGlyph::Bar
    }

    fn wants_axes(&self) -> bool {
        // ArcMark is polar; cartesian axis chrome would draw outside the
        // disk. Match PieMark's behavior so polar-only figures suppress
        // axes when every mark agrees.
        false
    }
}

// ── arc geometry ─────────────────────────────────────────────────────────────────────────────────

/// Build a closed wedge path in compass coords: outer arc from `a → b`,
/// straight line to inner-arc end, inner arc back from `b → a`, close.
///
/// Compass convention: `(angle.sin, -angle.cos) * r` — angle = 0 lands above
/// the center. Matches [`PolarCoord::data_to_pixel`]. Shared with sibling
/// polar marks (`PolarBarMark`, `PolarRectMark`) via `pub(crate)`.
pub(crate) fn build_arc_wedge(center: Point, r_in: f32, r_out: f32, a: f64, b: f64) -> Path {
    let cx = center.x;
    let cy = center.y;
    let outer_start = compass_point(cx, cy, r_out, a);
    let mut path = Path::new();
    if r_in <= 0.5 {
        // Pie slice: center → outer arc → close.
        path = path.move_to(Point::new(cx, cy)).line_to(outer_start);
        arc_compass(&mut path, cx, cy, r_out, a, b);
        path.close()
    } else {
        // Donut wedge: outer_start → outer arc to b → inner end → inner arc back → close.
        let inner_end = compass_point(cx, cy, r_in, b);
        path = path.move_to(outer_start);
        arc_compass(&mut path, cx, cy, r_out, a, b);
        path = path.line_to(inner_end);
        arc_compass(&mut path, cx, cy, r_in, b, a);
        path.close()
    }
}

/// Compass-space `(cx + r·sin(angle), cy - r·cos(angle))`. Shared with sibling
/// polar marks via `pub(crate)`.
pub(crate) fn compass_point(cx: f32, cy: f32, r: f32, angle: f64) -> Point {
    let s = angle.sin() as f32;
    let c = angle.cos() as f32;
    Point::new(cx + r * s, cy - r * c)
}

/// Approximate an arc from `start → end` in compass space using cubic Beziers.
/// Mirrors `pie::arc_to` but in compass coords (theta = 0 up, increasing
/// clockwise) instead of mathematical (theta = 0 right, increasing CCW).
/// Shared with sibling polar marks via `pub(crate)`.
pub(crate) fn arc_compass(path: &mut Path, cx: f32, cy: f32, r: f32, start: f64, end: f64) {
    let segments = ((end - start).abs() / FRAC_PI_2).ceil().max(1.0) as usize;
    let step = (end - start) / segments as f64;
    for s in 0..segments {
        let a0 = start + s as f64 * step;
        let a1 = a0 + step;
        let k = (4.0 / 3.0) * ((a1 - a0) / 4.0).tan();
        let (sin0, cos0) = (a0.sin(), a0.cos());
        let (sin1, cos1) = (a1.sin(), a1.cos());
        // Tangent of compass-space (sin, -cos) is (cos, sin).
        let p1 = Point::new(cx + r * sin1 as f32, cy - r * cos1 as f32);
        let c0 = Point::new(
            cx + r * (sin0 + k * cos0) as f32,
            cy - r * (cos0 - k * sin0) as f32,
        );
        let c1 = Point::new(
            cx + r * (sin1 - k * cos1) as f32,
            cy - r * (cos1 + k * sin1) as f32,
        );
        path.commands.push(PathCommand::CubicTo(c0, c1, p1));
    }
}

#[cfg(test)]
mod tests {
    use super::ArcMark;
    use crate::marks::{LegendGlyph, Mark};
    use starsight_layer_1::primitives::Color;

    #[test]
    fn new_truncates_to_shorter_input() {
        let mark = ArcMark::new(vec![0.0, 1.0, 2.0], vec![10.0, 20.0]);
        assert_eq!(mark.thetas.len(), 2);
        assert_eq!(mark.rs.len(), 2);
    }

    #[test]
    fn default_palette_cycles_when_user_palette_empty() {
        let mark = ArcMark::new(vec![0.0, 1.0], vec![10.0, 20.0]);
        // Different indices pick different default colors.
        assert_ne!(mark.color_at(0), mark.color_at(1));
        // Same index after cycling returns the same color.
        assert_eq!(
            mark.color_at(0),
            mark.color_at(super::DEFAULT_PALETTE.len())
        );
    }

    #[test]
    fn user_palette_cycles_too() {
        let mark = ArcMark::new(vec![0.0, 1.0, 2.0], vec![10.0, 20.0, 30.0])
            .colors(vec![Color::RED, Color::BLUE]);
        assert_eq!(mark.color_at(0), Color::RED);
        assert_eq!(mark.color_at(1), Color::BLUE);
        assert_eq!(mark.color_at(2), Color::RED);
    }

    #[test]
    fn r_inner_default_is_zero() {
        let mark = ArcMark::new(vec![0.0], vec![10.0]);
        assert_eq!(mark.r_inner_at(0), 0.0);
    }

    #[test]
    fn r_inner_uses_provided_when_set() {
        let mark = ArcMark::new(vec![0.0, 1.0], vec![10.0, 20.0]).r_inner(vec![3.0, 5.0]);
        assert_eq!(mark.r_inner_at(0), 3.0);
        assert_eq!(mark.r_inner_at(1), 5.0);
    }

    #[test]
    fn half_width_uniform_builder_sets_all() {
        let mark = ArcMark::new(vec![0.0, 1.0, 2.0], vec![1.0, 2.0, 3.0]).theta_half_width(0.4);
        for i in 0..3 {
            assert!((mark.half_width_at(i) - 0.4).abs() < 1e-9);
        }
    }

    #[test]
    fn half_width_default_uses_min_neighbor_gap() {
        // Three thetas at 0, 1, 3. Min gap = 1 → half-width = 0.5.
        let mark = ArcMark::new(vec![0.0, 1.0, 3.0], vec![10.0, 20.0, 30.0]);
        for i in 0..3 {
            assert!((mark.half_width_at(i) - 0.5).abs() < 1e-9);
        }
    }

    #[test]
    fn legend_glyph_is_bar_and_color_uses_first_slice() {
        let mark = ArcMark::new(vec![0.0, 1.0], vec![10.0, 20.0])
            .colors(vec![Color::GREEN, Color::BLUE])
            .label("series");
        assert_eq!(mark.legend_glyph(), LegendGlyph::Bar);
        assert_eq!(mark.legend_color(), Some(Color::GREEN));
    }

    #[test]
    fn no_legend_when_unlabeled() {
        let mark = ArcMark::new(vec![0.0], vec![10.0]);
        assert!(mark.legend_color().is_none());
    }

    #[test]
    fn does_not_want_axes() {
        let mark = ArcMark::new(vec![0.0], vec![10.0]);
        assert!(!mark.wants_axes());
    }

    #[test]
    fn data_extent_is_none() {
        let mark = ArcMark::new(vec![0.0, 1.0], vec![10.0, 20.0]);
        assert!(mark.data_extent().is_none());
    }
}
