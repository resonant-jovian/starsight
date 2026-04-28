//! Kernel-density violin mark.
//!
//! `ViolinMark` shows a sample's density rather than the five-number summary
//! a [`BoxPlotMark`](super::BoxPlotMark) emphasises. Each [`ViolinGroup`] gets
//! a vertically-oriented mirrored shape: the right half traces the kernel
//! density estimate from the data range top to bottom; the left half
//! mirrors it. An optional inner box overlay (driven by
//! [`BoxPlotStats`](crate::statistics::BoxPlotStats)) carries the quartile
//! summary inside the density envelope. With `split = true` and exactly two
//! groups, the two densities share an x-band: group A occupies the left
//! half, group B the right.
//!
//! Status: lands in 0.3.0. Density math comes from
//! [`Kde`](crate::statistics::Kde); bandwidth selection is configurable per
//! mark. The 256-point evaluation grid is wide enough to read smoothly at
//! gallery sizes without paying for finer resolution most readers will
//! never see.

#![allow(clippy::cast_precision_loss)]

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathStyle};
use starsight_layer_1::primitives::{Color, Point, Rect};
use starsight_layer_2::coords::CartesianCoord;
use starsight_layer_2::scales::Scale;

use crate::marks::{DataExtent, LegendGlyph, Mark, Orientation};
use crate::statistics::{Bandwidth, BoxPlotStats, Kde, Kernel};

/// Number of points in the per-group KDE evaluation grid. 256 is the
/// sweet-spot reported by the spec: enough for smooth contours at typical
/// figure sizes, cheap enough for several groups per chart.
const GRID_POINTS: usize = 256;

// ── ViolinGroup ──────────────────────────────────────────────────────────────────────────────────

/// A labelled column of samples rendered as one violin.
#[derive(Clone, Debug, PartialEq)]
pub struct ViolinGroup {
    /// Group name shown on the category axis.
    pub label: String,
    /// Raw samples. NaN values are filtered before density estimation.
    pub data: Vec<f64>,
}

impl ViolinGroup {
    /// Construct a group from a label and a set of samples.
    #[must_use]
    pub fn new(label: impl Into<String>, data: Vec<f64>) -> Self {
        Self {
            label: label.into(),
            data,
        }
    }
}

// ── ViolinScale ──────────────────────────────────────────────────────────────────────────────────

/// How the per-group densities are normalised against each other.
///
/// Using `Area` (the default) keeps the integral under each violin equal
/// across groups — the right call when the reader is comparing shapes, not
/// magnitudes. `Count` weights each violin by its sample size so groups with
/// more data look bigger. `Width` forces every violin to the same peak
/// width, which is honest when group sample sizes are wildly different and
/// you want shape comparison without sample-size visual bias.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ViolinScale {
    /// Equal area per violin (default).
    #[default]
    Area,
    /// Scale by sample size.
    Count,
    /// Equal max width per violin.
    Width,
}

// ── ViolinMark ───────────────────────────────────────────────────────────────────────────────────

/// Density-driven companion to [`BoxPlotMark`](super::BoxPlotMark).
#[derive(Clone, Debug)]
pub struct ViolinMark {
    /// Groups in display order.
    pub groups: Vec<ViolinGroup>,
    /// KDE config (bandwidth strategy + kernel). Defaults to Silverman /
    /// Gaussian — the right call for unimodal-ish data.
    pub kde: Kde,
    /// Shared body color when `palette` is `None`.
    pub color: Color,
    /// Optional per-group color cycle.
    pub palette: Option<Vec<Color>>,
    /// Half-width of each violin as a fraction of its band. `0.4` ≈ 80% of
    /// the band, leaving breathing room.
    pub half_width: f32,
    /// When true, render an inner [`BoxPlotStats`] mini box plot inside each
    /// violin. Default: true.
    pub show_box: bool,
    /// When true, draw a horizontal median line across the violin even when
    /// `show_box` is false. Default: true.
    pub show_median: bool,
    /// Extend the density grid by `cut * bandwidth` past min and max. The
    /// usual default is `2.0`, which shows the full Gaussian-tail decay
    /// without dragging the violin halfway off the chart on heavy-tailed
    /// data.
    pub cut: f64,
    /// Per-group normalisation strategy.
    pub scale: ViolinScale,
    /// When true and `groups.len() == 2`, draw group 0 on the left half of a
    /// single band and group 1 on the right half — handy for paired
    /// before/after comparisons.
    pub split: bool,
    /// Legend label.
    pub label: Option<String>,
    /// Cache of per-group labels for [`Mark::as_bar_data`].
    cached_x_labels: Vec<String>,
}

