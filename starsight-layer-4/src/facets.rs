//! Faceting: split data by a grouping variable and draw one panel per group.
//!
//! `FacetWrap` lays panels out in a grid that wraps. `FacetGrid` puts one
//! variable on rows and another on columns. `FacetMatrix` is a full pairwise
//! matrix (one panel per combination).
//!
//! Status: stub. Implementation lands in 0.4.0.

// ── FacetWrap ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct FacetWrap { variable: String, ncol: Option<usize>, nrow: Option<usize> }

// ── FacetGrid ────────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct FacetGrid { rows: String, cols: String }

// ── FacetMatrix ──────────────────────────────────────────────────────────────────────────────────
// TODO(0.5.0): pub struct FacetMatrix { variables: Vec<String>, diag: DiagonalMark }
