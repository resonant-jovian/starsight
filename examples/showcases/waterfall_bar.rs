//! Waterfall chart — starsight 0.2.0 showcase
//!
//! Waterfall charts show how a starting value changes through a series of
//! positive and negative adjustments to reach a final value.
//! Each bar "floats" at the running total, with the bar height showing the change.
//! Color indicates increase (green) or decrease (red), with subtotals and totals in blue.

use starsight::prelude::*;
use starsight_layer_3::marks::{BarMark, Orientation};

// ── Waterfall data ────────────────────────────────────────────────────────────────────

fn build_waterfall_data() -> (Vec<String>, Vec<f64>, Vec<f64>, Vec<bool>, Vec<bool>) {
    let data = [
        ("Revenue", 4_200_000.0, 0.0, false, false),
        ("COGS", -1_800_000.0, 4_200_000.0, false, false),
        ("Gross Profit", 2_400_000.0, 0.0, true, false), // subtotal
        ("OpEx", -900_000.0, 2_400_000.0, false, false),
        ("R&D", -500_000.0, 1_500_000.0, false, false),
        ("Marketing", -300_000.0, 1_000_000.0, false, false),
        ("EBITDA", 700_000.0, 0.0, true, false), // subtotal
        ("D&A", -150_000.0, 700_000.0, false, false),
        ("Interest", -50_000.0, 550_000.0, false, false),
        ("Net Income", 500_000.0, 0.0, false, true), // total
    ];

    let labels: Vec<String> = data.iter().map(|r| r.0.to_string()).collect();
    let values: Vec<f64> = data.iter().map(|r| r.1).collect();
    let bases: Vec<f64> = data.iter().map(|r| r.2).collect();
    let is_subtotal: Vec<bool> = data.iter().map(|r| r.3).collect();
    let is_total: Vec<bool> = data.iter().map(|r| r.4).collect();

    (labels, values, bases, is_subtotal, is_total)
}

// ── main ────────────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let (labels, values, bases, is_subtotal, is_total) = build_waterfall_data();

    let green = Color::new(34, 139, 34); // forest green for increases
    let red = Color::new(220, 20, 60); // crimson for decreases
    let blue = Color::new(65, 105, 225); // royal blue for subtotals/totals

    let mut fig = Figure::new(1200, 700);
    fig = fig
        .title("Waterfall Chart — P&L Walk")
        .x_label("")
        .y_label("Amount ($)");

    // Add each bar - we need separate marks because base varies per bar
    for i in 0..labels.len() {
        let label = &labels[i];
        let value = values[i];
        let base = bases[i];
        let subtotal = is_subtotal[i];
        let total = is_total[i];

        let color = if subtotal || total {
            blue
        } else if value >= 0.0 {
            green
        } else {
            red
        };

        fig = fig.add(BarMark {
            x: vec![label.to_string()],
            y: vec![value],
            color: Some(color),
            width: Some(0.6),
            orientation: Orientation::Vertical,
            group: None,
            stack: None,
            base: Some(base),
        });
    }

    fig.save("examples/showcases/waterfall_bar.png")?;
    println!("saved waterfall_bar.png");
    Ok(())
}
