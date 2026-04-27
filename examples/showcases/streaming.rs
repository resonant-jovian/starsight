//! Streaming — placeholder until 0.3.0
//!
//! The real demo will plot a live windowed data source. Static placeholder
//! for now.

use starsight::prelude::*;

fn main() -> Result<()> {
    let xs: Vec<f64> = (0..10).map(f64::from).collect();
    let ys = xs.clone();
    Figure::new(800, 600)
        .title("streaming — demo lands in 0.3.0 (windowed data source)")
        .add(LineMark::new(xs, ys).color(Color::from_hex(0x888888)).width(1.0))
        .save("examples/showcases/streaming.png")
}
