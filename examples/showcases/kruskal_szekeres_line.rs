//! Kruskal–Szekeres diagram — starsight 0.1.0 showcase

use starsight::prelude::*;

const M: f64 = 1.0;
const CLIP: f64 = 5.0;

// ── Core transform ───────────────────────────────────────────────────────────

fn kruskal(r: f64, t: f64, su: f64, sv: f64) -> Option<(f64, f64)> {
    let exp = (r / (4.0 * M)).exp();
    let tau = t / (4.0 * M);

    if r > 2.0 * M {
        let f = ((r / (2.0 * M)) - 1.0).sqrt() * exp;

        let u = su * f * tau.cosh();
        let v = sv * f * tau.sinh();

        Some((u, v))
    } else if r < 2.0 * M {
        let f = (1.0 - (r / (2.0 * M))).sqrt() * exp;

        let u = su * f * tau.sinh();
        let v = sv * f * tau.cosh();

        Some((u, v))
    } else {
        None
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn clip(u: f64, v: f64) -> bool {
    u.is_finite() && v.is_finite() && u.abs() < CLIP && v.abs() < CLIP
}

fn signs() -> [(f64, f64); 4] {
    [
        ( 1.0,  1.0),
        (-1.0,  1.0),
        ( 1.0, -1.0),
        (-1.0, -1.0),
    ]
}

// ── Constant r curves ─────────────────────────────────────────────────────────

fn constant_r_curve(r: f64, steps: usize) -> (Vec<f64>, Vec<f64>) {
    let mut x = Vec::new();
    let mut y = Vec::new();

    for i in 0..steps {
        let t = -20.0 + 40.0 * (i as f64) / (steps as f64);

        for (su, sv) in signs() {
            if let Some((u, v)) = kruskal(r, t, su, sv) {
                if clip(u, v) {
                    x.push(u);
                    y.push(v);
                }
            }
        }
    }

    (x, y)
}

// ── Constant t curves ─────────────────────────────────────────────────────────

fn constant_t_curve(t: f64, steps: usize) -> (Vec<f64>, Vec<f64>) {
    let mut x = Vec::new();
    let mut y = Vec::new();

    for i in 0..steps {
        let r = 0.01 + (8.0 - 0.01) * (i as f64) / (steps as f64);

        if (r - 2.0 * M).abs() < 1e-6 {
            continue;
        }

        for (su, sv) in signs() {
            if let Some((u, v)) = kruskal(r, t, su, sv) {
                if clip(u, v) {
                    x.push(u);
                    y.push(v);
                }
            }
        }
    }

    (x, y)
}

// ── Event horizon (r = 2M) ────────────────────────────────────────────────────

fn horizon(steps: usize) -> (Vec<f64>, Vec<f64>) {
    let mut x = Vec::new();
    let mut y = Vec::new();

    for i in 0..steps {
        let t = -20.0 + 40.0 * (i as f64) / (steps as f64);
        let v = t / (4.0 * M);

        let pts = [
            ( v,  v),
            (-v,  v),
            ( v, -v),
            (-v, -v),
        ];

        for (u, vv) in pts {
            if clip(u, vv) {
                x.push(u);
                y.push(vv);
            }
        }
    }

    (x, y)
}

// ── Singularity (r = 0) ───────────────────────────────────────────────────────

fn singularity(steps: usize) -> (Vec<f64>, Vec<f64>) {
    let mut x = Vec::new();
    let mut y = Vec::new();

    for i in 0..steps {
        let s = -3.0 + 6.0 * (i as f64) / (steps as f64);

        let u = s.sinh();
        let v = s.cosh();

        let pts = [
            ( u,  v),
            (-u,  v),
            ( u, -v),
            (-u, -v),
        ];

        for (uu, vv) in pts {
            if clip(uu, vv) {
                x.push(uu);
                y.push(vv);
            }
        }
    }

    (x, y)
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let mut fig = Figure::new(1600, 1000);

    fig = fig
        .title("Kruskal–Szekeres diagram")
        .x_label("u")
        .y_label("v");

    let r_values = [0.5, 1.0, 1.5, 1.9, 2.5, 3.0, 5.0, 10.0];

    for &r in &r_values {
        let (x, y) = constant_r_curve(r, 500);

        fig = fig.add(LineMark {
            x,
            y,
            color: Color::BLUE,
            width: 0.8,
        });
    }

    let t_values = [-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0];

    for &t in &t_values {
        let (x, y) = constant_t_curve(t, 500);

        fig = fig.add(LineMark {
            x,
            y,
            color: Color::RED,
            width: 0.8,
        });
    }

    let (x_h, y_h) = horizon(500);

    fig = fig.add(LineMark {
        x: x_h,
        y: y_h,
        color: Color::BLACK,
        width: 2.5,
    });

    let (x_s, y_s) = singularity(500);

    fig = fig.add(LineMark {
        x: x_s,
        y: y_s,
        color: Color::BLACK,
        width: 3.5,
    });

    fig.save("examples/showcases/kruskal.png")?;
    println!("saved kruskal.png");

    Ok(())
}