//! Custom colormap — starsight 0.2.0 showcase
//!
//! Same heatmap data as `heatmap`, rendered with the `INFERNO` colormap.
//! Available alternatives in `starsight::colormap`: VIRIDIS, PLASMA, MAGMA,
//! INFERNO, TURBO, BATLOW, BERLIN, VIK.

use starsight::colormap::INFERNO;
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
        .title("2D Density Heatmap (Inferno Colormap)")
        .x_label("X bin")
        .y_label("Y bin")
        .add(HeatmapMark::new(data).colormap(INFERNO))
        .save("examples/showcases/custom_colormap.png")
}
