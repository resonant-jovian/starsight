//! Pie chart — starsight 0.3.0 showcase.
//!
//! Five-slice energy mix pie with percentage labels at each midpoint. Default
//! palette (six perceptually-distinct hues) renders the slices in the order
//! given, white slice borders separate adjacent wedges.

use starsight::prelude::*;

fn main() -> Result<()> {
    Figure::new(600, 600)
        .title("Energy mix")
        .x_label("")
        .y_label("")
        .add(
            PieMark::new(
                vec![32.0, 24.0, 18.0, 14.0, 12.0],
                vec![
                    "Solar".into(),
                    "Wind".into(),
                    "Hydro".into(),
                    "Nuclear".into(),
                    "Other".into(),
                ],
            )
            .show_percent(),
        )
        .save("examples/composition/pie.png")
}
