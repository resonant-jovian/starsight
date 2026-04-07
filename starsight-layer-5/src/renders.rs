//! Rendering helpers used by [`Figure`](crate::figures::Figure).
//!
//! These functions take a coordinate system and a backend and emit the static
//! pieces of the chart that are not associated with a specific mark: the plot
//! background fill, the axis lines, and the tick labels.

#![allow(clippy::cast_possible_truncation)]

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathStyle};
use starsight_layer_1::primitives::{Color, Point, Rect};
use starsight_layer_2::coords::CartesianCoord;

// ── render_axes ──────────────────────────────────────────────────────────────────────────────────

/// Render tick marks and labels for both axes.
///
/// # Errors
/// Forwards any error from the backend's `draw_path` or `draw_text` calls.
pub fn render_axes(coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
    let area = &coord.plot_area;
    let tick_len: f32 = 5.0;
    let label_offset: f32 = 14.0;
    let tick_color = Color::new(80, 80, 80);
    let font_size: f32 = 12.0;

    // X-axis ticks.
    for (pos, label) in coord
        .x_axis
        .tick_positions
        .iter()
        .zip(&coord.x_axis.tick_labels)
    {
        let px = coord.map_x(*pos) as f32;
        let path = Path::new()
            .move_to(Point::new(px, area.bottom))
            .line_to(Point::new(px, area.bottom + tick_len));
        backend.draw_path(&path, &PathStyle::stroke(tick_color, 1.0))?;
        backend.draw_text(
            label,
            Point::new(px - 10.0, area.bottom + label_offset),
            font_size,
            tick_color,
        )?;
    }

    // Y-axis ticks.
    for (pos, label) in coord
        .y_axis
        .tick_positions
        .iter()
        .zip(&coord.y_axis.tick_labels)
    {
        let py = coord.map_y(*pos) as f32;
        let path = Path::new()
            .move_to(Point::new(area.left - tick_len, py))
            .line_to(Point::new(area.left, py));
        backend.draw_path(&path, &PathStyle::stroke(tick_color, 1.0))?;
        backend.draw_text(
            label,
            Point::new(area.left - 40.0, py - 6.0),
            font_size,
            tick_color,
        )?;
    }

    // Axis lines (bottom and left edges of plot area).
    let axis_style = PathStyle::stroke(Color::BLACK, 1.0);
    let bottom_line = Path::new()
        .move_to(Point::new(area.left, area.bottom))
        .line_to(Point::new(area.right, area.bottom));
    backend.draw_path(&bottom_line, &axis_style)?;
    let left_line = Path::new()
        .move_to(Point::new(area.left, area.top))
        .line_to(Point::new(area.left, area.bottom));
    backend.draw_path(&left_line, &axis_style)?;

    Ok(())
}

// ── render_background ────────────────────────────────────────────────────────────────────────────

/// Fill the plot area background with white.
///
/// # Errors
/// Forwards any error from the backend's `fill_rect` call.
pub fn render_background(plot_area: &Rect, backend: &mut dyn DrawBackend) -> Result<()> {
    backend.fill_rect(*plot_area, Color::WHITE)
}

// ── render_legend ────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub fn render_legend(legend: &Legend, backend: &mut dyn DrawBackend) -> Result<()>
