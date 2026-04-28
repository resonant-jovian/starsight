//! Box-and-whisker mark.
//!
//! `BoxPlotMark` renders one box per `BoxPlotGroup`: the box body covers the
//! [Q1, Q3] interquartile range, a horizontal line marks the median, whiskers
//! extend to the most extreme non-outlier samples, and any 1.5×IQR outliers
//! ride alongside as small dots. Groups are positioned by integer band index
//! along x — same approach `BarMark` uses for categorical data, kept here to
//! avoid a dependency on a not-yet-built `BandScale`.
//!
//! Status: lands in 0.3.0 alongside `ViolinMark`. The five-number summary and
//! outlier classification come from
//! [`statistics::BoxPlotStats`](crate::statistics::BoxPlotStats).

#![allow(clippy::cast_precision_loss)]

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathStyle};
use starsight_layer_1::primitives::{Color, Point, Rect};
use starsight_layer_2::coords::CartesianCoord;
use starsight_layer_2::scales::Scale;

use crate::marks::{DataExtent, LegendGlyph, Mark, Orientation};
use crate::statistics::BoxPlotStats;

// ── BoxPlotGroup ─────────────────────────────────────────────────────────────────────────────────

/// One labelled column of samples rendered as a single box-and-whisker.
#[derive(Clone, Debug, PartialEq)]
pub struct BoxPlotGroup {
    /// Group name shown on the category axis.
    pub label: String,
    /// Raw samples. NaN values are filtered before the five-number summary is
    /// computed, so callers can pass sparse columns through directly.
    pub data: Vec<f64>,
}

impl BoxPlotGroup {
    /// Construct a group from a label and a set of samples.
    #[must_use]
    pub fn new(label: impl Into<String>, data: Vec<f64>) -> Self {
        Self {
            label: label.into(),
            data,
        }
    }
}

// ── BoxPlotMark ──────────────────────────────────────────────────────────────────────────────────

/// Box-and-whisker chart driven by [`BoxPlotStats`].
///
/// One box per [`BoxPlotGroup`]. Box width (the `half_width` builder) defaults
/// to roughly 70% of the per-group band; whiskers + caps are drawn in the
/// theme's axis color, the box body in the per-group palette color (or the
/// shared `color` if no palette is supplied).
#[derive(Clone, Debug)]
pub struct BoxPlotMark {
    /// Groups in display order. Each group yields one box.
    pub groups: Vec<BoxPlotGroup>,
    /// Single box-body color; broadcast across every group when `palette` is
    /// `None`. Defaults to the standard cycle blue.
    pub color: Color,
    /// Optional per-group color cycle. When set, the i-th group uses
    /// `palette[i % palette.len()]`. Falls back to `color` if empty.
    pub palette: Option<Vec<Color>>,
    /// Half-width of the box body and cap, as a fraction of the per-group
    /// band. `0.35` reads as a 70%-wide box — a comfortable default that
    /// leaves whitespace between adjacent groups.
    pub half_width: f32,
    /// When `false`, suppress outlier dots even if `BoxPlotStats::compute`
    /// classifies points as outliers. Useful on dense scatter overlays where
    /// the dots become noise.
    pub show_outliers: bool,
    /// Legend label. Boxes don't repeat per-group in the legend; the mark
    /// contributes a single entry that uses the broadcast color.
    pub label: Option<String>,
    /// Cache of `groups[i].label` so [`Mark::as_bar_data`] can hand back a
    /// borrowed slice without allocating per call. Kept in sync with `groups`
    /// in every constructor / builder; not exposed publicly.
    cached_x_labels: Vec<String>,
}

impl BoxPlotMark {
    /// New box plot from a list of groups. Defaults: blue box bodies,
    /// half-width 0.35 of the band, outliers shown.
    #[must_use]
    pub fn new(groups: Vec<BoxPlotGroup>) -> Self {
        let cached_x_labels = groups.iter().map(|g| g.label.clone()).collect();
        Self {
            groups,
            color: Color::BLUE,
            palette: None,
            half_width: 0.35,
            show_outliers: true,
            label: None,
            cached_x_labels,
        }
    }

    /// Builder: broadcast a single body color to every group. Resets any
    /// previously-set `palette`.
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self.palette = None;
        self
    }

    /// Builder: assign a color cycle. The i-th group uses
    /// `palette[i % palette.len()]`; pass an empty vec to clear.
    #[must_use]
    pub fn palette(mut self, palette: Vec<Color>) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Builder: set the half-width as a fraction of the per-group band.
    /// Clamped to `[0.05, 0.5]` at render time so adjacent groups never
    /// overlap.
    #[must_use]
    pub fn half_width(mut self, w: f32) -> Self {
        self.half_width = w;
        self
    }

    /// Builder: enable or disable outlier dots (default: enabled).
    #[must_use]
    pub fn show_outliers(mut self, show: bool) -> Self {
        self.show_outliers = show;
        self
    }

    /// Builder: legend label for this mark.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Color for group `i`, after palette / single-color broadcast.
    fn group_color(&self, i: usize) -> Color {
        match self.palette.as_deref() {
            Some(palette) if !palette.is_empty() => palette[i % palette.len()],
            _ => self.color,
        }
    }

    /// Effective half-width in band-fraction units, clamped so adjacent groups
    /// never collide.
    fn clamped_half_width(&self) -> f32 {
        self.half_width.clamp(0.05, 0.5)
    }
}

