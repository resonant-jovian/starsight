//! Layout: arrange multiple charts on a single figure.
//!
//! Layout types control how panels are sized and positioned. `GridLayout` is
//! the workhorse: a fixed grid of rows and columns with optional gaps and
//! padding. `FreeLayout` allows arbitrary positioning. `FlowLayout` packs
//! variable-size panels left-to-right with wrapping.
//!
//! Status: stub. Implementation lands in 0.4.0.

// ── Layout ───────────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub trait Layout { fn allocate(&self, viewport: Rect) -> Vec<Rect>; }

// ── GridLayout ───────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct GridLayout { rows: usize, cols: usize, gap: f32, padding: f32 }

// ── FreeLayout ───────────────────────────────────────────────────────────────────────────────────
// TODO(0.5.0): pub struct FreeLayout { panels: Vec<(Rect, PanelId)> }

// ── FlowLayout ───────────────────────────────────────────────────────────────────────────────────
// TODO(0.5.0): pub struct FlowLayout { row_height: f32, horizontal_gap: f32, vertical_gap: f32 }
