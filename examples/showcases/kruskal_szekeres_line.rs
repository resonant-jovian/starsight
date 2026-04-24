//! Kruskal–Szekeres diagram — starsight 0.1.0 showcase (fixed)
//!
//! Root cause of the original output: constant_r_curve pushed all four
//! sign-quadrant sweeps into one Vec, so LineMark connected Region I
//! points to Region III points with diagonal slashes. Fix: one LineMark
//! per (su, sv) pair.

use starsight::prelude::*;

const M: f64 = 1.0;
const CLIP: f64 = 4.5;

// ── Transform ──────────────────────────────────────────────────────────────────

fn kruskal(r: f64, t: f64, su: f64, sv: f64) -> Option<(f64, f64)> {
    if (r - 2.0 * M).abs() < 1e-9 {
        return None;
    }
    let exp_r = (r / (4.0 * M)).exp();
    let tau = t / (4.0 * M);
    let (u, v) = if r > 2.0 * M {
        let f = ((r / (2.0 * M)) - 1.0).sqrt() * exp_r;
        (su * f * tau.cosh(), sv * f * tau.sinh())
    } else {
        let f = (1.0 - r / (2.0 * M)).sqrt() * exp_r;
        (su * f * tau.sinh(), sv * f * tau.cosh())
    };
    if u.is_finite() && v.is_finite() && u.abs() <= CLIP && v.abs() <= CLIP {
        Some((u, v))
    } else {
        None
    }
}

// ── Curve builders ─────────────────────────────────────────────────────────────

/// One continuous arc: fixed r, sweep t, single (su, sv) quadrant.
/// NaN is inserted only at clip-boundary exits so LineMark draws a gap
/// instead of a line back across the diagram.
fn r_arc(r: f64, su: f64, sv: f64, steps: usize) -> (Vec<f64>, Vec<f64>) {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for i in 0..=steps {
        let t = -20.0 + 40.0 * (i as f64) / (steps as f64);
        match kruskal(r, t, su, sv) {
            Some((u, v)) => {
                xs.push(u);
                ys.push(v);
            }
            None if !xs.is_empty() => {
                xs.push(f64::NAN);
                ys.push(f64::NAN);
            }
            None => {}
        }
    }
    (xs, ys)
}

/// One constant-t ray in the exterior (r > 2M only), single su sign.
/// Constant-Schwarzschild-t has no meaning inside the horizon; omitting
/// those points avoids the confused interior fan in the original.
fn t_arc(t: f64, su: f64, steps: usize) -> (Vec<f64>, Vec<f64>) {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for i in 0..=steps {
        // sweep from just above the horizon outward
        let r = 2.0 * M + 0.005 + 18.0 * (i as f64) / (steps as f64);
        // sv=1.0: for r > 2M with su fixed, sv is redundant (sinh covers ±)
        match kruskal(r, t, su, 1.0) {
            Some((u, v)) => {
                xs.push(u);
                ys.push(v);
            }
            None if !xs.is_empty() => {
                xs.push(f64::NAN);
                ys.push(f64::NAN);
            }
            None => {}
        }
    }
    (xs, ys)
}

// ── Main ───────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let mut fig = Figure::new(1200, 1200);
    fig = fig
        .title("Kruskal–Szekeres Diagram  (M = 1)")
        .x_label("u  (spacelike Kruskal)")
        .y_label("v  (timelike Kruskal)");

    // ── Constant-r arcs (blue) ─────────────────────────────────────────────────
    // r > 2M → left/right hyperbolas in Regions I (su=+1) and III (su=−1)
    // r < 2M → top/bottom hyperbolas in Regions II (sv=+1) and IV (sv=−1)
    let r_exterior = [2.5, 3.0, 4.0, 5.0, 7.0, 10.0];
    let r_interior = [0.25, 0.5, 1.0, 1.5, 1.9];

    for &r in &r_exterior {
        for su in [1.0_f64, -1.0] {
            let (xs, ys) = r_arc(r, su, 1.0, 600);
            if xs.iter().any(|x| x.is_finite()) {
                fig = fig.add(LineMark {
                    x: xs,
                    y: ys,
                    color: Color::BLUE,
                    width: 0.9,
                    label: None,
                });
            }
        }
    }
    for &r in &r_interior {
        for sv in [1.0_f64, -1.0] {
            let (xs, ys) = r_arc(r, 1.0, sv, 600);
            if xs.iter().any(|x| x.is_finite()) {
                fig = fig.add(LineMark {
                    x: xs,
                    y: ys,
                    color: Color::BLUE,
                    width: 0.9,
                    label: None,
                });
            }
        }
    }

    // ── Constant-t rays (red, exterior Regions I and III only) ────────────────
    let t_vals = [-4.0, -3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0];
    for &t in &t_vals {
        for su in [1.0_f64, -1.0] {
            let (xs, ys) = t_arc(t, su, 500);
            if xs.iter().any(|x| x.is_finite()) {
                fig = fig.add(LineMark {
                    x: xs,
                    y: ys,
                    color: Color::RED,
                    width: 0.8,
                    label: None,
                });
            }
        }
    }

    // ── Event horizon: u = +v and u = −v — two separate LineMark objects ───────
    // Original bug: all four diagonal half-lines were concatenated into one Vec.
    let n = 300usize;
    let ss: Vec<f64> = (0..=n)
        .map(|i| -CLIP + 2.0 * CLIP * (i as f64) / (n as f64))
        .collect();
    let neg: Vec<f64> = ss.iter().map(|&s| -s).collect();
    // u = v
    fig = fig.add(LineMark {
        x: ss.clone(),
        y: ss.clone(),
        color: Color::BLACK,
        width: 2.5,
        label: None,
    });
    // u = -v
    fig = fig.add(LineMark {
        x: neg,
        y: ss.clone(),
        color: Color::BLACK,
        width: 2.5,
        label: None,
    });

    // ── Singularity r = 0: v² − u² = 1 (future and past, separate) ───────────
    // Parametrize by u; clip where |v| > CLIP (occurs near |u| ≈ sqrt(CLIP²−1)).
    let u_max_sing = (CLIP * CLIP - 1.0).sqrt(); // ≈ 4.39 for CLIP=4.5
    let m = 400usize;
    let us: Vec<f64> = (0..=m)
        .map(|i| -u_max_sing + 2.0 * u_max_sing * (i as f64) / (m as f64))
        .collect();
    let vs_fut: Vec<f64> = us.iter().map(|&u| (1.0 + u * u).sqrt()).collect();
    let vs_past: Vec<f64> = us.iter().map(|&u| -(1.0 + u * u).sqrt()).collect();

    fig = fig.add(LineMark {
        x: us.clone(),
        y: vs_fut,
        color: Color::BLACK,
        width: 3.5,
        label: None,
    });
    fig = fig.add(LineMark {
        x: us.clone(),
        y: vs_past,
        color: Color::BLACK,
        width: 3.5,
        label: None,
    });

    fig.save("examples/showcases/kruskal_szekeres_line.png")?;
    println!("Saved kruskal_szekeres_line.png");
    Ok(())
}
