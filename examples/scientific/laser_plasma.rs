//! Laser-plasma phase-space — starsight 0.3.0 showcase
//!
//! Stimulated Raman scattering snapshot of warm-plasma electron phase-space
//! density. Picks the middle of the spec's three-snapshot sequence (δ=0.15).
//! Single-panel form for 0.3.0 — full multi-panel layout (with the density
//! profile and electric-field overlays from spec #7) needs `GridLayout`,
//! which lands at 0.4.0.
//!
//! Implements spec example #7 from `.spec/SHOWCASE_INPUTS.md`. Reuses the
//! 0.3.0 `HeatmapMark::log_scale()` builder added for spec #16.

use starsight::colormap::VIRIDIS;
use starsight::marks::HeatmapMark;
use starsight::prelude::*;

fn main() -> Result<()> {
    // Spec #7 grid: x ∈ [159.0, 160.0] µm, p_e ∈ [-5, 5] keV/c.
    let nx = 200;
    let np = 200;
    let (x_lo, x_hi) = (159.0_f64, 160.0_f64);
    let (p_lo, p_hi) = (-5.0_f64, 5.0_f64);

    // Warm-plasma electron parameters.
    let t_e: f64 = 2.0;
    let delta: f64 = 0.15;
    let k_l: f64 = std::f64::consts::TAU / 0.2;

    // f_e(x, p) = n0/√(2π T_e) · exp(−p²/(2 T_e)) · [1 + δ · cos(k_L · x)]
    // The Gaussian factor in p produces a horizontal band centered at p = 0;
    // the cos(k_L x) factor modulates the band into ~5 vertical fringes
    // (k_L · 1µm = 10π → 5 full periods over the x range).
    let n0 = 1.0 / (std::f64::consts::TAU * t_e).sqrt();
    let cells: Vec<Vec<f64>> = (0..np)
        .map(|j| {
            let p = p_lo + (p_hi - p_lo) * (f64::from(j) + 0.5) / f64::from(np);
            (0..nx)
                .map(|i| {
                    let x = x_lo + (x_hi - x_lo) * (f64::from(i) + 0.5) / f64::from(nx);
                    n0 * (-p * p / (2.0 * t_e)).exp() * (1.0 + delta * (k_l * x).cos())
                })
                .collect()
        })
        .collect();

    Figure::new(900, 700)
        .title("Stimulated Raman Scattering — electron phase space (δ = 0.15)")
        // 0.3.0 limitation: heatmap axes label cell indices, not the underlying
        // physical units. The labels below describe what each bin maps to.
        .x_label("x bin (159.0..160.0 µm across 200 cells)")
        .y_label("p bin (-5..5 keV/c across 200 cells)")
        .add(HeatmapMark::new(cells).colormap(VIRIDIS).log_scale())
        .save("examples/scientific/laser_plasma.png")?;

    println!("saved laser_plasma.png");
    Ok(())
}
