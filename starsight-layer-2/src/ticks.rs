//! Tick generation: the Wilkinson Extended algorithm.
//!
//! `wilkinson_extended` chooses tick positions that score well on simplicity,
//! coverage, density, and legibility. Weights are `0.2, 0.25, 0.5, 0.05`.
//! Reference: <https://vis.stanford.edu/files/2010-TickLabels-InfoVis.pdf>.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::float_cmp,
    clippy::many_single_char_names
)]

const Q: [f64; 6] = [1.0, 5.0, 2.0, 2.5, 4.0, 3.0];
const W: [f64; 4] = [0.25, 0.2, 0.5, 0.05];

// ── wilkinson_extended ───────────────────────────────────────────────────────────────────────────

/// Compute optimal tick positions for the data range `[dmin, dmax]`, targeting
/// roughly `m` ticks. When `only_loose` is true the chosen labels strictly bracket
/// the data; otherwise tighter ranges are also accepted.
#[must_use]
pub fn wilkinson_extended(mut dmin: f64, mut dmax: f64, m: usize, only_loose: bool) -> Vec<f64> {
    let eps = f64::EPSILON * 100.0;

    if dmin > dmax {
        std::mem::swap(&mut dmin, &mut dmax);
    }

    if dmax - dmin < eps || (dmax - dmin) > f64::MAX.sqrt() {
        return linspace(dmin, dmax, m);
    }

    let mut best_score = -2.0_f64;
    let mut best_lmin = 0.0;
    let mut best_lmax = 0.0;
    let mut best_lstep = 0.0;

    let mut j = 1.0;
    'outer: loop {
        for (i, &q) in Q.iter().enumerate() {
            let sm = simplicity_max(i, j);

            if W[0] * sm + W[1] + W[2] + W[3] < best_score {
                break 'outer;
            }

            let mut k = 2.0;
            loop {
                let dm = density_max(k, m as f64);

                if W[0] * sm + W[1] + W[2] * dm + W[3] < best_score {
                    break;
                }

                let delta = (dmax - dmin) / (k + 1.0) / j / q;
                let mut z = delta.log10().ceil() as i32;

                loop {
                    let step = j * q * 10.0_f64.powi(z);
                    let cm = coverage_max(dmin, dmax, step * (k - 1.0));

                    if W[0] * sm + W[1] * cm + W[2] * dm + W[3] < best_score {
                        break;
                    }

                    let min_start =
                        (dmax / step).floor() as i64 * j as i64 - (k as i64 - 1) * j as i64;
                    let max_start = (dmin / step).ceil() as i64 * j as i64;

                    if min_start > max_start {
                        z += 1;
                        continue;
                    }

                    for start in min_start..=max_start {
                        let lmin = start as f64 * (step / j);
                        let lmax = lmin + step * (k - 1.0);
                        let lstep = step;

                        let s = simplicity(i, j, lmin, lmax, lstep);
                        let c = coverage(dmin, dmax, lmin, lmax);
                        let g = density(k, m as f64, dmin, dmax, lmin, lmax);
                        let l = 1.0; // legibility

                        let score = W[0] * s + W[1] * c + W[2] * g + W[3] * l;

                        if score > best_score && (!only_loose || (lmin <= dmin && lmax >= dmax)) {
                            best_score = score;
                            best_lmin = lmin;
                            best_lmax = lmax;
                            best_lstep = lstep;
                        }
                    }

                    z += 1;
                }

                k += 1.0;
            }
        }

        j += 1.0;
    }

    linspace_step(best_lmin, best_lmax, best_lstep)
}

// ── helpers ──────────────────────────────────────────────────────────────────────────────────────

fn linspace(from: f64, to: f64, n: usize) -> Vec<f64> {
    if n <= 1 || (to - from).abs() < f64::EPSILON {
        return vec![from];
    }
    let step = (to - from) / (n - 1) as f64;
    (0..n).map(|i| from + i as f64 * step).collect()
}

fn linspace_step(from: f64, to: f64, step: f64) -> Vec<f64> {
    let n = ((to - from) / step).round() as usize + 1;
    (0..n).map(|i| from + i as f64 * step).collect()
}

