//! Colorbar — vertical scale-to-colormap legend on the right side of a
//! plot area.
//!
//! Auto-attached by [`crate::figures::Figure`] when at least one mark
//! returns a [`ColormapLegend`] (currently `HeatmapMark` and `ContourMark`
//! with a colormap), unless the user opts out via `Figure::colorbar(false)`.
//! The opt-out path keeps existing layouts unchanged for users who want a
//! manual legend instead.
//! Renders a vertical gradient strip with five tick labels (min, 25%, 50%,
//! 75%, max) and an optional rotated label to the right of the strip.
//!
//! Tracked as `starsight-kdi`.
//!
//! [`ColormapLegend`]: starsight_layer_3::marks::ColormapLegend

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::primitives::Rect;
use starsight_layer_1::theme::Theme;
use starsight_layer_3::marks::ColormapLegend;

use crate::layout::{LayoutComponent, LayoutCtx, LayoutFonts, Reservation, Side, Slot};

/// Pixel width of the colorbar strip itself (the gradient column).
pub(crate) const COLORBAR_STRIP_WIDTH: f32 = 16.0;
/// Pixel gap between the plot edge and the colorbar strip.
const STRIP_GAP_FROM_PLOT: f32 = 16.0;
/// Pixel gap between the colorbar strip and its tick labels.
const TICK_LABEL_GAP: f32 = 4.0;
/// Tick mark length.
const TICK_LEN: f32 = 4.0;
/// Number of color samples used to render the gradient.
const GRADIENT_SAMPLES: u32 = 64;

// ── Colorbar ─────────────────────────────────────────────────────────────────────────────────────

/// A colormap legend rendered as a vertical strip on the right side of the
/// plot area.
#[derive(Clone, Debug)]
pub struct Colorbar {
    /// Underlying colormap legend description (extracted from a mark via
    /// [`Mark::colormap_legend`]).
    ///
    /// [`Mark::colormap_legend`]: starsight_layer_3::marks::Mark::colormap_legend
    pub legend: ColormapLegend,
}

impl Colorbar {
    /// New colorbar from a colormap-legend description.
    #[must_use]
    pub fn new(legend: ColormapLegend) -> Self {
        Self { legend }
    }

    /// Compute five tick labels at min, 25%, 50%, 75%, max for the colorbar
    /// value range. Linear-spaced; log-scale ticks would require a separate
    /// generator.
    fn tick_values(&self) -> [f64; 5] {
        let lo = self.legend.value_min;
        let hi = self.legend.value_max;
        [
            lo,
            lo + (hi - lo) * 0.25,
            lo + (hi - lo) * 0.5,
            lo + (hi - lo) * 0.75,
            hi,
        ]
    }
}

// ── ColorbarComponent ────────────────────────────────────────────────────────────────────────────

/// `LayoutComponent` for the colorbar — reserves a Right-side slot wide
/// enough for the strip + tick labels (+ optional rotated axis label).
pub(crate) struct ColorbarComponent<'a> {
    pub colorbar: &'a Colorbar,
}

impl<'a> LayoutComponent for ColorbarComponent<'a> {
    fn id(&self) -> &'static str {
        "colorbar"
    }
    fn reserve(&self, ctx: &mut LayoutCtx) -> Vec<Reservation> {
        // Reserve enough for: gap-from-plot + strip + label-gap + max tick
        // label width + (axis label width if any).
        let mut max_label_w: f32 = 0.0;
        let ticks = self.colorbar.tick_values();
        for v in ticks {
            let label = format_tick(v);
            if let Ok((w, _)) = ctx.backend.text_extent(&label, ctx.fonts.label) {
                max_label_w = max_label_w.max(w);
            }
        }
        let axis_label_extra = if let Some(label) = &self.colorbar.legend.label {
            ctx.backend
                .text_extent(label, ctx.fonts.label)
                .map_or(0.0, |(_, h)| h + 8.0)
        } else {
            0.0
        };
        let size = STRIP_GAP_FROM_PLOT
            + COLORBAR_STRIP_WIDTH
            + TICK_LEN
            + TICK_LABEL_GAP
            + max_label_w
            + axis_label_extra
            + 8.0;
        vec![Reservation {
            side: Side::Right,
            size,
            // Higher priority than tick labels' Right reservation (which is
            // priority 0 for the ~2px overflow). Colorbar sits further out.
            priority: 1,
        }]
    }
}

// ── render_colorbar ──────────────────────────────────────────────────────────────────────────────

