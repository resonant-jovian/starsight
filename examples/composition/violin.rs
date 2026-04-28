//! Violin (kernel-density) plot — starsight 0.3.0 showcase.
//!
//! Compares response distributions across four quarters. Each violin shows
//! the kernel density estimate of its samples; the inner mini-box-plot
//! carries the quartile summary so the reader gets both shape (the
//! density envelope) and centre/spread (the box body) at one glance. The
//! palette runs from cool to warm so quarterly progression is readable
//! without leaning on the axis labels.

use starsight::prelude::*;
use starsight::statistics::Bandwidth;

fn main() -> Result<()> {
    // Slightly skewed samples per quarter — enough variance to make the
    // density estimate worth looking at.
    let q1 = vec![
        42.0, 44.0, 45.0, 46.0, 47.0, 48.0, 48.5, 49.0, 49.5, 50.0, 51.0, 52.0, 53.0, 54.0,
    ];
    let q2 = vec![
        46.0, 48.0, 49.0, 50.0, 51.0, 52.0, 52.5, 53.0, 53.5, 54.0, 55.0, 56.0, 57.0, 58.0,
    ];
    let q3 = vec![
        50.0, 52.0, 53.0, 54.0, 55.0, 56.0, 56.5, 57.0, 57.5, 58.0, 59.0, 60.0, 61.0, 62.0,
    ];
    let q4 = vec![
        54.0, 56.0, 57.0, 58.0, 59.0, 60.0, 60.5, 61.0, 61.5, 62.0, 63.0, 64.0, 65.0, 66.0,
    ];

    let groups = vec![
        ViolinGroup::new("Q1", q1),
        ViolinGroup::new("Q2", q2),
        ViolinGroup::new("Q3", q3),
        ViolinGroup::new("Q4", q4),
    ];

    let palette = vec![
        Color::from_hex(0x0033_77BB),
        Color::from_hex(0x0033_AA66),
        Color::from_hex(0x00CC_8800),
        Color::from_hex(0x00CC_3366),
    ];

    Figure::new(800, 500)
        .title("Quarterly throughput density")
        .x_label("quarter")
        .y_label("requests / s")
        .add(
            ViolinMark::new(groups)
                .bandwidth(Bandwidth::Silverman)
                .palette(palette),
        )
        .save("examples/composition/violin.png")
}
