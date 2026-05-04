//! Rendering helpers used by [`Figure`](crate::figures::Figure).
//!
//! These functions take a coordinate system and a backend and emit the static
//! pieces of the chart that are not associated with a specific mark: the plot
//! background fill, the axis lines, and the tick labels.

#![allow(clippy::cast_possible_truncation)]

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathCommand, PathStyle};
use starsight_layer_1::primitives::{Color, Point, Rect};
use starsight_layer_1::theme::Theme;
use starsight_layer_2::coords::{CartesianCoord, Coord, PolarCoord};
use starsight_layer_3::marks::LegendGlyph;

use crate::layout::{LayoutFonts, Slot};

// ── render_axes ──────────────────────────────────────────────────────────────────────────────────

/// Render tick marks and labels for both axes, plus category labels for bar charts.
///
/// `use_y_axis_labels`: true = categories on Y-axis (left), false = categories on X-axis (bottom)
///
/// # Errors
/// Returns the backend's error if any path or text draw call fails.
pub fn render_axes(
    coord: &CartesianCoord,
    backend: &mut dyn DrawBackend,
    category_labels: &[String],
    use_y_axis_labels: bool,
    theme: &Theme,
    fonts: &LayoutFonts,
) -> Result<()> {
    let area = &coord.plot_area;
    let tick_len: f32 = 5.0;
    let tick_color = theme.axis;
    let font_size = fonts.label;

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
        // Snap measured width up to the next integer pixel and start-anchor the
        // text there. The right edge becomes `area.left - tick_len - 4.0` for
        // every label up to ≤1px sub-pixel drift, fixing the inconsistent
        // flush-right rendering tracked as `starsight-cet`.
        let tw_px = tw.ceil();
        let x = (area.left - tick_len - 4.0 - tw_px).round();
        backend.draw_text(
            label,
            Point::new(x, py + font_size * 0.4),
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

/// Render light grid lines for the figure's coord system.
///
/// Dispatches by coord type: [`CartesianCoord`] gets vertical/horizontal grid
/// lines at each tick; [`PolarCoord`] gets concentric rings at each radial
/// tick plus radial spokes at each angular tick. Unknown coord types render
/// nothing — extension point for future ternary / 3D coords.
///
/// # Errors
/// Returns the backend's error if any line draw call fails.
pub fn render_grid_lines(
    coord: &dyn Coord,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
) -> Result<()> {
    if let Some(cart) = coord.as_any().downcast_ref::<CartesianCoord>() {
        render_cartesian_grid_lines(cart, backend, theme)
    } else if let Some(polar) = coord.as_any().downcast_ref::<PolarCoord>() {
        render_polar_grid_lines(polar, backend, theme)
    } else {
        Ok(())
    }
}

fn render_cartesian_grid_lines(
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

/// Polar grid: concentric rings at each radial tick + radial spokes at each
/// angular tick.
///
/// Ring/spoke styling follows the cartesian grid (theme.grid color, 1px
/// stroke). Spokes start at the polar center and end at the rim of the
/// inscribed disk; rings are full circles centered on the polar center.
/// Tick positions go through the axis scales — `polar_radial_sqrt` will
/// place rings closer together near the rim, `polar_angular_categorical`
/// will place spokes at band-center angles.
///
/// # Errors
/// Returns the backend's error if any draw call fails.
fn render_polar_grid_lines(
    coord: &PolarCoord,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
) -> Result<()> {
    let grid_color = theme.grid;
    let line_style = PathStyle::stroke(grid_color, 1.0);
    let center = coord.center;
    let radius = coord.radius;

    // Concentric rings at each r-tick (skip the degenerate r=0 ring).
    for &r_tick in &coord.r_axis.tick_positions {
        let r_norm = coord.r_axis.scale.map(r_tick) as f32;
        let r_pixels = radius * r_norm;
        if r_pixels <= 0.5 {
            continue;
        }
        let circle = build_circle_path(center, r_pixels);
        backend.draw_path(&circle, &line_style)?;
    }

    // Radial spokes at each theta-tick. Compass convention: theta=0 is up,
    // angle increases clockwise — matches PolarCoord::data_to_pixel.
    for &theta_tick in &coord.theta_axis.tick_positions {
        let theta_norm = coord.theta_axis.scale.map(theta_tick);
        let angle = theta_norm * std::f64::consts::TAU;
        let edge_x = center.x + radius * (angle.sin() as f32);
        let edge_y = center.y - radius * (angle.cos() as f32);
        let path = Path::new()
            .move_to(center)
            .line_to(Point::new(edge_x, edge_y));
        backend.draw_path(&path, &line_style)?;
    }

    Ok(())
}

/// Four-cubic-Bezier circle approximation. Magic constant
/// `k = 4/3 · tan(π/8) ≈ 0.5523` makes the cubic tangent match the true
/// circle within 0.027% — invisible at any practical chart scale.
fn build_circle_path(center: Point, r: f32) -> Path {
    let cx = center.x;
    let cy = center.y;
    let k = 0.552_284_8_f32;
    let kr = k * r;
    let mut path = Path::new().move_to(Point::new(cx + r, cy));
    path.commands.push(PathCommand::CubicTo(
        Point::new(cx + r, cy + kr),
        Point::new(cx + kr, cy + r),
        Point::new(cx, cy + r),
    ));
    path.commands.push(PathCommand::CubicTo(
        Point::new(cx - kr, cy + r),
        Point::new(cx - r, cy + kr),
        Point::new(cx - r, cy),
    ));
    path.commands.push(PathCommand::CubicTo(
        Point::new(cx - r, cy - kr),
        Point::new(cx - kr, cy - r),
        Point::new(cx, cy - r),
    ));
    path.commands.push(PathCommand::CubicTo(
        Point::new(cx + kr, cy - r),
        Point::new(cx + r, cy - kr),
        Point::new(cx + r, cy),
    ));
    path.close()
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
    /// Sample color drawn next to the label.
    pub color: Color,
    /// Display text for this legend row.
    pub label: String,
    /// Sample shape drawn next to the label. Honest legend glyphs let scatter
    /// plots show a dot (not a dash) and bar charts show a swatch (not a
    /// hairline). Fix for `starsight-f4t`.
    pub glyph: LegendGlyph,
}

/// Render a legend with colored line/box samples and labels.
///
/// # Errors
/// Returns the backend's error if any rect or text draw call fails.
pub fn render_legend(
    entries: &[LegendEntry],
    plot_area: &Rect,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
    fonts: &LayoutFonts,
) -> Result<()> {
    if entries.is_empty() {
        return Ok(());
    }

    let font_size = fonts.label;
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
    // 1-px border so the legend box has a visible edge against bright plot
    // backgrounds (yrp.1). Drawn after the fill so it sits on top.
    let border = Path::new()
        .move_to(Point::new(bg_rect.left, bg_rect.top))
        .line_to(Point::new(bg_rect.right, bg_rect.top))
        .line_to(Point::new(bg_rect.right, bg_rect.bottom))
        .line_to(Point::new(bg_rect.left, bg_rect.bottom))
        .close();
    backend.draw_path(&border, &PathStyle::stroke(theme.axis, 1.0))?;

    for (i, entry) in entries.iter().enumerate() {
        let y = legend_y + padding + (i as f32 * line_spacing) + sample_size / 2.0;
        let sample_left = legend_x + padding;
        let sample_right = sample_left + sample_size;

        match entry.glyph {
            LegendGlyph::Point => {
                // Filled disk centered on the sample slot, radius matching
                // PointMark's default visual weight in legends.
                let radius = sample_size / 2.5;
                let cx = f32::midpoint(sample_left, sample_right);
                draw_filled_disk(backend, Point::new(cx, y), radius, entry.color)?;
            }
            LegendGlyph::Bar => {
                let half = sample_size / 2.0;
                let rect = Rect::new(sample_left, y - half, sample_right, y + half);
                backend.fill_rect(rect, entry.color)?;
            }
            LegendGlyph::Area => {
                // Translucent fill + top stroke conveys the "area under a line"
                // shape — readable even at the small legend swatch size.
                let half = sample_size / 2.0;
                let rect = Rect::new(sample_left, y - half, sample_right, y + half);
                let fill = entry.color.with_alpha(140).without_alpha();
                backend.fill_rect(rect, fill)?;
                let top = Path::new()
                    .move_to(Point::new(sample_left, y - half))
                    .line_to(Point::new(sample_right, y - half));
                backend.draw_path(&top, &PathStyle::stroke(entry.color, 1.5))?;
            }
            // LegendGlyph::Line and any future variant fall back to a
            // horizontal stroke — the safe, readable default for unknown shapes.
            LegendGlyph::Line | _ => {
                let line = Path::new()
                    .move_to(Point::new(sample_left, y))
                    .line_to(Point::new(sample_right, y));
                backend.draw_path(&line, &PathStyle::stroke(entry.color, 2.0))?;
            }
        }

        backend.draw_text(
            &entry.label,
            Point::new(legend_x + padding + sample_size + 6.0, y + 4.0),
            font_size,
            label_color,
        )?;
    }

    Ok(())
}

/// Approximate a filled circle with four cubic Béziers (the standard
/// `0.5522847498` Kappa constant). The legend-internal helper avoids pulling in
/// any additional renderer dependencies for what is a one-off use today.
fn draw_filled_disk(
    backend: &mut dyn DrawBackend,
    center: Point,
    radius: f32,
    color: Color,
) -> Result<()> {
    // Kappa for cubic-Bézier circle approximation: 4·(√2 − 1)/3 ≈ 0.552_284_8.
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
    let style = PathStyle::fill(color);
    backend.draw_path(&path, &style)
}

// ── render_title ───────────────────────────────────────────────────────────────────────────────

/// Render the chart title horizontally centered over the *plot area* (so the
/// title balances over the data, not the full canvas including y-axis label
/// margin) and vertically centered inside the title slot.
///
/// Centering policy: title-x = midpoint of `plot_area`. Pre-fix `starsight-cet`
/// centered to the title slot's full width, which on axis-bearing charts left
/// the title offset to the right of the visible plot.
///
/// # Errors
/// Returns the backend's error if text measurement or drawing fails.
pub fn render_title(
    title: &str,
    slot: &Slot,
    plot_area: &Rect,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
    fonts: &LayoutFonts,
) -> Result<()> {
    let font_size = fonts.title;
    let title_color = theme.title;
    let (tw, th) = backend
        .text_extent(title, font_size)
        .unwrap_or((0.0, font_size));
    let cx = f32::midpoint(plot_area.left, plot_area.right);
    let x = cx - tw * 0.5;
    let y = f32::midpoint(slot.rect.height(), th) + slot.rect.top;
    backend.draw_text(title, Point::new(x, y), font_size, title_color)
}

/// Render axis-title labels centered along their respective axes, drawing into
/// the slots reserved by the layout (below/beside the tick-label band).
///
/// # Errors
/// Returns the backend's error if text measurement or drawing fails.
#[allow(clippy::too_many_arguments)]
pub fn render_axis_labels(
    x_label: Option<&str>,
    y_label: Option<&str>,
    x_slot: Option<&Slot>,
    y_slot: Option<&Slot>,
    plot_area: &Rect,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
    fonts: &LayoutFonts,
) -> Result<()> {
    let font_size = fonts.label;
    let label_color = theme.tick_label;

    if let (Some(label), Some(slot)) = (x_label, x_slot) {
        let (lw, lh) = backend
            .text_extent(label, font_size)
            .unwrap_or((0.0, font_size));
        let cx = f32::midpoint(plot_area.left, plot_area.right) - lw / 2.0;
        let cy = f32::midpoint(slot.rect.height(), lh) + slot.rect.top;
        backend.draw_text(label, Point::new(cx, cy), font_size, label_color)?;
    }

    if let (Some(label), Some(slot)) = (y_label, y_slot) {
        let (lw, lh) = backend
            .text_extent(label, font_size)
            .unwrap_or((0.0, font_size));
        // Rotated -90°: post-rotation width is lh, post-rotation height is lw.
        let cx = f32::midpoint(slot.rect.width(), lh) + slot.rect.left;
        let cy = f32::midpoint(plot_area.top, plot_area.bottom) + lw / 2.0;
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
    use crate::layout::{LayoutFonts, Side, Slot};
    use starsight_layer_1::backends::vectors::SvgBackend;
    use starsight_layer_1::primitives::{Color, Rect};
    use starsight_layer_1::theme::DEFAULT_LIGHT;
    use starsight_layer_2::axes::Axis;
    use starsight_layer_2::coords::CartesianCoord;
    use starsight_layer_2::scales::LinearScale;
    use starsight_layer_3::marks::LegendGlyph;

    fn coord_with_ticks(plot: Rect) -> CartesianCoord {
        CartesianCoord {
            x_axis: Axis {
                scale: Box::new(LinearScale {
                    domain_min: 0.0,
                    domain_max: 1.0,
                }),
                label: None,
                tick_positions: vec![0.0, 0.5, 1.0],
                tick_labels: vec!["0".into(), "0.5".into(), "1".into()],
            },
            y_axis: Axis {
                scale: Box::new(LinearScale {
                    domain_min: 0.0,
                    domain_max: 1.0,
                }),
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
        render_legend(
            &[],
            &Rect::new(0.0, 0.0, 100.0, 100.0),
            &mut backend,
            &DEFAULT_LIGHT,
            &LayoutFonts::default(),
        )
        .unwrap();
    }

    #[test]
    fn render_legend_with_entries() {
        let mut backend = SvgBackend::new(400, 200);
        let entries = vec![
            LegendEntry {
                color: Color::RED,
                label: "first".into(),
                glyph: LegendGlyph::Line,
            },
            LegendEntry {
                color: Color::BLUE,
                label: "second".into(),
                glyph: LegendGlyph::Point,
            },
        ];
        render_legend(
            &entries,
            &Rect::new(0.0, 0.0, 400.0, 200.0),
            &mut backend,
            &DEFAULT_LIGHT,
            &LayoutFonts::default(),
        )
        .unwrap();
        let svg = backend.svg_string();
        assert!(svg.contains("first"));
        assert!(svg.contains("second"));
    }

    #[test]
    fn render_background_fills_rect() {
        let mut backend = SvgBackend::new(50, 50);
        render_background(
            &Rect::new(0.0, 0.0, 50.0, 50.0),
            &mut backend,
            &DEFAULT_LIGHT,
        )
        .unwrap();
    }

    #[test]
    fn render_title_centers_over_plot_area() {
        let mut backend = SvgBackend::new(200, 50);
        let slot = Slot {
            rect: Rect::new(0.0, 0.0, 200.0, 30.0),
            side: Side::Top,
        };
        let plot_area = Rect::new(20.0, 30.0, 180.0, 50.0);
        render_title(
            "Hello",
            &slot,
            &plot_area,
            &mut backend,
            &DEFAULT_LIGHT,
            &LayoutFonts::default(),
        )
        .unwrap();
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
            &LayoutFonts::default(),
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
            &LayoutFonts::default(),
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
        render_axes(
            &coord,
            &mut backend,
            &[],
            false,
            &DEFAULT_LIGHT,
            &LayoutFonts::default(),
        )
        .unwrap();
    }

    #[test]
    fn render_axes_categories_on_x() {
        let mut backend = SvgBackend::new(200, 200);
        let coord = coord_with_ticks(Rect::new(20.0, 20.0, 180.0, 180.0));
        let cats = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        render_axes(
            &coord,
            &mut backend,
            &cats,
            false,
            &DEFAULT_LIGHT,
            &LayoutFonts::default(),
        )
        .unwrap();
    }

    #[test]
    fn render_axes_categories_on_y() {
        let mut backend = SvgBackend::new(200, 200);
        let coord = coord_with_ticks(Rect::new(20.0, 20.0, 180.0, 180.0));
        let cats = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        render_axes(
            &coord,
            &mut backend,
            &cats,
            true,
            &DEFAULT_LIGHT,
            &LayoutFonts::default(),
        )
        .unwrap();
    }
}
