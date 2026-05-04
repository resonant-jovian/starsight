//! Contour mark — isolines and filled bands over a 2-D scalar grid.
//!
//! Backed by [`crate::statistics::Contour`]'s marching-squares extractor for
//! the isoline path, and by per-cell Sutherland-Hodgman polygon clipping for
//! the filled-band path. `ContourMark` accepts a [`Grid`] plus a list of
//! levels and renders any combination of isolines and filled bands per
//! [`ContourMode`]. A colormap optionally tints each level (for isolines) or
//! each between-level band (for filled rendering).
//!
//! Status: lands in 0.3.0 — isolines, filled bands, and combined mode are
//! all supported.

#![allow(clippy::cast_precision_loss)]

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::colormap::Colormap;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathStyle};
use starsight_layer_1::primitives::{Color, Point};
use starsight_layer_2::coords::Coord;

use crate::marks::{DataExtent, LegendGlyph, Mark};
use crate::statistics::{Contour, Grid};

// ── ContourMode ──────────────────────────────────────────────────────────────────────────────────

/// How [`ContourMark`] renders the contour set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContourMode {
    /// Stroke each contour level with a 1-px line per cell crossing. Default.
    /// Honest, fast, and snapshot-friendly.
    #[default]
    Isolines,
    /// Fill each band between consecutive levels with the colormap-sampled
    /// hue, computed via per-cell Sutherland-Hodgman polygon clipping.
    /// Levels are pair-wise: `n` levels yields `n-1` filled bands.
    FilledBands,
    /// Both filled bands and overlaid isolines — the standard "filled
    /// contour with line overlay" output you'd get from matplotlib's
    /// `contourf` + `contour`.
    FilledWithLines,
}

// ── ContourMark ──────────────────────────────────────────────────────────────────────────────────

/// Contour plot of a 2-D scalar grid.
///
/// Each level produces one contour line traced by marching squares. With a
/// colormap, the i-th level samples the colormap at `i / (n_levels - 1)`,
/// going low → high; without one, every level renders in `stroke_color`
/// (default theme axis color).
#[derive(Clone, Debug)]
pub struct ContourMark {
    /// Source grid.
    pub grid: Grid,
    /// Contour levels (data values) to extract. Sorted ascending on construction.
    pub levels: Vec<f64>,
    /// Optional colormap; when set, each level samples it.
    pub colormap: Option<Colormap>,
    /// Single-color override applied when [`colormap`](Self::colormap) is `None`.
    pub stroke_color: Color,
    /// Stroke width in pixels.
    pub stroke_width: f32,
    /// Render mode.
    pub mode: ContourMode,
    /// Legend label.
    pub label: Option<String>,
}

impl ContourMark {
    /// New contour plot with the given grid and explicit levels.
    ///
    /// Levels are sorted on construction so colormap sampling and band
    /// pairing always run low → high regardless of input order.
    #[must_use]
    pub fn new(grid: Grid, levels: Vec<f64>) -> Self {
        let mut levels = levels;
        levels.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Self {
            grid,
            levels,
            colormap: None,
            stroke_color: Color::from_hex(0x0044_4444),
            stroke_width: 1.0,
            mode: ContourMode::Isolines,
            label: None,
        }
    }

    /// Builder: tint each level via the colormap (low → high).
    #[must_use]
    pub fn colormap(mut self, cm: Colormap) -> Self {
        self.colormap = Some(cm);
        self
    }

    /// Builder: single-color override for isolines (clears any colormap).
    #[must_use]
    pub fn stroke_color(mut self, c: Color) -> Self {
        self.stroke_color = c;
        self.colormap = None;
        self
    }

    /// Builder: stroke width in pixels.
    #[must_use]
    pub fn stroke_width(mut self, w: f32) -> Self {
        self.stroke_width = w;
        self
    }

    /// Builder: switch render mode.
    #[must_use]
    pub fn mode(mut self, mode: ContourMode) -> Self {
        self.mode = mode;
        self
    }

