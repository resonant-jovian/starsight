//! Custom theme — starsight 0.2.0 showcase
//!
//! Same data as `line_chart`, rendered against a real chromata editor theme.
//! `chromata` ships 1,104 themes as compile-time constants and starsight has
//! a `From<chromata::Theme> for Theme` impl, so any of them drops straight
//! into `Figure::theme(...)` via `.into()`. Swap `gruvbox::DARK_HARD` for
//! `dracula::DEFAULT`, `nord::DEFAULT`, `tokyo_night::STORM`, etc. and the
//! whole figure recolors — accent, grid, axis, tick labels and all.

use chromata::popular::gruvbox;
use starsight::prelude::*;

fn main() -> Result<()> {
    let months: Vec<f64> = (1..=24).map(f64::from).collect();
    let revenue: Vec<f64> = months
        .iter()
        .map(|&m| 100.0 + 8.0 * m + 25.0 * (m * 0.5).sin())
        .collect();

    let theme: Theme = gruvbox::DARK_HARD.into();

    Figure::new(1000, 600)
        .theme(theme)
        .title("Monthly Revenue — gruvbox dark hard (via chromata)")
        .x_label("Month")
        .y_label("Revenue (k USD)")
        .add(LineMark::new(months, revenue).color(theme.accent).width(2.5))
        .save("examples/theming/custom_theme.png")
}
