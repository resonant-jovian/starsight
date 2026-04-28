//! Box-and-whisker plot — starsight 0.3.0 showcase.
//!
//! Compares response distributions across four treatment cohorts. Each box
//! shows the interquartile range; the white line marks the median; whiskers
//! extend to the most extreme non-outlier samples; black dots flag points
//! beyond 1.5×IQR. The custom palette runs the four bands through a
//! perceptually-graded blue-to-red ramp so the cohort ordering reads at a
//! glance even before the axis labels are processed.

use starsight::prelude::*;

fn main() -> Result<()> {
    // Synthetic but plausible response measurements: each cohort gets a
    // tight unimodal sample plus a couple of long-tail outliers so the
    // boxplot exercises every visual element it produces.
    let groups = vec![
        BoxPlotGroup::new(
            "placebo",
            vec![
                42.0, 44.0, 45.0, 46.0, 47.0, 47.5, 48.0, 48.5, 49.0, 50.0, 51.0, 52.0, 80.0,
            ],
        ),
        BoxPlotGroup::new(
            "low dose",
            vec![
                48.0, 50.0, 51.0, 52.0, 53.0, 53.5, 54.0, 54.5, 55.0, 56.0, 57.0, 58.0, 22.0,
            ],
        ),
        BoxPlotGroup::new(
            "med dose",
            vec![
                55.0, 57.0, 58.0, 59.0, 60.0, 60.5, 61.0, 61.5, 62.0, 63.0, 64.0, 65.0,
            ],
        ),
        BoxPlotGroup::new(
            "high dose",
            vec![
                62.0, 64.0, 65.0, 66.0, 67.0, 67.5, 68.0, 68.5, 69.0, 70.0, 71.0, 72.0, 95.0, 30.0,
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