impl ViolinMark {
    /// New violin chart from a list of groups.
    #[must_use]
    pub fn new(groups: Vec<ViolinGroup>) -> Self {
        let cached_x_labels = groups.iter().map(|g| g.label.clone()).collect();
        Self {
            groups,
            kde: Kde::new(Bandwidth::Silverman, Kernel::Gaussian),
            color: Color::BLUE,
            palette: None,
            half_width: 0.4,
            show_box: true,
            show_median: true,
            cut: 2.0,
            scale: ViolinScale::Area,
            split: false,
            label: None,
            cached_x_labels,
        }
    }

    /// Builder: override the bandwidth strategy.
    #[must_use]
    pub fn bandwidth(mut self, b: Bandwidth) -> Self {
        self.kde.bandwidth = b;
        self
    }

    /// Builder: override the kernel.
    #[must_use]
    pub fn kernel(mut self, k: Kernel) -> Self {
        self.kde.kernel = k;
        self
    }

    /// Builder: broadcast a single body color (clears any palette).
    #[must_use]
    pub fn color(mut self, c: Color) -> Self {
        self.color = c;
        self.palette = None;
        self
    }

    /// Builder: per-group color cycle.
    #[must_use]
    pub fn palette(mut self, palette: Vec<Color>) -> Self {
        self.palette = Some(palette);
        self
    }

    /// Builder: half-width as a fraction of the band, clamped to `[0.05, 0.5]`.
    #[must_use]
    pub fn half_width(mut self, w: f32) -> Self {
        self.half_width = w;
        self
    }

    /// Builder: enable or disable the inner box overlay.
    #[must_use]
    pub fn show_box(mut self, show: bool) -> Self {
        self.show_box = show;
        self
    }

    /// Builder: enable or disable the standalone median line. Has no effect
    /// when [`show_box`](Self::show_box) is true (the inner box already
    /// carries the median).
    #[must_use]
    pub fn show_median(mut self, show: bool) -> Self {
        self.show_median = show;
        self
    }

    /// Builder: set the `cut` multiplier — grid extension past data range in
    /// units of bandwidth.
    #[must_use]
    pub fn cut(mut self, cut: f64) -> Self {
        self.cut = cut;
        self
    }

    /// Builder: per-group normalisation strategy.
    #[must_use]
    pub fn scale(mut self, scale: ViolinScale) -> Self {
        self.scale = scale;
        self
    }

    /// Builder: enable split-violin mode (only meaningful with exactly two
    /// groups).
    #[must_use]
    pub fn split(mut self, split: bool) -> Self {
        self.split = split;
        self
    }

    /// Builder: legend label for the mark.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn group_color(&self, i: usize) -> Color {
        match self.palette.as_deref() {
            Some(palette) if !palette.is_empty() => palette[i % palette.len()],
            _ => self.color,
        }
    }

    fn clamped_half_width(&self) -> f32 {
        self.half_width.clamp(0.05, 0.5)
    }
}

/// One group's density evaluation, reused by `render` and `data_extent`.
struct DensityRow {
    /// Y values where the kernel was sampled (data units).
    grid: Vec<f64>,
    /// Density at each grid point.
    density: Vec<f64>,
    /// Maximum density in this group — used for normalisation.
    d_max: f64,
}

impl ViolinMark {
    /// Compute the KDE for one group on the configured `cut`-extended grid.
    /// Returns `None` if the group has fewer than two finite samples or no
    /// spread (constant data); the renderer skips such groups instead of
    /// drawing a degenerate flat line.
    fn density_for(&self, group: &ViolinGroup) -> Option<DensityRow> {
        let data: Vec<f64> = group.data.iter().copied().filter(|v| !v.is_nan()).collect();
        if data.len() < 2 {
            return None;
        }
        let bw = self.kde.resolve_bandwidth(&data);
        if bw <= 0.0 {
            return None;
        }
        let y_min = data.iter().copied().fold(f64::INFINITY, f64::min);
        let y_max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let span = (y_max - y_min) + 2.0 * self.cut * bw;
        let start = y_min - self.cut * bw;
        let grid: Vec<f64> = (0..GRID_POINTS)
            .map(|i| start + span * (i as f64) / ((GRID_POINTS - 1) as f64))
            .collect();
        let density = self.kde.evaluate_grid(&grid, &data);
        let d_max = density.iter().copied().fold(0.0_f64, f64::max);
        if d_max <= 0.0 {
            return None;
        }
        Some(DensityRow {
            grid,
            density,
            d_max,
        })
    }

