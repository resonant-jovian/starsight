//! Recipe — starsight 0.2.0 showcase
//!
//! A polished, README-grade plot: three series, custom palette, applied
//! light theme, full chrome (title, axis labels, legend). Meant as the
//! reference for "what a good starsight chart looks like."

use starsight::prelude::*;

fn series(name: &str) -> (Vec<f64>, Vec<f64>) {
    let years: Vec<f64> = (2014..=2024).map(f64::from).collect();
    let mut state: u32 = name.bytes().fold(0x9E37_79B9u32, |s, b| {
        s.wrapping_mul(1_664_525).wrapping_add(u32::from(b))
    });
    let mut next = || {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        f64::from(state) / f64::from(u32::MAX) - 0.5
    };
    let baseline = match name {
        "North" => 38.0,
        "South" => 22.0,
        _ => 30.0,
    };
    let trend = match name {
        "North" => 1.6,
        "South" => 2.4,
        _ => 0.6,
    };
    let values: Vec<f64> = years
        .iter()
        .enumerate()
        .map(|(i, _)| baseline + trend * (i as f64) + 3.0 * next())
        .collect();
    (years, values)
}

fn main() -> Result<()> {
    let (yx, yn) = series("North");
    let (_, ys) = series("South");
    let (_, yc) = series("Central");

    Figure::new(1200, 700)
        .theme(DEFAULT_LIGHT)
        .title("Regional Sales Growth — 2014 to 2024")
        .x_label("Year")
        .y_label("Revenue (M USD)")
        .add(
            LineMark::new(yx.clone(), yn)
                .color(Color::from_hex(0x1F_77B4))
                .width(2.5)
                .label("North"),
        )
        .add(
            LineMark::new(yx.clone(), ys)
                .color(Color::from_hex(0xD6_2728))
                .width(2.5)
                .label("South"),
        )
        .add(
            LineMark::new(yx, yc)
                .color(Color::from_hex(0x2C_A02C))
                .width(2.5)
                .label("Central"),
        )
        .save("examples/composition/recipe.png")
}
