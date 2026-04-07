//! Snapshot tests for layer-3 mark rendering.
//!
//! Each test uses realistic, deterministic data drawn from a recognizable
//! domain (physics, weather, statistics) so the resulting PNGs both verify the
//! pipeline and double as honest demo material for the README screenshots.

use starsight_layer_1::primitives::Color;
use starsight_layer_3::marks::{LineMark, PointMark};
use starsight_layer_5::Figure;

// ── helpers ──────────────────────────────────────────────────────────────────────────────────────

/// Damped cosine: `y = e^(-0.05x) · cos(0.5x)` over `0..n`.
///
/// Looks like a damped harmonic oscillator. Crosses zero, decays toward zero,
/// has both positive and negative values — exercises the Wilkinson tick
/// algorithm on a symmetric range.
fn damped_cosine(n: usize) -> (Vec<f64>, Vec<f64>) {
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
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
    let bytes = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}

#[test]
fn snapshot_line_nan_gaps() {
    // 40-hour temperature reading from a sensor with two outage windows.
    // NaN gaps at hours 8–11 and 25–27 split the line into three sub-paths,
    // verifying that NaN handling produces visible breaks rather than artifacts.
    let x: Vec<f64> = (0..40).map(|i| i as f64).collect();
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
    for i in 8..12 {
        y[i] = f64::NAN;
    }
    for i in 25..28 {
        y[i] = f64::NAN;
    }
    let fig = Figure::new(800, 600).add(LineMark::new(x, y));
    let bytes = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}

#[test]
fn snapshot_line_multi() {
    // Daily high and low temperatures over four weeks — two related series
    // that share an x axis and never cross. Tests color differentiation and
    // overlapping (but distinct) line rendering.
    let days: Vec<f64> = (1..=28).map(|d| d as f64).collect();
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
    let bytes = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
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
    let bytes = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}

#[test]
fn snapshot_scatter_sizes() {
    // Two clusters distinguished by point size and color — a typical
    // bubble-chart use case. Cluster A (large, blue) sits upper-left;
    // cluster B (small, red) sits lower-right. Tests both `PointMark::radius`
    // and overlap-free coordinate placement.
    let cluster_a_x = vec![1.2, 2.1, 1.8, 2.5, 1.5, 2.8, 1.9, 2.2];
    let cluster_a_y = vec![7.8, 8.5, 7.2, 8.1, 8.9, 7.5, 8.3, 7.9];
    let cluster_b_x = vec![6.5, 7.2, 7.8, 6.9, 7.5, 8.1, 6.7, 7.4, 8.0, 7.0];
    let cluster_b_y = vec![2.1, 2.8, 1.5, 2.4, 1.9, 2.6, 1.8, 2.2, 2.5, 1.6];

    let fig = Figure::new(800, 600)
        .add(
            PointMark::new(cluster_a_x, cluster_a_y)
                .radius(8.0)
                .color(Color::BLUE),
        )
        .add(
            PointMark::new(cluster_b_x, cluster_b_y)
                .radius(3.0)
                .color(Color::RED),
        );
    let bytes = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", bytes);
}
