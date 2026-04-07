//! Chart-type auto-inference: pick the right mark from raw data shape.
//!
//! When the user calls `plot!(x, y)` without specifying a mark, this module
//! decides whether to draw a line, points, bars, or a histogram based on the
//! data's shape and types.
//!
//! Status: stub. Implementation lands in 0.2.0.

// ── infer_chart_kind ─────────────────────────────────────────────────────────────────────────────
// TODO(0.2.0): pub fn infer_chart_kind(x: &[f64], y: &[f64]) -> ChartKind { ... }
//              -- continuous numeric x → LineMark
//              -- categorical x → BarMark
//              -- single y array → Histogram
//              -- continuous + many points → PointMark (scatter)

// ── ChartKind ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.2.0): pub enum ChartKind { Line, Point, Bar, Histogram, Heatmap }