    /// Builder: convenience for `mode(ContourMode::Isolines)`.
    #[must_use]
    pub fn isolines(self) -> Self {
        self.mode(ContourMode::Isolines)
    }

    /// Builder: convenience for `mode(ContourMode::FilledBands)`.
    #[must_use]
    pub fn filled(self) -> Self {
        self.mode(ContourMode::FilledBands)
    }

    /// Builder: legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Color for the i-th level, accounting for colormap or `stroke_color`.
    fn level_color(&self, i: usize) -> Color {
        if let Some(cm) = &self.colormap {
            let n = self.levels.len();
            let t = if n <= 1 {
                0.5
            } else {
                i as f64 / (n - 1) as f64
            };
            cm.sample(t)
        } else {
            self.stroke_color
        }
    }

    /// Color for the i-th filled band (between `levels[i]` and `levels[i+1]`).
    /// Samples the colormap at `i / (n_bands - 1)` so the lowest band reads
    /// as the colormap's low end and the highest band as its high end.
    fn band_color(&self, i: usize) -> Color {
        if let Some(cm) = &self.colormap {
            let n_bands = self.levels.len().saturating_sub(1);
            let t = if n_bands <= 1 {
                0.5
            } else {
                i as f64 / (n_bands - 1) as f64
            };
            cm.sample(t)
        } else {
            self.stroke_color
        }
    }

    /// Filled-band rendering via per-cell Sutherland-Hodgman polygon clipping.
    ///
    /// For each consecutive pair `(L_i, L_{i+1})` and each grid cell, start
    /// with the cell rectangle as a 4-vertex polygon, clip below `L_i`
    /// (drop sub-region with values below the lower bound), then clip above
    /// `L_{i+1}` (drop sub-region above the upper bound). The remainder is
    /// the filled-band slice for that cell, drawn in the band color.
    fn render_filled_bands<F>(&self, to_px: &F, backend: &mut dyn DrawBackend) -> Result<()>
    where
        F: Fn(f64, f64) -> Point,
    {
        let nx = self.grid.nx;
        let ny = self.grid.ny;
        if nx < 2 || ny < 2 {
            return Ok(());
        }
        let dx = (self.grid.x_max - self.grid.x_min) / (nx - 1) as f64;
        let dy = (self.grid.y_max - self.grid.y_min) / (ny - 1) as f64;
        for band_idx in 0..self.levels.len() - 1 {
            let low = self.levels[band_idx];
            let high = self.levels[band_idx + 1];
            let color = self.band_color(band_idx);
            let style = PathStyle::fill(color);
            // Same-color hairline stroke covers the anti-aliased seam between
            // adjacent cells/bands (matplotlib's standard fix). Without it,
            // filled-contour output shows a 1-px white grid because each
            // cell's polygon AA leaves background pixels visible at edges.
            // Tracked as `starsight-3h6`.
            let seam_stroke = PathStyle::stroke(color, 1.0);
            for ci in 0..(ny - 1) {
                for cj in 0..(nx - 1) {
                    let v0 = self.grid.values[ci * nx + cj];
                    let v1 = self.grid.values[ci * nx + cj + 1];
                    let v2 = self.grid.values[(ci + 1) * nx + cj + 1];
                    let v3 = self.grid.values[(ci + 1) * nx + cj];
                    if !(v0.is_finite() && v1.is_finite() && v2.is_finite() && v3.is_finite()) {
                        continue;
                    }
                    // Quick reject: every corner below `low` or every corner
                    // above `high` → cell contributes nothing to this band.
                    if v0 < low && v1 < low && v2 < low && v3 < low {
                        continue;
                    }
                    if v0 > high && v1 > high && v2 > high && v3 > high {
                        continue;
                    }
                    let x0 = self.grid.x_min + cj as f64 * dx;
                    let x1 = x0 + dx;
                    let y0 = self.grid.y_min + ci as f64 * dy;
                    let y1 = y0 + dy;
                    let cell = [(x0, y0, v0), (x1, y0, v1), (x1, y1, v2), (x0, y1, v3)];
                    let after_low = clip_polygon_below(&cell, low);
                    let band = clip_polygon_above(&after_low, high);
                    if band.len() < 3 {
                        continue;
                    }
                    let mut path = Path::new();
                    let first = to_px(band[0].0, band[0].1);
                    path = path.move_to(first);
                    for v in &band[1..] {
                        path = path.line_to(to_px(v.0, v.1));
                    }
                    path = path.close();
                    backend.draw_path(&path, &style)?;
                    backend.draw_path(&path, &seam_stroke)?;
                }
            }
        }
        Ok(())
    }
}

