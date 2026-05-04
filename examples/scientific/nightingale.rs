//! Nightingale coxcomb (rose diagram) — starsight 0.3.0 showcase #34.
//!
//! Three-stack monthly mortality decomposition rendered with Florence
//! Nightingale's value-as-area invariant: the radial axis is sqrt-scaled, so
//! the visible area of each wedge is proportional to the value, not the
//! radius. Twelve months × three death-cause stacks (preventable disease,
//! battle wounds, other causes) form the canonical "rose."
//!
//! The synthetic dataset roughly mirrors the proportions Nightingale
//! published for the Crimean War; absolute scales are made-up but the
//! "preventable disease dwarfs combat" pattern is preserved so the chart
//! reads honestly.

use starsight::axes::Axis;
use starsight::prelude::*;

fn main() -> Result<()> {
    let thetas: Vec<f64> = (0..12u32).map(f64::from).collect();
    // Roughly Nightingale's first year: high preventable-disease deaths
    // through the winter months 0..6, dropping after the sanitary commission
    // arrives.
    let preventable = vec![
        450.0, 580.0, 720.0, 880.0, 1040.0, 1200.0, 950.0, 620.0, 420.0, 310.0, 240.0, 180.0,
    ];
    let wounds = vec![
        80.0, 90.0, 110.0, 130.0, 145.0, 160.0, 140.0, 110.0, 95.0, 85.0, 80.0, 75.0,
    ];
    let other = vec![
        50.0, 55.0, 60.0, 70.0, 80.0, 85.0, 75.0, 65.0, 55.0, 50.0, 45.0, 40.0,
    ];

    // Stacked: each month renders three wedges at the same theta with
    // increasing inner radius. ArcMark stacks them by setting r_inner to the
    // previous stack's outer.
    let theta_axis = Axis::polar_angular_categorical(12);
    let r_axis = Axis::polar_radial_sqrt(0.0, 1500.0);
    let r_inner_layer1: Vec<f64> = vec![0.0; 12];
    let r_outer_layer1: Vec<f64> = preventable.clone();
    let r_inner_layer2: Vec<f64> = preventable.clone();
    let r_outer_layer2: Vec<f64> = preventable
        .iter()
        .zip(&wounds)
        .map(|(p, w)| p + w)
        .collect();
    let r_inner_layer3: Vec<f64> = r_outer_layer2.clone();
    let r_outer_layer3: Vec<f64> = r_outer_layer2
        .iter()
        .zip(&other)
        .map(|(s, o)| s + o)
        .collect();

    Figure::new(900, 900)
        .title("Mortality by month — Nightingale's rose")
        .polar_axes(theta_axis, r_axis)
        .add(
            ArcMark::new(thetas.clone(), r_outer_layer1)
                .r_inner(r_inner_layer1)
                .theta_half_width(0.5)
                .colors(vec![Color::from_hex(0x00C2_5757); 12])
                .stroke(Color::WHITE, 0.8)
                .label("preventable disease"),
        )
        .add(
            ArcMark::new(thetas.clone(), r_outer_layer2)
                .r_inner(r_inner_layer2)
                .theta_half_width(0.5)
                .colors(vec![Color::from_hex(0x002E_4D77); 12])
                .stroke(Color::WHITE, 0.8)
                .label("wounds in battle"),
        )
        .add(
            ArcMark::new(thetas, r_outer_layer3)
                .r_inner(r_inner_layer3)
                .theta_half_width(0.5)
                .colors(vec![Color::from_hex(0x008A_A38E); 12])
                .stroke(Color::WHITE, 0.8)
                .label("other causes"),
        )
        .save("examples/scientific/nightingale.png")
}
