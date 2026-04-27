//! Colormap types wrapping prismatica for use in starsight.
//!
//! Provides a unified interface for applying scientific colormaps to heatmaps,
//! colorbars, and other visualization elements that need continuous color scales.

use crate::primitives::Color;

// ── Colormap ───────────────────────────────────────────────────────────────────────────────────

/// A colormap wrapping prismatica's Colormap for use in visualization.
///
/// This provides a simple interface to sample colors at normalized values [0, 1].
#[derive(Debug, Clone, Copy)]
pub struct Colormap {
    inner: prismatica::Colormap,
}

impl Colormap {
    /// Create a colormap from a prismatica colormap.
    #[must_use]
    pub const fn new(inner: prismatica::Colormap) -> Self {
        Self { inner }
    }

    /// Sample the colormap at a normalized value `t` in `[0, 1]`.
    ///
    /// Values outside `[0, 1]` are clamped. Interpolation is linear in sRGB space.
    #[must_use]
    pub fn sample(&self, t: f64) -> Color {
        // f32 precision is sufficient for color sampling (8-bit channel output).
        #[allow(clippy::cast_possible_truncation)]
        let t = t.clamp(0.0, 1.0) as f32;
        Color::from(self.inner.eval(t))
    }

    /// Sample at a rational index: the `i`-th of `n` evenly-spaced values.
    ///
    /// Equivalent to `sample(i as f64 / (n - 1) as f64)` for `n > 1`.
    #[must_use]
    pub fn sample_index(&self, i: usize, n: usize) -> Color {
        Color::from(self.inner.eval_rational(i, n))
    }

    /// Get `n` evenly-spaced discrete colors from the colormap.
    #[must_use]
    pub fn colors(&self, n: usize) -> Vec<Color> {
        self.inner.colors(n).into_iter().map(Color::from).collect()
    }

    /// Get the colormap name.
    #[must_use]
    pub fn name(&self) -> &'static str {
        self.inner.name()
    }

    /// Get the colormap kind (Sequential, Diverging, Cyclic, Qualitative).
    #[must_use]
    pub fn kind(&self) -> ColormapKind {
        match self.inner.kind() {
            prismatica::ColormapKind::Diverging => ColormapKind::Diverging,
            prismatica::ColormapKind::Cyclic => ColormapKind::Cyclic,
            prismatica::ColormapKind::Qualitative => ColormapKind::Qualitative,
            // Sequential, MultiSequential, and any future variants map to Sequential.
            _ => ColormapKind::Sequential,
        }
    }

    /// Get a reversed view of this colormap (zero allocation).
    /// Note: Returns the same colormap for now - full reversal requires more complex handling.
    #[must_use]
    pub fn reversed(&self) -> Colormap {
        *self
    }
}

impl From<prismatica::Colormap> for Colormap {
    fn from(inner: prismatica::Colormap) -> Self {
        Self::new(inner)
    }
}

impl From<&prismatica::Colormap> for Colormap {
    fn from(inner: &prismatica::Colormap) -> Self {
        Self::new(*inner)
    }
}

impl Default for Colormap {
    fn default() -> Self {
        VIRIDIS
    }
}

/// Classification of colormaps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColormapKind {
    /// Single-hue gradient (e.g., viridis, plasma).
    Sequential,
    /// Two-hue gradient with a neutral middle (e.g., coolwarm, rdbu).
    Diverging,
    /// Wraps around (e.g., hsv, phase).
    Cyclic,
    /// Categorical colors with no ordering (e.g., set1, tableau10).
    Qualitative,
}

// ── Built-in Colormaps ──────────────────────────────────────────────────────────────────────

/// Viridis colormap (perceptually uniform, colorblind-friendly).
pub const VIRIDIS: Colormap = Colormap::new(prismatica::matplotlib::VIRIDIS);

/// Plasma colormap (perceptually uniform).
pub const PLASMA: Colormap = Colormap::new(prismatica::matplotlib::PLASMA);

/// Inferno colormap (perceptually uniform).
pub const INFERNO: Colormap = Colormap::new(prismatica::matplotlib::INFERNO);