/// Polygon vertex for Sutherland-Hodgman clipping: `(x, y, value)`. Value is
/// the scalar field at that point; clipping at a threshold linearly
/// interpolates the new vertex's `(x, y)` from the bracketing endpoints.
type Vertex = (f64, f64, f64);

/// Sutherland-Hodgman clip: keep the sub-polygon where `value >= threshold`.
fn clip_polygon_below(poly: &[Vertex], threshold: f64) -> Vec<Vertex> {
    if poly.is_empty() {
        return Vec::new();
    }
    let n = poly.len();
    let mut out = Vec::with_capacity(n + 4);
    for i in 0..n {
        let curr = poly[i];
        let next = poly[(i + 1) % n];
        let curr_in = curr.2 >= threshold;
        let next_in = next.2 >= threshold;
        if curr_in {
            out.push(curr);
            if !next_in {
                out.push(interp_vertex(curr, next, threshold));
            }
        } else if next_in {
            out.push(interp_vertex(curr, next, threshold));
        }
    }
    out
}

/// Sutherland-Hodgman clip: keep the sub-polygon where `value <= threshold`.
fn clip_polygon_above(poly: &[Vertex], threshold: f64) -> Vec<Vertex> {
    if poly.is_empty() {
        return Vec::new();
    }
    let n = poly.len();
    let mut out = Vec::with_capacity(n + 4);
    for i in 0..n {
        let curr = poly[i];
        let next = poly[(i + 1) % n];
        let curr_in = curr.2 <= threshold;
        let next_in = next.2 <= threshold;
        if curr_in {
            out.push(curr);
            if !next_in {
                out.push(interp_vertex(curr, next, threshold));
            }
        } else if next_in {
            out.push(interp_vertex(curr, next, threshold));
        }
    }
    out
}

/// Linear interpolation between two vertices at the value crossing
/// `threshold`. Returns the (x, y, threshold) point along edge `a → b`.
fn interp_vertex(a: Vertex, b: Vertex, threshold: f64) -> Vertex {
    let denom = b.2 - a.2;
    let t = if denom.abs() < f64::EPSILON {
        0.5
    } else {
        ((threshold - a.2) / denom).clamp(0.0, 1.0)
    };
    (a.0 + t * (b.0 - a.0), a.1 + t * (b.1 - a.1), threshold)
}

