//! Sunburst — starsight 0.3.0 showcase #39 variant C.
//!
//! Three-level nested wedge layout: one inner ring, four middle wedges, eight
//! outer wedges. Each ring stacks via increasing `r_inner` so the rings sit
//! concentrically without overlap. The pattern lifts directly from a typical
//! filesystem-usage / decision-tree drilldown.

use starsight::axes::Axis;
use starsight::prelude::*;

fn main() -> Result<()> {
    let theta_axis = Axis::polar_angular(0.0, std::f64::consts::TAU);
    let r_axis = Axis::polar_radial(0.0, 1.0);

    Figure::new(800, 800)
        .title("Sunburst — three-level drilldown")
        .polar_axes(theta_axis, r_axis)
        // Innermost ring: a single full-disk wedge as the root.
        .add(
            ArcMark::new(vec![0.0], vec![0.30])
                .theta_half_widths(vec![std::f64::consts::PI])
                .colors(vec![Color::from_hex(0x002E_4D77)])
                .wedge_labels(vec!["root".into()]),
        )
        // Middle ring: 4 quadrants.
        .add(
            ArcMark::new(
                vec![
                    std::f64::consts::PI * 0.25,
                    std::f64::consts::PI * 0.75,
                    std::f64::consts::PI * 1.25,
                    std::f64::consts::PI * 1.75,
                ],
                vec![0.60; 4],
            )
            .r_inner(vec![0.30; 4])
            .theta_half_widths(vec![std::f64::consts::PI * 0.25; 4])
            .colors(vec![
                Color::from_hex(0x00C2_5757),
                Color::from_hex(0x008A_A38E),
                Color::from_hex(0x00E8_C547),
                Color::from_hex(0x00B0_6AB3),
            ])
            .stroke(Color::WHITE, 1.0)
            .wedge_labels(vec![
                "frontend".into(),
                "backend".into(),
                "infra".into(),
                "data".into(),
            ]),
        )
        // Outer ring: 8 octants.
        .add(
            ArcMark::new(
                (0..8u32)
                    .map(|i| {
                        std::f64::consts::PI * 0.125 + std::f64::consts::PI * 0.25 * f64::from(i)
                    })
                    .collect(),
                vec![0.92; 8],
            )
            .r_inner(vec![0.60; 8])
            .theta_half_widths(vec![std::f64::consts::PI * 0.125; 8])
            .colors(vec![
                Color::from_hex(0x00DD_8888),
                Color::from_hex(0x00BC_5050),
                Color::from_hex(0x00A2_C0AC),
                Color::from_hex(0x0072_8E7C),
                Color::from_hex(0x00F1_D470),
                Color::from_hex(0x00CD_AA3F),
                Color::from_hex(0x00C5_8FC9),
                Color::from_hex(0x008F_5894),
            ])
            .stroke(Color::WHITE, 1.0)
            .wedge_labels(vec![
                "react".into(),
                "css".into(),
                "rust".into(),
                "db".into(),
                "k8s".into(),
                "ci".into(),
                "etl".into(),
                "ml".into(),
            ]),
        )
        .save("examples/composition/donut_sunburst.png")
}
