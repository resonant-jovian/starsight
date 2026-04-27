//! Statistical transforms: binning, density estimation, regression, aggregation.
//!
//! Each transform takes raw data and produces output data ready for a mark.
//! Status: Implementation for 0.2.0 includes `BinMethod`, Bin, `BinTransform`.

#![allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]

// ── BinMethod ─────────────────────────────────────────────────────────────────────────────────────

/// Method for automatic bin count selection.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BinMethod {
    #[default]
    /// Sturges' rule: `ceil(1 + log2(n))` bins.
    Sturges,
    /// Freedman-Diaconis: `2 * IQR * n^(-1/3)` bin width.
    FreedmanDiaconis,
    /// Scott's rule: `3.5 * std * n^(-1/3)` bin width.
    Scott,
    /// Exact number of bins.
    Count(usize),
    /// Exact bin width.
    Width(f64),
}

impl BinMethod {
    /// Compute the number of bins for the given data.
    pub fn bin_count(&self, data: &[f64]) -> usize {
        let n = data.len();
        if n == 0 {
            return 1;
        }
        match self {
            Self::Count(k) => *k,
            Self::Sturges => (1.0 + (n as f64).log2()).ceil() as usize,
            Self::FreedmanDiaconis => {
                let q1 = percentile(data, 0.25);
                let q3 = percentile(data, 0.75);
                let iqr = q3 - q1;
                if iqr <= 0.0 {
                    return Self::Sturges.bin_count(data);
                }
                let h = 2.0 * iqr * (n as f64).powf(-1.0 / 3.0);
                let range = data.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                    - data.iter().copied().fold(f64::INFINITY, f64::min);
                (range / h).ceil() as usize
            }
            Self::Scott => {
                let mean = data.iter().sum::<f64>() / n as f64;
                let var = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;
                let std = var.sqrt();
                if std <= 0.0 {
                    return Self::Sturges.bin_count(data);
                }
                let h = 3.5 * std * (n as f64).powf(-1.0 / 3.0);
                let range = data.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                    - data.iter().copied().fold(f64::INFINITY, f64::min);
                (range / h).ceil() as usize
            }
            Self::Width(w) => {
                let range = data.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                    - data.iter().copied().fold(f64::INFINITY, f64::min);
                if *w > 0.0 {
                    (range / w).ceil() as usize
                } else {
                    1
                }
            }
        }
    }
}

// ── Bin ────────────────────────────────────────────────────────────────────────────────────────

/// A single histogram bin.
#[derive(Debug, Clone, PartialEq)]
pub struct Bin {
    /// Left edge of the bin (inclusive).
    pub left: f64,
    /// Right edge of the bin (exclusive).
    pub right: f64,
    /// Number of data points in the bin.
    pub count: usize,
}

// ── BinTransform ────────────────────────────────────────────────────────────────────────

/// Transform that bins raw data into histogram-ready bins.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BinTransform {
    /// Bin calculation method.
    pub method: BinMethod,
}

impl BinTransform {
    /// Create a new bin transform with the given method.
    #[must_use]
    pub fn new(method: BinMethod) -> Self {
        Self { method }
    }

    /// Compute bins from the given data.
    #[must_use]
    pub fn compute(&self, data: &[f64]) -> Vec<Bin> {
        if data.is_empty() {
            return vec![Bin {
                left: 0.0,
                right: 1.0,
                count: 0,
            }];
        }

        let k = self.method.bin_count(data);
        let dmin = data.iter().copied().fold(f64::INFINITY, f64::min);
        let dmax = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);

        if (dmax - dmin).abs() < f64::EPSILON {
            return vec![Bin {
                left: dmin - 0.5,
                right: dmax + 0.5,
                count: data.len(),
            }];
        }

        let width = (dmax - dmin) / k as f64;

        let mut bins = Vec::with_capacity(k);
        for i in 0..k {
            bins.push(Bin {
                left: dmin + i as f64 * width,
                right: dmin + (i + 1) as f64 * width,
                count: 0,
            });
        }

        for &v in data {
            let idx = ((v - dmin) / width).floor() as usize;
            let idx = idx.min(k - 1);
            bins[idx].count += 1;
        }

