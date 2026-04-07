//! Aesthetic mappings: how data columns become visual properties.
//!
//! In the grammar of graphics, an aesthetic binds a data column to a visual
//! channel (color, size, shape, alpha, fill, linetype). The aesthetic resolves
//! discrete categories into concrete values.
//!
//! Status: stub. Implementations land in 0.3.0+.

// ── Aesthetic ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub trait Aesthetic { type Output; fn resolve(&self, data: &[f64]) -> Vec<Self::Output>; }

// ── ColorAesthetic ───────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct ColorAesthetic { palette: Vec<Color>, mapping: Mapping }

// ── SizeAesthetic ────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct SizeAesthetic { range: (f32, f32), mapping: Mapping }

// ── ShapeAesthetic ───────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct ShapeAesthetic { shapes: Vec<Shape> }

// ── AlphaAesthetic ───────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct AlphaAesthetic { range: (f32, f32) }

// ── FillAesthetic ────────────────────────────────────────────────────────────────────────────────
// TODO(0.3.0): pub struct FillAesthetic { palette: Vec<Color> }

// ── LinetypeAesthetic ────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct LinetypeAesthetic { types: Vec<Linetype> }
