//! Animation framework: render figures over a timeline.
//!
//! Status: stub. Implementation lands in 0.10.0.

// ── Animation ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.10.0): pub struct Animation { duration: Duration, fps: u32, frames: Vec<Frame> }

// ── Timeline ─────────────────────────────────────────────────────────────────────────────────────
// TODO(0.10.0): pub struct Timeline { keyframes: Vec<(Duration, FigureState)>, easing: Easing }

// ── Frame ────────────────────────────────────────────────────────────────────────────────────────
// TODO(0.10.0): pub struct Frame { time: Duration, figure: Figure }

// ── Interpolation ────────────────────────────────────────────────────────────────────────────────
// TODO(0.10.0): pub trait Interpolation { fn lerp(&self, t: f64) -> Self; }
