//! Pie / donut mark.
//!
//! `PieMark` carves the plot area into wedge-shaped slices proportional to a
//! list of values. Setting [`PieMark::inner_radius`] to a non-zero fraction
//! turns it into a donut. Slices are filled from a palette and optionally
//! labelled with their share (percentage or absolute value) at the midpoint
//! angle.
//!
//! Status: lands in 0.3.0. The arc geometry uses the four-cubic-Bezier-per-
//! quarter-circle approximation with the standard `4·tan((θ_end−θ_start)/4)/3`
//! tangent length, which keeps SVG and raster output deterministic without a
//! backend-specific arc primitive.

#![allow(clippy::cast_precision_loss)]

use std::f64::consts::{FRAC_PI_2, TAU};

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathCommand, PathStyle};
use starsight_layer_1::primitives::{Color, Point};
use starsight_layer_2::coords::CartesianCoord;

use crate::marks::{DataExtent, LegendGlyph, Mark};

// ── PieMark ──────────────────────────────────────────────────────────────────────────────────────

/// What text to draw on each slice (if any).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum PieLabelMode {
    /// No labels.
    #[default]
    None,
    /// Render `XX%` of the total at each slice midpoint.
    Percent,
    /// Render the raw value at each slice midpoint.
    Value,
}

/// Pie or donut chart.
#[derive(Clone, Debug)]
pub struct PieMark {
    /// Slice values. Negative or zero values are skipped at render time.
    pub values: Vec<f64>,
    /// Slice labels, parallel to `values`. Optional; falls back to indices.
    pub labels: Vec<String>,
    /// Color cycle. Slice `i` uses `palette[i % palette.len()]`. Empty
    /// palette → BLUE for every slice.
    pub palette: Vec<Color>,
    /// Inner radius as a fraction of the outer radius. `0.0` → solid pie;
    /// `0.5` → typical donut. Clamped to `[0.0, 0.95]` at render time.
    pub inner_radius_fraction: f32,
    /// Outer radius as a fraction of `min(plot_width, plot_height) / 2`.
    /// `0.85` leaves a small breathing margin so labels and slice edges
    /// don't graze the plot edge.
    pub outer_radius_fraction: f32,
    /// Where the first slice starts. `-π/2` (top) is the conventional
    /// default; `0.0` starts at the right.
    pub start_angle: f64,
    /// What labels to render on each slice.
    pub label_mode: PieLabelMode,
    /// Color of slice text labels.
    pub label_color: Color,
    /// Legend label for the mark as a whole.
    pub label: Option<String>,
}

impl PieMark {
    /// New pie chart from values and slice labels. Pass an empty `Vec` for
    /// labels to fall back to slice indices.
    #[must_use]
    pub fn new(values: Vec<f64>, labels: Vec<String>) -> Self {
        Self {
            values,
            labels,
            palette: default_palette(),
            inner_radius_fraction: 0.0,
            outer_radius_fraction: 0.85,
            start_angle: -FRAC_PI_2,
            label_mode: PieLabelMode::None,
            label_color: Color::BLACK,
            label: None,
        }
    }

    /// Builder: convenience for the donut variant. `0.5` is the conventional
    /// donut hole; `0.7` reads as a thin ring.
    #[must_use]
    pub fn inner_radius(mut self, fraction: f32) -> Self {
        self.inner_radius_fraction = fraction;
        self
    }

    /// Builder: shrink or expand the pie inside the plot area. Useful when
    /// labels need extra room outside the slices.
    #[must_use]
    pub fn outer_radius(mut self, fraction: f32) -> Self {
        self.outer_radius_fraction = fraction;
        self
    }

    /// Builder: rotate the first slice's start angle. Radians.
    #[must_use]
    pub fn start_angle(mut self, angle: f64) -> Self {
        self.start_angle = angle;
        self
    }

    /// Builder: replace the palette. Empty vec → fall back to BLUE.
    #[must_use]
    pub fn palette(mut self, palette: Vec<Color>) -> Self {
        self.palette = palette;
        self
    }

    /// Builder: render percentages at each slice midpoint.
    #[must_use]
    pub fn show_percent(mut self) -> Self {
        self.label_mode = PieLabelMode::Percent;
        self
    }

    /// Builder: render raw values at each slice midpoint.
    #[must_use]
    pub fn show_values(mut self) -> Self {
        self.label_mode = PieLabelMode::Value;
        self
    }

    /// Builder: legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn slice_color(&self, i: usize) -> Color {
        if self.palette.is_empty() {
            Color::BLUE
        } else {
            self.palette[i % self.palette.len()]
        }
    }
}

