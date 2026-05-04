//! Error bars — starsight 0.3.0 showcase for [`ErrorBarMark`].
//!
//! Synthetic linear-fit residuals: 12 measurements at evenly-spaced x with
//! Gaussian error bars on y. The error magnitudes vary across the range to
//! show how `ErrorBarMark` handles asymmetric uncertainty (here: confidence
//! intervals that broaden at the endpoints).

use starsight::prelude::*;

fn main() -> Result<()> {
    // Synthetic dataset: y = 2x + 3 + noise, with confidence-interval
    // half-widths growing toward the ends of the x range.
    let xs: Vec<f64> = (0..12).map(|i| f64::from(i)).collect();
    let ys: Vec<f64> = xs
        .iter()
        .map(|x| 2.0 * x + 3.0 + 0.7 * (x * 0.7).sin())
        .collect();

    // Symmetric error magnitudes — narrower at center (near the regression
    // mean), wider at the endpoints.
    let n = xs.len();
    let half = (n - 1) as f64 / 2.0;
    let symmetric_errors: Vec<f64> = (0..n)
        .map(|i| {
            let d = (i as f64 - half).abs() / half;
            0.6 + 1.2 * d * d
        })
        .collect();

    // Asymmetric variant for the second series: lower half-width grows
    // faster than the upper, mimicking a log-skewed posterior.
    let xs2: Vec<f64> = xs.iter().map(|x| x + 0.25).collect();
    let ys2: Vec<f64> = xs2.iter().map(|x| 1.5 * x + 5.5).collect();
    let asym_pairs: Vec<(f64, f64)> = (0..n)
        .map(|i| {
            let d = (i as f64 - half).abs() / half;
            let lo = 0.4 + 1.6 * d;
            let hi = 0.4 + 0.8 * d;
            (lo, hi)
        })
        .collect();

    Figure::new(900, 600)
        .title("Regression-fit residuals — symmetric vs. asymmetric errors")
        .x_label("x")
        .y_label("y")
        .add(
            PointMark::new(xs.clone(), ys.clone())
                .color(Color::from_hex(0x004E_79A7))
                .radius(6.0)
                .label("series A — symmetric"),
        )
        .add(
            ErrorBarMark::new(xs, ys, symmetric_errors)
                .color(Color::from_hex(0x004E_79A7))
                .cap_width(8.0)
                .width(1.5),
        )
        .add(
            PointMark::new(xs2.clone(), ys2.clone())
                .color(Color::from_hex(0x00E1_5759))
                .radius(6.0)
                .label("series B — asymmetric"),
        )
        .add(
            ErrorBarMark::new(xs2, ys2, vec![0.0; 12])
                .errors_pair(asym_pairs)
                .color(Color::from_hex(0x00E1_5759))
                .cap_width(8.0)
                .width(1.5),
        )
        .save("examples/scientific/error_bars.png")
}