        bins
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────────────────────────

fn percentile(data: &[f64], p: f64) -> f64 {
    if data.is_empty() {
        return f64::NAN;
    }
    let mut sorted: Vec<f64> = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = (p * (sorted.len() - 1) as f64).floor() as usize;
    let frac = p * (sorted.len() - 1) as f64 - idx as f64;
    if idx + 1 >= sorted.len() {
        return sorted[idx];
    }
    sorted[idx] * (1.0 - frac) + sorted[idx + 1] * frac
}

// ── KDE ──────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct KDE { bandwidth: f64, kernel: Kernel }
//              -- kernel density estimation

// ── Boxplot ──────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct Boxplot { whisker_method: Whisker }
//              -- five-number summary

// ── Regression ───────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct Regression { kind: RegressionKind, ci: Option<f64> }
//              -- linear, polynomial, loess fits

// ── Aggregate ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct Aggregate { group_by: String, agg: Aggregation }
//              -- group-and-summarize

// ── Density ──────────���──────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct Density { bandwidth: f64 }

// ── Smooth ───────────────────────────────────────────────────────────────────────────────────────
// TODO(0.5.0): pub struct Smooth { window: usize, method: Smoother }

#[cfg(test)]
mod tests {
    use super::{Bin, BinMethod, BinTransform, percentile};

    #[test]
    fn bin_transform_new() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let transform = BinTransform::new(BinMethod::Count(5));
        let bins = transform.compute(&data);
        assert_eq!(bins.len(), 5);
    }

    #[test]
    fn bin_transform_empty() {
        let data = vec![];
        let transform = BinTransform::new(BinMethod::default());
        let bins = transform.compute(&data);
        assert_eq!(bins.len(), 1);
    }

    #[test]
    fn bin_method_count() {
        let data = vec![1.0; 100];
        assert_eq!(BinMethod::Count(10).bin_count(&data), 10);
    }

    #[test]
    fn bin_method_width() {
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let method = BinMethod::Width(10.0);
        let count = method.bin_count(&data);
        assert_eq!(count, 10);
    }

    #[test]
    fn bin_method_freedman_diaconis() {
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let method = BinMethod::FreedmanDiaconis;
        let count = method.bin_count(&data);
        assert!(count > 0);
    }

    #[test]
    fn bin_method_freedman_diaconis_equal_iqr() {
        let data = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let method = BinMethod::FreedmanDiaconis;
        let count = method.bin_count(&data);
        assert!(count > 0);
    }

    #[test]
    fn bin_method_scott() {
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let method = BinMethod::Scott;
        let count = method.bin_count(&data);
        assert!(count > 0);
    }

    #[test]
    fn bin_method_scott_zero_std() {
        let data = vec![5.0, 5.0, 5.0, 5.0, 5.0];
        let method = BinMethod::Scott;
        let count = method.bin_count(&data);
        assert!(count > 0);
    }

    #[test]
    fn bin_method_width_zero_range() {
        let data = vec![5.0, 5.0, 5.0];
        let method = BinMethod::Width(1.0);
        let count = method.bin_count(&data);
        assert_eq!(count, 0);
    }

    #[test]
    fn bin_method_width_negative() {
        let data = vec![1.0, 2.0, 3.0];
        let method = BinMethod::Width(-1.0);
        let count = method.bin_count(&data);
        assert_eq!(count, 1);
    }

    #[test]
    fn bin_values() {
        let bins = vec![
            Bin {
                left: 0.0,
                right: 10.0,
                count: 5,
            },
            Bin {
                left: 10.0,
                right: 20.0,
                count: 3,
            },
        ];
        assert_eq!(bins[0].count, 5);
        assert_eq!(bins[1].left, 10.0);
    }

    #[test]
    fn bin_transform_compute_even_distribution() {
        let data: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let transform = BinTransform::new(BinMethod::Count(4));
        let bins = transform.compute(&data);
        assert_eq!(bins.len(), 4);
        assert_eq!(bins[0].count, 5);
    }

    #[test]
    fn bin_transform_single_value() {
        let data = vec![5.0, 5.0, 5.0];
        let transform = BinTransform::new(BinMethod::Count(1));
        let bins = transform.compute(&data);
        assert_eq!(bins[0].count, 3);
    }

    #[test]
    fn percentile_empty() {
        let result = percentile(&[], 0.5);
        assert!(result.is_nan());
    }

    #[test]
    fn percentile_median() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let p = percentile(&data, 0.5);
        assert!((p - 3.0).abs() < 0.01);
    }

    #[test]
    fn percentile_100() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let p = percentile(&data, 1.0);
        assert!((p - 5.0).abs() < 0.01);
    }

    #[test]
    fn bin_count_empty_returns_one() {
        for method in [
            BinMethod::Sturges,
            BinMethod::FreedmanDiaconis,
            BinMethod::Scott,
            BinMethod::Count(5),
            BinMethod::Width(1.0),
        ] {
            assert_eq!(method.bin_count(&[]), 1, "{:?}", method);
        }
    }
}
