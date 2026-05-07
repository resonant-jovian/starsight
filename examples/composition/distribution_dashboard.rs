//! Multi-panel distribution dashboard — starsight 0.3.0 showcase #2.
//!
//! Four views of the same synthetic player-rating dataset stitched into a
//! 2×2 [`MultiPanelFigure`]. Each panel highlights a different aspect of
//! the distribution that a single chart would obscure:
//!
//! - **Top-left**: histogram of `overall` (20 bins). Shows the asymmetric
//!   peak around 70 — the body of typical players.
//! - **Top-right**: smoothed kernel-density of the same `overall` series,
//!   for comparison with the histogram bars. Drawn as an `AreaMark` from a
//!   manually-evaluated `Kde` so the example exercises the public stat API.
//! - **Bottom-left**: `BoxPlotMark` per country — five-number summary +
//!   Tukey-fence outliers across 6 categorical groups.
//! - **Bottom-right**: `PointMark` scatter of `overall` vs `potential`,
//!   showing the strong positive correlation plus the upward bias for
//!   younger / underdeveloped players.
//!
//! Spec calls for `n = 18 000` samples drawn from a Beta(5, 3) marginal;
//! we ship `n = 600` here — enough for the histogram, KDE, and box plots
//! to look smooth, small enough that the scatter doesn't blur into a wall.
//! Random draws come from a deterministic xorshift LCG so the gallery PNG
//! reproduces bit-for-bit without an `rand` dependency.

#![allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]

use starsight::common::MultiPanelFigure;
use starsight::prelude::*;
use starsight::statistics::{Bandwidth, Kde, Kernel};

const N: usize = 600;
const COUNTRIES: [&str; 6] = ["BRA", "ARG", "ENG", "ESP", "GER", "FRA"];

fn main() -> Result<()> {
    // Per-country mean shift: every country has the same Beta(5, 3) shape
    // but a different mean, so the BoxPlotMark panel reads as 6 distinct
    // distributions instead of 6 identical ones.
    let country_means: [f64; 6] = [4.0, 1.5, -0.5, 2.0, 0.0, -2.0];
    let country_idx: Vec<usize> = (0..N).map(|i| (i * 7 + 3) % COUNTRIES.len()).collect();
    let overall: Vec<f64> = (0..N)
        .map(|i| {
            let u = beta_int_sample(0xCAFE, i as u32, 5, 3);
            // Beta(5, 3) → mean ≈ 0.625; map [0, 1] → [45, 92] then add the
            // per-country shift so panel C shows real spread.
            45.0 + u * (92.0 - 45.0) + country_means[country_idx[i]]
        })
        .collect();
    let potential: Vec<f64> = overall
        .iter()
        .enumerate()
        .map(|(i, o)| {
            // Bounded headroom: Beta(2, 5) mean ≈ 0.286, so the player gains
            // roughly 30% of the gap to 92 plus a small noise term. The
            // original spec's Exp(0.08) is unbounded and pushes ratings past
            // the natural ceiling.
            let bonus_factor = beta_int_sample(0xBEEF, i as u32, 2, 5);
            let bonus = bonus_factor * (92.0 - o);
            let noise = normal_one(0xF00D, i as u32, 0.0, 2.0);
            (o + bonus + noise).clamp(45.0, 99.0)
        })
        .collect();

    let palette = vec![
        Color::from_hex(0x0033_77BB),
        Color::from_hex(0x0033_AA66),
        Color::from_hex(0x00CC_8800),
        Color::from_hex(0x00CC_3366),
        Color::from_hex(0x0088_3399),
        Color::from_hex(0x0011_AABB),
    ];

    // Panel A — histogram of overall ratings
    let panel_a = Figure::new(500, 400)
        .theme(theme_from_env())
        .title("A — overall rating, histogram")
        .x_label("overall")
        .y_label("count")
        .add(
            HistogramMark::new(overall.clone())
                .method(BinMethod::Count(20))
                .color(palette[0]),
        );

    // Panel B — KDE of overall ratings, sampled on a 200-point grid
    let kde = Kde::new(Bandwidth::Silverman, Kernel::Gaussian);
    let grid_min = overall.iter().copied().fold(f64::INFINITY, f64::min) - 2.0;
    let grid_max = overall.iter().copied().fold(f64::NEG_INFINITY, f64::max) + 2.0;
    let grid: Vec<f64> = (0..200)
        .map(|i| grid_min + (grid_max - grid_min) * f64::from(i) / 199.0)
        .collect();
    let density = kde.evaluate_grid(&grid, &overall);
    let panel_b = Figure::new(500, 400)
        .theme(theme_from_env())
        .title("B — overall rating, KDE")
        .x_label("overall")
        .y_label("density")
        .add(AreaMark::new(grid, density).color(palette[1]).opacity(0.55));

    // Panel C — box plot of overall, grouped by country
    let mut by_country: Vec<Vec<f64>> = vec![Vec::new(); COUNTRIES.len()];
    for (i, &v) in overall.iter().enumerate() {
        by_country[country_idx[i]].push(v);
    }
    let groups: Vec<BoxPlotGroup> = COUNTRIES
        .iter()
        .zip(by_country)
        .map(|(label, data)| BoxPlotGroup::new(*label, data))
        .collect();
    let panel_c = Figure::new(500, 400)
        .theme(theme_from_env())
        .title("C — overall by country")
        .x_label("country")
        .y_label("overall")
        .add(BoxPlotMark::new(groups).palette(palette.clone()));

    // Panel D — scatter overall vs potential (correlation + bias)
    let panel_d = Figure::new(500, 400)
        .theme(theme_from_env())
        .title("D — overall vs potential")
        .x_label("overall")
        .y_label("potential")
        .add(
            PointMark::new(overall.clone(), potential)
                .color(palette[2])
                .radius(2.0)
                .alpha(0.45),
        );

    MultiPanelFigure::new(1100, 850, 2, 2)
        .theme(theme_from_env())
        .padding(16.0)
        .add(panel_a)
        .add(panel_b)
        .add(panel_c)
        .add(panel_d)
        .save(format!(
            "examples/composition/distribution_dashboard{}.{}",
            theme_suffix_from_env(),
            format_extension_from_env()
        ))
}

