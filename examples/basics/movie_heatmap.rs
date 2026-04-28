//! Movie ratings cross-tab — starsight 0.3.0 showcase
//!
//! Synthetic Rotten Tomatoes × IMDB cross-tabulation. The dual-Gaussian formula
//! produces a bright primary mode along the positive-correlation diagonal and a
//! dimmer secondary lobe at low ratings; the log-scale color mapping lifts the
//! secondary lobe out of the noise floor where linear scale would crush it.
//!
//! Implements spec example #16 from `.spec/SHOWCASE_INPUTS.md`. Demonstrates
//! the 0.3.0 `HeatmapMark::log_scale()` builder.

use starsight::colormap::VIRIDIS;
use starsight::marks::HeatmapMark;
use starsight::prelude::*;

/// Univariate Gaussian PDF.
fn phi(x: f64, mu: f64, sigma: f64) -> f64 {
    let z = (x - mu) / sigma;
    (-0.5 * z * z).exp() / (sigma * (std::f64::consts::TAU).sqrt())
}

fn main() -> Result<()> {
    let nx = 30; // Rotten Tomatoes buckets across [0, 100]
    let ny = 30; // IMDB buckets across [1.0, 10.0]

    let cells: Vec<Vec<f64>> = (0..ny)
        .map(|j| {
            // Center of bucket j on the IMDB axis.
            let y = 1.0 + 9.0 * (f64::from(j) + 0.5) / f64::from(ny);
            (0..nx)
                .map(|i| {
                    // Center of bucket i on the Rotten Tomatoes axis.
                    let x = 100.0 * (f64::from(i) + 0.5) / f64::from(nx);
                    let primary = phi(x, 65.0, 15.0) * phi(y, 7.0, 1.0) * 2e9;
                    let secondary = phi(x, 30.0, 10.0) * phi(y, 4.0, 0.8) * 5e8;
                    (primary - secondary).max(0.0)
                })
                .collect()
        })
        .collect();

    Figure::new(900, 800)
        .title("Movie Ratings Cross-Tab — Rotten Tomatoes × IMDB (log scale)")
        // HeatmapMark axes are cell indices in 0.3.0; categorical/continuous
        // axis scales for heatmaps land with the broader scale infrastructure
        // at 0.5.0. The labels below explain what each bin maps to.
        .x_label("Rotten Tomatoes bin (0..100% across 30 buckets)")
        .y_label("IMDB bin (1.0..10.0 across 30 buckets)")
        .add(HeatmapMark::new(cells).colormap(VIRIDIS).log_scale())
        .save("examples/basics/movie_heatmap.png")?;

    println!("saved movie_heatmap.png");
    Ok(())
}
