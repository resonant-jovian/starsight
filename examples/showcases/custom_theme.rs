//! Custom theme — starsight 0.2.0 showcase
//!
//! Same data as `line_chart`, rendered against `DEFAULT_DARK` plus an
//! explicit accent override to demonstrate `Figure::theme(...)`.

use starsight::prelude::*;

fn main() -> Result<()> {
    let months: Vec<f64> = (1..=24).map(f64::from).collect();
    let revenue: Vec<f64> = months
        .iter()
        .map(|&m| 100.0 + 8.0 * m + 25.0 * (m * 0.5).sin())
        .collect();

    let theme = Theme {
        accent: Color::from_hex(0xFFD166),
        ..DEFAULT_DARK
    };

    Figure::new(1000, 600)
        .theme(theme)
        .title("Monthly Revenue (Dark Theme)")
        .x_label("Month")
        .y_label("Revenue (k USD)")
        .add(LineMark::new(months, revenue).color(theme.accent).width(2.5))
        .save("examples/showcases/custom_theme.png")
}
