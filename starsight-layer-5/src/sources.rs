//! Data sources: how user-supplied data enters the library.
//!
//! Layer 5 accepts a wide variety of input types (slices, vectors, Polars
//! `DataFrames`, ndarray arrays, Arrow record batches) and converts them to the
//! `Vec<f64>` format consumed by marks. Each source is feature-gated.
//!
//! Status: stub. `SliceSource` lands in 0.1.0; richer sources land in 0.3.0+.

// ── DataSource ───────────────────────────────────────────────────────────────────────────────────
// TODO(0.2.0): pub trait DataSource { fn into_columns(self) -> Vec<Vec<f64>>; }

// ── SliceSource ──────────────────────────────────────────────────────────────────────────────────
// TODO(0.2.0): pub struct SliceSource<'a> { columns: &'a [&'a [f64]] }

// ── FrameSource (feature: polars) ────────────────────────────────────────────────────────────────
// TODO(0.3.0): #[cfg(feature = "polars")] pub struct FrameSource { df: polars::DataFrame, x: String, y: String }

// ── NdArraySource (feature: ndarray) ─────────────────────────────────────────────────────────────
// TODO(0.4.0): #[cfg(feature = "ndarray")] pub struct NdArraySource { x: ndarray::Array1<f64>, y: ndarray::Array1<f64> }

// ── ArrowSource (feature: arrow) ─────────────────────────────────────────────────────────────────
// TODO(0.4.0): #[cfg(feature = "arrow")] pub struct ArrowSource { batch: arrow::record_batch::RecordBatch }
