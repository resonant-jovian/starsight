//! Gauge / radial-progress meter — starsight 0.3.0 showcase #41.
//!
//! Single-value gauge sweeping a 270° arc (start at the bottom-left,
//! end at the bottom-right). The displayed value (78/100) fills its
//! proportional share of the sweep; the rest is rendered as a muted
//! "background" arc.

use starsight::axes::Axis;
use starsight::prelude::*;

fn main() -> Result<()> {
    let value: f64 = 78.0;
    let max: f64 = 100.0;
    let total_sweep = 1.5 * std::f64::consts::PI; // 270°
    let half_total = total_sweep / 2.0;
    let value_sweep = (value / max) * total_sweep;
    let half_value = value_sweep / 2.0;
    // Center the value-arc on the start of the gauge (i.e. its left edge
    // sits at the bottom-left of the dial). The full gauge is centered too,
    // so its midpoint lines up with the top of the disk.
    let value_center = -half_total + half_value;
    let bg_center: f64 = 0.0;

    let theta_axis = Axis::polar_angular(0.0, std::f64::consts::TAU);
    let r_axis = Axis::polar_radial(0.0, 1.0);

    Figure::new(800, 600)
        .title(format!("Battery — {value:.0}%"))
        .polar_axes(theta_axis, r_axis)
        // Background arc (the unfilled portion).
        .add(
            ArcMark::new(vec![bg_center], vec![1.0])
                .theta_half_widths(vec![half_total])
                .r_inner(vec![0.7])
                .colors(vec![Color::from_hex(0x00DD_DDDD)]),
        )
        // Foreground arc (the filled portion).
        .add(
            ArcMark::new(vec![value_center], vec![1.0])
                .theta_half_widths(vec![half_value])
                .r_inner(vec![0.7])
                .colors(vec![Color::from_hex(0x004C_AF50)]),
        )
        .save("examples/scientific/gauge.png")
}
