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

/// Linear-interpolation percentile of `data` at probability `p ∈ [0, 1]`.
///
/// Returns `NaN` for empty input. Sorts a copy of the slice; callers that
/// already have sorted data are encouraged to inline the lerp themselves to
/// skip the allocation.
#[must_use]
pub fn percentile(data: &[f64], p: f64) -> f64 {
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

/// Population standard deviation of `data`. Returns `0.0` for empty input or
/// a single sample (no spread to measure).
#[must_use]
pub fn std_dev(data: &[f64]) -> f64 {
    let n = data.len();
    if n < 2 {
        return 0.0;
    }
    let mean = data.iter().sum::<f64>() / n as f64;
    let var = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    var.sqrt()
}

/// Silverman's rule-of-thumb bandwidth for a 1-D Gaussian KDE:
///   `h = 0.9 · min(σ, IQR/1.34) · n^(-1/5)`
///
/// Returns `0.0` when the data has no spread (constant sample); the caller
/// should treat that as "use a tiny manual bandwidth instead of dividing by
/// zero downstream."
#[must_use]
pub fn silverman_bandwidth(data: &[f64]) -> f64 {
    let n = data.len();
    if n < 2 {
        return 0.0;
    }
    let sigma = std_dev(data);
    let q1 = percentile(data, 0.25);
    let q3 = percentile(data, 0.75);
    let iqr = q3 - q1;
    let spread = if iqr > 0.0 {
        sigma.min(iqr / 1.34)
    } else {
        sigma
    };
    if spread <= 0.0 {
        return 0.0;
    }
    0.9 * spread * (n as f64).powf(-0.2)
}

/// Scott's rule bandwidth for a 1-D Gaussian KDE:
///   `h = 1.06 · σ · n^(-1/5)`
#[must_use]
pub fn scott_bandwidth(data: &[f64]) -> f64 {
    let n = data.len();
    if n < 2 {
        return 0.0;
    }
    let sigma = std_dev(data);
    if sigma <= 0.0 {
        return 0.0;
    }
    1.06 * sigma * (n as f64).powf(-0.2)
}

// ── BoxPlotStats ─────────────────────────────────────────────────────────────────────────────────

/// Five-number summary plus 1.5×IQR Tukey outliers — the data shape `BoxPlotMark`
/// renders and `ViolinMark`'s inner-box overlay reuses.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxPlotStats {
    /// Smallest non-outlier value (≥ Q1 − 1.5·IQR).
    pub min: f64,
    /// 25th-percentile value.
    pub q1: f64,
    /// 50th-percentile value (the median line in the box body).
    pub median: f64,
    /// 75th-percentile value.
    pub q3: f64,
    /// Largest non-outlier value (≤ Q3 + 1.5·IQR).
    pub max: f64,
    /// Points that fall beyond `[min, max]`. Plotted as individual dots.
    pub outliers: Vec<f64>,
}

impl BoxPlotStats {
    /// Compute the five-number summary and outlier set from a non-empty slice.
    /// NaN values are filtered out. Returns a degenerate summary
    /// `(min=q1=median=q3=max=0.0, outliers empty)` for an empty post-filter
    /// slice so callers can render a blank slot without panicking.
    #[must_use]
    pub fn compute(data: &[f64]) -> Self {
        let filtered: Vec<f64> = data.iter().copied().filter(|v| !v.is_nan()).collect();
        if filtered.is_empty() {
            return Self {
                min: 0.0,
                q1: 0.0,
                median: 0.0,
                q3: 0.0,
                max: 0.0,
                outliers: Vec::new(),
            };
        }
        let q1 = percentile(&filtered, 0.25);
        let median = percentile(&filtered, 0.50);
        let q3 = percentile(&filtered, 0.75);
        let iqr = q3 - q1;
        let lower_fence = q1 - 1.5 * iqr;
        let upper_fence = q3 + 1.5 * iqr;
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut outliers = Vec::new();
        for &v in &filtered {
            if v < lower_fence || v > upper_fence {
                outliers.push(v);
            } else {
                if v < min {
                    min = v;
                }
                if v > max {
                    max = v;
                }
            }
        }
        if !min.is_finite() {
            // Every value is an outlier (degenerate IQR=0 with values either
            // side). Fall back to the q1/q3 envelope so the box still has a
            // sensible whisker range.
            min = q1;
            max = q3;
        }
        Self {
            min,
            q1,
            median,
            q3,
            max,
            outliers,
        }
    }
}