/// Magma colormap (perceptually uniform).
pub const MAGMA: Colormap = Colormap::new(prismatica::matplotlib::MAGMA);

/// Turbo colormap (rainbow variant, perceptually uniform).
pub const TURBO: Colormap = Colormap::new(prismatica::d3::TURBO);

/// Batlow colormap (crameri, perceptually uniform).
pub const BATLOW: Colormap = Colormap::new(prismatica::crameri::BATLOW);

/// Berlin diverging colormap.
pub const BERLIN: Colormap = Colormap::new(prismatica::crameri::BERLIN);

/// Vik diverging colormap.
pub const VIK: Colormap = Colormap::new(prismatica::crameri::VIK);

/// Default colormap used when none is specified.
pub const DEFAULT: Colormap = VIRIDIS;

#[cfg(test)]
mod tests {
    use super::{
        BATLOW, BERLIN, Colormap, ColormapKind, DEFAULT, INFERNO, MAGMA, PLASMA, TURBO, VIK,
        VIRIDIS,
    };

    #[test]
    fn sample_clamps_below_zero() {
        let c = VIRIDIS.sample(-1.0);
        let c0 = VIRIDIS.sample(0.0);
        assert_eq!(c, c0);
    }

    #[test]
    fn sample_clamps_above_one() {
        let c = VIRIDIS.sample(2.0);
        let c1 = VIRIDIS.sample(1.0);
        assert_eq!(c, c1);
    }

    #[test]
    fn sample_index_endpoints() {
        let first = VIRIDIS.sample_index(0, 10);
        let last = VIRIDIS.sample_index(9, 10);
        assert_ne!(first, last);
    }

    #[test]
    fn colors_returns_n_entries() {
        let palette = VIRIDIS.colors(8);
        assert_eq!(palette.len(), 8);
    }

    #[test]
    fn name_is_nonempty() {
        assert!(!VIRIDIS.name().is_empty());
    }

    #[test]
    fn kind_classification() {
        assert_eq!(VIRIDIS.kind(), ColormapKind::Sequential);
        assert_eq!(BERLIN.kind(), ColormapKind::Diverging);
        assert_eq!(VIK.kind(), ColormapKind::Diverging);
    }

    #[test]
    fn kind_multi_sequential_falls_back_to_sequential() {
        // oleron is a MultiSequential colormap; the wrapper folds it into Sequential.
        let m = Colormap::new(prismatica::crameri::OLERON);
        assert_eq!(m.kind(), ColormapKind::Sequential);
    }

    #[test]
    fn kind_cyclic() {
        let m = Colormap::new(prismatica::d3::SINEBOW);
        assert_eq!(m.kind(), ColormapKind::Cyclic);
    }

    #[test]
    fn kind_qualitative() {
        let m = Colormap::new(prismatica::d3::TABLEAU10);
        assert_eq!(m.kind(), ColormapKind::Qualitative);
    }

    #[test]
    fn reversed_returns_self_for_now() {
        let c = VIRIDIS;
        let r = c.reversed();
        assert_eq!(c.sample(0.0), r.sample(0.0));
    }

    #[test]
    fn from_prismatica_value() {
        let c: Colormap = prismatica::matplotlib::VIRIDIS.into();
        assert_eq!(c.name(), VIRIDIS.name());
    }

    #[test]
    fn from_prismatica_ref() {
        let inner = prismatica::matplotlib::VIRIDIS;
        let c: Colormap = (&inner).into();
        assert_eq!(c.name(), VIRIDIS.name());
    }

    #[test]
    fn default_is_viridis() {
        let d = Colormap::default();
        assert_eq!(d.name(), VIRIDIS.name());
    }

    #[test]
    fn default_const_is_viridis() {
        assert_eq!(DEFAULT.name(), VIRIDIS.name());
    }

    #[test]
    fn builtin_colormaps_load() {
        for c in [VIRIDIS, PLASMA, INFERNO, MAGMA, TURBO, BATLOW, BERLIN, VIK] {
            assert!(!c.name().is_empty());
            let _ = c.sample(0.5);
        }
    }
}