impl Mark for BoxPlotMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        if self.groups.is_empty() {
            return Ok(());
        }
        let area = &coord.plot_area;
        let n = self.groups.len();
        let band_width = area.width() / n as f32;
        let half_width_px = band_width * self.clamped_half_width();
        let cap_half = half_width_px * 0.5;

        for (i, group) in self.groups.iter().enumerate() {
            let center_x = area.left + (i as f32 + 0.5) * band_width;
            let stats = BoxPlotStats::compute(&group.data);
            self.render_one(
                coord,
                backend,
                &stats,
                center_x,
                half_width_px,
                cap_half,
                self.group_color(i),
            )?;
        }
        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        if self.groups.is_empty() {
            return None;
        }
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        let mut any = false;
        for group in &self.groups {
            for &v in &group.data {
                if v.is_nan() {
                    continue;
                }
                any = true;
                if v < y_min {
                    y_min = v;
                }
                if v > y_max {
                    y_max = v;
                }
            }
        }
        if !any {
            return None;
        }
        // y range covers the absolute min/max — including outliers — so dots
        // sit safely inside the plot rather than getting clipped by the axis.
        Some(DataExtent {
            x_min: 0.0,
            x_max: self.groups.len() as f64,
            y_min,
            y_max,
        })
    }

    fn legend_color(&self) -> Option<Color> {
        // No label → no legend entry. The `?` consumes the None case so the
        // following expression always returns Some.
        self.label.as_ref()?;
        Some(
            self.palette
                .as_deref()
                .and_then(|p| p.first().copied())
                .unwrap_or(self.color),
        )
    }

    fn legend_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Bar
    }

    fn as_bar_info(&self) -> Option<(Option<&str>, Option<&str>, Orientation)> {
        // Marker for the figure that this mark wants the x-axis treated as a
        // category axis. Group/stack are always None — boxes don't dodge or
        // stack — so the bar-context render paths short-circuit cleanly.
        Some((None, None, Orientation::Vertical))
    }

    fn as_bar_data(&self) -> Option<(&[String], &[f64])> {
        // Hand the cached label list to `Figure::category_labels` so the
        // x-axis shows "placebo" / "low dose" / … instead of band indices.
        // The values half is unused for BoxPlot (no stacking) — empty slice.
        Some((&self.cached_x_labels, &[]))
    }
}

impl BoxPlotMark {
    /// Render a single box: body rect (Q1↔Q3), median line, whiskers, caps,
    /// outliers. Pulled out so the per-group loop in [`Mark::render`] reads
    /// at one level of abstraction.
    #[allow(clippy::too_many_arguments)]
    fn render_one(
        &self,
        coord: &CartesianCoord,
        backend: &mut dyn DrawBackend,
        stats: &BoxPlotStats,
        center_x: f32,
        half_width_px: f32,
        cap_half: f32,
        color: Color,
    ) -> Result<()> {
        let area = &coord.plot_area;
        let to_y =
            |v: f64| -> f32 { area.bottom - coord.y_axis.scale.map(v) as f32 * area.height() };

        let q1_px = to_y(stats.q1);
        let q3_px = to_y(stats.q3);
        let median_px = to_y(stats.median);
        let min_px = to_y(stats.min);
        let max_px = to_y(stats.max);

        // Box body. The y-axis is inverted in screen space so q3 (larger y in
        // data space) lives above q1 (smaller y in data space) — pass them in
        // top/bottom order via the rect ctor.
        let body_top = q3_px.min(q1_px);
        let body_bottom = q3_px.max(q1_px);
        let body = Rect::new(
            center_x - half_width_px,
            body_top,
            center_x + half_width_px,
            body_bottom,
        );
        backend.fill_rect(body, color)?;

        let outline = PathStyle::stroke(Color::BLACK, 1.0);
        // Outline around the box body so the fill reads cleanly against busy
        // backgrounds.
        let outline_path = Path::new()
            .move_to(Point::new(body.left, body.top))
            .line_to(Point::new(body.right, body.top))
            .line_to(Point::new(body.right, body.bottom))
            .line_to(Point::new(body.left, body.bottom))
            .close();
        backend.draw_path(&outline_path, &outline)?;

        // Median line — drawn after the outline so it sits on top.
        let median_line = Path::new()
            .move_to(Point::new(center_x - half_width_px, median_px))
            .line_to(Point::new(center_x + half_width_px, median_px));
        let median_style = PathStyle::stroke(Color::WHITE, 2.0);
        backend.draw_path(&median_line, &median_style)?;

        // Whiskers + caps.
        let whisker_style = PathStyle::stroke(Color::BLACK, 1.0);
        let upper = Path::new()
            .move_to(Point::new(center_x, body_top))
            .line_to(Point::new(center_x, max_px));
        backend.draw_path(&upper, &whisker_style)?;
        let upper_cap = Path::new()
            .move_to(Point::new(center_x - cap_half, max_px))
            .line_to(Point::new(center_x + cap_half, max_px));
        backend.draw_path(&upper_cap, &whisker_style)?;
        let lower = Path::new()
            .move_to(Point::new(center_x, body_bottom))
            .line_to(Point::new(center_x, min_px));
        backend.draw_path(&lower, &whisker_style)?;
        let lower_cap = Path::new()
            .move_to(Point::new(center_x - cap_half, min_px))
            .line_to(Point::new(center_x + cap_half, min_px));
        backend.draw_path(&lower_cap, &whisker_style)?;

        if self.show_outliers {
            for &v in &stats.outliers {
                let py = to_y(v);
                draw_outlier_dot(backend, Point::new(center_x, py), 3.0, Color::BLACK)?;
            }
        }
        Ok(())
    }
}

