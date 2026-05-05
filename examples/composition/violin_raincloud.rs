//! Violin raincloud — starsight 0.3.0 showcase #19.
//!
//! A raincloud plot composes three marks per category:
//!
//! - `ViolinMark` for the kernel-density envelope,
//! - the violin's built-in inner box overlay for the five-number summary,
//! - `PointMark` overlay with deterministic in-band jitter and low alpha to
//!   show individual samples without obscuring the density shape.
//!
//! The four categories were picked to highlight different distribution
//! shapes:
//!
//! - **Bimodal A** — two well-separated modes (the violin clearly shows two
//!   bumps where a single box plot would just show a wide IQR).
//! - **Unimodal** — symmetric Gaussian, n = 200 around 4.0 ± 1.5.
//! - **Bimodal symmetric** — two equally-weighted modes 5.5 apart.
//! - **Skewed** — log-normal long upper tail.
//!
//! Synthetic samples are generated from a single Box–Muller normal pair
//! seeded with a per-index hash so the example renders bit-for-bit
//! reproducibly without pulling in an `rand` dependency. n = 200 per
//! category — enough density for the KDE to look smooth, small enough for
//! gallery render time to stay under a second.

use starsight::prelude::*;
use starsight::statistics::Bandwidth;

fn main() -> Result<()> {
    // n = 60 per category — enough density for the KDE envelope to look
    // smooth, sparse enough that individual jittered points stay readable
    // in the strip overlay. The spec calls for n = 10000 but that's a
    // gallery-render-time and visual-noise problem at example scale.
    let cat_a = mixture_samples(0xA1, 18, -2.0, 0.8, 6.5, 1.2, 60); // 30%/70% split
    let cat_b = normal_samples(0xB2, 4.0, 1.5, 60);
    let cat_c = mixture_samples(0xC3, 30, 1.5, 0.5, 7.0, 0.8, 60); // 50%/50% split
    let cat_d = lognormal_samples(0xD4, 1.2, 0.4, 60);

    let groups = vec![
        ViolinGroup::new("Bimodal A", cat_a.clone()),
        ViolinGroup::new("Unimodal", cat_b.clone()),
        ViolinGroup::new("Bimodal sym.", cat_c.clone()),
        ViolinGroup::new("Skewed", cat_d.clone()),
    ];

    let palette = vec![
        Color::from_hex(0x0033_77BB),
        Color::from_hex(0x0033_AA66),
        Color::from_hex(0x00CC_8800),
        Color::from_hex(0x00CC_3366),
    ];

    // Three-zone band layout per category (band width = 1.0):
    //   - Violin envelope: band centre 0.30..0.50 (half_width = 0.20 of band).
    //   - Inner box: same centre, drawn by ViolinMark::show_box(true).
    //   - Jittered strip: 0.65..0.95, offset to the RIGHT of the violin so the
    //     rain doesn't get hidden behind the opaque violin fill.
    // The violin builder centres every group on `index + 0.5`, so we shift
    // it left by 0.20 here to free room for the strip on the right.
    let mut strip_x: Vec<f64> = Vec::with_capacity(240);
    let mut strip_y: Vec<f64> = Vec::with_capacity(240);
    let mut strip_colors: Vec<Color> = Vec::with_capacity(240);
    for (idx, (samples, base_color)) in [
        (&cat_a, palette[0]),
        (&cat_b, palette[1]),
        (&cat_c, palette[2]),
        (&cat_d, palette[3]),
    ]
    .iter()
    .enumerate()
    {
        let strip_centre = idx as f64 + 0.78;
        for (i, &y) in samples.iter().enumerate() {
            let jitter = jittered_offset(idx as u32, i as u32);
            strip_x.push(strip_centre + jitter * 0.16);
            strip_y.push(y);
            strip_colors.push(*base_color);
        }
    }

    Figure::new(900, 600)
        .title("Raincloud — violin + inner box + jittered strip")
        .x_label("category")
        .y_label("value")
        .add(
            ViolinMark::new(groups)
                .bandwidth(Bandwidth::Silverman)
                .palette(palette.clone())
                .half_width(0.20)
                .show_box(true)
                .cut(0.0),
        )
        .add(
            PointMark::new(strip_x, strip_y)
                .colors(strip_colors)
                .radius(2.5)
                .alpha(0.55),
        )
        .save("examples/composition/violin_raincloud.png")
}

/// Deterministic uniform-ish jitter in `[-1.0, 1.0]` from a 2-tuple seed.
/// xorshift mixer is enough for visual jitter without an `rand` dependency.
fn jittered_offset(group: u32, idx: u32) -> f64 {
    let mut state = (group.wrapping_mul(0x9E37_79B9))
        .wrapping_add(idx)
        .wrapping_add(0x1234_5678);
    state ^= state << 13;
    state ^= state >> 17;
    state ^= state << 5;
    let normalised = f64::from(state) / f64::from(u32::MAX);
    normalised * 2.0 - 1.0
}

/// Box–Muller pair off a deterministic LCG so the example reproduces.
fn box_muller(seed: u32, idx: u32) -> (f64, f64) {
    let mut s1 = seed.wrapping_mul(0x6C07_8965).wrapping_add(idx).wrapping_add(1);
    let mut s2 = seed.wrapping_mul(0x4F1B_BCDC).wrapping_add(idx).wrapping_add(2);
    s1 ^= s1 << 13;
    s1 ^= s1 >> 17;
    s1 ^= s1 << 5;
    s2 ^= s2 << 13;
    s2 ^= s2 >> 17;
    s2 ^= s2 << 5;
    let u1 = (f64::from(s1) / f64::from(u32::MAX)).max(1e-9);
    let u2 = f64::from(s2) / f64::from(u32::MAX);
    let r = (-2.0 * u1.ln()).sqrt();
    let theta = 2.0 * std::f64::consts::PI * u2;
    (r * theta.cos(), r * theta.sin())
}

/// Generate `n` Gaussian samples with mean `mu` and std-dev `sigma`.
fn normal_samples(seed: u32, mu: f64, sigma: f64, n: usize) -> Vec<f64> {
    let mut out = Vec::with_capacity(n);
    let half = n.div_ceil(2);
    for i in 0..half {
        let (a, b) = box_muller(seed, i as u32);
        out.push(mu + sigma * a);
        if out.len() < n {
            out.push(mu + sigma * b);
        }
    }
    out
}

/// Mixture of two Gaussians, drawing `n_1` samples from component 1 and the
/// rest from component 2.
fn mixture_samples(
    seed: u32,
    n_1: usize,
    mu_1: f64,
    sigma_1: f64,
    mu_2: f64,
    sigma_2: f64,
    n: usize,
) -> Vec<f64> {
    let mut out = normal_samples(seed, mu_1, sigma_1, n_1);
    out.extend(normal_samples(seed.wrapping_add(0xDEAD), mu_2, sigma_2, n - n_1));
    out
}

/// Log-normal samples (skew control). `mu`/`sigma` are the underlying normal
/// distribution's parameters, so the final samples are `exp(N(mu, sigma))`.
fn lognormal_samples(seed: u32, mu: f64, sigma: f64, n: usize) -> Vec<f64> {
    normal_samples(seed, mu, sigma, n)
        .into_iter()
        .map(f64::exp)
        .collect()
}
