//! Terminal — placeholder until 0.3.0
//!
//! The real demo will render a chart inside a terminal via the planned
//! ratatui backend. Until then, this writes a static 2D PNG announcing the
//! deferred feature so `cargo xtask gallery` still has a uniform output set.

use starsight::prelude::*;

fn main() -> Result<()> {
    let xs: Vec<f64> = (0..10).map(f64::from).collect();
    let ys = xs.clone();
    Figure::new(800, 600)
        .title("[placeholder] terminal — demo lands in 0.3.0 (ratatui backend)")
        .add(
            LineMark::new(xs, ys)
                .color(Color::from_hex(0x888888))
                .width(1.0),
        )
        .save("examples/planned/terminal.png")
}
