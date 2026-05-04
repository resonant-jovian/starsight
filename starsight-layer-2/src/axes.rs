//! Axes: a [`Scale`](crate::scales::Scale) bundled with tick positions, labels, and an optional title.

use crate::scales::{CategoricalScale, LinearScale, LogScale, Scale, SqrtScale};

// ── Axis ─────────────────────────────────────────────────────────────────────────────────────────

/// One chart axis: scale + ticks + tick labels + optional axis label.
///
/// `scale` is a `Box<dyn Scale>` so the same `Axis` type can carry linear /
/// log / sqrt / categorical mappings — required by polar radial axes
/// (`Nightingale` wants sqrt) and log heatmap color bars.
#[derive(Clone)]
pub struct Axis {
    /// The scale that maps data values to the normalized range.
    pub scale: Box<dyn Scale>,
    /// Optional axis title displayed alongside the tick labels.
    pub label: Option<String>,
    /// Tick positions in data space.
    pub tick_positions: Vec<f64>,
    /// Pre-formatted tick labels, one per `tick_positions`.
    pub tick_labels: Vec<String>,
}

impl Axis {
    /// Build an axis whose ticks are chosen by the Wilkinson Extended algorithm.
    ///
    /// Applies a 5% inset on both ends of the scale so points / bars at the
    /// data extremes don't visually sit on the plot edge — matches
    /// matplotlib's default `margins(0.05)`. Tracked as `starsight-3bp.9.1`
    /// (Epic I.1). Categorical axes (`Axis::category`) skip this inset
    /// because band-edge ticks are intentional.
    pub fn auto_from_data(values: &[f64], target_ticks: usize) -> Option<Self> {
        let dmin = values.iter().copied().fold(f64::INFINITY, f64::min);
        let dmax = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        if !dmin.is_finite() || !dmax.is_finite() {
            return None;
        }
        let ticks = crate::ticks::wilkinson_extended(dmin, dmax, target_ticks, true);
        let labels: Vec<String> = ticks.iter().map(|t| format!("{t}")).collect();
        let raw_min = ticks[0];
        let raw_max = *ticks.last()?;
        let pad = (raw_max - raw_min).max(f64::EPSILON) * 0.05;
        Some(Self {
            scale: Box::new(LinearScale {
                domain_min: raw_min - pad,
                domain_max: raw_max + pad,
            }),
            label: None,
            tick_positions: ticks,
            tick_labels: labels,
        })
    }

    /// Build a category axis covering exactly `[0, n]` for `n` labels, with
    /// tick positions at band edges so grid lines fall between categories.
    ///
    /// # Invariants
    ///
    /// - `tick_positions.len() == labels.len() + 1`. Positions land at the band
    ///   edges (0, 1, …, n), and tick labels are always empty strings; the
    ///   "one `tick_label` per `tick_position`" contract is preserved by
    ///   aligning lengths, not by writing the category names into them.
    /// - Bar-style marks bypass [`scale`](Self::scale) on a category axis and
    ///   compute band-center positions directly with
    ///   `area.left + (i as f32 + 0.5) * band_width`. Iterating
    ///   `tick_labels` to recover category names will yield empty strings —
    ///   read the upstream `Vec<String>` that produced this axis instead.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `labels` is empty. With no categories the
    /// scale degenerates to `[0, 0]` and bars collapse to the plot midpoint;
    /// callers should gate construction on a non-empty list.
    #[must_use]
    pub fn category(labels: &[String]) -> Self {
        debug_assert!(
            !labels.is_empty(),
            "Axis::category requires at least one label",
        );
        let n = labels.len();
        Self {
            scale: Box::new(LinearScale {
                domain_min: 0.0,
                domain_max: n as f64,
            }),
            label: None,
            tick_positions: (0..=n).map(|i| i as f64).collect(),
            tick_labels: vec![String::new(); n + 1],
        }
    }

    /// Linear angular axis spanning `[domain_min, domain_max]`. The data range
    /// maps to a full `2π` sweep through `theta_axis.scale`. Pass
    /// `(0.0, 360.0)` for degrees, `(0.0, std::f64::consts::TAU)` for
    /// radians, or any other range that suits the user's data.
    ///
    /// Wraps around the disk: a value at `domain_max` lands at the same angle
    /// as one at `domain_min`. Callers that want a partial sweep (e.g.
    /// `Gauge` covering only 270°) should construct the axis manually so the
    /// scale's normalized range stays inside `[0.0, 0.75]`.
    #[must_use]
    pub fn polar_angular(domain_min: f64, domain_max: f64) -> Self {
        Self {
            scale: Box::new(LinearScale {
                domain_min,
                domain_max,
            }),
            label: None,
            tick_positions: vec![],
            tick_labels: vec![],
        }
    }

