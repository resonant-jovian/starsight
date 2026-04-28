//! Polars integration — starsight 0.3.0 showcase.
//!
//! Builds a synthetic measurement `DataFrame` and feeds it to the `plot!`
//! macro's `DataFrame` arm. The macro inspects column types: numeric x →
//! `LineMark` for the no-grouping case, but with `color = "group"` the
//! helper switches to one `PointMark` per unique group value with a cycled
//! palette and per-group legend labels.
//!
//! Build and run with the `polars` feature enabled:
//!
//! ```sh
//! cargo run --release --example polars_integration --features polars
//! ```

use polars::prelude::*;
use starsight::prelude::*;
use starsight::sources::plot_dataframe;

fn main() -> Result<()> {
    // Three-group cluster scatter: each group has a different mean and spread.
    let df = df!(
        "x" => &[
            // group A
            0.5_f64, 1.0, 0.8, 1.2, 1.4, 1.0, 0.7, 1.3,
            // group B
            3.0, 3.4, 2.8, 3.6, 3.2, 3.0, 2.9, 3.5,
            // group C
            5.5, 6.0, 5.7, 6.3, 5.9, 6.1, 5.6, 6.2,
        ],
        "y" => &[
            1.5_f64, 1.8, 1.6, 2.0, 1.9, 1.7, 1.4, 1.8,
            3.5, 3.9, 3.6, 4.1, 3.8, 3.7, 3.4, 4.0,
            6.0, 6.4, 6.1, 6.6, 6.3, 6.2, 5.9, 6.5,
        ],
        "group" => &[
            "A", "A", "A", "A", "A", "A", "A", "A",
            "B", "B", "B", "B", "B", "B", "B", "B",
            "C", "C", "C", "C", "C", "C", "C", "C",
        ],
    )
    .map_err(|e| StarsightError::Data(format!("build dataframe: {e}")))?;

    plot_dataframe(&df, "x", "y", Some("group"))
        .title("Cluster scatter from a Polars DataFrame")
        .x_label("feature 1")
        .y_label("feature 2")
        .save("examples/data/polars_integration.png")
}
