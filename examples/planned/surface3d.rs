//! 3D surface — placeholder until 0.3.0
//!
//! The real demo will render a 3D surface via the vello/wgpu backend.
//! Static placeholder for now.

use starsight::prelude::*;

fn main() -> Result<()> {
    let xs: Vec<f64> = (0..10).map(f64::from).collect();
    let ys = xs.clone();
    Figure::new(800, 600)
        .title("[placeholder] 3D surface — demo lands in 0.3.0 (vello/wgpu backend)")
        .add(
            LineMark::new(xs, ys)
                .color(Color::from_hex(0x88_8888))
                .width(1.0),
        )
        .save("examples/planned/surface3d.png")
}
