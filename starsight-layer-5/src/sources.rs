//! Data sources: how user-supplied data enters the library.
//!
//! Layer 5 accepts a wide variety of input types (slices, vectors, Polars
//! `DataFrames`, ndarray arrays, Arrow record batches) and converts them to the
//! `Vec<f64>` format consumed by marks. Each source is feature-gated.
//!
//! Status: stub. `SliceSource` lands in 0.1.0; richer sources land in 0.3.0+.

// ── DataSource ───────────────────────────────────────────────────────────────────────────────────

/// Trait for converting arbitrary data into columns of f64 values.
pub trait DataSource {
    /// Convert the source into a vector of columns, each column being f64 values.
    fn into_columns(self) -> Vec<Vec<f64>>;
}

// ── SliceSource ──────────────────────────────────────────────────────────────────────────────────

/// Data source from a slice of columns (e.g., from `&[x, y]`).
#[derive(Debug, Clone)]
pub struct SliceSource<'a> {
    columns: &'a [&'a [f64]],
}

impl<'a> SliceSource<'a> {
    /// Create a new slice source from columns.
    #[must_use]
    pub fn new(columns: &'a [&'a [f64]]) -> Self {
        Self { columns }
    }
}

impl DataSource for SliceSource<'_> {
    fn into_columns(self) -> Vec<Vec<f64>> {
        self.columns.iter().map(|c| c.to_vec()).collect()
    }
}

// ── VecSource ───────────────────────────────────────────────────────────────────────────────────

/// Data source from owned vectors.
#[derive(Debug, Clone)]
pub struct VecSource {
    columns: Vec<Vec<f64>>,
}

impl VecSource {
    /// Create from vectors.
    #[must_use]
    pub fn new(columns: Vec<Vec<f64>>) -> Self {
        Self { columns }
    }

    /// Create from a single vector (convenience).
    #[must_use]
    pub fn single(values: Vec<f64>) -> Self {
        Self {
            columns: vec![values],
        }
    }

    /// Create from two vectors (x and y).
    #[must_use]
    pub fn xy(x: Vec<f64>, y: Vec<f64>) -> Self {
        Self {
            columns: vec![x, y],
        }
    }
}

impl DataSource for VecSource {
    fn into_columns(self) -> Vec<Vec<f64>> {
        self.columns
    }
}

// ── FrameSource (feature: polars) ────────────────────────────────────────────────────────────────
// TODO(0.3.0): #[cfg(feature = "polars")] pub struct FrameSource { df: polars::DataFrame, x: String, y: String }

// ── NdArraySource (feature: ndarray) ─────────────────────────────────────────────────────────────
// TODO(0.4.0): #[cfg(feature = "ndarray")] pub struct NdArraySource { x: ndarray::Array1<f64>, y: ndarray::Array1<f64> }

// ── ArrowSource (feature: arrow) ─────────────────────────────────────────────────────────────────
// TODO(0.4.0): #[cfg(feature = "arrow")] pub struct ArrowSource { batch: arrow::record_batch::RecordBatch }

#[cfg(test)]
mod tests {
    use super::{DataSource, SliceSource, VecSource};

    #[test]
    fn slice_source_new() {
        let x: &[f64] = &[1.0, 2.0, 3.0];
        let y: &[f64] = &[4.0, 5.0, 6.0];
        let slices: &[&[f64]] = &[x, y];
        let source = SliceSource::new(slices);
        let cols = source.into_columns();
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0], vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn slice_source_single_column() {
        let x: &[f64] = &[1.0, 2.0, 3.0];
        let slices: &[&[f64]] = &[x];
        let source = SliceSource::new(slices);
        let cols = source.into_columns();
        assert_eq!(cols.len(), 1);
    }

    #[test]
    fn vec_source_new() {
        let source = VecSource::new(vec![vec![1.0, 2.0], vec![3.0, 4.0]]);
        let cols = source.into_columns();
        assert_eq!(cols.len(), 2);
    }

    #[test]
    fn vec_source_single() {
        let source = VecSource::single(vec![1.0, 2.0, 3.0]);
        let cols = source.into_columns();
        assert_eq!(cols.len(), 1);
        assert_eq!(cols[0], vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn vec_source_xy() {
        let source = VecSource::xy(vec![1.0, 2.0], vec![3.0, 4.0]);
        let cols = source.into_columns();
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0], vec![1.0, 2.0]);
        assert_eq!(cols[1], vec![3.0, 4.0]);
    }
}
