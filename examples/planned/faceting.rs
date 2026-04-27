//! Faceting — placeholder until 0.3.0
//!
//! The real demo will render a small-multiples grid via the planned layer-4
//! grid layout composer. Static placeholder for now.

use starsight::prelude::*;

fn main() -> Result<()> {
    let xs: Vec<f64> = (0..10).map(f64::from).collect();
    let ys = xs.clone();
    Figure::new(800, 600)
        .title("[placeholder] faceting — demo lands in 0.3.0 (layer-4 grid layout)")
        .add(
            LineMark::new(xs, ys)
                .color(Color::from_hex(0x888888))
                .width(1.0),
        )
        .save("examples/planned/faceting.png")
}
