//! Violin (kernel-density) plot — starsight 0.3.0 showcase.
//!
//! Four cohorts with *visually distinct* density shapes so the violin
//! envelopes actually carry information:
//!
//! - **Q1** — unimodal, near-Gaussian
//! - **Q2** — bimodal (two delivery channels with different conversion
//!   ceilings)
//! - **Q3** — right-skewed long tail (a viral campaign pulled the upper
//!   end)
//! - **Q4** — narrow, peaked distribution (process matured, variance
//!   collapsed)
//!
//! The inner mini box-plot carries the quartile summary so reader gets
//! both shape (the density envelope) and centre/spread (the box body) at
//! one glance. Palette is the same blue→pink ramp as the box-plot
//! example so the two read as a matched pair.

use starsight::prelude::*;
use starsight::statistics::Bandwidth;

fn main() -> Result<()> {
    let groups = vec![
        ViolinGroup::new("Q1", unimodal()),
        ViolinGroup::new("Q2", bimodal()),
        ViolinGroup::new("Q3", right_skewed()),
        ViolinGroup::new("Q4", narrow_peaked()),
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

/// Near-Gaussian unimodal samples around 50.
fn unimodal() -> Vec<f64> {
    vec![
        42.0, 44.0, 46.0, 47.0, 48.0, 48.5, 49.0, 49.5, 50.0, 50.0, 50.5, 51.0, 51.5, 52.0, 53.0,
        54.0, 56.0, 58.0,
    ]
}

/// Two clusters around 46 and 60 — a bimodal distribution that a single
/// box plot would summarise as one wide IQR but the violin shows clearly.
fn bimodal() -> Vec<f64> {
    vec![
        // mode 1
        43.0, 44.5, 45.0, 45.5, 46.0, 46.0, 46.5, 47.0, 48.0, 49.0, // gap
        51.0, 53.0, // mode 2
        57.0, 58.5, 59.0, 59.5, 60.0, 60.0, 60.5, 61.0, 62.0, 63.0,
    ]
}

/// Right-skewed (lower mode + long upper tail) — a campaign-effect shape.
fn right_skewed() -> Vec<f64> {
    vec![
        50.0, 51.0, 52.0, 52.5, 53.0, 53.5, 54.0, 54.5, 55.0, 55.5, 56.0, 56.5, 57.0, 58.0, 59.0,
        60.5, 62.0, 64.0, 67.0, 71.0,
    ]
}

/// Narrow, sharply-peaked distribution — the variance collapsed.
fn narrow_peaked() -> Vec<f64> {
    vec![
        58.5, 59.0, 59.5, 59.5, 60.0, 60.0, 60.0, 60.5, 60.5, 61.0, 61.0, 61.5, 62.0,
    ]
}
