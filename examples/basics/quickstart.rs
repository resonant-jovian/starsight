//! Quickstart — starsight 0.2.0 showcase
//!
//! The absolute simplest path: the `plot!` macro, two arrays, one save call,
//! plus three optional setters for a shippable-looking chart. This is what
//! gets advertised at the top of `starsight/src/lib.rs`.
//!
//! The data is a synthetic monthly metric — a slow upward trend with seasonal
//! oscillation — so the curve has a recognizable shape rather than the
//! arbitrary zigzag a placeholder dataset gives.

use starsight::prelude::*;

fn main() -> Result<()> {
    let xs: Vec<f64> = (0..30).map(f64::from).collect();
    // Linear growth with a gentle ±8 oscillation; lands in the 50–95 range.
    let ys: Vec<f64> = xs
        .iter()
        .map(|x| 50.0 + x * 1.3 + 8.0 * (x * 0.4).sin())
        .collect();

    plot!(&xs, &ys)
        .title("Quickstart — daily metric")
        .x_label("day")
        .y_label("value")
        .save("examples/basics/quickstart.png")
}
