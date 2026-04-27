//! Axes: a [`Scale`](crate::scales::Scale) bundled with tick positions, labels, and an optional title.

use crate::scales::LinearScale;

// ── Axis ─────────────────────────────────────────────────────────────────────────────────────────

/// One chart axis: scale + ticks + tick labels + optional axis label.
pub struct Axis {
    /// The scale that maps data values to the normalized range.
    pub scale: LinearScale,
    /// Optional axis title displayed alongside the tick labels.
    pub label: Option<String>,
    /// Tick positions in data space.
    pub tick_positions: Vec<f64>,
    /// Pre-formatted tick labels, one per `tick_positions`.
    pub tick_labels: Vec<String>,
}

impl Axis {
    /// Build an axis whose ticks are chosen by the Wilkinson Extended algorithm.
    pub fn auto_from_data(values: &[f64], target_ticks: usize) -> Option<Self> {
        let dmin = values.iter().copied().fold(f64::INFINITY, f64::min);
        let dmax = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let ticks = crate::ticks::wilkinson_extended(dmin, dmax, target_ticks, true);
        let labels: Vec<String> = ticks.iter().map(|t| format!("{t}")).collect();
        Some(Self {
            scale: LinearScale {
                domain_min: ticks[0],
                domain_max: *ticks.last()?,
            },
            label: None,
            tick_positions: ticks,
            tick_labels: labels,
        })
    }

    /// Build a category axis covering exactly `[0, n]` for `n` labels, with
    /// tick positions at band edges so grid lines fall between categories.
    /// Tick labels are kept empty since renderers use the upstream category
    /// label list directly; the contract "one tick_label per tick_position"
    /// is preserved by aligning lengths.
    #[must_use]
    pub fn category(labels: &[String]) -> Self {
        let n = labels.len();
        Self {
            scale: LinearScale {
                domain_min: 0.0,
                domain_max: n as f64,
            },
            label: None,
            tick_positions: (0..=n).map(|i| i as f64).collect(),
            tick_labels: vec![String::new(); n + 1],
        }
    }
}
