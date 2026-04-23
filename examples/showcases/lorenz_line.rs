//! Lorenz attractor family — starsight 0.1.0 showcase
//!
//! 11 trajectories sweeping ρ ∈ {13, 15, 18, 21, 24.06, 28, 35, 50, 100, 160, 250}.
//! σ = 10, β = 8/3.  IC (1+ε, 1+ε, 1+ε), ε = 0.001·i.
//! RK4, dt = 0.005, 80 000 steps, first 5 000 discarded as transient.
//! 2D projection onto the x–z plane (the classic butterfly).
//! Each trajectory coloured by ρ on prismatica's inferno map.

use starsight::prelude::*;

// ── Lorenz derivatives ───────────────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct State {
    x: f64,
    y: f64,
    z: f64,
}

fn deriv(s: State, sigma: f64, rho: f64, beta: f64) -> State {
    State {
        x: sigma * (s.y - s.x),
        y: s.x * (rho - s.z) - s.y,
        z: s.x * s.y - beta * s.z,
    }
}

fn rk4(s: State, dt: f64, sigma: f64, rho: f64, beta: f64) -> State {
    let k1 = deriv(s, sigma, rho, beta);
    let s2 = State {
        x: s.x + 0.5 * dt * k1.x,
        y: s.y + 0.5 * dt * k1.y,
        z: s.z + 0.5 * dt * k1.z,
    };
    let k2 = deriv(s2, sigma, rho, beta);
    let s3 = State {
        x: s.x + 0.5 * dt * k2.x,
        y: s.y + 0.5 * dt * k2.y,
        z: s.z + 0.5 * dt * k2.z,
    };
    let k3 = deriv(s3, sigma, rho, beta);
    let s4 = State {
        x: s.x + dt * k3.x,
        y: s.y + dt * k3.y,
        z: s.z + dt * k3.z,
    };
    let k4 = deriv(s4, sigma, rho, beta);
    State {
        x: s.x + dt / 6.0 * (k1.x + 2.0 * k2.x + 2.0 * k3.x + k4.x),
        y: s.y + dt / 6.0 * (k1.y + 2.0 * k2.y + 2.0 * k3.y + k4.y),
        z: s.z + dt / 6.0 * (k1.z + 2.0 * k2.z + 2.0 * k3.z + k4.z),
    }
}

// ── Integrate one trajectory ─────────────────────────────────────────────────────────────────────

fn integrate(rho: f64, ic_jitter: f64) -> (Vec<f64>, Vec<f64>) {
    const SIGMA: f64 = 10.0;
    const BETA: f64 = 8.0 / 3.0;
    const DT: f64 = 0.005;
    const STEPS: usize = 80_000;
    const DISCARD: usize = 5_000;

    let mut s = State {
        x: 1.0 + ic_jitter,
        y: 1.0 + ic_jitter,
        z: 1.0 + ic_jitter,
    };

    // burn-in: discard transient
    for _ in 0..DISCARD {
        s = rk4(s, DT, SIGMA, rho, BETA);
    }

    let keep = STEPS - DISCARD;
    let mut xs = Vec::with_capacity(keep);
    let mut zs = Vec::with_capacity(keep);

    for _ in 0..keep {
        xs.push(s.x);
        zs.push(s.z); // x–z projection = the butterfly
        s = rk4(s, DT, SIGMA, rho, BETA);
    }
    (xs, zs)
}

// ── main ─────────────────────────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    // let rho_values: &[f64] = &[13.0, 15.0, 18.0, 21.0, 24.06, 28.0, 35.0, 50.0, 100.0, 160.0, 250.0];
    // series above if you want that instead
    let rho_values: &[f64] = &[28.];
    let n = rho_values.len() as f64;

    let mut fig = Figure::new(1600, 1000);
    fig = fig
        .title("Lorenz attractor family — x–z projection")
        .x_label("x")
        .y_label("z");

    for (i, &rho) in rho_values.iter().enumerate() {
        let jitter = 0.001 * i as f64;
        let (x_data, z_data) = integrate(rho, jitter);

        // map trajectory index to [0, 1] for inferno colormap
        let t = i as f64 / (n - 1.0);
        // only valid for series
        let c: prismatica::Color = prismatica::matplotlib::INFERNO.eval(t as f32);

        fig = fig.add(LineMark {
            x: x_data,
            y: z_data,
            color: Color::from(c),
            width: 0.5,
        });
    }

    fig.save("examples/showcases/lorenz_line.png")?;
    println!("saved lorenz_line.png");
    Ok(())
}
