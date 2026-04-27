//! Waterfall chart — starsight 0.2.0 showcase
//!
//! Waterfall charts show how a starting value changes through a series of
//! positive and negative adjustments to reach a final value.
//! Each bar "floats" at the running total, with the bar height showing the change.
//! Color indicates increase (green) or decrease (red), with subtotals and totals in blue.

use starsight::prelude::*;

// ── Waterfall data ────────────────────────────────────────────────────────────────────

struct WaterfallData {
    labels: Vec<String>,
    values: Vec<f64>,
    bases: Vec<f64>,
    is_subtotal: Vec<bool>,
    is_total: Vec<bool>,
}

fn build_waterfall_data() -> WaterfallData {
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

    WaterfallData {
        labels: data.iter().map(|r| r.0.to_string()).collect(),
        values: data.iter().map(|r| r.1).collect(),
        bases: data.iter().map(|r| r.2).collect(),
        is_subtotal: data.iter().map(|r| r.3).collect(),
        is_total: data.iter().map(|r| r.4).collect(),
    }
}

// ── main ────────────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let data = build_waterfall_data();

    let green = Color::new(34, 139, 34); // forest green for increases
    let red = Color::new(220, 20, 60); // crimson for decreases
    let blue = Color::new(65, 105, 225); // royal blue for subtotals/totals

    let mut fig = Figure::new(1200, 700);
    fig = fig
        .title("Waterfall Chart — P&L Walk")
        .x_label("")
        .y_label("Amount ($)");

    // Add each bar - we need separate marks because base varies per bar
    for i in 0..data.labels.len() {
        let label = &data.labels[i];
        let value = data.values[i];
        let base = data.bases[i];
        let subtotal = data.is_subtotal[i];
        let total = data.is_total[i];

        let color = if subtotal || total {
            blue
        } else if value >= 0.0 {
            green
        } else {
            red
        };

        fig = fig.add(BarMark {
            x: vec![label.clone()],
            y: vec![value],
            color: Some(color),
            width: Some(0.6),
            orientation: Orientation::Vertical,
            group: None,
            stack: None,
            base: Some(base),
            label: None,
        });
    }

    fig.save("examples/composition/waterfall_bar.png")?;
    println!("saved waterfall_bar.png");
    Ok(())
}
