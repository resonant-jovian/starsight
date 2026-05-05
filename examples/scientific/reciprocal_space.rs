//! Reciprocal-space scattering — starsight 0.3.0 showcase #17.
//!
//! 2 × 3 [`MultiPanelFigure`]: top row of `S(h, k)` heatmaps at three
//! temperatures, bottom row of horizontal cuts at `k = 0.5` with √I error
//! bars. The Bragg-peak structure factor is
//!
//! ```text
//!     S(h, k) = Σ_Q  A_Q · T_amp / ((h − Q_h)² + (k − Q_k)² + Γ²)
//! ```
//!
//! with `Q ∈ {(1, 0), (0, 1), (2, 1), (1, 2), (½, ½), (3/2, 1/2)}`,
//! Γ = 0.05, and the temperature-dependent amplitude scaling `T_amp ∈
//! {1.0, 0.7, 0.3}`. Spec calls for a 3 × 3 layout, but the third row
//! collapses cleanly into the cut row by carrying the per-temperature label
//! in the panel title — six panels read better than nine here.
//!
//! Each top-row heatmap auto-attaches a [`Colorbar`](starsight::Colorbar)
//! via the colormap-legend dispatch shipped with Epic G — the bordered
//! legend dedup that Epic I.5 applied prevents the figure from showing both
//! a colorbar and a redundant labeled-mark legend entry.

#![allow(clippy::cast_precision_loss)]

use starsight::colormap::VIRIDIS;
use starsight::common::MultiPanelFigure;
use starsight::prelude::*;

const N_GRID: usize = 80;
const H_RANGE: f64 = 2.5;
const K_RANGE: f64 = 2.5;
const GAMMA_SQ: f64 = 0.05 * 0.05;
const TEMPERATURES: [(&str, f64); 3] = [("T = 30 K", 1.0), ("T = 150 K", 0.7), ("T = 300 K", 0.3)];

fn bragg_peaks() -> [(f64, f64, f64); 6] {
    // (Q_h, Q_k, A_Q) — equal-amplitude lattice peaks; the two diagonals
    // sit at half-integer positions to break the obvious square symmetry.
    [
        (1.0, 0.0, 1.0),
        (0.0, 1.0, 1.0),
        (2.0, 1.0, 0.8),
        (1.0, 2.0, 0.8),
        (0.5, 0.5, 0.6),
        (1.5, 0.5, 0.6),
    ]
}

fn structure_factor(h: f64, k: f64, t_amp: f64) -> f64 {
    let mut total = 0.0;
    for &(q_h, q_k, a_q) in &bragg_peaks() {
        let dh = h - q_h;
        let dk = k - q_k;
        total += a_q * t_amp / (dh * dh + dk * dk + GAMMA_SQ);
    }
    total
}

fn heatmap_panel(title: &str, t_amp: f64) -> Figure {
    let mut data = Vec::with_capacity(N_GRID);
    for j in 0..N_GRID {
        let k = K_RANGE * (j as f64) / (N_GRID as f64 - 1.0);
        let mut row = Vec::with_capacity(N_GRID);
        for i in 0..N_GRID {
            let h = H_RANGE * (i as f64) / (N_GRID as f64 - 1.0);
            row.push(structure_factor(h, k, t_amp));
        }
        data.push(row);
    }
    Figure::new(420, 360)
        .title(title)
        .x_label("h")
        .y_label("k")
        .add(
            HeatmapMark::new(data)
                .colormap(VIRIDIS)
                .log_scale()
                .label("S(h, k)"),
        )
}

fn cut_panel(title: &str, t_amp: f64) -> Figure {
    // Horizontal line cut at k = 0.5, sampled on a 64-point h-grid. Each
    // intensity carries a Poisson √I error bar; both intensities and
    // counts use the same scaling factor so the error bars shrink with
    // temperature exactly as the heatmap does.
    let n: u32 = 64;
    let xs: Vec<f64> = (0..n)
        .map(|i: u32| H_RANGE * f64::from(i) / f64::from(n - 1))
        .collect();
    let ys: Vec<f64> = xs.iter().map(|h| structure_factor(*h, 0.5, t_amp)).collect();
    // Treat S(h, k) as a count rate ~ I; std deviation ≈ √I.
    let errs: Vec<f64> = ys.iter().map(|y| y.max(0.0).sqrt() * 0.6).collect();
    Figure::new(420, 320)
        .title(title)
        .x_label("h (cut at k = 0.5)")
        .y_label("intensity")
        .add(LineMark::new(xs.clone(), ys.clone()).color(Color::from_hex(0x0033_77BB)))
        .add(
            ErrorBarMark::new(xs, ys, errs)
                .color(Color::from_hex(0x0040_4040))
                .cap_width(4.0),
        )
}

fn main() -> Result<()> {
    let mut mp = MultiPanelFigure::new(1320, 720, 2, 3).padding(14.0);
    for &(label, t_amp) in &TEMPERATURES {
        mp = mp.add(heatmap_panel(label, t_amp));
    }
    for &(label, t_amp) in &TEMPERATURES {
        mp = mp.add(cut_panel(&format!("{label} — h cut"), t_amp));
    }
    mp.save("examples/scientific/reciprocal_space.png")
}