    /// Resolve the visual scale factor that turns raw density values into
    /// half-widths in pixel-space units. Combines the user-facing
    /// [`ViolinScale`] choice with each row's `d_max` and the band-relative
    /// half-width.
    fn width_factor(&self, row: &DensityRow, group: &ViolinGroup, max_d_max: f64) -> f64 {
        match self.scale {
            ViolinScale::Area => 1.0 / row.d_max,
            ViolinScale::Count => {
                (group.data.len() as f64) / (row.d_max * row.density.len() as f64).max(1e-12)
            }
            ViolinScale::Width => {
                if max_d_max <= 0.0 {
                    1.0 / row.d_max
                } else {
                    1.0 / max_d_max
                }
            }
        }
    }
}

impl Mark for ViolinMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        if self.groups.is_empty() {
            return Ok(());
        }
        // Pre-compute every group's KDE once. Skipping degenerate rows here
        // keeps the band-layout math below from getting tangled with NaN
        // / constant-data branching.
        let rows: Vec<(usize, DensityRow)> = self
            .groups
            .iter()
            .enumerate()
            .filter_map(|(i, g)| self.density_for(g).map(|r| (i, r)))
            .collect();
        if rows.is_empty() {
            return Ok(());
        }
        let max_d_max = rows.iter().map(|(_, r)| r.d_max).fold(0.0_f64, f64::max);

        let area = &coord.plot_area;

        if self.split && self.groups.len() == 2 && rows.len() == 2 {
            // Split mode: both groups share band 0.5..1.5 (the only band).
            // Group 0 → left half, group 1 → right half.
            let center_x = area.left + 0.5 * area.width();
            let half_width_px = self.clamped_half_width() * area.width();
            self.render_half(
                coord,
                backend,
                &rows[0].1,
                &self.groups[0],
                max_d_max,
                center_x,
                half_width_px,
                self.group_color(0),
                Side::Left,
            )?;
            self.render_half(
                coord,
                backend,
                &rows[1].1,
                &self.groups[1],
                max_d_max,
                center_x,
                half_width_px,
                self.group_color(1),
                Side::Right,
            )?;
            return Ok(());
        }

        let n = self.groups.len();
        let band_width = area.width() / n as f32;
        let half_width_px = band_width * self.clamped_half_width();

        for (i, row) in rows {
            let group = &self.groups[i];
            let center_x = area.left + (i as f32 + 0.5) * band_width;
            let color = self.group_color(i);
            // Full violin: left half + right half drawn as a closed mirrored path.
            self.render_full(
                coord,
                backend,
                &row,
                group,
                max_d_max,
                center_x,
                half_width_px,
                color,
            )?;
        }

        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        // y range covers each group's [min, max] (with NaN filtered) plus the
        // cut-extension, so the violins don't get clipped at the axis. Groups
        // that won't render — fewer than two samples or a zero bandwidth —
        // are skipped here so the axis isn't sized by data the renderer
        // ignores.
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        let mut any = false;
        for group in &self.groups {
            let data: Vec<f64> = group.data.iter().copied().filter(|v| !v.is_nan()).collect();
            if data.len() < 2 {
                continue;
            }
            let bw = self.kde.resolve_bandwidth(&data);
            if bw <= 0.0 {
                continue;
            }
            let local_min = data.iter().copied().fold(f64::INFINITY, f64::min);
            let local_max = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            any = true;
            let lower = local_min - self.cut * bw;
            let upper = local_max + self.cut * bw;
            if lower < y_min {
                y_min = lower;
            }
            if upper > y_max {
                y_max = upper;
            }
        }
        if !any {
            return None;
        }
        let n = if self.split && self.groups.len() == 2 {
            1
        } else {
            self.groups.len()
        };
        Some(DataExtent {
            x_min: 0.0,
            x_max: n as f64,
            y_min,
            y_max,
        })
    }

    fn legend_color(&self) -> Option<Color> {
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
        Some((None, None, Orientation::Vertical))
    }

    fn as_bar_data(&self) -> Option<(&[String], &[f64])> {
        Some((&self.cached_x_labels, &[]))
    }
}

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

