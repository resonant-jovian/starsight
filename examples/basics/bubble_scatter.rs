//! Bubble scatter — starsight 0.3.0 showcase
//!
//! Per-point continuous color and per-point radius, applied to wine-shaped
//! synthetic data. Color encodes color-intensity through `ColorBrewer`'s `RdPu`
//! sequential colormap; radius encodes proline (size ∝ √proline). Implements
//! spec example #3 from `.spec/SHOWCASE_INPUTS.md`.
//!
//! Full `ColorScale` infrastructure lands at 0.5.0 — at 0.3.0 we apply the
//! colormap manually and pass per-point colors via `PointMark::colors()`.

use starsight::prelude::*;

/// Deterministic LCG → normalized `[0, 1)` floats. Seed-stable across runs and
/// platforms so the generated PNG matches the snapshot pixel-for-pixel.
fn make_rng(seed: u32) -> impl FnMut() -> f64 {
    let mut state = seed;
    move || {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        f64::from(state) / f64::from(u32::MAX)
    }
}

/// One Box-Muller draw from `N(mean, stddev)`. Two `next` calls per sample.
fn gauss(next: &mut impl FnMut() -> f64, mean: f64, stddev: f64) -> f64 {
    let u1 = next().max(1e-9);
    let u2 = next();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    mean + stddev * z
}

fn main() -> Result<()> {
    let n = 178;
    let mut rng_alc = make_rng(1);
    let mut rng_ci = make_rng(2);
    let mut rng_pro = make_rng(3);

    // alcohol ~ Uniform(11.0, 15.0)
    let alcohol: Vec<f64> = (0..n).map(|_| 11.0 + 4.0 * rng_alc()).collect();

    // color_intensity = 0.8 · alcohol − 4.0 + N(0, 1.5)
    let color_intensity: Vec<f64> = alcohol
        .iter()
        .map(|&a| 0.8 * a - 4.0 + gauss(&mut rng_ci, 0.0, 1.5))
        .collect();

    // proline = 300 + 80 · alcohol + N(0, 200)
    let proline: Vec<f64> = alcohol
        .iter()
        .map(|&a| 300.0 + 80.0 * a + gauss(&mut rng_pro, 0.0, 200.0))
        .collect();

    // Map color_intensity → [0, 1] → RdPu colormap (ColorBrewer sequential).
    let cmin = color_intensity
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let cmax = color_intensity
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let cspan = (cmax - cmin).max(f64::EPSILON);
    let colors: Vec<Color> = color_intensity
        .iter()
        .map(|&c| {
            let t = ((c - cmin) / cspan).clamp(0.0, 1.0) as f32;
            prismatica::colorbrewer::RDPU.eval(t).into()
        })
        .collect();

    // Per-point radius: √proline · 0.5, clamped to a readable pixel range so
    // outliers don't dominate the canvas.
    let radii: Vec<f32> = proline
        .iter()
        .map(|&p| (p.max(0.0).sqrt() * 0.5).clamp(2.0, 14.0) as f32)
        .collect();

    Figure::new(900, 700)
        .title("Wine: alcohol × proline (size = √proline, color = intensity)")
        .x_label("Alcohol (% vol)")
        .y_label("Proline (mg/L)")
        .add(
            PointMark::new(alcohol, proline)
                .colors(colors)
                .radii(radii)
                .alpha(0.5)
                .label("Wine samples"),
        )
        .save("examples/basics/bubble_scatter.png")?;

    println!("saved bubble_scatter.png");
    Ok(())
}
