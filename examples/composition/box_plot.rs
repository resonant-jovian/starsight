//! Box-and-whisker plot — starsight 0.3.0 showcase.
//!
//! Compares response distributions across four cohorts in a synthetic
//! dose-response trial. Each cohort has *genuinely different* characteristics
//! so the box plot's distinguishing features (whisker reach, IQR width,
//! outliers in different directions) actually show up:
//!
//! - **placebo** — tight unimodal distribution, no outliers
//! - **low dose** — wider IQR, one high outlier (a non-responder
//!   over-corrected upward)
//! - **med dose** — moderate spread, two low outliers (under-responders)
//! - **high dose** — narrowest IQR (drug worked), one extreme high outlier
//!   (idiosyncratic reaction)
//!
//! The custom palette runs the cohort sequence through a perceptually
//! uniform blue → green → orange → pink ramp.

use starsight::prelude::*;

fn main() -> Result<()> {
    // Deterministic data; values chosen so each cohort's box-and-whisker
    // exercises a different visual element (whisker length, outlier side,
    // box width). No RNG so the snapshot stays byte-stable.
    let groups = vec![
        BoxPlotGroup::new(
            "placebo",
            // Tight, near-symmetric: IQR ≈ 4, no Tukey outliers.
            vec![
                44.0, 45.5, 46.5, 47.0, 47.5, 48.0, 48.0, 48.5, 49.0, 49.5, 50.0, 50.5, 51.0, 52.0,
                53.5,
            ],
        ),
        BoxPlotGroup::new(
            "low dose",
            // Wider IQR + one high outlier. The 78.0 sits well past
            // q3 + 1.5·IQR.
            vec![
                48.0, 49.5, 51.0, 52.0, 53.0, 53.5, 54.0, 54.5, 55.5, 56.5, 58.0, 59.5, 61.0, 78.0,
            ],
        ),
        BoxPlotGroup::new(
            "med dose",
            // Moderate spread + two low outliers (under-responders) and a
            // slightly negative-skewed body.
            vec![
                28.0, 32.0, 55.0, 56.5, 58.0, 59.0, 60.0, 60.5, 61.0, 62.0, 63.0, 64.5, 66.0,
            ],
        ),
        BoxPlotGroup::new(
            "high dose",
            // Narrowest IQR (drug worked uniformly) plus one extreme high
            // outlier representing an idiosyncratic over-response.
            vec![
                65.0, 66.0, 66.5, 67.0, 67.0, 67.5, 68.0, 68.0, 68.5, 69.0, 69.5, 70.0, 92.0,
            ],
        ),
    ];

    let palette = vec![
        Color::from_hex(0x0033_77BB),
        Color::from_hex(0x0033_AA66),
        Color::from_hex(0x00CC_8800),
        Color::from_hex(0x00CC_3366),
    ];

    Figure::new(800, 500)
        .title("Dose-response distribution")
        .x_label("cohort")
        .y_label("response (% baseline)")
        .add(BoxPlotMark::new(groups).palette(palette))
        .save("examples/composition/box_plot.png")
}
