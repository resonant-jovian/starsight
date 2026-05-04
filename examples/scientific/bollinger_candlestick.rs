//! Bollinger candlestick — starsight 0.3.0 multi-panel showcase #38.
//!
//! Two stacked panels share an x-axis (per panel — shared axes across rows
//! land in 0.4.0):
//!
//! - **Top panel:** OHLC candles (90 trading days of synthetic data) with a
//!   20-day simple moving average and Bollinger bands (`SMA ± 2σ`) drawn as
//!   blue/grey envelope `LineMark`s.
//! - **Bottom panel:** A `BarMark` of daily volume.
//!
//! `MultiPanelFigure(width, height, 2, 1)` partitions the canvas into the two
//! rows; each panel keeps its own `Figure` builder chain.

#![allow(clippy::cast_precision_loss)]

use starsight::common::MultiPanelFigure;
use starsight::prelude::*;

fn synthetic_ohlc(n: usize) -> (Vec<Ohlc>, Vec<f64>) {
    // Deterministic mean-reverting walk with regime change near the middle.
    let mut ohlc = Vec::with_capacity(n);
    let mut volume = Vec::with_capacity(n);
    let mut price: f64 = 100.0;
    for i in 0..n {
        let i_f = i as f64;
        // Drift + sinusoid + a "crash" in the middle quartile.
        let drift = if (n / 3..2 * n / 3).contains(&i) {
            -0.18
        } else {
            0.05
        };
        let wave = (i_f * 0.18).sin() * 0.6;
        let shock = (i_f * 1.7).sin() * 0.45;
        let prev = price;
        price = (price + drift + wave + shock).max(20.0);
        let high = price.max(prev) + 1.5 + (i_f * 0.31).cos().abs() * 1.2;
        let low = price.min(prev) - 1.5 - (i_f * 0.27).sin().abs() * 1.2;
        ohlc.push(Ohlc {
            timestamp: i_f,
            open: prev,
            high,
            low,
            close: price,
        });
        // Volume spikes on big-move days.
        let body = (price - prev).abs();
        let vol = 1500.0 + body * 2200.0 + (i_f * 0.7).cos().abs() * 600.0;
        volume.push(vol);
    }
    (ohlc, volume)
}

fn sma(values: &[f64], window: usize) -> Vec<f64> {
    if values.len() < window {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(values.len() - window + 1);
    let mut sum: f64 = values[..window].iter().sum();
    out.push(sum / window as f64);
    for i in window..values.len() {
        sum += values[i] - values[i - window];
        out.push(sum / window as f64);
    }
    out
}

fn rolling_std(values: &[f64], window: usize) -> Vec<f64> {
    if values.len() < window {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(values.len() - window + 1);
    for i in window..=values.len() {
        let slice = &values[i - window..i];
        let mean = slice.iter().sum::<f64>() / window as f64;
        let var = slice.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / window as f64;
        out.push(var.sqrt());
    }
    out
}

fn main() -> Result<()> {
    let (ohlc, volume) = synthetic_ohlc(90);
    let closes: Vec<f64> = ohlc.iter().map(|o| o.close).collect();
    let window = 20;
    let sma_values = sma(&closes, window);
    let std_values = rolling_std(&closes, window);
    let xs: Vec<f64> = (window - 1..closes.len()).map(|i| i as f64).collect();
    let upper: Vec<f64> = sma_values
        .iter()
        .zip(&std_values)
        .map(|(m, s)| m + 2.0 * s)
        .collect();
    let lower: Vec<f64> = sma_values
        .iter()
        .zip(&std_values)
        .map(|(m, s)| m - 2.0 * s)
        .collect();

    let candles_panel = Figure::new(1200, 500)
        .title("Synthetic asset — candles + 20d Bollinger (±2σ)")
        .y_label("price")
        .add(CandlestickMark::new(ohlc).label("OHLC"))
        .add(
            LineMark::new(xs.clone(), sma_values)
                .color(Color::from_hex(0x002A_5099))
                .width(2.0)
                .label("20-day SMA"),
        )
        .add(
            LineMark::new(xs.clone(), upper)
                .color(Color::from_hex(0x0099_99AA))
                .width(1.0)
                .label("+2σ"),
        )
        .add(
            LineMark::new(xs, lower)
                .color(Color::from_hex(0x0099_99AA))
                .width(1.0)
                .label("-2σ"),
        );

    let volume_labels: Vec<String> = (0..volume.len()).map(|i| format!("{i}")).collect();
    let volume_panel = Figure::new(1200, 250)
        .title("Volume")
        .y_label("contracts")
        .add(BarMark::new(volume_labels, volume).color(Color::from_hex(0x0044_8855)));

    MultiPanelFigure::new(1200, 800, 2, 1)
        .padding(10.0)
        .add(candles_panel)
        .add(volume_panel)
        .save("examples/scientific/bollinger_candlestick.png")
}
