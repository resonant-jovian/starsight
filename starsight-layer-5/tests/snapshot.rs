//! Snapshot tests for layer-3 mark rendering.
//!
//! Each test uses realistic, deterministic data drawn from a recognizable
//! domain (physics, weather, statistics) so the resulting snapshots both
//! verify the pipeline and double as honest demo material.
//!
//! Note: snapshots are taken against the SVG backend, not PNG. SVG keeps text
//! as `<text>` elements (no glyph rasterization), so the output is byte-exact
//! reproducible across operating systems and font setups. The PNG raster path
//! is exercised by `starsight/tests/integration.rs` and the layer-1
//! `blue_rect_on_white` test, neither of which depends on font rendering.

use starsight_layer_1::primitives::Color;
use starsight_layer_3::marks::{
    AreaMark, BarMark, HistogramMark, LineMark, PointMark, StepMark, StepPosition,
};
use starsight_layer_5::Figure;

// ── helpers ──────────────────────────────────────────────────────────────────────────────────────

/// Damped cosine: `y = e^(-0.05x) · cos(0.5x)` over `0..n`.
///
/// Looks like a damped harmonic oscillator. Crosses zero, decays toward zero,
/// has both positive and negative values — exercises the Wilkinson tick
/// algorithm on a symmetric range. The `u32` parameter type lets us use the
/// lossless `f64::from` conversion instead of an `as` cast.
fn damped_cosine(n: u32) -> (Vec<f64>, Vec<f64>) {
    let x: Vec<f64> = (0..n).map(f64::from).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|&xi| (-xi * 0.05).exp() * (xi * 0.5).cos())
        .collect();
    (x, y)
}