impl ViolinMark {
    /// Draw a single full (mirrored) violin: right side top→bottom, left side
    /// bottom→top, closed.
    #[allow(clippy::too_many_arguments)]
    fn render_full(
        &self,
        coord: &CartesianCoord,
        backend: &mut dyn DrawBackend,
        row: &DensityRow,
        group: &ViolinGroup,
        max_d_max: f64,
        center_x: f32,
        half_width_px: f32,
        color: Color,
    ) -> Result<()> {
        let factor = self.width_factor(row, group, max_d_max);
        let area = &coord.plot_area;
        let to_y =
            |v: f64| -> f32 { area.bottom - coord.y_axis.scale.map(v) as f32 * area.height() };
        let to_dx = |d: f64| -> f32 { (d * factor) as f32 * half_width_px };

        let mut path = Path::new();
        // Right side top→bottom.
        for i in (0..row.grid.len()).rev() {
            let py = to_y(row.grid[i]);
            let dx = to_dx(row.density[i]);
            let p = Point::new(center_x + dx, py);
            if i == row.grid.len() - 1 {
                path = path.move_to(p);
            } else {
                path = path.line_to(p);
            }
        }
        // Left side bottom→top, mirrored.
        for i in 0..row.grid.len() {
            let py = to_y(row.grid[i]);
            let dx = to_dx(row.density[i]);
            path = path.line_to(Point::new(center_x - dx, py));
        }
        path = path.close();

        let style = PathStyle {
            stroke_color: color.darker_for_outline(),
            stroke_width: 1.0,
            fill_color: Some(color.with_alpha(180).without_alpha()),
            opacity: 1.0,
            ..PathStyle::default()
        };
        backend.draw_path(&path, &style)?;

        // Inner box / median overlays.
        let inner_half = half_width_px * 0.18;
        if self.show_box {
            self.render_inner_box(coord, backend, group, center_x, inner_half)?;
        } else if self.show_median {
            let stats = BoxPlotStats::compute(&group.data);
            let median_px = to_y(stats.median);
            let median_line = Path::new()
                .move_to(Point::new(center_x - half_width_px * 0.5, median_px))
                .line_to(Point::new(center_x + half_width_px * 0.5, median_px));
            backend.draw_path(&median_line, &PathStyle::stroke(Color::BLACK, 1.5))?;
        }
        Ok(())
    }

    /// Draw a half violin (left or right side of the band) for split mode.
    #[allow(clippy::too_many_arguments)]
    fn render_half(
        &self,
        coord: &CartesianCoord,
        backend: &mut dyn DrawBackend,
        row: &DensityRow,
        group: &ViolinGroup,
        max_d_max: f64,
        center_x: f32,
        half_width_px: f32,
        color: Color,
        side: Side,
    ) -> Result<()> {
        let factor = self.width_factor(row, group, max_d_max);
        let area = &coord.plot_area;
        let to_y =
            |v: f64| -> f32 { area.bottom - coord.y_axis.scale.map(v) as f32 * area.height() };
        let to_dx = |d: f64| -> f32 { (d * factor) as f32 * half_width_px };

        let signed = |dx: f32| match side {
            Side::Right => center_x + dx,
            Side::Left => center_x - dx,
        };

        let mut path = Path::new();
        // Outer arc: top→bottom along the side.
        for i in (0..row.grid.len()).rev() {
            let py = to_y(row.grid[i]);
            let dx = to_dx(row.density[i]);
            let p = Point::new(signed(dx), py);
            if i == row.grid.len() - 1 {
                path = path.move_to(p);
            } else {
                path = path.line_to(p);
            }
        }
        // Close along the centre line.
        path = path.line_to(Point::new(center_x, to_y(row.grid[0])));
        path = path.line_to(Point::new(center_x, to_y(row.grid[row.grid.len() - 1])));
        path = path.close();

        let style = PathStyle {
            stroke_color: color.darker_for_outline(),
            stroke_width: 1.0,
            fill_color: Some(color.with_alpha(180).without_alpha()),
            opacity: 1.0,
            ..PathStyle::default()
        };
        backend.draw_path(&path, &style)?;

        if self.show_median {
            let stats = BoxPlotStats::compute(&group.data);
            let median_px = to_y(stats.median);
            let inner_x = match side {
                Side::Right => center_x + half_width_px * 0.5,
                Side::Left => center_x - half_width_px * 0.5,
            };
            let line = Path::new()
                .move_to(Point::new(center_x, median_px))
                .line_to(Point::new(inner_x, median_px));
            backend.draw_path(&line, &PathStyle::stroke(Color::BLACK, 1.5))?;
        }
        Ok(())
    }

