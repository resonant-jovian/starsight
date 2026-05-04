//! Four classic 2-D scalar fields rendered as contour plots — starsight
//! 0.3.0 showcase #22.
//!
//! Rosenbrock, Himmelblau, Rastrigin, and a 3-mode Gaussian mixture, each
//! sampled on a 100×100 grid and rendered with `ContourMark::isolines` plus a
//! viridis colormap. The four panels share the same canvas via
//! [`MultiPanelFigure`], so the topology of each surface reads against a
//! consistent visual baseline.

#![allow(clippy::cast_precision_loss)]

use starsight::colormap::VIRIDIS;
use starsight::prelude::*;
use starsight::statistics::Grid;

fn rosenbrock(x: f64, y: f64) -> f64 {
    let a = 1.0 - x;
    let b = y - x * x;
    a * a + 100.0 * b * b
}

fn himmelblau(x: f64, y: f64) -> f64 {
    let a = x * x + y - 11.0;
    let b = x + y * y - 7.0;
    a * a + b * b
}

fn rastrigin(x: f64, y: f64) -> f64 {
    let two_pi = std::f64::consts::TAU;
    20.0 + x * x - 10.0 * (two_pi * x).cos() + y * y - 10.0 * (two_pi * y).cos()
}

fn gaussian_mixture(x: f64, y: f64) -> f64 {
    let g = |cx: f64, cy: f64, sigma: f64, weight: f64| -> f64 {
        let dx = x - cx;
        let dy = y - cy;
        weight * (-(dx * dx + dy * dy) / (2.0 * sigma * sigma)).exp()
    };
    g(-1.5, -1.5, 0.6, 1.0) + g(1.0, 1.5, 0.8, 0.7) + g(2.0, -1.0, 0.4, 0.4)
}

fn contour_panel(
    title: &str,
    grid_factory: impl Fn(usize, usize, f64, f64, f64, f64) -> Grid,
    levels: Vec<f64>,
) -> Figure {
    let grid = grid_factory(80, 80, -3.0, 3.0, -3.0, 3.0);
    Figure::new(400, 400)
        .title(title)
        .x_label("x")
        .y_label("y")
        .add(
            ContourMark::new(grid, levels)
                .colormap(VIRIDIS)
                .stroke_width(1.2),
        )
}

fn main() -> Result<()> {
    let mp = MultiPanelFigure::new(1000, 1000, 2, 2)
        .padding(12.0)
        .add(contour_panel(
            "Rosenbrock",
            |nx, ny, x0, x1, y0, y1| Grid::sample(nx, ny, x0, x1, y0, y1, rosenbrock),
            vec![1.0, 5.0, 20.0, 100.0, 500.0, 2000.0],
        ))
        .add(contour_panel(
            "Himmelblau",
            |nx, ny, x0, x1, y0, y1| Grid::sample(nx, ny, x0, x1, y0, y1, himmelblau),
            vec![1.0, 5.0, 20.0, 80.0, 200.0, 500.0],
        ))
        .add(contour_panel(
            "Rastrigin",
            |nx, ny, x0, x1, y0, y1| Grid::sample(nx, ny, x0, x1, y0, y1, rastrigin),
            vec![5.0, 15.0, 30.0, 50.0, 70.0],
        ))
        .add(contour_panel(
            "Gaussian mixture",
            |nx, ny, x0, x1, y0, y1| Grid::sample(nx, ny, x0, x1, y0, y1, gaussian_mixture),
            vec![0.05, 0.1, 0.3, 0.5, 0.7, 0.9],
        ));
    mp.save("examples/scientific/contour_fields.png")
}