// ── Kde ──────────────────────────────────────────────────────────────────────────────────────────

/// Kernel function for [`Kde`]. Currently only Gaussian; left as an enum so
/// future kernels (Epanechnikov, Triangular, …) slot in without an API break.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Kernel {
    /// Standard normal kernel: `K(u) = (1/√(2π)) · exp(−u²/2)`.
    #[default]
    Gaussian,
}

/// Bandwidth selection strategy for [`Kde`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum Bandwidth {
    /// Silverman's rule-of-thumb. Robust default; works well unimodal-ish data.
    #[default]
    Silverman,
    /// Scott's rule. Slightly wider than Silverman, more forgiving on tails.
    Scott,
    /// Caller-supplied bandwidth `h` in data units. Useful when the
    /// auto-selected value oversmooths (use a smaller `h`) or undersmooths.
    Manual(f64),
}

/// 1-D kernel density estimator. Used by `ViolinMark` to build mirrored
/// density paths and by histogram-overlay recipes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Kde {
    /// Bandwidth selection strategy.
    pub bandwidth: Bandwidth,
    /// Kernel function.
    pub kernel: Kernel,
}

impl Kde {
    /// New KDE configured with the given bandwidth strategy and kernel.
    #[must_use]
    pub fn new(bandwidth: Bandwidth, kernel: Kernel) -> Self {
        Self { bandwidth, kernel }
    }

    /// Resolve the configured bandwidth strategy against the actual sample
    /// data. The returned value is in data units; pair it with
    /// [`evaluate_at`](Self::evaluate_at) and friends.
    #[must_use]
    pub fn resolve_bandwidth(&self, data: &[f64]) -> f64 {
        match self.bandwidth {
            Bandwidth::Silverman => silverman_bandwidth(data),
            Bandwidth::Scott => scott_bandwidth(data),
            Bandwidth::Manual(h) => h,
        }
    }

    /// Density at a single point `y`, given the sample `data`. Returns `0.0`
    /// when the data has no spread (so caller paths still close cleanly even
    /// when fed a constant sample).
    #[must_use]
    pub fn evaluate_at(&self, y: f64, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let h = self.resolve_bandwidth(data);
        if h <= 0.0 {
            return 0.0;
        }
        let n = data.len() as f64;
        let inv_nh = 1.0 / (n * h);
        match self.kernel {
            Kernel::Gaussian => {
                // Inline Gaussian PDF avoids one allocation per call vs. statrs
                // and is hot-loop friendly when ViolinMark sweeps a 256-point
                // grid per group.
                const INV_SQRT_2PI: f64 = 0.398_942_280_401_433;
                let mut sum = 0.0;
                for &xi in data {
                    let u = (y - xi) / h;
                    sum += INV_SQRT_2PI * (-0.5 * u * u).exp();
                }
                inv_nh * sum
            }
        }
    }

    /// Density at each `y` in `points`, given the sample `data`. Allocates a
    /// fresh `Vec<f64>`; for tight loops keep one buffer and call
    /// [`evaluate_at`](Self::evaluate_at) directly.
    #[must_use]
    pub fn evaluate_grid(&self, points: &[f64], data: &[f64]) -> Vec<f64> {
        points.iter().map(|&y| self.evaluate_at(y, data)).collect()
    }
}

// ── Grid / Polyline / Contour ────────────────────────────────────────────────────────────────────

/// Polyline in data coordinates, tagged with the contour level it belongs to.
///
/// The 0.3.0 [`Contour`] extractor emits one `Polyline` per cell crossing —
/// each carries exactly two points (a single line segment). `level` lets a
/// `ContourMark` colormap or label each band. Future versions may merge
/// adjacent segments into longer chains; consumers should not assume a fixed
/// number of points.
#[derive(Clone, Debug, PartialEq)]
pub struct Polyline {
    /// Ordered points in data space.
    pub points: Vec<(f64, f64)>,
    /// Contour level this polyline belongs to.
    pub level: f64,
}

