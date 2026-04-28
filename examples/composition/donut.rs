//! Donut chart — starsight 0.3.0 showcase.
//!
//! Three-option vote distribution as a donut (`inner_radius` = 0.5) with raw
//! vote counts at each slice midpoint. The hollow center keeps the chart
//! reading as a "share of total" rather than a literal pie.

use starsight::prelude::*;

fn main() -> Result<()> {
    Figure::new(600, 600)
        .title("Vote distribution")
        .x_label("")
        .y_label("")
        .add(
            PieMark::new(
                vec![1240.0, 980.0, 540.0],
                vec!["Yes".into(), "No".into(), "Abstain".into()],
            )
            .inner_radius(0.5)
            .show_values(),
        )
        .save("examples/composition/donut.png")
}