/// Render the colorbar's gradient strip + tick labels + optional rotated
/// axis label inside `slot`.
///
/// # Errors
/// Returns the backend's error if any draw call fails.
pub fn render_colorbar(
    colorbar: &Colorbar,
    slot: &Slot,
    plot_area: &Rect,
    backend: &mut dyn DrawBackend,
    theme: &Theme,
    fonts: &LayoutFonts,
) -> Result<()> {
    // Anchor the strip vertically to the plot area so its color samples line
    // up with the data column they describe.
    let strip_left = slot.rect.left + STRIP_GAP_FROM_PLOT;
    let strip_right = strip_left + COLORBAR_STRIP_WIDTH;
    let strip_top = plot_area.top;
    let strip_bottom = plot_area.bottom;
    let strip_height = strip_bottom - strip_top;

    // Gradient: stack `GRADIENT_SAMPLES` thin rects, each colored by sampling
    // the colormap at its normalized vertical position. Top of strip = max
    // value (t=1), bottom = min value (t=0) — matplotlib convention.
    let n = GRADIENT_SAMPLES;
    let n_f64 = f64::from(n);
    let n_f32 = n as f32;
    for i in 0..n {
        let i_f64 = f64::from(i);
        let t_top = 1.0 - i_f64 / n_f64;
        let t_bot = 1.0 - (i_f64 + 1.0) / n_f64;
        let t_mid = (t_top + t_bot) * 0.5;
        let color = colorbar.legend.colormap.sample(t_mid);
        let band_top = strip_top + (i as f32 / n_f32) * strip_height;
        let band_bottom = strip_top + ((i as f32 + 1.0) / n_f32) * strip_height;
        let rect = Rect::new(strip_left, band_top, strip_right, band_bottom);
        backend.fill_rect(rect, color)?;
    }

    // 1px outline so the strip reads as a defined element against the
    // figure background.
    let outline = starsight_layer_1::paths::Path::new()
        .move_to(starsight_layer_1::primitives::Point::new(strip_left, strip_top))
        .line_to(starsight_layer_1::primitives::Point::new(strip_right, strip_top))
        .line_to(starsight_layer_1::primitives::Point::new(
            strip_right,
            strip_bottom,
        ))
        .line_to(starsight_layer_1::primitives::Point::new(strip_left, strip_bottom))
        .close();
    backend.draw_path(
        &outline,
        &starsight_layer_1::paths::PathStyle::stroke(theme.axis, 1.0),
    )?;

    // Tick marks + labels at five positions (min, 25, 50, 75, max). Top of
    // strip = max, so y(value) = top + (1 - normalized) * height.
    let lo = colorbar.legend.value_min;
    let hi = colorbar.legend.value_max;
    let range = (hi - lo).max(f64::EPSILON);
    let font_size = fonts.label;
    let tick_color = theme.axis;
    let label_color = theme.tick_label;
    let label_x = strip_right + TICK_LEN + TICK_LABEL_GAP;
    let mut max_label_right = label_x;
    for v in colorbar.tick_values() {
        let normalized = (v - lo) / range;
        let py = strip_top + (1.0 - normalized as f32) * strip_height;
        // Tick mark.
        let tick_path = starsight_layer_1::paths::Path::new()
            .move_to(starsight_layer_1::primitives::Point::new(strip_right, py))
            .line_to(starsight_layer_1::primitives::Point::new(
                strip_right + TICK_LEN,
                py,
            ));
        backend.draw_path(
            &tick_path,
            &starsight_layer_1::paths::PathStyle::stroke(tick_color, 1.0),
        )?;
        // Label.
        let text = format_tick(v);
        let (tw, _) = backend
            .text_extent(&text, font_size)
            .unwrap_or((0.0, font_size));
        backend.draw_text(
            &text,
            starsight_layer_1::primitives::Point::new(label_x, py + font_size * 0.4),
            font_size,
            label_color,
        )?;
        max_label_right = max_label_right.max(label_x + tw);
    }

    // Optional axis label rotated 90° CW to the right of the labels.
    if let Some(label) = &colorbar.legend.label {
        let axis_x = max_label_right + 12.0;
        let axis_y = (strip_top + strip_bottom) * 0.5;
        let (tw, _) = backend
            .text_extent(label, font_size)
            .unwrap_or((0.0, font_size));
        backend.draw_rotated_text(
            label,
            starsight_layer_1::primitives::Point::new(axis_x, axis_y + tw * 0.5),
            font_size,
            label_color,
            -90.0,
        )?;
    }

    Ok(())
}

/// Render a numeric tick value as a compact label. Picks a fixed format
/// suitable for typical heatmap value ranges.
fn format_tick(v: f64) -> String {
    if v == 0.0 {
        "0".to_string()
    } else if v.abs() >= 1000.0 || v.abs() < 0.01 {
        format!("{v:.1e}")
    } else if v.abs() >= 100.0 {
        format!("{v:.0}")
    } else if v.abs() >= 10.0 {
        format!("{v:.1}")
    } else {
        format!("{v:.2}")
    }
}

#[cfg(test)]
mod tests {
    use super::{Colorbar, format_tick};
    use starsight_layer_1::colormap::VIRIDIS;
    use starsight_layer_3::marks::ColormapLegend;

    fn make_legend(min: f64, max: f64) -> ColormapLegend {
        ColormapLegend {
            colormap: VIRIDIS,
            value_min: min,
            value_max: max,
            label: None,
            log_scale: false,
        }
    }

    #[test]
    fn tick_values_are_evenly_spaced() {
        let bar = Colorbar::new(make_legend(0.0, 100.0));
        let ticks = bar.tick_values();
        assert!((ticks[0] - 0.0).abs() < f64::EPSILON);
        assert!((ticks[1] - 25.0).abs() < f64::EPSILON);
        assert!((ticks[2] - 50.0).abs() < f64::EPSILON);
        assert!((ticks[3] - 75.0).abs() < f64::EPSILON);
        assert!((ticks[4] - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn format_tick_rounds_by_magnitude() {
        assert_eq!(format_tick(0.0), "0");
        assert_eq!(format_tick(0.5), "0.50");
        assert_eq!(format_tick(15.0), "15.0");
        assert_eq!(format_tick(150.0), "150");
        assert_eq!(format_tick(15000.0), "1.5e4");
    }

    #[test]
    fn small_values_use_scientific() {
        assert_eq!(format_tick(0.005), "5.0e-3");
    }
}
