//! Interactive — placeholder until 0.3.0
//!
//! The real demo will open a winit window and render via the wgpu backend.
//! Static placeholder for now so the gallery has a uniform output set.

use starsight::prelude::*;

fn main() -> Result<()> {
    let xs: Vec<f64> = (0..10).map(f64::from).collect();
    let ys = xs.clone();
    Figure::new(800, 600)
        .title("[placeholder] interactive — demo lands in 0.3.0 (winit + wgpu backend)")
        .add(
            LineMark::new(xs, ys)
                .color(Color::from_hex(0x88_8888))
                .width(1.0),
        )
        .save("examples/planned/interactive.png")
}
