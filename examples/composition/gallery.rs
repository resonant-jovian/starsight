//! Gallery — starsight 0.2.0 showcase
//!
//! Layered composite: a LineMark on top of PointMark scatter, all in a
//! single Figure. Multi-figure faceting (real grid layout) lands in 0.3.0
//! once layer-4 ships its grid composer.

use starsight::prelude::*;

fn main() -> Result<()> {
    let mut state: u32 = 0xCAFE_F00D;
    let mut noise = || {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        f64::from(state) / f64::from(u32::MAX) - 0.5
    };

    let xs: Vec<f64> = (0..40).map(|i| f64::from(i) * 0.25).collect();
    let truth: Vec<f64> = xs.iter().map(|&x| 3.0 + 1.4 * x).collect();
    let noisy: Vec<f64> = xs
        .iter()
        .zip(&truth)
        .map(|(_, &y)| y + 1.5 * noise())
        .collect();

    Figure::new(1100, 650)
        .title("Linear Fit Over Noisy Observations")
        .x_label("x")
        .y_label("y")
        .add(
            PointMark::new(xs.clone(), noisy)
                .color(Color::from_hex(0x6B7280))
                .radius(3.5)
                .label("observations"),
        )
        .add(
            LineMark::new(xs, truth)
                .color(Color::from_hex(0xD62728))
                .width(2.5)
                .label("model"),
        )
        .save("examples/composition/gallery.png")
}
