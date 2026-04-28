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

#[cfg(test)]
mod tests {
    use super::Axis;

    #[test]
    fn category_axis_preserves_invariants() {
        let labels: Vec<String> = ["A", "B", "C"].iter().map(|s| (*s).to_string()).collect();
        let axis = Axis::category(&labels);
        assert_eq!(axis.scale.domain_min, 0.0);
        assert_eq!(axis.scale.domain_max, 3.0);
        assert_eq!(axis.tick_positions, vec![0.0, 1.0, 2.0, 3.0]);
        assert_eq!(axis.tick_labels.len(), axis.tick_positions.len());
        assert!(axis.tick_labels.iter().all(String::is_empty));
    }

    #[test]
    #[should_panic(expected = "Axis::category requires at least one label")]
    fn category_axis_panics_on_empty_labels() {
        let _ = Axis::category(&[]);
    }
}