/// Regular 2-D scalar grid backing [`Contour::compute`].
///
/// `values` is row-major (`values[i * nx + j]` = value at column `j`, row
/// `i`), with row 0 at `y_min` and the last row at `y_max`. `nx`/`ny` are
/// vertex counts, not cell counts — at least `2 × 2` to produce any cells.
#[derive(Clone, Debug)]
pub struct Grid {
    /// Row-major scalar values; `len() == nx * ny`.
    pub values: Vec<f64>,
    /// Vertex count along x.
    pub nx: usize,
    /// Vertex count along y.
    pub ny: usize,
    /// Domain x lower bound (column 0).
    pub x_min: f64,
    /// Domain x upper bound (column `nx - 1`).
    pub x_max: f64,
    /// Domain y lower bound (row 0).
    pub y_min: f64,
    /// Domain y upper bound (row `ny - 1`).
    pub y_max: f64,
}

impl Grid {
    /// Build a grid from a row-major value buffer plus domain bounds.
    ///
    /// # Errors
    ///
    /// Returns `None` if `values.len() != nx * ny` or if `nx < 2 || ny < 2`
    /// (no cells).
    #[must_use]
    pub fn new(
        values: Vec<f64>,
        nx: usize,
        ny: usize,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
    ) -> Option<Self> {
        if nx < 2 || ny < 2 || values.len() != nx * ny {
            return None;
        }
        Some(Self {
            values,
            nx,
            ny,
            x_min,
            x_max,
            y_min,
            y_max,
        })
    }

