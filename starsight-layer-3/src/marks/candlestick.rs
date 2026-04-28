//! OHLC / candlestick mark.
//!
//! `CandlestickMark` renders financial-style open/high/low/close bars: a
//! filled body spans `open` to `close`, vertical wicks extend from each end
//! of the body to the high (above) and low (below). Body color encodes
//! direction — green for up days (`close ≥ open`), red for down days. Pass
//! the `up_color` / `down_color` builders to override the defaults for a
//! palette-aware chart.
//!
//! Status: lands in 0.3.0. The mark uses the cartesian coordinate system
//! directly — `timestamp` maps through `coord.x_axis.scale`, prices through
//! `coord.y_axis.scale` — so it composes with date-time x-axes once those
//! land in 0.5.0 without any change here.

#![allow(clippy::cast_precision_loss)]

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::errors::Result;
use starsight_layer_1::paths::{Path, PathStyle};
use starsight_layer_1::primitives::{Color, Point, Rect};
use starsight_layer_2::coords::CartesianCoord;
use starsight_layer_2::scales::Scale;

use crate::marks::{DataExtent, LegendGlyph, Mark};

// ── Ohlc ─────────────────────────────────────────────────────────────────────────────────────────

/// One OHLC bar.
///
/// `timestamp` is the x-axis coordinate (epoch seconds, day index, whatever
/// numeric scale the figure carries). The mark doesn't interpret it, just
/// passes it through `coord.x_axis.scale.map`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ohlc {
    /// X-axis position.
    pub timestamp: f64,
    /// Opening price.
    pub open: f64,
    /// High during the period.
    pub high: f64,
    /// Low during the period.
    pub low: f64,
    /// Closing price.
    pub close: f64,
}

// ── CandlestickMark ──────────────────────────────────────────────────────────────────────────────

/// OHLC candlestick chart.
#[derive(Clone, Debug)]
pub struct CandlestickMark {
    /// One row per period in chronological order.
    pub data: Vec<Ohlc>,
    /// Body color when `close ≥ open`. Default: muted green (#26a69a).
    pub up_color: Color,
    /// Body color when `close < open`. Default: muted red (#ef5350).
    pub down_color: Color,
    /// Body width as a fraction of the per-bar slot. `0.7` ≈ 70% of the
    /// allotted x-space, leaving margins between adjacent candles.
    pub body_width: f32,
    /// Wick stroke width in pixels.
    pub wick_width: f32,
    /// Legend label for the mark as a whole.
    pub label: Option<String>,
}

impl CandlestickMark {
    /// New candlestick chart from a list of OHLC rows.
    #[must_use]
    pub fn new(data: Vec<Ohlc>) -> Self {
        Self {
            data,
            up_color: Color::from_hex(0x0026_A69A),
            down_color: Color::from_hex(0x00EF_5350),
            body_width: 0.7,
            wick_width: 1.0,
            label: None,
        }
    }

    /// Builder: override the up-day body color.
    #[must_use]
    pub fn up_color(mut self, c: Color) -> Self {
        self.up_color = c;
        self
    }

    /// Builder: override the down-day body color.
    #[must_use]
    pub fn down_color(mut self, c: Color) -> Self {
        self.down_color = c;
        self
    }

    /// Builder: body width as a fraction of the per-bar x-slot, clamped to
    /// `[0.05, 0.98]`.
    #[must_use]
    pub fn body_width(mut self, w: f32) -> Self {
        self.body_width = w;
        self
    }

    /// Builder: wick stroke width in pixels.
    #[must_use]
    pub fn wick_width(mut self, w: f32) -> Self {
        self.wick_width = w;
        self
    }

    /// Builder: legend label.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    fn body_color(&self, row: &Ohlc) -> Color {
        if row.close >= row.open {
            self.up_color
        } else {
            self.down_color
        }
    }

    /// Per-bar half-width in pixels. Uses the median spacing between
    /// consecutive timestamps so non-uniform x sampling doesn't produce
    /// either over- or under-wide candles.
    fn half_body_px(&self, coord: &CartesianCoord) -> f32 {
        let area = &coord.plot_area;
        let n = self.data.len();
        if n < 2 {
            // Single bar — give it a respectable width relative to the plot.
            return area.width() * 0.05 * self.body_width.clamp(0.05, 0.98);
        }
        let mut spacings: Vec<f64> = self
            .data
            .windows(2)
            .map(|w| (w[1].timestamp - w[0].timestamp).abs())
            .filter(|s| *s > 0.0)
            .collect();
        if spacings.is_empty() {
            return area.width() * 0.05 * self.body_width.clamp(0.05, 0.98);
        }
        spacings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = spacings[spacings.len() / 2];
        let span = (coord.x_axis.scale.map(median) - coord.x_axis.scale.map(0.0)).abs();
        let band = (span as f32) * area.width();
        band * 0.5 * self.body_width.clamp(0.05, 0.98)
    }
}

