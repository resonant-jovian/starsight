//! Statistical — starsight 0.2.0 showcase
//!
//! Noisy daily measurements with a 7-day rolling mean overlay. Demonstrates
//! layering two LineMarks on one Figure and the legend auto-derived from
//! `.label(...)`. KDE / density bands land in 0.3.0.

use starsight::prelude::*;

fn rolling_mean(data: &[f64], window: usize) -> Vec<f64> {
    let mut out = Vec::with_capacity(data.len());
    for i in 0..data.len() {
        let lo = i.saturating_sub(window / 2);
        let hi = (i + window / 2 + 1).min(data.len());
        let slice = &data[lo..hi];
        out.push(slice.iter().sum::<f64>() / slice.len() as f64);
    }
    out
}

fn main() -> Result<()> {
    let mut state: u32 = 0x0DEF_ACED;
    let mut next = || {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        f64::from(state) / f64::from(u32::MAX) - 0.5
    };

    let n = 120;
    let xs: Vec<f64> = (0..n).map(|i| f64::from(i)).collect();
    let raw: Vec<f64> = xs
        .iter()
        .map(|&x| 50.0 + 0.3 * x + 8.0 * (x * 0.15).sin() + 5.0 * next())
        .collect();
    let smoothed = rolling_mean(&raw, 7);

    Figure::new(1100, 600)
        .title("Daily Measurement vs. 7-day Rolling Mean")
        .x_label("Day")
        .y_label("Reading")
        .add(
            LineMark::new(xs.clone(), raw)
                .color(Color::from_hex(0xCBD5E1))
                .width(1.5)
                .label("raw"),
        )
        .add(
            LineMark::new(xs, smoothed)
                .color(Color::from_hex(0x2E7CB8))
                .width(2.5)
                .label("rolling mean (7d)"),
        )
        .save("examples/composition/statistical.png")
}
