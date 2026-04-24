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

/// Render tick marks and labels for both axes, plus category labels for bar charts.
///
/// `use_y_axis_labels`: true = categories on Y-axis (left), false = categories on X-axis (bottom)
pub fn render_axes(
    coord: &CartesianCoord,
    backend: &mut dyn DrawBackend,
    category_labels: &[String],
    use_y_axis_labels: bool,
) -> Result<()> {
    let area = &coord.plot_area;
    let tick_len: f32 = 5.0;
    let label_offset: f32 = 14.0;
    let tick_color = Color::new(80, 80, 80);
    let font_size: f32 = 12.0;

    let n_categories = category_labels.len();

    // === Category labels (bar charts) ===
    if n_categories > 0 {
        if use_y_axis_labels {
            // Labels on Y-axis (left side) - horizontal bars, only X ticks
            let band_height = area.height() / n_categories as f32;
            for (i, label) in category_labels.iter().enumerate() {
                let py = area.top + (i as f32 + 0.5) * band_height;
                backend.draw_text(label, Point::new(10.0, py - 6.0), font_size, tick_color)?;
            }
            // X-axis: ticks to left (label side)
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
            // NO Y-axis ticks - category labels are on Y positions
        } else {
            // Labels on X-axis (bottom) - vertical bars, only Y ticks
            let band_width = area.width() / n_categories as f32;
            for (i, label) in category_labels.iter().enumerate() {
                let px = area.left + (i as f32 + 0.5) * band_width;
                backend.draw_text(
                    label,
                    Point::new(px - 20.0, area.bottom + label_offset + 10.0),
                    font_size,
                    tick_color,
                )?;
            }
            // NO X-axis ticks - category labels replace them
            // Y-axis: ticks right (data side)
            for (pos, label) in coord
                .y_axis
                .tick_positions
                .iter()
                .zip(&coord.y_axis.tick_labels)
            {
                let py = coord.map_y(*pos) as f32;
                let path = Path::new()
                    .move_to(Point::new(area.left, py))
                    .line_to(Point::new(area.left + tick_len, py));
                backend.draw_path(&path, &PathStyle::stroke(tick_color, 1.0))?;
                backend.draw_text(
                    label,
                    Point::new(area.left - 40.0, py - 6.0),
                    font_size,
                    tick_color,
                )?;
            }
        }
    } else {
        // === No category labels - regular numeric axes ===
        // X-axis: ticks up (data side)
        for (pos, label) in coord
            .x_axis
            .tick_positions
            .iter()
            .zip(&coord.x_axis.tick_labels)
        {
            let px = coord.map_x(*pos) as f32;
            let path = Path::new()
                .move_to(Point::new(px, area.bottom))
                .line_to(Point::new(px, area.bottom - tick_len));
            backend.draw_path(&path, &PathStyle::stroke(tick_color, 1.0))?;
            backend.draw_text(
                label,
                Point::new(px - 10.0, area.bottom + label_offset),
                font_size,
                tick_color,
            )?;
        }
        // Y-axis: ticks right (data side)
        for (pos, label) in coord
            .y_axis
            .tick_positions
            .iter()
            .zip(&coord.y_axis.tick_labels)
        {
            let py = coord.map_y(*pos) as f32;
            let path = Path::new()
                .move_to(Point::new(area.left, py))
                .line_to(Point::new(area.left + tick_len, py));
            backend.draw_path(&path, &PathStyle::stroke(tick_color, 1.0))?;
            backend.draw_text(
                label,
                Point::new(area.left - 40.0, py - 6.0),
                font_size,
                tick_color,
            )?;
        }
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

// ── render_legend ───────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub fn render_legend(legend: &Legend, backend: &mut dyn DrawBackend) -> Result<()>

// ── render_title ───────────────────────────────────────────────────────────────────────────────

/// Render chart title above the plot area.
pub fn render_title(title: &str, width: u32, backend: &mut dyn DrawBackend) -> Result<()> {
    let font_size: f32 = 16.0;
    let title_color = Color::new(30, 30, 30);
    let x = width as f32 / 2.0;
    let y = 15.0;
    backend.draw_text(title, Point::new(x, y), font_size, title_color)
}

/// Render axis labels.
pub fn render_axis_labels(
    x_label: Option<&str>,
    y_label: Option<&str>,
    plot_area: &Rect,
    backend: &mut dyn DrawBackend,
) -> Result<()> {
    let font_size: f32 = 12.0;
    let label_color = Color::new(60, 60, 60);

    if let Some(label) = x_label {
        let x = plot_area.left + plot_area.width() / 2.0;
        let y = plot_area.bottom + 35.0;
        backend.draw_text(label, Point::new(x, y), font_size, label_color)?;
    }

    if let Some(label) = y_label {
        let x = 15.0;
        let y = plot_area.top + plot_area.height() / 2.0;
        backend.draw_text(label, Point::new(x, y), font_size, label_color)?;
    }

    Ok(())
}