impl Mark for CandlestickMark {
    fn render(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        if self.data.is_empty() {
            return Ok(());
        }
        let area = &coord.plot_area;
        let to_x = |t: f64| -> f32 { area.left + coord.x_axis.scale.map(t) as f32 * area.width() };
        let to_y =
            |v: f64| -> f32 { area.bottom - coord.y_axis.scale.map(v) as f32 * area.height() };
        let half = self.half_body_px(coord);

        for row in &self.data {
            let x_px = to_x(row.timestamp);
            let open_px = to_y(row.open);
            let close_px = to_y(row.close);
            let high_px = to_y(row.high);
            let low_px = to_y(row.low);
            let color = self.body_color(row);

            // Body: rect from min(open, close) to max(open, close). Doji
            // (open == close) collapses to a 1-px thin bar so the day still
            // shows on the chart.
            let body_top = open_px.min(close_px);
            let body_bottom = open_px.max(close_px);
            let mut body = Rect::new(x_px - half, body_top, x_px + half, body_bottom);
            if (body.bottom - body.top).abs() < 1.0 {
                body.top -= 0.5;
                body.bottom += 0.5;
            }
            backend.fill_rect(body, color)?;

            // Upper wick: top-of-body → high. Lower wick: bottom-of-body → low.
            let upper = Path::new()
                .move_to(Point::new(x_px, body.top))
                .line_to(Point::new(x_px, high_px));
            backend.draw_path(&upper, &PathStyle::stroke(color, self.wick_width))?;
            let lower = Path::new()
                .move_to(Point::new(x_px, body.bottom))
                .line_to(Point::new(x_px, low_px));
            backend.draw_path(&lower, &PathStyle::stroke(color, self.wick_width))?;
        }
        Ok(())
    }

    fn data_extent(&self) -> Option<DataExtent> {
        if self.data.is_empty() {
            return None;
        }
        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;
        for row in &self.data {
            if row.timestamp < x_min {
                x_min = row.timestamp;
            }
            if row.timestamp > x_max {
                x_max = row.timestamp;
            }
            if row.low < y_min {
                y_min = row.low;
            }
            if row.high > y_max {
                y_max = row.high;
            }
        }
        // Inset the reported x range by half a band so the leftmost and
        // rightmost candle bodies sit fully inside the plot rect rather
        // than half-spilling onto the y-axis (yrp.5). Half-band is half
        // the median timestamp spacing for n ≥ 2; degenerate single-row
        // case widens by ±0.5 around the lone timestamp.
        let half_band = if self.data.len() < 2 {
            0.5
        } else {
            let mut spacings: Vec<f64> = self
                .data
                .windows(2)
                .map(|w| (w[1].timestamp - w[0].timestamp).abs())
                .filter(|s| *s > 0.0)
                .collect();
            if spacings.is_empty() {
                0.5
            } else {
                spacings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                spacings[spacings.len() / 2] * 0.5
            }
        };
        Some(DataExtent {
            x_min: x_min - half_band,
            x_max: x_max + half_band,
            y_min,
            y_max,
        })
    }

    fn legend_color(&self) -> Option<Color> {
        self.label.as_ref()?;
        Some(self.up_color)
    }

    fn legend_label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    fn legend_glyph(&self) -> LegendGlyph {
        LegendGlyph::Bar
    }
}

#[cfg(test)]
mod tests {
    use super::{CandlestickMark, Ohlc};
    use crate::marks::{LegendGlyph, Mark};
    use starsight_layer_1::primitives::Color;

    fn sample() -> Vec<Ohlc> {
        vec![
            Ohlc {
                timestamp: 0.0,
                open: 100.0,
                high: 110.0,
                low: 95.0,
                close: 105.0,
            },
            Ohlc {
                timestamp: 1.0,
                open: 105.0,
                high: 115.0,
                low: 100.0,
                close: 98.0,
            },
            Ohlc {
                timestamp: 2.0,
                open: 98.0,
                high: 108.0,
                low: 90.0,
                close: 107.0,
            },
        ]
    }

    #[test]
    fn data_extent_covers_low_high_per_row() {
        // x range insets by half a band on each side so the leftmost and
        // rightmost candles fit inside the plot rect (yrp.5). Sample uses
        // unit spacing → half_band = 0.5.
        let mark = CandlestickMark::new(sample());
        let extent = mark.data_extent().expect("non-empty extent");
        assert!((extent.x_min - (-0.5)).abs() < 1e-9);
        assert!((extent.x_max - 2.5).abs() < 1e-9);
        assert_eq!(extent.y_min, 90.0);
        assert_eq!(extent.y_max, 115.0);
    }

    #[test]
    fn data_extent_single_row_widens_by_half() {
        let mark = CandlestickMark::new(vec![Ohlc {
            timestamp: 10.0,
            open: 1.0,
            high: 2.0,
            low: 0.5,
            close: 1.5,
        }]);
        let extent = mark.data_extent().expect("non-empty extent");
        assert!((extent.x_min - 9.5).abs() < 1e-9);
        assert!((extent.x_max - 10.5).abs() < 1e-9);
    }

    #[test]
    fn empty_has_no_extent() {
        let mark = CandlestickMark::new(vec![]);
        assert!(mark.data_extent().is_none());
    }

    #[test]
    fn body_color_picks_up_for_close_ge_open() {
        let mark = CandlestickMark::new(sample());
        let up_row = Ohlc {
            timestamp: 0.0,
            open: 100.0,
            high: 110.0,
            low: 95.0,
            close: 110.0,
        };
        let down_row = Ohlc {
            timestamp: 0.0,
            open: 110.0,
            high: 115.0,
            low: 100.0,
            close: 100.0,
        };
        assert_eq!(mark.body_color(&up_row), mark.up_color);
        assert_eq!(mark.body_color(&down_row), mark.down_color);
    }

    #[test]
    fn legend_glyph_is_bar_and_color_uses_up_color() {
        let mark = CandlestickMark::new(sample())
            .up_color(Color::GREEN)
            .label("ticker");
        assert_eq!(mark.legend_glyph(), LegendGlyph::Bar);
        assert_eq!(mark.legend_color(), Some(Color::GREEN));
    }

    #[test]
    fn no_legend_when_unlabeled() {
        let mark = CandlestickMark::new(sample());
        assert!(mark.legend_color().is_none());
    }
}
