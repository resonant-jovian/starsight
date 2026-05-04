//! Wind rose — starsight 0.3.0 showcase #33.
//!
//! Stacked polar bars: 16 compass directions × 4 wind-speed bins. Each
//! direction gets one stack of 4 wedges; each speed bin maps to a single
//! [`PolarBarMark`] layer that uses `r_base` to sit on top of the previous
//! layer. Backed by `Figure::polar_axes` with a categorical angular axis.
//!
//! Synthetic data — frequencies are chosen so the rose has a clear
//! prevailing wind from the SW (slot 11) tapering elsewhere.

use starsight::axes::Axis;
use starsight::prelude::*;

fn main() -> Result<()> {
    // 16 compass slots: N, NNE, NE, ENE, E, ESE, SE, SSE, S, SSW, SW, WSW,
    // W, WNW, NW, NNW. Indexed 0..16; data goes through the categorical axis.
    let n = 16usize;
    let thetas: Vec<f64> = (0..n).map(|i| i as f64).collect();

    // Synthetic frequencies (% of total observations) per (direction, speed_bin).
    // 4 speed bins: 0–5, 5–10, 10–15, 15+ m/s.
    // Prevailing SW wind (idx 11): heaviest in mid-speed bins.
    #[rustfmt::skip]
    let bin_0_5 = vec![
        2.0, 1.8, 1.6, 1.4, 1.2, 1.0, 1.2, 1.4,
        1.6, 1.8, 2.0, 3.5, 3.0, 2.5, 2.2, 2.0,
    ];
    #[rustfmt::skip]
    let bin_5_10 = vec![
        1.5, 1.4, 1.3, 1.2, 1.1, 1.0, 1.2, 1.4,
        1.6, 1.8, 2.0, 4.5, 3.8, 3.0, 2.4, 1.8,
    ];
    #[rustfmt::skip]
    let bin_10_15 = vec![
        0.8, 0.7, 0.6, 0.5, 0.4, 0.4, 0.5, 0.6,
        0.7, 0.9, 1.2, 3.5, 2.6, 1.8, 1.2, 0.9,
    ];
    #[rustfmt::skip]
    let bin_15_plus = vec![
        0.2, 0.2, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1,
        0.2, 0.3, 0.5, 1.5, 0.8, 0.4, 0.2, 0.2,
    ];

    // Stack r_base[i] = sum of all preceding bins at index i.
    let bin_5_10_base: Vec<f64> = bin_0_5.clone();
    let bin_10_15_base: Vec<f64> = bin_0_5
        .iter()
        .zip(&bin_5_10)
        .map(|(a, b)| a + b)
        .collect();
    let bin_15_plus_base: Vec<f64> = bin_10_15_base
        .iter()
        .zip(&bin_10_15)
        .map(|(a, b)| a + b)
        .collect();

    let theta_axis = Axis::polar_angular_categorical(n);
    let r_axis = Axis::polar_radial(0.0, 12.0);

    Figure::new(800, 800)
        .title("Wind rose — frequency × direction × speed")
        .polar_axes(theta_axis, r_axis)
        .add(
            PolarBarMark::new(thetas.clone(), bin_0_5)
                .color(Color::from_hex(0x00B5_D8E8))
                .stroke(Color::WHITE, 0.6)
                .label("0–5 m/s"),
        )
        .add(
            PolarBarMark::new(thetas.clone(), bin_5_10)
                .r_base(bin_5_10_base)
                .color(Color::from_hex(0x0076_B7B2))
                .stroke(Color::WHITE, 0.6)
                .label("5–10 m/s"),
        )
        .add(
            PolarBarMark::new(thetas.clone(), bin_10_15)
                .r_base(bin_10_15_base)
                .color(Color::from_hex(0x00F2_8E2B))
                .stroke(Color::WHITE, 0.6)
                .label("10–15 m/s"),
        )
        .add(
            PolarBarMark::new(thetas, bin_15_plus)
                .r_base(bin_15_plus_base)
                .color(Color::from_hex(0x00E1_5759))
                .stroke(Color::WHITE, 0.6)
                .label("15+ m/s"),
        )
        .save("examples/scientific/wind_rose.png")
}
