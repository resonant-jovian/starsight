//! Energy transition donut — starsight 0.3.0 showcase #39 variant B.
//!
//! Two concentric donut rings comparing the global energy mix between 2020
//! (inner ring) and 2025 (outer ring). Each color keys a fixed source across
//! both years, so the per-source share shift between rings reads at a
//! glance — solar and wind grow, gas and coal contract, hydro and nuclear
//! roughly hold.
//!
//! Inner ring sits at `r_inner = 0.40 .. r_outer = 0.65`; outer ring at
//! `r_inner = 0.75 .. r_outer = 1.00`. The 0.10-unit radial gap visually
//! separates the two years without crowding either.
//!
//! Pairs with `donut.rs` (single donut, Variant A) and `donut_sunburst.rs`
//! (three-level sunburst, Variant C) to cover the spec's three concentric-
//! arc patterns.

use starsight::axes::Axis;
use starsight::prelude::*;

fn main() -> Result<()> {
    // Six energy sources, same order in both rings so the color → source
    // mapping is consistent year-over-year.
    let labels = ["Solar", "Wind", "Hydro", "Nuclear", "Gas", "Coal"];
    let colors = [
        Color::from_hex(0x00F1_C40F), // Solar    — yellow
        Color::from_hex(0x003A_A39C), // Wind     — teal
        Color::from_hex(0x002E_86AB), // Hydro    — blue
        Color::from_hex(0x008E_44AD), // Nuclear  — purple
        Color::from_hex(0x00E6_7E22), // Gas      — orange
        Color::from_hex(0x004A_4A4A), // Coal     — neutral dark
    ];

    let outer_pct = [22.0, 19.0, 16.0, 18.0, 15.0, 10.0]; // 2025
    let inner_pct = [10.0, 12.0, 18.0, 20.0, 22.0, 18.0]; // 2020

    let theta_axis = Axis::polar_angular(0.0, std::f64::consts::TAU);
    let r_axis = Axis::polar_radial(0.0, 1.0);

    let outer = ring_arc(&outer_pct, &colors, 1.00, 0.75, &labels);
    let inner = ring_arc(&inner_pct, &colors, 0.65, 0.40, &labels);

    Figure::new(800, 800)
        .theme(theme_from_env())
        .title("Energy transition — 2020 (inner) vs 2025 (outer)")
        .polar_axes(theta_axis, r_axis)
        .add(inner)
        .add(outer)
        .save(format!(
            "examples/composition/energy_transition{}.{}",
            theme_suffix_from_env(),
            format_extension_from_env()
        ))
}

/// Build one donut ring: each wedge's angular span is proportional to its
/// percentage, the ring sits between `r_inner` and `r_outer`, and the
/// per-wedge color cycle keys label[i] → palette[i].
fn ring_arc(
    pct: &[f64],
    palette: &[Color],
    r_outer: f64,
    r_inner: f64,
    labels: &[&str],
) -> ArcMark {
    let total: f64 = pct.iter().sum();
    let mut thetas = Vec::with_capacity(pct.len());
    let mut half_widths = Vec::with_capacity(pct.len());
    let mut cum = 0.0;
    for &p in pct {
        let frac = p / total;
        let center = (cum + p * 0.5) / total * std::f64::consts::TAU;
        let half = frac * std::f64::consts::PI;
        thetas.push(center);
        half_widths.push(half);
        cum += p;
    }
    ArcMark::new(thetas, vec![r_outer; pct.len()])
        .r_inner(vec![r_inner; pct.len()])
        .theta_half_widths(half_widths)
        .colors(palette[..pct.len()].to_vec())
        .stroke(Color::WHITE, 1.0)
        .wedge_labels(labels.iter().map(|s| (*s).to_string()).collect())
}
