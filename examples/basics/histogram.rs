//! Histogram — starsight 0.2.0 showcase
//!
//! `HistogramMark` on 5000 deterministic Gaussian samples (Box–Muller against an
//! LCG seed, no `rand` dep). KDE overlay lands in 0.3.0.

use starsight::prelude::*;

fn main() -> Result<()> {
    let mut state: u32 = 0x1234_5678;
    let mut next = || {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        f64::from(state) / f64::from(u32::MAX)
    };
    let mut samples = Vec::with_capacity(5000);
    for _ in 0..5000 {
        let u1 = next().max(1e-9);
        let u2 = next();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        samples.push(50.0 + 12.0 * z);
    }

    Figure::new(1000, 600)
        .title("Distribution of Synthetic Measurements")
        .x_label("Value")
        .y_label("Frequency")
        .add(HistogramMark::new(samples).color(Color::from_hex(0x4F_8AB8)))
        .save("examples/basics/histogram.png")
}
