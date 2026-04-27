//! Scatter plot — starsight 0.2.0 showcase
//!
//! Two color-coded clusters on a single PointMark per series. Demonstrates
//! `.label(name)` driving legend rendering and the `.radius(px)` builder.

use starsight::prelude::*;

fn cluster(cx: f64, cy: f64, n: usize, seed: u32) -> (Vec<f64>, Vec<f64>) {
    // Tiny LCG so output is deterministic without pulling in `rand`.
    let mut state = seed;
    let mut next = || {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        f64::from(state) / f64::from(u32::MAX)
    };
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for _ in 0..n {
        let u1 = next().max(1e-9);
        let u2 = next();
        let r = (-2.0 * u1.ln()).sqrt();
        xs.push(cx + r * (2.0 * std::f64::consts::PI * u2).cos());
        ys.push(cy + r * (2.0 * std::f64::consts::PI * u2).sin());
    }
    (xs, ys)
}

fn main() -> Result<()> {
    let (xa, ya) = cluster(2.5, 3.0, 80, 42);
    let (xb, yb) = cluster(6.0, 5.5, 80, 4711);

    Figure::new(900, 700)
        .title("Two Clusters in 2D Feature Space")
        .x_label("Feature A")
        .y_label("Feature B")
        .add(
            PointMark::new(xa, ya)
                .color(Color::from_hex(0x1F77B4))
                .radius(4.0)
                .label("Group α"),
        )
        .add(
            PointMark::new(xb, yb)
                .color(Color::from_hex(0xD62728))
                .radius(4.0)
                .label("Group β"),
        )
        .save("examples/basics/scatter.png")
}
