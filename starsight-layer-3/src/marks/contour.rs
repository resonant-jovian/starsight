//! Contour mark — isolines (and, eventually, filled bands) over a 2-D scalar
//! grid.
//!
//! Backed by [`crate::statistics::Contour`]'s marching-squares extractor.
//! `ContourMark` accepts a [`Grid`] plus a list of levels, then strokes one
//! line segment per cell crossing per level. A colormap optionally tints each
//! level so the iso-elevation order is visible at a glance.
//!
//! Status: lands in 0.3.0 — isoline mode is fully supported. `FilledBands`
//! (proper polygon-tracing across cells) is scaffolded but currently falls
//! back to isoline rendering; a follow-up task delivers it.

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
    /// Fill the band between consecutive levels. **0.3.0 caveat:** falls back
    /// to isoline rendering until polygon-tracing lands; the API is in place
    /// so call sites compile against the final shape.
    FilledBands,
    /// Both filled bands and overlaid isolines. Same caveat as `FilledBands`.
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
}

impl Mark for ContourMark {
    fn render(&self, coord: &dyn Coord, backend: &mut dyn DrawBackend) -> Result<()> {
        let coord = crate::marks::require_cartesian(coord)?;
        if self.levels.is_empty() {
            return Ok(());
        }

        // Render each level separately so we can color independently. For each
        // level, run marching squares once over the grid; the segment list
        // returns Polylines (currently 2-point segments per cell crossing).
        // Each level's segments stroke in the level's color.
        let area = &coord.plot_area;
        let to_px = |x: f64, y: f64| -> Point {
            let x_norm = coord.x_axis.scale.map(x) as f32;
            let y_norm = coord.y_axis.scale.map(y) as f32;
            Point::new(
                area.left + x_norm * area.width(),
                area.bottom - y_norm * area.height(),
            )
        };

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
}
