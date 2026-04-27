//! Polars integration — placeholder until 0.3.0
//!
//! The real demo will accept a `polars::DataFrame` and convert columns into
//! marks via the planned `polars` optional feature. Static placeholder for
//! now.

use starsight::prelude::*;

fn main() -> Result<()> {
    let xs: Vec<f64> = (0..10).map(f64::from).collect();
    let ys = xs.clone();
    Figure::new(800, 600)
        .title("[placeholder] Polars — demo lands in 0.3.0 (polars optional feature)")
        .add(
            LineMark::new(xs, ys)
                .color(Color::from_hex(0x88_8888))
                .width(1.0),
        )
        .save("examples/planned/polars_integration.png")
}
