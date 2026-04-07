//! Statistical transforms: binning, density estimation, regression, aggregation.
//!
//! Each transform takes raw data and produces output data ready for a mark.
//! Status: stub. Implementations land in 0.2.0–0.5.0 behind the `stats` feature.

// ── Stat ─────────────────────────────────────────────────────────────────────────────────────────
// TODO(0.2.0): pub trait Stat { type Input; type Output;
//                  fn compute(&self, input: Self::Input) -> Self::Output; }

// ── Bin ──────────────────────────────────────────────────────────────────────────────────────────
// TODO(0.2.0): pub struct Bin { bins: usize, range: Option<(f64, f64)> }
//              -- histograms: count values per bin

// ── KDE ──────────────────────────────────────────────────────────────────────────────────────────
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

// ── Density ──────────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct Density { bandwidth: f64 }

// ── Smooth ───────────────────────────────────────────────────────────────────────────────────────
// TODO(0.5.0): pub struct Smooth { window: usize, method: Smoother }