// ── line tests ───────────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_line_basic() {
    // Damped harmonic oscillator — looks like a real physical measurement.
    // Exercises smooth-curve rendering across positive and negative y values.
    let (x, y) = damped_cosine(50);
    let fig = Figure::new(800, 600).add(LineMark::new(x, y));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_line_nan_gaps() {
    // 40-hour temperature reading from a sensor with two outage windows.
    // NaN gaps at hours 8–11 and 25–27 split the line into three sub-paths,
    // verifying that NaN handling produces visible breaks rather than artifacts.
    let x: Vec<f64> = (0u32..40).map(f64::from).collect();
    let mut y: Vec<f64> = vec![
        18.2, 17.8, 17.5, 17.1, 16.9, 17.4, 18.6, 20.1, // 0–7
        21.8, 23.4, 24.7, 25.3, // 8–11 (overwritten as NaN below)
        25.8, 26.1, 26.0, 25.6, // 12–15
        24.9, 23.8, 22.4, 21.0, // 16–19
        19.8, 18.9, 18.3, 17.9, // 20–23
        17.5, 17.2, 17.0, 17.1, // 24–27 (25–27 overwritten as NaN below)
        17.8, 18.9, 20.4, 22.1, // 28–31
        23.7, 24.9, 25.6, 25.9, // 32–35
        25.7, 25.0, 23.8, 22.3, // 36–39
    ];
    y[8..12].fill(f64::NAN);
    y[25..28].fill(f64::NAN);
    let fig = Figure::new(800, 600).add(LineMark::new(x, y));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_line_multi() {
    // Daily high and low temperatures over four weeks — two related series
    // that share an x axis and never cross. Tests color differentiation and
    // overlapping (but distinct) line rendering.
    let days: Vec<f64> = (1u32..=28).map(f64::from).collect();
    let highs: Vec<f64> = vec![
        12.5, 13.1, 14.0, 13.8, 14.5, 15.2, 16.1, 16.8, 17.5, 18.2, 17.9, 17.4, 18.0, 19.3, 20.5,
        21.2, 20.8, 19.6, 18.4, 17.2, 17.8, 18.5, 19.2, 20.0, 20.9, 21.4, 21.0, 20.3,
    ];
    let lows: Vec<f64> = vec![
        4.2, 4.8, 5.5, 5.1, 5.9, 6.4, 7.0, 7.3, 7.8, 8.2, 8.0, 7.6, 8.1, 9.0, 10.1, 10.5, 10.2,
        9.4, 8.7, 7.9, 8.3, 8.9, 9.5, 10.2, 10.8, 11.1, 10.7, 10.0,
    ];
    let fig = Figure::new(800, 600)
        .add(LineMark::new(days.clone(), highs).color(Color::RED))
        .add(LineMark::new(days, lows).color(Color::BLUE));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
}

// ── scatter tests ────────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_scatter_basic() {
    // Anscombe's quartet, set I — the classic 11-point dataset from
    // Anscombe (1973), "Graphs in Statistical Analysis". Real positive
    // correlation; mean(y) ≈ 7.50, var(y) ≈ 4.13, slope ≈ 0.50.
    let x = vec![10.0, 8.0, 13.0, 9.0, 11.0, 14.0, 6.0, 4.0, 12.0, 7.0, 5.0];
    let y = vec![
        8.04, 6.95, 7.58, 8.81, 8.33, 9.96, 7.24, 4.26, 10.84, 4.82, 5.68,
    ];
    let fig = Figure::new(800, 600).add(PointMark::new(x, y));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_scatter_sizes() {
    // Two clusters distinguished by point size and color — a typical
    // bubble-chart use case. The "large" cluster (radius 8, blue) sits
    // upper-left; the "small" cluster (radius 3, red) sits lower-right.
    // Tests both `PointMark::radius` and overlap-free coordinate placement.
    // Names are `large_*` / `small_*` (not `cluster_a_*` / `cluster_b_*`)
    // to avoid `clippy::similar_names`.
    let large_x = vec![1.2, 2.1, 1.8, 2.5, 1.5, 2.8, 1.9, 2.2];
    let large_y = vec![7.8, 8.5, 7.2, 8.1, 8.9, 7.5, 8.3, 7.9];
    let small_x = vec![6.5, 7.2, 7.8, 6.9, 7.5, 8.1, 6.7, 7.4, 8.0, 7.0];
    let small_y = vec![2.1, 2.8, 1.5, 2.4, 1.9, 2.6, 1.8, 2.2, 2.5, 1.6];

    let fig = Figure::new(800, 600)
        .add(
            PointMark::new(large_x, large_y)
                .radius(8.0)
                .color(Color::BLUE),
        )
        .add(
            PointMark::new(small_x, small_y)
                .radius(3.0)
                .color(Color::RED),
        );
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

// ── bar tests ────────────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_bar_vertical() {
    // Monthly rainfall totals (mm) — simple positive-only dataset that
    // exercises the baseline-to-value vertical bar path.
    let fig = Figure::new(800, 600).add(BarMark::new(
        vec!["Jan".into(), "Feb".into(), "Mar".into()],
        vec![42.0, 18.5, 67.3],
    ));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_bar_horizontal() {
    // City populations (millions) — horizontal layout exercises the
    // transposed band/value axis path and left-to-right bar growth.
    let fig = Figure::new(800, 600).add(
        BarMark::new(
            vec![
                "Oslo".into(),
                "Stockholm".into(),
                "Copenhagen".into(),
                "Helsinki".into(),
            ],
            vec![1.0, 2.4, 0.8, 0.7],
        )
        .horizontal(),
    );
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_bar_grouped() {
    // Quarterly sales vs costs — two series sharing the same category axis.
    // Exercises dodge positioning so bars sit side-by-side within each band.
    let fig = Figure::new(800, 600)
        .add(
            BarMark::new(
                vec!["Q1".into(), "Q2".into(), "Q3".into()],
                vec![40.0, 55.0, 48.0],
            )
            .group("Sales")
            .color(Color::BLUE),
        )
        .add(
            BarMark::new(
                vec!["Q1".into(), "Q2".into(), "Q3".into()],
                vec![30.0, 38.0, 35.0],
            )
            .group("Costs")
            .color(Color::RED),
        );
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_bar_stacked() {
    // Energy mix (GWh) stacked by source — exercises cumulative baseline
    // offsets so the second series begins where the first ends.
    let fig = Figure::new(800, 600)
        .add(
            BarMark::new(vec!["2022".into(), "2023".into()], vec![120.0, 135.0])
                .stack("wind")
                .color(Color::BLUE),
        )
        .add(
            BarMark::new(vec!["2022".into(), "2023".into()], vec![80.0, 95.0])
                .stack("solar")
                .color(Color::RED),
        );
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

// ── step tests ─────────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_step_pre() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&xi| (xi * 0.5).floor() % 5.0).collect();
    let fig = Figure::new(800, 600).add(StepMark::new(x, y).position(StepPosition::Pre));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_step_mid() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&xi| (xi * 0.5).floor() % 5.0).collect();
    let fig = Figure::new(800, 600).add(StepMark::new(x, y).position(StepPosition::Mid));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_step_post() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&xi| (xi * 0.5).floor() % 5.0).collect();
    let fig = Figure::new(800, 600).add(StepMark::new(x, y).position(StepPosition::Post));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

// ── area tests ─────────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_area_basic() {
    // Temperature over a year — area chart exercises closed polygon path.
    let x: Vec<f64> = (0..365).map(|i| i as f64).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|&xi| 15.0 + 10.0 * (xi * 2.0 * std::f64::consts::PI / 365.0).sin())
        .collect();
    let fig = Figure::new(800, 600).add(AreaMark::new(x, y).opacity(0.5));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_area_nan_gaps() {
    let x = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let y = vec![0.0, 3.0, 1.0, 4.0, f64::NAN, 2.0, 5.0, 3.0];
    let fig = Figure::new(800, 600).add(AreaMark::new(x, y).opacity(0.5));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_area_with_baseline() {
    let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|&xi| (xi * 0.5).sin() * 10.0 + 15.0).collect();
    let fig = Figure::new(800, 600).add(AreaMark::new(x, y).baseline(10.0).opacity(0.6));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

// ── histogram tests ───────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_histogram_basic() {
    let mut rng = 42u32;
    let mut next_rand = || {
        rng = rng.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        rng as f64 / u32::MAX as f64
    };
    let data: Vec<f64> = (0..1000)
        .map(|_| {
            let u1 = next_rand();
            let u2 = next_rand();
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            50.0 + 15.0 * z
        })
        .collect();
    let fig = Figure::new(800, 600).add(HistogramMark::new(data));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

// ── title and label tests ─────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_with_title_and_labels() {
    let (x, y) = damped_cosine(30);
    let fig = Figure::new(800, 600)
        .title("Damped Oscillation")
        .x_label("Time (s)")
        .y_label("Amplitude (m)")
        .add(LineMark::new(x, y));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}