/// Tiny filled disk for outlier marks. Cubic-bezier circle approximation
/// keeps SVG output deterministic — no backend-specific circle primitive.
fn draw_outlier_dot(
    backend: &mut dyn DrawBackend,
    center: Point,
    radius: f32,
    color: Color,
) -> Result<()> {
    use starsight_layer_1::paths::PathCommand;
    let k = 0.552_284_8 * radius;
    let cx = center.x;
    let cy = center.y;
    let mut path = Path::new().move_to(Point::new(cx + radius, cy));
    path.commands.push(PathCommand::CubicTo(
        Point::new(cx + radius, cy + k),
        Point::new(cx + k, cy + radius),
        Point::new(cx, cy + radius),
    ));
    path.commands.push(PathCommand::CubicTo(
        Point::new(cx - k, cy + radius),
        Point::new(cx - radius, cy + k),
        Point::new(cx - radius, cy),
    ));
    path.commands.push(PathCommand::CubicTo(
        Point::new(cx - radius, cy - k),
        Point::new(cx - k, cy - radius),
        Point::new(cx, cy - radius),
    ));
    path.commands.push(PathCommand::CubicTo(
        Point::new(cx + k, cy - radius),
        Point::new(cx + radius, cy - k),
        Point::new(cx + radius, cy),
    ));
    backend.draw_path(&path, &PathStyle::fill(color))
}

#[cfg(test)]
mod tests {
    use super::{BoxPlotGroup, BoxPlotMark};
    use crate::marks::{LegendGlyph, Mark};
    use starsight_layer_1::primitives::Color;

    #[test]
    fn data_extent_covers_outliers() {
        let mark = BoxPlotMark::new(vec![BoxPlotGroup::new(
            "A",
            // Outlier 100 must be inside the y-extent so it doesn't get
            // clipped at render time.
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 100.0],
        )]);
        let extent = mark.data_extent().expect("non-empty extent");
        assert_eq!(extent.x_min, 0.0);
        assert_eq!(extent.x_max, 1.0);
        assert!(extent.y_min <= 1.0);
        assert!(extent.y_max >= 100.0);
    }

    #[test]
    fn empty_groups_have_no_extent() {
        let mark = BoxPlotMark::new(vec![]);
        assert!(mark.data_extent().is_none());
    }

    #[test]
    fn all_nan_groups_have_no_extent() {
        let mark = BoxPlotMark::new(vec![BoxPlotGroup::new("A", vec![f64::NAN, f64::NAN])]);
        assert!(mark.data_extent().is_none());
    }

    #[test]
    fn legend_color_only_when_labelled() {
        let unlabeled = BoxPlotMark::new(vec![BoxPlotGroup::new("A", vec![1.0, 2.0])]);
        assert!(unlabeled.legend_color().is_none());
        let labeled = unlabeled.color(Color::RED).label("samples");
        assert_eq!(labeled.legend_color(), Some(Color::RED));
        assert_eq!(labeled.legend_glyph(), LegendGlyph::Bar);
    }

    #[test]
    fn palette_overrides_color_for_first_legend_swatch() {
        let mark = BoxPlotMark::new(vec![BoxPlotGroup::new("A", vec![1.0, 2.0])])
            .palette(vec![Color::GREEN, Color::BLUE])
            .label("groups");
        assert_eq!(mark.legend_color(), Some(Color::GREEN));
    }

    #[test]
    fn half_width_clamps_to_safe_range() {
        let mark = BoxPlotMark::new(vec![BoxPlotGroup::new("A", vec![1.0, 2.0])]).half_width(10.0);
        assert!(mark.clamped_half_width() <= 0.5);
        let tiny = BoxPlotMark::new(vec![BoxPlotGroup::new("A", vec![1.0, 2.0])]).half_width(-1.0);
        assert!(tiny.clamped_half_width() >= 0.05);
    }
}
