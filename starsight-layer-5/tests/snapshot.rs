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

use starsight_layer_1::colormap::{PLASMA, VIRIDIS};
use starsight_layer_1::primitives::Color;
use starsight_layer_3::marks::{
    AreaMark, BarMark, HeatmapMark, HistogramMark, LineMark, PointMark, StepMark, StepPosition,
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
    let (x, y) = damped_cosine(100);
    let fig = Figure::new(1200, 800).add(LineMark::new(x, y));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_line_nan_gaps() {
    let x: Vec<f64> = (0u32..168).map(f64::from).collect();
    let mut y: Vec<f64> = x
        .iter()
        .map(|&xi| 20.0 + 5.0 * (xi * std::f64::consts::PI / 12.0).sin() + 2.0 * (xi * 0.3).cos())
        .collect();
    for i in [24..32, 72..80, 120..128].iter() {
        for j in i.clone() {
            y[j as usize] = f64::NAN;
        }
    }
    let fig = Figure::new(1200, 800).add(LineMark::new(x, y));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_line_multi() {
    let days: Vec<f64> = (1u32..=90).map(f64::from).collect();
    let highs: Vec<f64> = vec![
        12.5, 13.1, 14.0, 13.8, 14.5, 15.2, 16.1, 16.8, 17.5, 18.2, 17.9, 17.4, 18.0, 19.3, 20.5,
        21.2, 20.8, 19.6, 18.4, 17.2, 17.8, 18.5, 19.2, 20.0, 20.9, 21.4, 21.0, 20.3, 19.5, 18.1,
        17.3, 16.8, 15.9, 14.7, 13.5, 12.8, 11.9, 11.2, 10.5, 9.8, 10.1, 10.8, 11.5, 12.3, 13.2,
        14.1, 15.0, 15.8, 16.5, 17.2, 18.0, 18.8, 19.5, 20.2, 21.0, 21.8, 22.5, 23.0, 22.8, 22.0,
        21.2, 20.5, 19.8, 19.0, 18.2, 17.5, 16.8, 16.2, 15.8, 15.5, 15.0, 14.5, 13.9, 13.2, 12.5,
        11.8, 11.2, 10.8, 10.5, 10.2, 10.0, 9.8, 9.9, 10.2, 10.5, 11.0, 11.5, 12.0,
    ];
    let lows: Vec<f64> = vec![
        4.2, 4.8, 5.5, 5.1, 5.9, 6.4, 7.0, 7.3, 7.8, 8.2, 8.0, 7.6, 8.1, 9.0, 10.1, 10.5, 10.2,
        9.4, 8.7, 7.9, 8.3, 8.9, 9.5, 10.2, 10.8, 11.1, 10.7, 10.0, 9.4, 8.8, 8.2, 7.8, 7.2, 6.8,
        6.2, 5.5, 4.8, 4.2, 4.5, 4.8, 5.2, 5.8, 6.4, 7.0, 7.6, 8.2, 8.8, 9.4, 10.0, 10.5, 11.0,
        11.5, 12.0, 12.5, 13.0, 13.5, 14.0, 14.2, 14.0, 13.5, 12.8, 12.2, 11.5, 10.8, 10.2, 9.5,
        8.8, 8.2, 7.6, 7.0, 6.5, 6.0, 5.5, 5.2, 5.0, 4.8, 4.5, 4.2, 4.0, 3.8, 3.5, 3.2, 3.0, 2.8,
        2.5, 2.4, 2.5, 2.8, 3.0, 3.2, 3.5, 3.8,
    ];
    let fig = Figure::new(1200, 800)
        .title("Daily High and Low Temperatures (Spring Quarter)")
        .x_label("Day of Year")
        .y_label("Temperature (°C)")
        .add(LineMark::new(days.clone(), highs).color(Color::RED))
        .add(LineMark::new(days, lows).color(Color::BLUE));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

// ── scatter tests ────────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_scatter_basic() {
    let x = vec![
        10.0, 8.0, 13.0, 9.0, 11.0, 14.0, 6.0, 4.0, 12.0, 7.0, 5.0, 11.0, 12.0, 9.0, 8.0, 13.0,
        10.0, 9.0, 11.0, 14.0, 6.0, 4.0, 12.0, 7.0, 5.0, 10.0, 8.0, 13.0, 9.0, 11.0,
    ];
    let y = vec![
        8.04, 6.95, 7.58, 8.81, 8.33, 9.96, 7.24, 4.26, 10.84, 4.82, 5.68, 8.50, 9.20, 7.80, 8.10,
        7.90, 8.20, 8.60, 8.40, 10.10, 7.10, 4.50, 10.60, 5.00, 5.90, 7.90, 7.10, 7.40, 8.70, 8.50,
    ];
    let fig = Figure::new(1200, 800)
        .title("Regression Analysis")
        .x_label("X Values")
        .y_label("Y Values")
        .add(PointMark::new(x, y).radius(4.0));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_scatter_sizes() {
    let large_x = vec![
        1.2, 2.1, 1.8, 2.5, 1.5, 2.8, 1.9, 2.2, 1.6, 2.4, 2.0, 1.7, 2.3, 1.4, 2.6,
    ];
    let large_y = vec![
        7.8, 8.5, 7.2, 8.1, 8.9, 7.5, 8.3, 7.9, 8.0, 8.2, 7.6, 8.4, 7.7, 8.8, 7.4,
    ];
    let small_x = vec![
        6.5, 7.2, 7.8, 6.9, 7.5, 8.1, 6.7, 7.4, 8.0, 7.0, 7.3, 6.8, 7.6, 7.1, 7.9, 6.6, 7.7, 7.2,
    ];
    let small_y = vec![
        2.1, 2.8, 1.5, 2.4, 1.9, 2.6, 1.8, 2.2, 2.5, 1.6, 2.3, 1.7, 2.0, 2.4, 1.4, 2.7, 1.9, 2.1,
    ];

    let fig = Figure::new(1200, 800)
        .title("Cluster Analysis: Two Distinct Groups")
        .x_label("Feature A")
        .y_label("Feature B")
        .add(
            PointMark::new(large_x, large_y)
                .radius(10.0)
                .color(Color::BLUE),
        )
        .add(
            PointMark::new(small_x, small_y)
                .radius(4.0)
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
    let fig = Figure::new(1200, 800)
        .title("Monthly Average Rainfall")
        .x_label("Month")
        .y_label("Rainfall (mm)")
        .add(BarMark::new(
            vec![
                "Jan".into(),
                "Feb".into(),
                "Mar".into(),
                "Apr".into(),
                "May".into(),
                "Jun".into(),
                "Jul".into(),
                "Aug".into(),
                "Sep".into(),
                "Oct".into(),
                "Nov".into(),
                "Dec".into(),
            ],
            vec![
                68.0, 52.0, 74.0, 85.0, 92.0, 78.0, 65.0, 70.0, 88.0, 95.0, 102.0, 78.0,
            ],
        ));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_bar_horizontal() {
    let fig = Figure::new(1200, 800)
        .title("Population by City")
        .x_label("Population (millions)")
        .y_label("City")
        .add(
            BarMark::new(
                vec![
                    "Tokyo".into(),
                    "Delhi".into(),
                    "Shanghai".into(),
                    "São Paulo".into(),
                    "Mexico City".into(),
                    "Cairo".into(),
                    "Mumbai".into(),
                    "Beijing".into(),
                    "Dhaka".into(),
                    "Osaka".into(),
                    "New York".into(),
                    "Karachi".into(),
                ],
                vec![3.7, 3.3, 2.8, 2.2, 2.1, 2.1, 2.0, 2.0, 2.1, 1.9, 1.8, 1.6],
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
    let fig = Figure::new(1200, 800)
        .title("Quarterly Revenue by Region")
        .x_label("Quarter")
        .y_label("Revenue (millions)")
        .add(
            BarMark::new(
                vec!["Q1".into(), "Q2".into(), "Q3".into(), "Q4".into()],
                vec![420.0, 580.0, 510.0, 680.0],
            )
            .group("North America")
            .color(Color::BLUE),
        )
        .add(
            BarMark::new(
                vec!["Q1".into(), "Q2".into(), "Q3".into(), "Q4".into()],
                vec![280.0, 340.0, 390.0, 420.0],
            )
            .group("Europe")
            .color(Color::RED),
        )
        .add(
            BarMark::new(
                vec!["Q1".into(), "Q2".into(), "Q3".into(), "Q4".into()],
                vec![180.0, 220.0, 250.0, 310.0],
            )
            .group("Asia Pacific")
            .color(Color::GREEN),
        );
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_bar_stacked() {
    let fig = Figure::new(1200, 800)
        .title("Energy Generation Mix by Year")
        .x_label("Year")
        .y_label("Energy (TWh)")
        .add(
            BarMark::new(
                vec![
                    "2019".into(),
                    "2020".into(),
                    "2021".into(),
                    "2022".into(),
                    "2023".into(),
                    "2024".into(),
                ],
                vec![180.0, 195.0, 210.0, 225.0, 240.0, 260.0],
            )
            .stack("Wind")
            .color(Color {
                r: 76,
                g: 175,
                b: 80,
            }),
        )
        .add(
            BarMark::new(
                vec![
                    "2019".into(),
                    "2020".into(),
                    "2021".into(),
                    "2022".into(),
                    "2023".into(),
                    "2024".into(),
                ],
                vec![120.0, 135.0, 150.0, 165.0, 180.0, 195.0],
            )
            .stack("Solar")
            .color(Color {
                r: 255,
                g: 193,
                b: 7,
            }),
        )
        .add(
            BarMark::new(
                vec![
                    "2019".into(),
                    "2020".into(),
                    "2021".into(),
                    "2022".into(),
                    "2023".into(),
                    "2024".into(),
                ],
                vec![80.0, 75.0, 70.0, 60.0, 55.0, 50.0],
            )
            .stack("Coal")
            .color(Color {
                r: 96,
                g: 125,
                b: 139,
            }),
        );
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

// ── step tests ─────────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_step_pre() {
    let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|&xi| ((xi * 0.4).floor() % 6.0) * 5.0 + 10.0)
        .collect();
    let fig = Figure::new(1200, 800)
        .title("Step Chart: Pre Position")
        .x_label("Time")
        .y_label("Value")
        .add(StepMark::new(x, y).position(StepPosition::Pre));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_step_mid() {
    let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|&xi| ((xi * 0.4).floor() % 6.0) * 5.0 + 10.0)
        .collect();
    let fig = Figure::new(1200, 800)
        .title("Step Chart: Mid Position")
        .x_label("Time")
        .y_label("Value")
        .add(StepMark::new(x, y).position(StepPosition::Mid));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_step_post() {
    let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|&xi| ((xi * 0.4).floor() % 6.0) * 5.0 + 10.0)
        .collect();
    let fig = Figure::new(1200, 800)
        .title("Step Chart: Post Position")
        .x_label("Time")
        .y_label("Value")
        .add(StepMark::new(x, y).position(StepPosition::Post));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

// ── area tests ─────────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_area_basic() {
    let x: Vec<f64> = (0..365).map(|i| i as f64).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|&xi| {
            15.0 + 10.0 * (xi * 2.0 * std::f64::consts::PI / 365.0).sin()
                + 3.0 * (xi * 2.0 * std::f64::consts::PI / 365.0 * 4.0).sin()
        })
        .collect();
    let fig = Figure::new(1200, 800)
        .title("Daily Temperature Profile (Annual)")
        .x_label("Day of Year")
        .y_label("Temperature (°C)")
        .add(AreaMark::new(x, y).opacity(0.6));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_area_nan_gaps() {
    let x: Vec<f64> = (0..30).map(|i| i as f64).collect();
    let y: Vec<f64> = vec![
        0.0,
        2.5,
        4.0,
        3.5,
        5.0,
        6.5,
        5.0,
        4.5,
        f64::NAN,
        f64::NAN,
        3.0,
        4.5,
        5.5,
        4.0,
        3.5,
        2.0,
        1.5,
        2.5,
        3.0,
        4.5,
        5.0,
        f64::NAN,
        f64::NAN,
        f64::NAN,
        3.5,
        2.0,
        1.5,
        2.0,
        3.5,
        4.0,
    ];
    let fig = Figure::new(1200, 800)
        .title("Area Chart with Data Gaps")
        .x_label("Time")
        .y_label("Value")
        .add(AreaMark::new(x, y).opacity(0.6));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_area_with_baseline() {
    let x: Vec<f64> = (0..100).map(|i| i as f64).collect();
    // Range [0, 60] so the curve crosses baseline=30 each period — exercises both
    // above- and below-baseline rendering rather than the baseline acting as a floor.
    let y: Vec<f64> = x
        .iter()
        .map(|&xi| (xi * 0.15).sin() * 30.0 + 30.0)
        .collect();
    let fig = Figure::new(1200, 800)
        .title("Area Chart with Custom Baseline")
        .x_label("Observation")
        .y_label("Value")
        .add(AreaMark::new(x, y).baseline(30.0).opacity(0.6));
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
    let data: Vec<f64> = (0..5000)
        .map(|_| {
            let u1 = next_rand();
            let u2 = next_rand();
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            50.0 + 15.0 * z
        })
        .collect();
    let fig = Figure::new(1200, 800)
        .title("Distribution of Simulated Measurements")
        .x_label("Value")
        .y_label("Frequency")
        .add(HistogramMark::new(data));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

// ── title and label tests ─────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_with_title_and_labels() {
    let (x, y) = damped_cosine(80);
    let fig = Figure::new(1200, 800)
        .title("Damped Harmonic Oscillator")
        .x_label("Time (s)")
        .y_label("Amplitude (m)")
        .add(LineMark::new(x, y));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

// ── heatmap tests ───────────────────────────────────────────────────────────────────────────────

#[test]
fn snapshot_heatmap_basic() {
    let data: Vec<Vec<f64>> = (0..40)
        .map(|i| {
            (0..40)
                .map(|j| {
                    let x = i as f64 - 20.0;
                    let y = j as f64 - 20.0;
                    (x * x + y * y).sqrt()
                })
                .collect()
        })
        .collect();
    let fig = Figure::new(800, 800)
        .title("Distance from Center Heatmap")
        .add(HeatmapMark::new(data));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_heatmap_viridis() {
    let data: Vec<Vec<f64>> = (0..50)
        .map(|i| {
            (0..50)
                .map(|j| {
                    let x = i as f64 - 25.0;
                    let y = j as f64 - 25.0;
                    100.0 - (x * x + y * y).sqrt() + 20.0 * ((x * 0.1).sin() + (y * 0.1).cos())
                })
                .collect()
        })
        .collect();
    let fig = Figure::new(800, 800)
        .title("Sensor Reading Intensity (VIRIDIS)")
        .add(HeatmapMark::new(data).colormap(VIRIDIS));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}

#[test]
fn snapshot_heatmap_plasma() {
    let data: Vec<Vec<f64>> = (0..50)
        .map(|i| {
            (0..50)
                .map(|j| {
                    let x = i as f64 - 25.0;
                    let y = j as f64 - 25.0;
                    ((x * x + y * y) * 0.02).cos() * 50.0 + 50.0
                })
                .collect()
        })
        .collect();
    let fig = Figure::new(800, 800)
        .title("Wave Interference Pattern (PLASMA)")
        .add(HeatmapMark::new(data).colormap(PLASMA));
    let svg = fig.render_svg().unwrap();
    insta::assert_snapshot!(svg);
    let png = fig.render_png().unwrap();
    insta::assert_binary_snapshot!(".png", png);
}