fn simplicity(i: usize, j: f64, lmin: f64, lmax: f64, lstep: f64) -> f64 {
    let eps = f64::EPSILON;
    let n = Q.len();
    let v = if (lmin % lstep < eps || lstep - (lmin % lstep) < lstep) && lmin <= 0.0 && lmax >= 0.0
    {
        1.0
    } else {
        0.0
    };
    1. - (i as f64 - 1.) / (n as f64 - 1.) - j + v
}

fn simplicity_max(i: usize, j: f64) -> f64 {
    let n = Q.len();
    let v = 1.;
    1. - (i as f64 - 1.) / (n as f64 - 1.) - j + v
}

fn coverage(dmin: f64, dmax: f64, lmin: f64, lmax: f64) -> f64 {
    let range = dmax - dmin;
    1. - 0.5 * ((dmax - lmax).powi(2) + (dmin - lmin).powi(2)) / ((0.1 * range).powi(2))
}

fn coverage_max(dmin: f64, dmax: f64, span: f64) -> f64 {
    let range = dmax - dmin;
    if span > range {
        let half = (span - range) / 2.;
        1. - 0.5 * (half.powi(2) + half.powi(2)) / ((0.1 * range).powi(2))
    } else {
        1.
    }
}

fn density(k: f64, m: f64, dmin: f64, dmax: f64, lmin: f64, lmax: f64) -> f64 {
    let r = (k - 1.) / (lmax - lmin);
    let rt = (m - 1.) / (lmax.max(dmax) - dmin.min(lmin));
    2. - (r / rt).max(rt / r)
}

fn density_max(k: f64, m: f64) -> f64 {
    if k >= m { 2. - (k - 1.) / (m - 1.) } else { 1. }
}

// ── tests ────────────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::wilkinson_extended;

    #[test]
    fn ticks_0_to_100() {
        let ticks = wilkinson_extended(0.0, 100.0, 5, true);
        assert!(!ticks.is_empty());
        assert!(ticks[0] <= 0.0);
        assert!(*ticks.last().unwrap() >= 100.0);
    }

    #[test]
    fn ticks_0_to_1() {
        let ticks = wilkinson_extended(0.0, 1.0, 5, true);
        assert!(!ticks.is_empty());
        let step = ticks[1] - ticks[0];
        assert!(step > 0.0 && step <= 0.5);
    }

    #[test]
    fn ticks_negative_range() {
        let ticks = wilkinson_extended(-50.0, 50.0, 5, true);
        assert!(ticks[0] <= -50.0);
        assert!(*ticks.last().unwrap() >= 50.0);
    }

    #[test]
    fn ticks_zero_width() {
        let ticks = wilkinson_extended(42.0, 42.0, 5, true);
        assert_eq!(ticks, vec![42.0]);
    }

    #[test]
    fn ticks_swapped_input_is_normalized() {
        let normal = wilkinson_extended(0.0, 100.0, 5, true);
        let swapped = wilkinson_extended(100.0, 0.0, 5, true);
        assert_eq!(normal, swapped);
    }

    #[test]
    fn ticks_zero_count_returns_singleton() {
        // Triggers the `n <= 1` branch in linspace via the zero-width fast path.
        let ticks = wilkinson_extended(7.0, 7.0, 0, true);
        assert_eq!(ticks, vec![7.0]);
    }

    #[test]
    fn ticks_huge_range_uses_linspace() {
        // Range exceeds f64::MAX.sqrt() so linspace fallback triggers.
        let ticks = wilkinson_extended(-f64::MAX / 2.0, f64::MAX / 2.0, 5, true);
        assert!(!ticks.is_empty());
    }
}

use proptest::prelude::*;
proptest! {
    #[test]
    fn ticks_monotonic(min in -1e6f64..0.0, max in 0.1f64..1e6) {
        let ticks = wilkinson_extended(min, max, 5, true);
        for pair in ticks.windows(2) {
            prop_assert!(pair[0] < pair[1], "ticks not monotonic: {:?}", ticks);
        }
    }
}
