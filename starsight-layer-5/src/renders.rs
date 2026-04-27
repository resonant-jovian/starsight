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
use starsight_layer_1::theme::Theme;
use starsight_layer_2::coords::CartesianCoord;

use crate::layout::Slot;

// ── render_axes ──────────────────────────────────────────────────────────────────────────────────

/// Render tick marks and labels for both axes, plus category labels for bar charts.
///
/// `use_y_axis_labels`: true = categories on Y-axis (left), false = categories on X-axis (bottom)
pub fn render_axes(
    coord: &CartesianCoord,
    backend: &mut dyn DrawBackend,
    category_labels: &[String],
    use_y_axis_labels: bool,
    theme: &Theme,
) -> Result<()> {
    let area = &coord.plot_area;
    let tick_len: f32 = 5.0;
    let tick_color = theme.axis;
    let font_size: f32 = 12.0;

    let n_categories = category_labels.len();

    // Both backends now treat draw_text's y as the SVG baseline. X labels are
    // positioned so the glyph top sits ~2px below the tick (baseline =
    // tick_end + ascent), and Y labels so the glyph is vertically centered on
    // the tick (baseline = py + font_size * 0.4).
    let draw_x_label = |backend: &mut dyn DrawBackend, label: &str, px: f32| -> Result<()> {
        let (tw, _) = backend
            .text_extent(label, font_size)
            .unwrap_or((0.0, font_size));
        backend.draw_text(
            label,
            Point::new(px - tw / 2.0, area.bottom + tick_len + font_size),
            font_size,
            tick_color,
        )
    };
    let draw_y_label = |backend: &mut dyn DrawBackend, label: &str, py: f32| -> Result<()> {
        let (tw, _) = backend
            .text_extent(label, font_size)
            .unwrap_or((0.0, font_size));
        backend.draw_text(
            label,
            Point::new(area.left - tick_len - 4.0 - tw, py + font_size * 0.4),
            font_size,
            tick_color,
        )
    };

    // === Category labels (bar charts) ===
    if n_categories > 0 {
        if use_y_axis_labels {
            // Labels on Y-axis (left side) - horizontal bars, only X ticks
            let band_height = area.height() / n_categories as f32;
            for (i, label) in category_labels.iter().enumerate() {
                let py = area.top + (i as f32 + 0.5) * band_height;
                draw_y_label(backend, label, py)?;
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
                draw_x_label(backend, label, px)?;
            }
            // NO Y-axis ticks - category labels are on Y positions
        } else {
            // Labels on X-axis (bottom) - vertical bars, only Y ticks
            let band_width = area.width() / n_categories as f32;
            for (i, label) in category_labels.iter().enumerate() {
                let px = area.left + (i as f32 + 0.5) * band_width;
                draw_x_label(backend, label, px)?;
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
                draw_y_label(backend, label, py)?;
            }
        }
    } else {
        // === No category labels - regular numeric axes ===
        // X-axis: ticks down (below plot area), labels below ticks
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
            draw_x_label(backend, label, px)?;
        }
        // Y-axis: ticks left (to the left of plot area), labels to the left of ticks
        for (pos, label) in coord
            .y_axis
            .tick_positions
            .iter()
            .zip(&coord.y_axis.tick_labels)
        {
            let py = coord.map_y(*pos) as f32;
            let path = Path::new()
                .move_to(Point::new(area.left, py))
                .line_to(Point::new(area.left - tick_len, py));
            backend.draw_path(&path, &PathStyle::stroke(tick_color, 1.0))?;
            draw_y_label(backend, label, py)?;
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

// ── render_grid_lines ────────────────────────────────────────────────────────────────────────────

/// Render light grid lines for both axes.
pub fn render_grid_lines(
    coord: &CartesianCoord,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
) -> Result<()> {
    let area = &coord.plot_area;
    let grid_color = theme.grid;
    let line_style = PathStyle::stroke(grid_color, 1.0);

    for pos in &coord.x_axis.tick_positions {
        let px = coord.map_x(*pos) as f32;
        let path = Path::new()
            .move_to(Point::new(px, area.bottom))
            .line_to(Point::new(px, area.top));
        backend.draw_path(&path, &line_style)?;
    }

    for pos in &coord.y_axis.tick_positions {
        let py = coord.map_y(*pos) as f32;
        let path = Path::new()
            .move_to(Point::new(area.left, py))
            .line_to(Point::new(area.right, py));
        backend.draw_path(&path, &line_style)?;
    }

    Ok(())
}

// ── render_background ────────────────────────────────────────────────────────────────────────────

/// Fill the plot area background with the theme background color.
///
/// # Errors
/// Forwards any error from the backend's `fill_rect` call.
pub fn render_background(
    plot_area: &Rect,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
) -> Result<()> {
    backend.fill_rect(*plot_area, theme.background)
}

// ── render_legend ───────────────────────────────────────────────────────────────────────────────

/// A single entry in the legend.
pub struct LegendEntry {
    pub color: Color,
    pub label: String,
}

/// Render a legend with colored line/box samples and labels.
pub fn render_legend(
    entries: &[LegendEntry],
    plot_area: &Rect,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let font_size: f32 = 12.0;
    let label_color = theme.tick_label;
    let sample_size: f32 = 12.0;
    let padding: f32 = 8.0;
    let line_spacing: f32 = 20.0;

    let max_label_len = entries.iter().map(|e| e.label.len()).max().unwrap_or(0);
    let legend_width = max_label_len as f32 * 7.0 + 30.0;
    let legend_height = (entries.len() as f32 * line_spacing) + padding * 2.0;

    let legend_x = plot_area.right - legend_width - 10.0;
    let legend_y = plot_area.top + 10.0;

    let bg_color = theme.background.with_alpha(230).without_alpha();
    let bg_rect = Rect::new(
        legend_x,
        legend_y,
        legend_x + legend_width,
        legend_y + legend_height,
    );
    backend.fill_rect(bg_rect, bg_color)?;

    for (i, entry) in entries.iter().enumerate() {
        let y = legend_y + padding + (i as f32 * line_spacing) + sample_size / 2.0;

        let line = Path::new()
            .move_to(Point::new(legend_x + padding, y))
            .line_to(Point::new(legend_x + padding + sample_size, y));
        backend.draw_path(&line, &PathStyle::stroke(entry.color, 2.0))?;

        backend.draw_text(
            &entry.label,
            Point::new(legend_x + padding + sample_size + 6.0, y + 4.0),
            font_size,
            label_color,
        )?;
    }

    Ok(())
}

// ── render_title ───────────────────────────────────────────────────────────────────────────────

/// Render the chart title centered inside its layout slot.
pub fn render_title(
    title: &str,
    slot: &Slot,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
) -> Result<()> {
    let font_size: f32 = 16.0;
    let title_color = theme.title;
    let (tw, th) = backend.text_extent(title, font_size).unwrap_or((0.0, font_size));
    let x = slot.rect.left + (slot.rect.width() - tw) / 2.0;
    let y = slot.rect.top + (slot.rect.height() + th) / 2.0;
    backend.draw_text(title, Point::new(x, y), font_size, title_color)
}

/// Render axis-title labels centered along their respective axes, drawing into
/// the slots reserved by the layout (below/beside the tick-label band).
pub fn render_axis_labels(
    x_label: Option<&str>,
    y_label: Option<&str>,
    x_slot: Option<&Slot>,
    y_slot: Option<&Slot>,
    plot_area: &Rect,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
) -> Result<()> {
    let font_size: f32 = 12.0;
    let label_color = theme.tick_label;

    if let (Some(label), Some(slot)) = (x_label, x_slot) {
        let (lw, lh) = backend
            .text_extent(label, font_size)
            .unwrap_or((0.0, font_size));
        let cx = (plot_area.left + plot_area.right) / 2.0 - lw / 2.0;
        let cy = slot.rect.top + (slot.rect.height() + lh) / 2.0;
        backend.draw_text(label, Point::new(cx, cy), font_size, label_color)?;
    }

    if let (Some(label), Some(slot)) = (y_label, y_slot) {
        let (lw, lh) = backend
            .text_extent(label, font_size)
            .unwrap_or((0.0, font_size));
        // Rotated -90°: post-rotation width is lh, post-rotation height is lw.
        let cx = slot.rect.left + (slot.rect.width() + lh) / 2.0;
        let cy = (plot_area.top + plot_area.bottom) / 2.0 + lw / 2.0;
        backend.draw_rotated_text(label, Point::new(cx, cy), font_size, label_color, -90.0)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        LegendEntry, render_axes, render_axis_labels, render_background, render_grid_lines,
        render_legend, render_title,
    };
    use crate::layout::{Side, Slot};
    use starsight_layer_1::backends::vectors::SvgBackend;
    use starsight_layer_1::primitives::{Color, Rect};
    use starsight_layer_1::theme::DEFAULT_LIGHT;
    use starsight_layer_2::axes::Axis;
    use starsight_layer_2::coords::CartesianCoord;
    use starsight_layer_2::scales::LinearScale;

    fn coord_with_ticks(plot: Rect) -> CartesianCoord {
        CartesianCoord {
            x_axis: Axis {
                scale: LinearScale {
                    domain_min: 0.0,
                    domain_max: 1.0,
                },
                label: None,
                tick_positions: vec![0.0, 0.5, 1.0],
                tick_labels: vec!["0".into(), "0.5".into(), "1".into()],
            },
            y_axis: Axis {
                scale: LinearScale {
                    domain_min: 0.0,
                    domain_max: 1.0,
                },
                label: None,
                tick_positions: vec![0.0, 0.5, 1.0],
                tick_labels: vec!["0".into(), "0.5".into(), "1".into()],
            },
            plot_area: plot,
        }
    }

    #[test]
    fn render_legend_empty_returns_ok() {
        let mut backend = SvgBackend::new(100, 100);
        render_legend(&[], &Rect::new(0.0, 0.0, 100.0, 100.0), &mut backend, &DEFAULT_LIGHT)
            .unwrap();
    }

    #[test]
    fn render_legend_with_entries() {
        let mut backend = SvgBackend::new(400, 200);
        let entries = vec![
            LegendEntry {
                color: Color::RED,
                label: "first".into(),
            },
            LegendEntry {
                color: Color::BLUE,
                label: "second".into(),
            },
        ];
        render_legend(
            &entries,
            &Rect::new(0.0, 0.0, 400.0, 200.0),
            &mut backend,
            &DEFAULT_LIGHT,
        )
        .unwrap();
        let svg = backend.svg_string();
        assert!(svg.contains("first"));
        assert!(svg.contains("second"));
    }

    #[test]
    fn render_background_fills_rect() {
        let mut backend = SvgBackend::new(50, 50);
        render_background(&Rect::new(0.0, 0.0, 50.0, 50.0), &mut backend, &DEFAULT_LIGHT).unwrap();
    }

    #[test]
    fn render_title_centers_in_slot() {
        let mut backend = SvgBackend::new(200, 50);
        let slot = Slot {
            rect: Rect::new(0.0, 0.0, 200.0, 30.0),
            side: Side::Top,
        };
        render_title("Hello", &slot, &mut backend, &DEFAULT_LIGHT).unwrap();
        assert!(backend.svg_string().contains("Hello"));
    }

    #[test]
    fn render_axis_labels_writes_when_provided() {
        let mut backend = SvgBackend::new(200, 200);
        let plot = Rect::new(20.0, 20.0, 180.0, 180.0);
        let x_slot = Slot {
            rect: Rect::new(20.0, 180.0, 180.0, 200.0),
            side: Side::Bottom,
        };
        let y_slot = Slot {
            rect: Rect::new(0.0, 20.0, 20.0, 180.0),
            side: Side::Left,
        };
        render_axis_labels(
            Some("X"),
            Some("Y"),
            Some(&x_slot),
            Some(&y_slot),
            &plot,
            &mut backend,
            &DEFAULT_LIGHT,
        )
        .unwrap();
        let svg = backend.svg_string();
        assert!(svg.contains('X'));
        assert!(svg.contains('Y'));
    }

    #[test]
    fn render_axis_labels_skips_when_missing() {
        let mut backend = SvgBackend::new(200, 200);
        render_axis_labels(
            None,
            None,
            None,
            None,
            &Rect::new(0.0, 0.0, 200.0, 200.0),
            &mut backend,
            &DEFAULT_LIGHT,
        )
        .unwrap();
    }

    #[test]
    fn render_grid_lines_runs_for_each_tick() {
        let mut backend = SvgBackend::new(200, 200);
        let coord = coord_with_ticks(Rect::new(20.0, 20.0, 180.0, 180.0));
        render_grid_lines(&coord, &mut backend, &DEFAULT_LIGHT).unwrap();
    }

    #[test]
    fn render_axes_numeric_branch() {
        let mut backend = SvgBackend::new(200, 200);
        let coord = coord_with_ticks(Rect::new(20.0, 20.0, 180.0, 180.0));
        render_axes(&coord, &mut backend, &[], false, &DEFAULT_LIGHT).unwrap();
    }

    #[test]
    fn render_axes_categories_on_x() {
        let mut backend = SvgBackend::new(200, 200);
        let coord = coord_with_ticks(Rect::new(20.0, 20.0, 180.0, 180.0));
        let cats = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        render_axes(&coord, &mut backend, &cats, false, &DEFAULT_LIGHT).unwrap();
    }

    #[test]
    fn render_axes_categories_on_y() {
        let mut backend = SvgBackend::new(200, 200);
        let coord = coord_with_ticks(Rect::new(20.0, 20.0, 180.0, 180.0));
        let cats = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        render_axes(&coord, &mut backend, &cats, true, &DEFAULT_LIGHT).unwrap();
    }
}
