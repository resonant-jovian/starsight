//! Waterfall chart — starsight 0.3.0 showcase
//!
//! Waterfall charts show how a starting value changes through a series of
//! positive and negative adjustments to reach a final value. Each bar floats
//! at the running total, with bar height showing the change. Color encodes
//! direction (green = increase, red = decrease, blue = subtotal/total). Thin
//! gray connector lines link consecutive bars at the running-total level.
//!
//! Implements spec example #37 from `.spec/SHOWCASE_INPUTS.md`. The 0.3.0
//! `BarMark` carries per-bar `bases` and `colors` plus the `connectors` flag,
//! so the whole chart is a single mark.

use starsight::prelude::*;

fn main() -> Result<()> {
    let labels: Vec<String> = [
        "Revenue",
        "COGS",
        "Gross Profit",
        "OpEx",
        "R&D",
        "Marketing",
        "EBITDA",
        "D&A",
        "Interest",
        "Net Income",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();

    let values = vec![
        4_200_000.0,
        -1_800_000.0,
        2_400_000.0,
        -900_000.0,
        -500_000.0,
        -300_000.0,
        700_000.0,
        -150_000.0,
        -50_000.0,
        500_000.0,
    ];

    // bases[i] is the running total *before* row i is applied, so each bar's
    // top edge lands at bases[i] + values[i] — the running total after row i.
    // Subtotal/total rows reset to base 0 because they show the cumulative figure
    // as a height from zero, not as an adjustment.
    let bases = vec![
        0.0,
        4_200_000.0,
        0.0,
        2_400_000.0,
        1_500_000.0,
        1_000_000.0,
        0.0,
        700_000.0,
        550_000.0,
        0.0,
    ];

    let kind = [
        "inc", "dec", "sub", "dec", "dec", "dec", "sub", "dec", "dec", "tot",
    ];

    // Spec colors (.spec/SHOWCASE_INPUTS.md:175).
    let green = Color::from_hex(0x2E_7D32);
    let red = Color::from_hex(0xC6_2828);
    let blue = Color::from_hex(0x15_65C0);
    let colors: Vec<Color> = kind
        .iter()
        .map(|k| match *k {
            "inc" => green,
            "dec" => red,
            _ => blue,
        })
        .collect();

    Figure::new(1200, 700)
        .title("Waterfall Chart — P&L Walk")
        .y_label("Amount ($)")
        .add(
            BarMark::new(labels, values)
                .bases(bases)
                .colors(colors)
                .width(0.6)
                .connectors(true),
        )
        .save("examples/composition/waterfall_bar.png")?;

    println!("saved waterfall_bar.png");
    Ok(())
}