    /// Sample a regular grid by evaluating `f(x, y)` over the rectangular
    /// domain. Convenience for tests and examples.
    #[must_use]
    pub fn sample<F: Fn(f64, f64) -> f64>(
        nx: usize,
        ny: usize,
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        f: F,
    ) -> Self {
        let dx = if nx > 1 {
            (x_max - x_min) / (nx - 1) as f64
        } else {
            0.0
        };
        let dy = if ny > 1 {
            (y_max - y_min) / (ny - 1) as f64
        } else {
            0.0
        };
        let mut values = Vec::with_capacity(nx * ny);
        for i in 0..ny {
            let y = y_min + i as f64 * dy;
            for j in 0..nx {
                let x = x_min + j as f64 * dx;
                values.push(f(x, y));
            }
        }
        Self {
            values,
            nx,
            ny,
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    fn at(&self, j: usize, i: usize) -> f64 {
        self.values[i * self.nx + j]
    }
}

/// Marching-squares contour extractor.
///
/// `Contour::compute(grid, &levels)` emits one [`Polyline`] per cell crossing
/// per level, using the standard marching-squares 16-case lookup with
/// average-of-corners saddle disambiguation (matplotlib default). Each
/// resulting polyline carries exactly two points — a single segment in cell
/// coordinates — which `ContourMark` strokes for isolines or pairs across
/// levels for filled bands.
pub struct Contour;

impl Contour {
    /// Extract contour segments from `grid` at each level in `levels`.
    ///
    /// Output is unordered — segments from one level interleave with
    /// segments from another. Filter by [`Polyline::level`] if a single
    /// level's segments are needed in isolation.
    #[must_use]
    #[allow(clippy::many_single_char_names)] // t, u, v, x, y are math-vocab in marching squares.
    pub fn compute(grid: &Grid, levels: &[f64]) -> Vec<Polyline> {
        if grid.nx < 2 || grid.ny < 2 || levels.is_empty() {
            return Vec::new();
        }
        let dx = (grid.x_max - grid.x_min) / (grid.nx - 1) as f64;
        let dy = (grid.y_max - grid.y_min) / (grid.ny - 1) as f64;
        let mut out = Vec::new();
        for &level in levels {
            for cell_i in 0..(grid.ny - 1) {
                for cell_j in 0..(grid.nx - 1) {
                    let v0 = grid.at(cell_j, cell_i); // BL
                    let v1 = grid.at(cell_j + 1, cell_i); // BR
                    let v2 = grid.at(cell_j + 1, cell_i + 1); // TR
                    let v3 = grid.at(cell_j, cell_i + 1); // TL
                    if !(v0.is_finite() && v1.is_finite() && v2.is_finite() && v3.is_finite()) {
                        continue;
                    }
                    let mut mask = 0u8;
                    if v0 > level {
                        mask |= 1;
                    }
                    if v1 > level {
                        mask |= 2;
                    }
                    if v2 > level {
                        mask |= 4;
                    }
                    if v3 > level {
                        mask |= 8;
                    }
                    if mask == 0 || mask == 15 {
                        continue;
                    }
                    let edge_pt = |e: u8| -> (f64, f64) {
                        // Local cell coords (u, v) in [0, 1]².
                        let (a_v, b_v, a_uv, b_uv) = match e {
                            0 => (v0, v1, (0.0, 0.0), (1.0, 0.0)),
                            1 => (v1, v2, (1.0, 0.0), (1.0, 1.0)),
                            2 => (v2, v3, (1.0, 1.0), (0.0, 1.0)),
                            3 => (v3, v0, (0.0, 1.0), (0.0, 0.0)),
                            _ => unreachable!(),
                        };
                        let denom = b_v - a_v;
                        let t = if denom.abs() < f64::EPSILON {
                            0.5
                        } else {
                            ((level - a_v) / denom).clamp(0.0, 1.0)
                        };
                        let u = a_uv.0 + t * (b_uv.0 - a_uv.0);
                        let v = a_uv.1 + t * (b_uv.1 - a_uv.1);
                        // Convert local (u, v) to data coords.
                        let x = grid.x_min + (cell_j as f64 + u) * dx;
                        let y = grid.y_min + (cell_i as f64 + v) * dy;
                        (x, y)
                    };
                    let segments: &[(u8, u8)] = match mask {
                        1 | 14 => &[(3, 0)],
                        2 | 13 => &[(0, 1)],
                        3 | 12 => &[(3, 1)],
                        4 | 11 => &[(1, 2)],
                        6 | 9 => &[(0, 2)],
                        7 | 8 => &[(2, 3)],
                        5 => {
                            // Corners 0,2 above; saddle. avg > level → contour
                            // wraps the two below corners (1 and 3) separately;
                            // avg < level → contour wraps the two above corners
                            // (0 and 2) separately.
                            let avg = (v0 + v1 + v2 + v3) * 0.25;
                            if avg > level {
                                &[(0, 1), (2, 3)]
                            } else {
                                &[(3, 0), (1, 2)]
                            }
                        }
                        10 => {
                            // Corners 1,3 above; saddle.
                            let avg = (v0 + v1 + v2 + v3) * 0.25;
                            if avg > level {
                                &[(3, 0), (1, 2)]
                            } else {
                                &[(0, 1), (2, 3)]
                            }
                        }
                        _ => &[],
                    };
                    for &(ea, eb) in segments {
                        let pa = edge_pt(ea);
                        let pb = edge_pt(eb);
                        out.push(Polyline {
                            points: vec![pa, pb],
                            level,
                        });
                    }
                }
            }
        }
        out
    }
}

// ── Regression ───────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct Regression { kind: RegressionKind, ci: Option<f64> }
//              -- linear, polynomial, loess fits

// ── Aggregate ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct Aggregate { group_by: String, agg: Aggregation }
//              -- group-and-summarize

// ── Smooth ───────────────────────────────────────────────────────────────────────────────────────
// TODO(0.5.0): pub struct Smooth { window: usize, method: Smoother }

#[cfg(test)]
mod tests {
    use super::{
        Bandwidth, Bin, BinMethod, BinTransform, BoxPlotStats, Contour, Grid, Kde, Kernel,
        percentile, scott_bandwidth, silverman_bandwidth, std_dev,
    };

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
        let data: Vec<f64> = (0..100).map(f64::from).collect();
        let method = BinMethod::Width(10.0);
        let count = method.bin_count(&data);
        assert_eq!(count, 10);
    }

    #[test]
    fn bin_method_freedman_diaconis() {
        let data: Vec<f64> = (0..100).map(f64::from).collect();
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
        let data: Vec<f64> = (0..100).map(f64::from).collect();
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
        let bins = [
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
        let data: Vec<f64> = (0..20).map(f64::from).collect();
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
            assert_eq!(method.bin_count(&[]), 1, "{method:?}");
        }
    }

    // ── std_dev ──────────────────────────────────────────────────────────

    #[test]
    fn std_dev_empty_or_single_is_zero() {
        assert_eq!(std_dev(&[]), 0.0);
        assert_eq!(std_dev(&[42.0]), 0.0);
    }

    #[test]
    fn std_dev_known_sample() {
        // Population std of [1..=5] = sqrt(2) ≈ 1.4142.
        let s = std_dev(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((s - 2.0_f64.sqrt()).abs() < 1e-12);
    }

    // ── bandwidth helpers ────────────────────────────────────────────────

    #[test]
    fn silverman_bandwidth_constant_data_returns_zero() {
        assert_eq!(silverman_bandwidth(&[5.0; 10]), 0.0);
    }

    #[test]
    fn silverman_bandwidth_matches_reference() {
        // For [1..=10]: σ ≈ 2.872; IQR = 4.5 → IQR/1.34 ≈ 3.358 → spread = σ.
        // h = 0.9 · 2.872 · 10^(-1/5) ≈ 1.6312.
        let data: Vec<f64> = (1..=10).map(f64::from).collect();
        let h = silverman_bandwidth(&data);
        assert!((h - 1.631_2).abs() < 1e-3);
    }

    #[test]
    fn scott_bandwidth_matches_reference() {
        // Scott on [1..=10]: 1.06 · σ · n^(-1/5) ≈ 1.92.
        let data: Vec<f64> = (1..=10).map(f64::from).collect();
        let h = scott_bandwidth(&data);
        assert!((h - 1.921_2).abs() < 1e-3);
    }

    // ── BoxPlotStats ─────────────────────────────────────────────────────

    #[test]
    fn boxplot_stats_textbook_example() {
        // Symmetric data with a single high outlier. q1=2.5, median=4.5,
        // q3=6.5, IQR=4 → fences [-3.5, 12.5], so 20.0 is the outlier.
        let stats = BoxPlotStats::compute(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 20.0]);
        assert!((stats.q1 - 3.25).abs() < 1e-9);
        assert!((stats.median - 5.5).abs() < 1e-9);
        assert!((stats.q3 - 7.75).abs() < 1e-9);
        assert!((stats.min - 1.0).abs() < 1e-9);
        assert!((stats.max - 9.0).abs() < 1e-9);
        assert_eq!(stats.outliers, vec![20.0]);
    }

    #[test]
    fn boxplot_stats_empty_is_degenerate() {
        let stats = BoxPlotStats::compute(&[]);
        assert_eq!(stats.q1, 0.0);
        assert_eq!(stats.median, 0.0);
        assert_eq!(stats.q3, 0.0);
        assert!(stats.outliers.is_empty());
    }

    #[test]
    fn boxplot_stats_filters_nan() {
        let stats = BoxPlotStats::compute(&[1.0, f64::NAN, 2.0, 3.0]);
        assert!((stats.median - 2.0).abs() < 1e-9);
        assert!(stats.outliers.is_empty());
    }

    // ── Kde ──────────────────────────────────────────────────────────────

    #[test]
    fn kde_evaluate_at_constant_data_returns_zero() {
        let kde = Kde::new(Bandwidth::Silverman, Kernel::Gaussian);
        assert_eq!(kde.evaluate_at(1.0, &[1.0; 5]), 0.0);
    }

    #[test]
    fn kde_evaluate_at_known_density() {
        // KDE of standard-normal-shaped sample should peak near zero. With a
        // small synthetic sample [-2, -1, 0, 1, 2] and Silverman bandwidth, the
        // density at 0 should be greater than at ±2.
        let data = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        let kde = Kde::new(Bandwidth::Silverman, Kernel::Gaussian);
        let center = kde.evaluate_at(0.0, &data);
        let edge = kde.evaluate_at(2.0, &data);
        assert!(center > edge, "center {center} should exceed edge {edge}");
    }

    #[test]
    fn kde_evaluate_grid_matches_pointwise() {
        let data = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let kde = Kde::new(Bandwidth::Manual(0.5), Kernel::Gaussian);
        let points = vec![-1.0, 0.5, 2.0, 3.5];
        let grid = kde.evaluate_grid(&points, &data);
        for (g, &y) in grid.iter().zip(&points) {
            assert!((*g - kde.evaluate_at(y, &data)).abs() < 1e-12);
        }
    }

    // ── Contour ──────────────────────────────────────────────────────────

    #[test]
    fn grid_new_rejects_size_mismatch() {
        assert!(Grid::new(vec![0.0; 3], 2, 2, 0.0, 1.0, 0.0, 1.0).is_none());
    }

    #[test]
    fn grid_new_rejects_too_small() {
        // 1x2 has no cells; 2x1 likewise.
        assert!(Grid::new(vec![0.0; 2], 1, 2, 0.0, 1.0, 0.0, 1.0).is_none());
        assert!(Grid::new(vec![0.0; 2], 2, 1, 0.0, 1.0, 0.0, 1.0).is_none());
    }

    #[test]
    fn grid_sample_evaluates_at_corners() {
        // f(x,y) = x; with 2 columns at x=0, x=1, values should be 0,1,0,1.
        let g = Grid::sample(2, 2, 0.0, 1.0, 0.0, 1.0, |x, _| x);
        assert_eq!(g.values, vec![0.0, 1.0, 0.0, 1.0]);
    }

    #[test]
    fn contour_empty_levels_returns_empty() {
        let g = Grid::sample(4, 4, 0.0, 1.0, 0.0, 1.0, |x, y| x + y);
        assert!(Contour::compute(&g, &[]).is_empty());
    }

    #[test]
    fn contour_plane_x_at_level_05_cuts_vertically() {
        // f(x,y) = x; level 0.5 should produce horizontal-ish line segments
        // at x ≈ 0.5 across all rows. With a 5x5 grid, expect 4 vertically
        // stacked segments (one per cell row).
        let g = Grid::sample(5, 5, 0.0, 1.0, 0.0, 1.0, |x, _| x);
        let polys = Contour::compute(&g, &[0.5]);
        // 4 cells per row × 4 rows = 16 cells; only the column whose range
        // straddles 0.5 will produce a segment. With 5 grid points x = 0,
        // 0.25, 0.5, 0.75, 1.0 — the cell j=1 (x in [0.25, 0.5]) and j=2
        // (x in [0.5, 0.75]) both touch 0.5 because the level matches a
        // grid line. matplotlib accepts that. Just check the segments lie
        // on the right vertical band.
        assert!(!polys.is_empty(), "expected contour segments at level 0.5");
        for p in &polys {
            assert_eq!(p.points.len(), 2);
            assert_eq!(p.level, 0.5);
            for &(x, _) in &p.points {
                assert!(
                    (0.25..=0.75).contains(&x),
                    "segment x out of expected band: {x}"
                );
            }
        }
    }

    #[test]
    fn contour_saddle_emits_two_segments_per_saddle_cell() {
        // Single 2×2 grid (one cell). Row-major order is [BL, BR, TL, TR]
        // because row i=0 is at y_min, row i=1 at y_max. To trigger saddle
        // case 5 (corners 0=BL and 2=TR above; corners 1=BR and 3=TL below),
        // we need BL/TR > level and BR/TL < level. Use 1.5/1.0/0.0/0.0 in
        // row-major so avg = 0.625 > 0.5 → saddle "above" branch.
        let values = vec![1.5, 0.0, 0.0, 1.0]; // BL, BR, TL, TR
        let g = Grid::new(values, 2, 2, 0.0, 1.0, 0.0, 1.0).unwrap();
        let polys = Contour::compute(&g, &[0.5]);
        // Saddle case 5 emits 2 segments (one per below-corner wrap).
        assert_eq!(polys.len(), 2);
        for p in &polys {
            assert_eq!(p.points.len(), 2);
            assert_eq!(p.level, 0.5);
        }
    }

    #[test]
    fn contour_skips_cells_with_nan() {
        let mut values = vec![0.0; 9];
        values[4] = f64::NAN; // center vertex of 3x3 grid
        let g = Grid::new(values, 3, 3, 0.0, 1.0, 0.0, 1.0).unwrap();
        // Levels far from any vertex value — should still skip NaN cells.
        let polys = Contour::compute(&g, &[10.0]);
        assert!(polys.is_empty());
    }

    #[test]
    fn contour_uniform_grid_emits_nothing() {
        // No crossing anywhere.
        let g = Grid::sample(5, 5, 0.0, 1.0, 0.0, 1.0, |_, _| 1.0);
        assert!(Contour::compute(&g, &[0.5]).is_empty());
    }
}
