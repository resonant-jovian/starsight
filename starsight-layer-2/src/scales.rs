//! Scales: map data values onto normalized `[0, 1]` ranges.
//!
//! Every scale implements the [`Scale`] trait. The simplest is [`LinearScale`];
//! later milestones add `LogScale`, `SymLogScale`, `BandScale`, `DateTimeScale`.

// ── Scale ────────────────────────────────────────────────────────────────────────────────────────

/// A monotonic mapping from a data domain to the normalized `[0, 1]` range.
pub trait Scale {
    /// Map a data value to its normalized position.
    fn map(&self, value: f64) -> f64;
    /// Inverse: map a normalized position back to data space.
    fn inverse(&self, normalized: f64) -> f64;
}

// ── LinearScale ──────────────────────────────────────────────────────────────────────────────────

/// Linear (`y = ax + b`) mapping.
pub struct LinearScale {
    /// Lower bound of the data domain.
    pub domain_min: f64,
    /// Upper bound of the data domain.
    pub domain_max: f64,
}

impl Scale for LinearScale {
    fn map(&self, value: f64) -> f64 {
        if (self.domain_max - self.domain_min).abs() < f64::EPSILON {
            return 0.5;
        }
        (value - self.domain_min) / (self.domain_max - self.domain_min)
    }

    fn inverse(&self, normalized: f64) -> f64 {
        normalized * (self.domain_max - self.domain_min) + self.domain_min
    }
}

// ── LogScale ─────────────────────────────────────────────────────────────────────────────────────
// TODO(0.2.0): pub struct LogScale { pub domain_min: f64, pub domain_max: f64, pub base: f64 }

// ── SymLogScale ──────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct SymLogScale { pub domain_min: f64, pub domain_max: f64, pub linthresh: f64 }

// ── BandScale ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.2.0): pub struct BandScale { pub categories: Vec<String>, pub padding: f64 }

// ── DateTimeScale ────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct DateTimeScale { pub domain: (Timestamp, Timestamp) }

// ── tests ────────────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::Scale;

    #[test]
    fn linear_scale_map() {
        let scale = super::LinearScale {
            domain_min: 0.0,
            domain_max: 10.0,
        };
        assert_eq!(scale.map(5.0), 0.5);
        assert_eq!(scale.map(0.0), 0.0);
        assert_eq!(scale.map(10.0), 1.0);
    }

    #[test]
    fn linear_scale_inverse() {
        let scale = super::LinearScale {
            domain_min: 0.0,
            domain_max: 10.0,
        };
        assert_eq!(scale.inverse(0.5), 5.0);
        assert_eq!(scale.inverse(0.0), 0.0);
        assert_eq!(scale.inverse(1.0), 10.0);
    }

    #[test]
    fn linear_scale_inverse_clamped() {
        let scale = super::LinearScale {
            domain_min: 0.0,
            domain_max: 10.0,
        };
        assert_eq!(scale.inverse(-0.5), -5.0);
        assert_eq!(scale.inverse(1.5), 15.0);
    }

    #[test]
    fn linear_scale_zero_domain_maps_to_midpoint() {
        let scale = super::LinearScale {
            domain_min: 5.0,
            domain_max: 5.0,
        };
        assert_eq!(scale.map(5.0), 0.5);
        assert_eq!(scale.map(0.0), 0.5);
    }
}
