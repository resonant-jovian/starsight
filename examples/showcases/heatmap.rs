//! Heatmap — starsight 0.2.0 showcase
//!
//! 30×30 HeatmapMark of a synthetic 2D Gaussian-like field with the default
//! colormap. Annotated colorbar / value annotations land in 0.3.0.

use starsight::marks::HeatmapMark;
use starsight::prelude::*;

fn main() -> Result<()> {
    let n = 30;
    let data: Vec<Vec<f64>> = (0..n)
        .map(|j| {
            (0..n)
                .map(|i| {
                    let x = (f64::from(i) - 15.0) / 6.0;
                    let y = (f64::from(j) - 12.0) / 5.0;
                    (-(x * x + y * y)).exp() + 0.3 * (x * 0.7 + y).sin()
                })
                .collect()
        })
        .collect();

    Figure::new(720, 720)
        .title("2D Density Heatmap")
        .x_label("X bin")
        .y_label("Y bin")
        .add(HeatmapMark::new(data))
        .save("examples/showcases/heatmap.png")
}