    /// Inner box overlay: a slim BoxPlotStats-driven mini box plot rendered
    /// on top of the violin shape.
    #[allow(clippy::unused_self)] // Kept as a method for grouping with the
    // other render_* helpers; future extensions (configurable inner-box
    // width, alpha, etc.) will read from `self`.
    fn render_inner_box(
        &self,
        coord: &CartesianCoord,
        backend: &mut dyn DrawBackend,
        group: &ViolinGroup,
        center_x: f32,
        half_width_px: f32,
    ) -> Result<()> {
        let stats = BoxPlotStats::compute(&group.data);
        let area = &coord.plot_area;
        let to_y =
            |v: f64| -> f32 { area.bottom - coord.y_axis.scale.map(v) as f32 * area.height() };
        let q1_px = to_y(stats.q1);
        let q3_px = to_y(stats.q3);
        let median_px = to_y(stats.median);
        let min_px = to_y(stats.min);
        let max_px = to_y(stats.max);

        let body_top = q3_px.min(q1_px);
        let body_bottom = q3_px.max(q1_px);
        let body = Rect::new(
            center_x - half_width_px,
            body_top,
            center_x + half_width_px,
            body_bottom,
        );
        backend.fill_rect(body, Color::BLACK)?;

        let median_line = Path::new()
            .move_to(Point::new(center_x - half_width_px, median_px))
            .line_to(Point::new(center_x + half_width_px, median_px));
        backend.draw_path(&median_line, &PathStyle::stroke(Color::WHITE, 1.5))?;

        let whisker = Path::new()
            .move_to(Point::new(center_x, min_px))
            .line_to(Point::new(center_x, max_px));
        backend.draw_path(&whisker, &PathStyle::stroke(Color::BLACK, 1.0))?;
        Ok(())
    }
}

/// Color helper kept inline so violin.rs doesn't reach into layer-1
/// primitives for a one-off operation. Returns a darker variant of `color`
/// suitable for an outline against the translucent body fill.
trait ColorOutlineExt {
    fn darker_for_outline(&self) -> Self;
}

impl ColorOutlineExt for Color {
    fn darker_for_outline(&self) -> Self {
        // 70% of each channel → readable outline against a translucent fill.
        let scale = 0.7;
        Color::new(
            (f32::from(self.r) * scale) as u8,
            (f32::from(self.g) * scale) as u8,
            (f32::from(self.b) * scale) as u8,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{ViolinGroup, ViolinMark, ViolinScale};
    use crate::marks::{LegendGlyph, Mark};
    use crate::statistics::Bandwidth;
    use starsight_layer_1::primitives::Color;

    #[test]
    fn data_extent_includes_cut_extension() {
        let mark = ViolinMark::new(vec![ViolinGroup::new(
            "A",
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        )])
        .cut(2.0)
        .bandwidth(Bandwidth::Manual(0.5));
        let extent = mark.data_extent().expect("non-empty extent");
        // bw=0.5, cut=2 → extension = 1.0 each side. Data [1, 10] → extent ≥ [0, 11].
        assert!(extent.y_min <= 0.0 + 1e-9);
        assert!(extent.y_max >= 11.0 - 1e-9);
    }

    #[test]
    fn empty_groups_have_no_extent() {
        let mark = ViolinMark::new(vec![]);
        assert!(mark.data_extent().is_none());
    }

    #[test]
    fn constant_data_groups_have_no_extent() {
        let mark = ViolinMark::new(vec![ViolinGroup::new("A", vec![5.0; 10])]);
        // Constant data → bandwidth 0 → no density → no extent.
        assert!(mark.data_extent().is_none());
    }

    #[test]
    fn split_collapses_x_range_to_one_band() {
        let mark = ViolinMark::new(vec![
            ViolinGroup::new("A", (1..=20).map(f64::from).collect()),
            ViolinGroup::new("B", (5..=24).map(f64::from).collect()),
        ])
        .split(true);
        let extent = mark.data_extent().expect("non-empty extent");
        assert_eq!(extent.x_min, 0.0);
        assert_eq!(extent.x_max, 1.0);
    }

    #[test]
    fn legend_dispatches_bar_glyph() {
        let mark = ViolinMark::new(vec![ViolinGroup::new("A", vec![1.0, 2.0, 3.0])])
            .label("samples")
            .color(Color::RED);
        assert_eq!(mark.legend_glyph(), LegendGlyph::Bar);
        assert_eq!(mark.legend_color(), Some(Color::RED));
    }

    #[test]
    fn scale_variants_are_distinct() {
        // Sanity: builders accept all three variants.
        for scale in [ViolinScale::Area, ViolinScale::Count, ViolinScale::Width] {
            let m = ViolinMark::new(vec![ViolinGroup::new("A", vec![1.0, 2.0, 3.0])]).scale(scale);
            assert_eq!(m.scale, scale);
        }
    }
}
