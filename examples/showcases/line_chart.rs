//! Line chart — starsight 0.2.0 showcase
//!
//! Single LineMark with a title and axis labels, the canonical "hello world"
//! plot. Synthetic monthly revenue series with seasonal variation.

use starsight::prelude::*;

fn main() -> Result<()> {
    let months: Vec<f64> = (1..=24).map(f64::from).collect();
    let revenue: Vec<f64> = months
        .iter()
        .map(|&m| 100.0 + 8.0 * m + 25.0 * (m * 0.5).sin())
        .collect();

    Figure::new(1000, 600)
        .title("Monthly Revenue")
        .x_label("Month")
        .y_label("Revenue (k USD)")
        .add(LineMark::new(months, revenue).color(Color::from_hex(0x2E7CB8)).width(2.5))
        .save("examples/showcases/line_chart.png")
}