impl Mark for ContourMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = crate::marks::require_cartesian(coord)?;
        if self.levels.is_empty() {
            return Ok(());
        }

        let area = &coord.plot_area;
        let to_px = |x: f64, y: f64| -> Point {
            let x_norm = coord.x_axis.scale.map(x) as f32;
            let y_norm = coord.y_axis.scale.map(y) as f32;
            Point::new(
                area.left + x_norm * area.width(),
                area.bottom - y_norm * area.height(),
            )
        };

        let render_filled = matches!(
            self.mode,
            ContourMode::FilledBands | ContourMode::FilledWithLines
        );
        let render_lines = matches!(
            self.mode,
            ContourMode::Isolines | ContourMode::FilledWithLines
        );

        if render_filled && self.levels.len() >= 2 {
            self.render_filled_bands(&to_px, backend)?;
        }

        if render_lines {
            // Stroke each level via marching-squares output. Per-cell
            // 2-point segments share endpoints exactly so antialiasing
            // doesn't visibly stitch across cell boundaries.
            for (i, level) in self.levels.iter().enumerate() {
                let polys = Contour::compute(&self.grid, &[*level]);
                if polys.is_empty() {
                    continue;
                }
                let color = self.level_color(i);
                let style = PathStyle::stroke(color, self.stroke_width);
                for poly in polys {
                    if poly.points.len() < 2 {
                        continue;
                    }
                    let mut path = Path::new();
                    let p0 = poly.points[0];
                    path = path.move_to(to_px(p0.0, p0.1));
                    for p in &poly.points[1..] {
                        path = path.line_to(to_px(p.0, p.1));
                    }
                    backend.draw_path(&path, &style)?;
                }
            }
        }

        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        Some(DataExtent {
            x_min: self.grid.x_min,
            x_max: self.grid.x_max,
            y_min: self.grid.y_min,
            y_max: self.grid.y_max,
        })
    }

    fn legend_color(&self) -> Option<Color> {
        self.label.as_ref()?;
        Some(if let Some(cm) = &self.colormap {
            cm.sample(0.5)
        } else {
            self.stroke_color
        })
    }

    fn legend_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn legend_glyph(&self) -> LegendGlyph {
        // Filled rect — best honest approximation of a contour band swatch.
        // Pure-isoline mode would also fit `Line`, but every contour figure
        // displays at least a filled-area sense (even strokes fence off
        // regions), so the rect glyph reads more accurately than a hairline.
        LegendGlyph::Bar
    }

    fn colormap_legend(&self) -> Option<crate::marks::ColormapLegend> {
        let cm = self.colormap?;
        let lo = *self.levels.first()?;
        let hi = *self.levels.last()?;
        if !lo.is_finite() || !hi.is_finite() || hi <= lo {
            return None;
        }
        Some(crate::marks::ColormapLegend {
            colormap: cm,
            value_min: lo,
            value_max: hi,
            label: self.label.clone(),
            log_scale: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ContourMark, ContourMode};
    use crate::marks::{LegendGlyph, Mark};
    use crate::statistics::Grid;
    use starsight_layer_1::primitives::Color;

    fn linear_grid() -> Grid {
        // f(x, y) = x + y over [0, 1]² on a 5×5 grid.
        Grid::sample(5, 5, 0.0, 1.0, 0.0, 1.0, |x, y| x + y)
    }

    #[test]
    fn new_sorts_levels() {
        let mark = ContourMark::new(linear_grid(), vec![0.5, 0.1, 1.5, 0.8]);
        assert_eq!(mark.levels, vec![0.1, 0.5, 0.8, 1.5]);
    }

    #[test]
    fn data_extent_matches_grid_bounds() {
        let mark = ContourMark::new(linear_grid(), vec![0.5]);
        let e = mark.data_extent().expect("non-empty");
        assert!((e.x_min - 0.0).abs() < 1e-9);
        assert!((e.x_max - 1.0).abs() < 1e-9);
        assert!((e.y_min - 0.0).abs() < 1e-9);
        assert!((e.y_max - 1.0).abs() < 1e-9);
    }

    #[test]
    fn empty_levels_data_extent_still_returns_grid() {
        let mark = ContourMark::new(linear_grid(), vec![]);
        // data_extent returns the grid bounds even with no levels — gives the
        // figure something to size axes against.
        assert!(mark.data_extent().is_some());
    }

    #[test]
    fn stroke_color_clears_colormap() {
        let mark = ContourMark::new(linear_grid(), vec![0.5])
            .colormap(starsight_layer_1::colormap::VIRIDIS)
            .stroke_color(Color::RED);
        assert!(mark.colormap.is_none());
        assert_eq!(mark.stroke_color, Color::RED);
    }

    #[test]
    fn legend_glyph_is_bar() {
        let mark = ContourMark::new(linear_grid(), vec![0.5]).label("ψ");
        assert_eq!(mark.legend_glyph(), LegendGlyph::Bar);
        assert!(mark.legend_color().is_some());
        assert_eq!(mark.legend_label(), Some("ψ"));
    }

    #[test]
    fn no_legend_when_unlabeled() {
        let mark = ContourMark::new(linear_grid(), vec![0.5]);
        assert!(mark.legend_color().is_none());
    }

    #[test]
    fn mode_builders_set_mode() {
        let mark = ContourMark::new(linear_grid(), vec![0.5]).filled();
        assert_eq!(mark.mode, ContourMode::FilledBands);
        let mark = mark.isolines();
        assert_eq!(mark.mode, ContourMode::Isolines);
    }

    // ── Sutherland-Hodgman polygon clipping ──────────────────────────────

    #[test]
    fn clip_polygon_below_keeps_above_threshold() {
        // Square cell with corners (0,0)→(1,0)→(1,1)→(0,1), values
        // [-1, 1, 1, -1]. Clipping at threshold 0 keeps the right half.
        let cell = vec![
            (0.0_f64, 0.0_f64, -1.0_f64),
            (1.0, 0.0, 1.0),
            (1.0, 1.0, 1.0),
            (0.0, 1.0, -1.0),
        ];
        let kept = super::clip_polygon_below(&cell, 0.0);
        // Result: 4 vertices forming the right-half rectangle: (0.5,0,0),
        // (1,0,1), (1,1,1), (0.5,1,0).
        assert_eq!(kept.len(), 4);
        let xs: Vec<f64> = kept.iter().map(|v| v.0).collect();
        assert!(xs.iter().all(|x| *x >= 0.5 - 1e-9));
    }

    #[test]
    fn clip_polygon_above_keeps_below_threshold() {
        let cell = vec![
            (0.0_f64, 0.0_f64, -1.0_f64),
            (1.0, 0.0, 1.0),
            (1.0, 1.0, 1.0),
            (0.0, 1.0, -1.0),
        ];
        let kept = super::clip_polygon_above(&cell, 0.0);
        // Result: 4 vertices forming the LEFT half rectangle.
        assert_eq!(kept.len(), 4);
        let xs: Vec<f64> = kept.iter().map(|v| v.0).collect();
        assert!(xs.iter().all(|x| *x <= 0.5 + 1e-9));
    }

    #[test]
    fn clip_polygon_below_drops_everything_below_threshold() {
        // All values below threshold → empty clip.
        let cell = vec![
            (0.0_f64, 0.0_f64, -1.0_f64),
            (1.0, 0.0, -1.0),
            (1.0, 1.0, -1.0),
            (0.0, 1.0, -1.0),
        ];
        assert!(super::clip_polygon_below(&cell, 0.0).is_empty());
    }

    #[test]
    fn clip_polygon_below_keeps_full_when_all_above() {
        let cell = vec![
            (0.0_f64, 0.0_f64, 5.0_f64),
            (1.0, 0.0, 5.0),
            (1.0, 1.0, 5.0),
            (0.0, 1.0, 5.0),
        ];
        let kept = super::clip_polygon_below(&cell, 0.0);
        assert_eq!(kept.len(), 4);
    }

    #[test]
    fn clip_polygon_handles_degenerate_zero_gradient() {
        // Edge endpoints with equal values shouldn't divide by zero in
        // interp_vertex. Values straddle threshold via the *other* edge.
        let cell = vec![
            (0.0_f64, 0.0_f64, 1.0_f64),
            (1.0, 0.0, 1.0), // equal value to v0; no crossing on this edge
            (1.0, 1.0, -1.0),
            (0.0, 1.0, -1.0),
        ];
        let kept = super::clip_polygon_below(&cell, 0.0);
        // Top half is below 0; bottom half is above. Should keep ~bottom half.
        assert!(!kept.is_empty());
        // Every kept vertex should have value >= 0.
        for v in &kept {
            assert!(v.2 >= -1e-9);
        }
    }

    #[test]
    fn interp_vertex_at_midpoint_for_symmetric_edge() {
        let a = (0.0_f64, 0.0_f64, -1.0_f64);
        let b = (10.0, 0.0, 1.0);
        let mid = super::interp_vertex(a, b, 0.0);
        assert!((mid.0 - 5.0).abs() < 1e-9);
        assert!((mid.1 - 0.0).abs() < 1e-9);
        assert!((mid.2 - 0.0).abs() < 1e-9);
    }
}