    /// Categorical angular axis: `n` evenly spaced compass-bin / month /
    /// category positions sweeping the disk. Index `i` lands at the
    /// band-center angle `(i + 0.5) / n * 2π`. Backs Nightingale (12 months),
    /// wind rose (16 directions), polar bar plots in general.
    #[must_use]
    pub fn polar_angular_categorical(n: usize) -> Self {
        Self {
            scale: Box::new(CategoricalScale { n_categories: n }),
            label: None,
            tick_positions: vec![],
            tick_labels: vec![],
        }
    }

    /// Linear radial axis spanning `[domain_min, domain_max]`. The range maps
    /// linearly to `[0, radius]` pixel-space. Suits gauges, bar height /
    /// fraction, and most radar / spider charts.
    #[must_use]
    pub fn polar_radial(domain_min: f64, domain_max: f64) -> Self {
        Self {
            scale: Box::new(LinearScale {
                domain_min,
                domain_max,
            }),
            label: None,
            tick_positions: vec![],
            tick_labels: vec![],
        }
    }

    /// Sqrt radial axis: `r ∝ √value` so slice area is proportional to value.
    /// Backs Nightingale's coxcomb invariant (Florence Nightingale's original
    /// design intent). `domain_min` must be ≥ 0.
    #[must_use]
    pub fn polar_radial_sqrt(domain_min: f64, domain_max: f64) -> Self {
        Self {
            scale: Box::new(SqrtScale {
                domain_min,
                domain_max,
            }),
            label: None,
            tick_positions: vec![],
            tick_labels: vec![],
        }
    }

    /// Log radial axis: `r ∝ log(value)`. Compresses wide value ranges onto a
    /// single disk. Both endpoints must be > 0.
    #[must_use]
    pub fn polar_radial_log(domain_min: f64, domain_max: f64) -> Self {
        Self {
            scale: Box::new(LogScale {
                domain_min,
                domain_max,
            }),
            label: None,
            tick_positions: vec![],
            tick_labels: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Axis;

    #[test]
    fn category_axis_preserves_invariants() {
        let labels: Vec<String> = ["A", "B", "C"].iter().map(|s| (*s).to_string()).collect();
        let axis = Axis::category(&labels);
        // Behavior: scale maps [0, 3] → [0, 1].
        assert!((axis.scale.map(0.0) - 0.0).abs() < 1e-9);
        assert!((axis.scale.map(3.0) - 1.0).abs() < 1e-9);
        assert_eq!(axis.tick_positions, vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(axis.tick_labels.len(), axis.tick_positions.len());
        assert!(axis.tick_labels.iter().all(String::is_empty));
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "Axis::category requires at least one label")]
    fn category_axis_panics_on_empty_labels() {
        let _ = Axis::category(&[]);
    }

    #[test]
    fn polar_angular_maps_full_sweep() {
        let a = Axis::polar_angular(0.0, 360.0);
        assert!((a.scale.map(0.0) - 0.0).abs() < 1e-9);
        assert!((a.scale.map(180.0) - 0.5).abs() < 1e-9);
        assert!((a.scale.map(360.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn polar_angular_categorical_band_centers() {
        let a = Axis::polar_angular_categorical(12);
        // Month 0 lands at the center of its 1/12 band.
        assert!((a.scale.map(0.0) - 0.5 / 12.0).abs() < 1e-9);
        assert!((a.scale.map(6.0) - 6.5 / 12.0).abs() < 1e-9);
    }

    #[test]
    fn polar_radial_sqrt_quarter_at_half() {
        let a = Axis::polar_radial_sqrt(0.0, 100.0);
        // Nightingale invariant: value 25 maps to r at 0.5 of the disk.
        assert!((a.scale.map(25.0) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn polar_radial_log_decade_endpoints() {
        let a = Axis::polar_radial_log(1.0, 100.0);
        assert!((a.scale.map(1.0) - 0.0).abs() < 1e-9);
        assert!((a.scale.map(10.0) - 0.5).abs() < 1e-9);
        assert!((a.scale.map(100.0) - 1.0).abs() < 1e-9);
    }
}
