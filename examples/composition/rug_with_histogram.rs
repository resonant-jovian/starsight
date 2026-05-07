//! Rug-with-histogram — starsight 0.3.0 showcase for [`RugMark`].
//!
//! 5 000 deterministic Gaussian samples rendered as a [`HistogramMark`]
//! (binned counts) with a [`RugMark`] underneath that exposes the
//! individual observations on the x-axis margin. The rug makes density
//! visible at the per-sample level — useful when histogram bin edges hide
//! clustering, or as a sanity check on the bin choice.

use starsight::components::statistics::BinMethod;
use starsight::prelude::*;

fn main() -> Result<()> {
    // Box-Muller PRNG seeded for determinism (matches basics/histogram.rs
    // pattern).
    let n = 5_000usize;
    let mut samples = Vec::with_capacity(n);
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next_u64 = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for _ in 0..n / 2 {
        let u1 = (next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        let u2 = (next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = std::f64::consts::TAU * u2;
        samples.push(r * theta.cos());
        samples.push(r * theta.sin());
    }

    // Subsample for the rug — 5000 ticks would crowd visually.
    let rug_samples: Vec<f64> = samples.iter().step_by(20).copied().collect();

    Figure::new(900, 600)
        .title("Gaussian samples — histogram + rug overlay")
        .x_label("value")
        .y_label("count")
        .add(
            HistogramMark::new(samples)
                .method(BinMethod::Count(40))
                .color(Color::from_hex(0x0076_B7B2)),
        )
        .add(
            RugMark::new(rug_samples, AxisDir::X)
                .length(10.0)
                .color(Color::from_hex(0x0030_303A))
                .width(0.8)
                .label("samples"),
        )
        .save("examples/composition/rug_with_histogram.png")
}
