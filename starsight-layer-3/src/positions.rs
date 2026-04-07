//! Position adjustments: resolve overlapping marks.
//!
//! When multiple marks share the same x position, position adjustments decide
//! whether to stack them, dodge them side by side, jitter them randomly, or
//! leave them at the original positions.
//!
//! Status: stub. Implementations land in 0.3.0+.

// ── Position ─────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub trait Position { fn adjust(&self, marks: &mut [Box<dyn Mark>]); }

// ── Stack ────────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct Stack { direction: Direction }
//              -- stacked bar charts, stacked area

// ── Dodge ────────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct Dodge { padding: f64 }
//              -- side-by-side grouped bars

// ── Jitter ───────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct Jitter { width: f64, height: f64, seed: u64 }
//              -- spread overlapping points randomly

// ── Identity ─────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct Identity;
//              -- no adjustment (default)

// ── Fill ─────────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct Fill;
//              -- normalize stack to 100%