impl Mark for PieMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let total: f64 = self.values.iter().filter(|&&v| v > 0.0).sum();
        if total <= 0.0 || self.values.is_empty() {
            return Ok(());
        }
        let area = &coord.plot_area;
        let cx = (area.left + area.right) * 0.5;
        let cy = (area.top + area.bottom) * 0.5;
        let plot_min = area.width().min(area.height());
        let outer_r = plot_min * self.outer_radius_fraction.clamp(0.05, 0.5);
        let inner_r = outer_r * self.inner_radius_fraction.clamp(0.0, 0.95);

        let mut start_angle = self.start_angle;
        for (i, &value) in self.values.iter().enumerate() {
            if value <= 0.0 {
                continue;
            }
            let sweep = (value / total) * TAU;
            let end_angle = start_angle + sweep;

            let path = if inner_r > 0.0 {
                build_donut_slice(cx, cy, inner_r, outer_r, start_angle, end_angle)
            } else {
                build_pie_slice(cx, cy, outer_r, start_angle, end_angle)
            };
            let slice_color = self.slice_color(i);
            let style = PathStyle {
                stroke_color: Color::WHITE,
                stroke_width: 1.0,
                fill_color: Some(slice_color),
                ..PathStyle::default()
            };
            backend.draw_path(&path, &style)?;

            if !matches!(self.label_mode, PieLabelMode::None) {
                let mid_angle = start_angle + sweep * 0.5;
                let label_r = if inner_r > 0.0 {
                    (inner_r + outer_r) * 0.5
                } else {
                    outer_r * 0.65
                };
                let lx = cx + (f64::from(label_r) * mid_angle.cos()) as f32;
                let ly = cy + (f64::from(label_r) * mid_angle.sin()) as f32;
                let text = match self.label_mode {
                    PieLabelMode::Percent => format!("{:.0}%", 100.0 * value / total),
                    PieLabelMode::Value => format!("{value:.0}"),
                    PieLabelMode::None => String::new(),
                };
                let font_size = 12.0_f32;
                let (tw, _) = backend
                    .text_extent(&text, font_size)
                    .unwrap_or((0.0, font_size));
                // Auto-pick label color for contrast against the slice fill
                // (yrp.3). The user-facing `.label_color(c)` builder still
                // wins when explicitly set away from the BLACK default.
                let resolved_label_color = if self.label_color == Color::BLACK {
                    if luminance(slice_color) < 0.5 {
                        Color::WHITE
                    } else {
                        Color::BLACK
                    }
                } else {
                    self.label_color
                };
                backend.draw_text(
                    &text,
                    Point::new(lx - tw * 0.5, ly + font_size * 0.4),
                    font_size,
                    resolved_label_color,
                )?;
            }
            start_angle = end_angle;
        }
        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        // Pie/donut uses absolute screen coordinates inside the plot area
        // so its slice values aren't mappable through the figure's
        // Wilkinson-driven numeric axis. We still return a unit-square
        // extent so the figure has *something* to scale to; without this
        // the figure's `merged_extent` rejects pie-only charts as "No data
        // to render". Callers usually pair this with `.x_label("")` /
        // `.y_label("")` for a label-free presentation.
        Some(DataExtent {
            x_min: 0.0,
            x_max: 1.0,
            y_min: 0.0,
            y_max: 1.0,
        })
    }

    fn legend_color(&self) -> Option<Color> {
        self.label.as_ref()?;
        Some(self.slice_color(0))
    }

    fn legend_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Bar
    }

    fn wants_axes(&self) -> bool {
        // Pie / donut charts are angular — numeric x/y axes around them
        // are visual noise. The figure suppresses axes + grid when every
        // mark on it returns `false` here (yrp.2).
        false
    }
}

/// Rec. 601 luminance for a sRGB color, in `[0.0, 1.0]`. Used to pick a
/// readable label color against an arbitrary slice fill.
fn luminance(c: Color) -> f32 {
    let r = f32::from(c.r) / 255.0;
    let g = f32::from(c.g) / 255.0;
    let b = f32::from(c.b) / 255.0;
    0.299 * r + 0.587 * g + 0.114 * b
}

// ── arc geometry ─────────────────────────────────────────────────────────────────────────────────

