//! Polar calendar — starsight 0.3.0 showcase #8.
//!
//! 21 years × 52 weeks of a synthetic daily metric, rendered as an annular
//! grid: theta = week-of-year (52 angular tiles), r = year offset (21
//! radial rings). Each tile's color is the colormap-mapped value, which
//! makes seasonal cycles legible at a glance — the metric peaks each
//! summer and dips in winter, so the rose shows alternating bands.
//!
//! Backed by [`PolarRectMark`] on `Figure::polar_axes`. The colormap
//! comes from `prismatica::matplotlib::VIRIDIS`.

use starsight::axes::Axis;
use starsight::colormap::VIRIDIS;
use starsight::prelude::*;

fn main() -> Result<()> {
    let years = 21usize;
    let weeks = 52usize;

    // Synthetic data: seasonal sine wave + slow upward trend across years.
    // Each (year, week) cell carries one value.
    let mut theta_min = Vec::with_capacity(years * weeks);
    let mut theta_max = Vec::with_capacity(years * weeks);
    let mut r_min = Vec::with_capacity(years * weeks);
    let mut r_max = Vec::with_capacity(years * weeks);
    let mut colors = Vec::with_capacity(years * weeks);

    let weeks_f = weeks as f64;
    let years_f = years as f64;
    let two_pi = std::f64::consts::TAU;

    for y in 0..years {
        let r0 = y as f64 / years_f;
        let r1 = (y as f64 + 1.0) / years_f;
        for w in 0..weeks {
            let t0 = w as f64 / weeks_f;
            let t1 = (w as f64 + 1.0) / weeks_f;
            // Seasonal sine peaking at week 26 (mid-summer), trending +0.4 over 21y.
            let seasonal = (two_pi * (w as f64 - 13.0) / weeks_f).sin();
            let trend = (y as f64 / years_f) * 0.4;
            let raw = 0.5 + 0.4 * seasonal + trend;
            let clamped = raw.clamp(0.0, 1.0);
            let color = VIRIDIS.sample(clamped);
            theta_min.push(t0);
            theta_max.push(t1);
            r_min.push(r0);
            r_max.push(r1);
            colors.push(color);
        }
    }

    let theta_axis = Axis::polar_angular(0.0, 1.0);
    let r_axis = Axis::polar_radial(0.0, 1.0);

    Figure::new(800, 800)
        .title("21-year seasonal calendar (week × year)")
        .polar_axes(theta_axis, r_axis)
        .add(PolarRectMark::new(theta_min, theta_max, r_min, r_max).colors(colors))
        .save("examples/scientific/polar_calendar.png")
}
