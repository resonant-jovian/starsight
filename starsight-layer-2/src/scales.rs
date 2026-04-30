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

/// Logarithmic mapping. Both `domain_min` and `domain_max` must be positive
/// and non-equal; values outside the domain extrapolate (`map` does not clamp).
///
/// The base of the log cancels in normalization, so internally `ln` is used —
/// the user's expected base only matters for tick label formatting (handled by
/// the axis layer, not the scale).
pub struct LogScale {
    /// Lower bound (must be > 0).
    pub domain_min: f64,
    /// Upper bound (must be > 0 and != `domain_min`).
    pub domain_max: f64,
}

impl Scale for LogScale {
    fn map(&self, value: f64) -> f64 {
        let lmin = self.domain_min.ln();
        let lmax = self.domain_max.ln();
        if (lmax - lmin).abs() < f64::EPSILON {
            return 0.5;
        }
        (value.ln() - lmin) / (lmax - lmin)
    }

    fn inverse(&self, normalized: f64) -> f64 {
        let lmin = self.domain_min.ln();
        let lmax = self.domain_max.ln();
        (lmin + normalized * (lmax - lmin)).exp()
    }
}

// ── SqrtScale ────────────────────────────────────────────────────────────────────────────────────

/// Square-root mapping. Useful when an area encodes value (Nightingale's
/// `r ∝ √v` makes the rendered slice area proportional to value, not to r).
/// `domain_min` and `domain_max` must be ≥ 0.
pub struct SqrtScale {
    /// Lower bound (must be ≥ 0).
    pub domain_min: f64,
    /// Upper bound (must be ≥ 0 and > `domain_min`).
    pub domain_max: f64,
}

impl Scale for SqrtScale {
    fn map(&self, value: f64) -> f64 {
        let smin = self.domain_min.sqrt();
        let smax = self.domain_max.sqrt();
        if (smax - smin).abs() < f64::EPSILON {
            return 0.5;
        }
        (value.sqrt() - smin) / (smax - smin)
    }

    fn inverse(&self, normalized: f64) -> f64 {
        let smin = self.domain_min.sqrt();
        let smax = self.domain_max.sqrt();
        let s = smin + normalized * (smax - smin);
        s * s
    }
}

// ── CategoricalScale ─────────────────────────────────────────────────────────────────────────────

/// Maps a category index (0..n) to its band-center position in `[0, 1]`.
///
/// Symmetric across band centers: index `i` maps to `(i + 0.5) / n`. This is
/// what polar bars / coxcombs / wind-rose marks want — the angular center of
/// each compass bin / month / category. For Cartesian band layouts use
/// [`Axis::category`](crate::axes::Axis::category) instead, which keeps tick
/// positions at band edges.
pub struct CategoricalScale {
    /// Total number of categories. Zero produces a degenerate midpoint scale.
    pub n_categories: usize,
}

impl Scale for CategoricalScale {
    fn map(&self, value: f64) -> f64 {
        if self.n_categories == 0 {
            return 0.5;
        }
        (value + 0.5) / self.n_categories as f64
    }

    fn inverse(&self, normalized: f64) -> f64 {
        if self.n_categories == 0 {
            return 0.0;
        }
        normalized * self.n_categories as f64 - 0.5
    }
}

// ── SymLogScale ──────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct SymLogScale { pub domain_min: f64, pub domain_max: f64, pub linthresh: f64 }

// ── BandScale ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct BandScale { pub categories: Vec<String>, pub padding: f64 }

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

    #[test]
    fn log_scale_endpoints_and_midpoint() {
        let s = super::LogScale {
            domain_min: 1.0,
            domain_max: 100.0,
        };
        assert!((s.map(1.0) - 0.0).abs() < 1e-9);
        assert!((s.map(100.0) - 1.0).abs() < 1e-9);
        // log10(10) is halfway between log10(1)=0 and log10(100)=2
        assert!((s.map(10.0) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn log_scale_inverse_round_trip() {
        let s = super::LogScale {
            domain_min: 1e-3,
            domain_max: 1e3,
        };
        for v in [1e-3, 1e-2, 1.0, 1e2, 1e3] {
            let n = s.map(v);
            let v2 = s.inverse(n);
            assert!(
                (v - v2).abs() / v.abs() < 1e-9,
                "log round trip on {v}: got {v2}"
            );
        }
    }

    #[test]
    fn sqrt_scale_quarter_value_at_half() {
        // Nightingale invariant: r ∝ √v, so r at 0.5 means area at 0.25.
        let s = super::SqrtScale {
            domain_min: 0.0,
            domain_max: 100.0,
        };
        assert!((s.map(25.0) - 0.5).abs() < 1e-9);
        assert!((s.map(0.0) - 0.0).abs() < 1e-9);
        assert!((s.map(100.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn sqrt_scale_inverse_round_trip() {
        let s = super::SqrtScale {
            domain_min: 0.0,
            domain_max: 81.0,
        };
        for v in [0.0_f64, 1.0, 16.0, 25.0, 49.0, 81.0] {
            let n = s.map(v);
            let v2 = s.inverse(n);
            assert!((v - v2).abs() < 1e-6, "sqrt round trip on {v}: got {v2}");
        }
    }

    #[test]
    fn categorical_scale_band_centers() {
        // 12-month wheel: month i lands at (i + 0.5) / 12.
        let s = super::CategoricalScale { n_categories: 12 };
        assert!((s.map(0.0) - (0.5 / 12.0)).abs() < 1e-12);
        assert!((s.map(11.0) - (11.5 / 12.0)).abs() < 1e-12);
        // Midpoint of the band that month 6 occupies.
        assert!((s.map(6.0) - (6.5 / 12.0)).abs() < 1e-12);
    }

    #[test]
    fn categorical_scale_zero_is_midpoint() {
        let s = super::CategoricalScale { n_categories: 0 };
        assert_eq!(s.map(0.0), 0.5);
        assert_eq!(s.map(7.0), 0.5);
    }
}