// ── Synthetic distributions, all deterministic ─────────────────────────────

/// Splitmix64 finalizer — the canonical hash for turning a counter into a
/// well-distributed pseudo-random `u64`. Rust's `std::hash` defaults pull in
/// `SipHash` which we'd rather not bundle into a dependency-free example, so
/// the splitmix variant lives inline.
fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Hash a (seed, idx, salt) triple into a uniform `(0, 1)` draw. The bits
/// are shifted into independent slots of the u64 input so different triples
/// produce uncorrelated outputs even when the components only differ by 1.
fn uniform(seed: u32, idx: u32, salt: u32) -> f64 {
    let mixed = splitmix(u64::from(seed) | (u64::from(idx) << 21) | (u64::from(salt) << 42));
    let frac = (mixed >> 11) as f64 / (1u64 << 53) as f64;
    frac.clamp(1e-12, 1.0 - 1e-12)
}

/// One Box–Muller draw at index `idx` for the requested mean / std-dev.
fn normal_one(seed: u32, idx: u32, mu: f64, sigma: f64) -> f64 {
    let u1 = uniform(seed, idx, 0x1111_1111);
    let u2 = uniform(seed, idx, 0x2222_2222);
    mu + sigma * (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
}

/// Sum-of-exponentials Gamma sample for integer shape `k`. Gamma(k, 1) is
/// the sum of `k` independent Exp(1) draws, and `−ln(U)` for `U ~ U(0, 1)`
/// is one Exp(1). This is the canonical Gamma method when the shape is a
/// known positive integer.
fn gamma_int(seed: u32, idx: u32, salt: u32, k: usize) -> f64 {
    (0..k)
        .map(|j| -uniform(seed, idx, salt.wrapping_add(j as u32)).ln())
        .sum()
}

/// Beta(a, b) via X / (X + Y) where X ~ Gamma(a, 1) and Y ~ Gamma(b, 1).
/// Works for any positive integer shape pair, unlike Johnk's method which
/// only behaves at small shape parameters.
fn beta_int_sample(seed: u32, idx: u32, a: usize, b: usize) -> f64 {
    let x = gamma_int(seed, idx, 0x4444_4444, a);
    let y = gamma_int(seed, idx, 0x5555_5555, b);
    x / (x + y)
}