/// Append a forward arc (radius `r`, angles `start → end`) onto `path`.
/// Uses the four-quadrant cubic-Bezier circle approximation: each ≤π/2
/// segment is a single cubic with tangent length `4/3 · tan(Δθ/4) · r`.
fn arc_to(path: &mut Path, cx: f32, cy: f32, r: f32, start: f64, end: f64) {
    let segments = ((end - start).abs() / FRAC_PI_2).ceil().max(1.0) as usize;
    let step = (end - start) / segments as f64;
    for s in 0..segments {
        let a0 = start + s as f64 * step;
        let a1 = a0 + step;
        let k = (4.0 / 3.0) * ((a1 - a0) / 4.0).tan();
        let (sin0, cos0) = (a0.sin(), a0.cos());
        let (sin1, cos1) = (a1.sin(), a1.cos());
        let p1 = Point::new(cx + r * cos1 as f32, cy + r * sin1 as f32);
        let c0 = Point::new(
            cx + r * (cos0 - k * sin0) as f32,
            cy + r * (sin0 + k * cos0) as f32,
        );
        let c1 = Point::new(
            cx + r * (cos1 + k * sin1) as f32,
            cy + r * (sin1 - k * cos1) as f32,
        );
        path.commands.push(PathCommand::CubicTo(c0, c1, p1));
    }
}

fn build_pie_slice(cx: f32, cy: f32, r: f32, start: f64, end: f64) -> Path {
    let p0 = Point::new(cx + r * start.cos() as f32, cy + r * start.sin() as f32);
    let mut path = Path::new().move_to(Point::new(cx, cy)).line_to(p0);
    arc_to(&mut path, cx, cy, r, start, end);
    path.close()
}

fn build_donut_slice(cx: f32, cy: f32, inner_r: f32, outer_r: f32, start: f64, end: f64) -> Path {
    // Outer arc start point. The path is: move to outer_start, arc forward to
    // outer_end, line in to inner_end, arc backward to inner_start, close.
    let outer_start = Point::new(
        cx + outer_r * start.cos() as f32,
        cy + outer_r * start.sin() as f32,
    );
    let mut path = Path::new().move_to(outer_start);
    arc_to(&mut path, cx, cy, outer_r, start, end);
    let inner_end = Point::new(
        cx + inner_r * end.cos() as f32,
        cy + inner_r * end.sin() as f32,
    );
    path = path.line_to(inner_end);
    arc_to(&mut path, cx, cy, inner_r, end, start);
    path.close()
}

// ── default palette ──────────────────────────────────────────────────────────────────────────────

/// Six perceptually-distinct hues — readable on light and dark themes alike.
/// Cheap to ship inline; users override via [`PieMark::palette`].
fn default_palette() -> Vec<Color> {
    vec![
        Color::from_hex(0x0033_77BB),
        Color::from_hex(0x00EE_7733),
        Color::from_hex(0x0033_AA66),
        Color::from_hex(0x00CC_3366),
        Color::from_hex(0x00AA_44AA),
        Color::from_hex(0x0099_AABB),
    ]
}

#[cfg(test)]
mod tests {
    use super::{PieLabelMode, PieMark};
    use crate::marks::{LegendGlyph, Mark};
    use starsight_layer_1::primitives::Color;

    #[test]
    fn data_extent_is_unit_square() {
        let mark = PieMark::new(
            vec![1.0, 2.0, 3.0],
            vec!["a".into(), "b".into(), "c".into()],
        );
        let extent = mark
            .data_extent()
            .expect("pie reports a placeholder extent");
        assert_eq!(extent.x_min, 0.0);
        assert_eq!(extent.x_max, 1.0);
        assert_eq!(extent.y_min, 0.0);
        assert_eq!(extent.y_max, 1.0);
    }

    #[test]
    fn donut_inner_radius_round_trip() {
        let mark = PieMark::new(vec![1.0, 1.0], vec!["a".into(), "b".into()]).inner_radius(0.5);
        assert!((mark.inner_radius_fraction - 0.5).abs() < 1e-9);
    }

    #[test]
    fn percent_and_value_label_modes() {
        let pct = PieMark::new(vec![1.0], vec!["a".into()]).show_percent();
        let val = PieMark::new(vec![1.0], vec!["a".into()]).show_values();
        assert_eq!(pct.label_mode, PieLabelMode::Percent);
        assert_eq!(val.label_mode, PieLabelMode::Value);
    }

    #[test]
    fn legend_glyph_is_bar_and_color_uses_palette_first() {
        let mark = PieMark::new(vec![1.0, 1.0], vec![])
            .palette(vec![Color::RED, Color::GREEN])
            .label("shares");
        assert_eq!(mark.legend_glyph(), LegendGlyph::Bar);
        assert_eq!(mark.legend_color(), Some(Color::RED));
    }

    #[test]
    fn no_legend_when_unlabeled() {
        let mark = PieMark::new(vec![1.0], vec![]);
        assert!(mark.legend_color().is_none());
    }
}
