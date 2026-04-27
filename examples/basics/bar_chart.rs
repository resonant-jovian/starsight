//! Bar chart — starsight 0.2.0 showcase
//!
//! Grouped bars: 4 quarters × 2 product lines. Demonstrates the per-series
//! `.group(name)` API, default theme, and a legend that auto-derives from
//! the marks' labels.

use starsight::prelude::*;

fn main() -> Result<()> {
    let quarters: Vec<String> = ["Q1", "Q2", "Q3", "Q4"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let widgets = vec![32.0, 41.0, 38.0, 47.0];
    let gizmos = vec![22.0, 28.0, 35.0, 42.0];

    Figure::new(900, 600)
        .title("Quarterly Revenue by Product Line")
        .x_label("Quarter")
        .y_label("Revenue (M USD)")
        .add(
            BarMark::new(quarters.clone(), widgets)
                .color(Color::from_hex(0x4F8AB8))
                .label("Widgets")
                .group("widgets"),
        )
        .add(
            BarMark::new(quarters, gizmos)
                .color(Color::from_hex(0xE57B49))
                .label("Gizmos")
                .group("gizmos"),
        )
        .save("examples/basics/bar_chart.png")
}
