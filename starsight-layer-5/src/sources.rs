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

/// Polars `DataFrame` source. Pulls two named columns out of a frame and
/// converts them to `Vec<f64>` slot-by-slot so the rest of the pipeline
/// (which still speaks `Vec<f64>`) stays unchanged.
///
/// Null values become `f64::NAN`, matching the NaN-as-gap convention the
/// `LineMark` / `PointMark` / `AreaMark` renderers already honour. Building
/// from a `LazyFrame` logs a warning and `.collect()` materialises the whole
/// frame — fine for the figure sizes a chart library cares about, but worth
/// the warning so callers aren't surprised by a streaming computation
/// suddenly running to completion.
#[cfg(feature = "polars")]
#[derive(Clone, Debug)]
pub struct FrameSource {
    df: polars::frame::DataFrame,
    x: String,
    y: String,
}

#[cfg(feature = "polars")]
impl FrameSource {
    /// Build a frame source from a `DataFrame` and the names of the x and y
    /// columns to extract.
    #[must_use]
    pub fn new(df: polars::frame::DataFrame, x: impl Into<String>, y: impl Into<String>) -> Self {
        Self {
            df,
            x: x.into(),
            y: y.into(),
        }
    }

    /// Borrow the underlying frame.
    #[must_use]
    pub fn frame(&self) -> &polars::frame::DataFrame {
        &self.df
    }

    /// X column name.
    #[must_use]
    pub fn x_column(&self) -> &str {
        &self.x
    }

    /// Y column name.
    #[must_use]
    pub fn y_column(&self) -> &str {
        &self.y
    }
}

#[cfg(feature = "polars")]
impl DataSource for FrameSource {
    fn into_columns(self) -> Vec<Vec<f64>> {
        let xs = extract_f64_with_nulls(&self.df, &self.x).unwrap_or_default();
        let ys = extract_f64_with_nulls(&self.df, &self.y).unwrap_or_default();
        vec![xs, ys]
    }
}

/// Extract a numeric column into a `Vec<f64>`, casting through Float64 when
/// the column is integer-typed.
///
/// # Errors
/// Returns a description string if the column is missing, can't be cast to
/// `f64`, or its `f64` chunked-array view fails to materialise.
#[cfg(feature = "polars")]
pub fn extract_f64(df: &polars::frame::DataFrame, name: &str) -> Result<Vec<f64>, String> {
    use polars::prelude::DataType;
    let column = df
        .column(name)
        .map_err(|e| format!("column '{name}': {e}"))?;
    let casted = column
        .cast(&DataType::Float64)
        .map_err(|e| format!("cast '{name}' to f64: {e}"))?;
    let chunked = casted
        .f64()
        .map_err(|e| format!("read '{name}' as f64: {e}"))?;
    Ok(chunked.into_no_null_iter().collect())
}

/// Extract a numeric column, mapping null entries to `f64::NAN`. The
/// gap-aware mark renderers (Line / Point / Area) skip NaN values, so a
/// sparse column flows through unchanged without a separate masking step.
///
/// # Errors
/// Returns a description string if the column is missing or can't be cast
/// to `f64`.
#[cfg(feature = "polars")]
pub fn extract_f64_with_nulls(
    df: &polars::frame::DataFrame,
    name: &str,
) -> Result<Vec<f64>, String> {
    use polars::prelude::DataType;
    let column = df
        .column(name)
        .map_err(|e| format!("column '{name}': {e}"))?;
    let casted = column
        .cast(&DataType::Float64)
        .map_err(|e| format!("cast '{name}' to f64: {e}"))?;
    let chunked = casted
        .f64()
        .map_err(|e| format!("read '{name}' as f64: {e}"))?;
    Ok(chunked.into_iter().map(|o| o.unwrap_or(f64::NAN)).collect())
}

/// Extract a string-typed column as `Vec<String>`. Useful for color- and
/// label-grouping in the `plot!` macro's `DataFrame` arm.
///
/// # Errors
/// Returns a description string if the column is missing or not string-typed.
#[cfg(feature = "polars")]
pub fn extract_strings(df: &polars::frame::DataFrame, name: &str) -> Result<Vec<String>, String> {
    let column = df
        .column(name)
        .map_err(|e| format!("column '{name}': {e}"))?;
    let chunked = column
        .str()
        .map_err(|e| format!("read '{name}' as str: {e}"))?;
    Ok(chunked
        .into_iter()
        .map(|opt| opt.unwrap_or("").to_string())
        .collect())
}

#[cfg(feature = "polars")]
impl From<polars::lazy::frame::LazyFrame> for FrameSource {
    /// Materialise the lazy frame eagerly. We're plotting, not streaming —
    /// the chart needs every row in memory anyway. Logged at warn-level so
    /// callers aren't surprised by a streaming pipeline suddenly running.
    fn from(lf: polars::lazy::frame::LazyFrame) -> Self {
        log::warn!(
            "Collecting LazyFrame for chart rendering — this materialises the full frame in memory."
        );
        let df = lf.collect().unwrap_or_default();
        Self {
            df,
            x: String::new(),
            y: String::new(),
        }
    }
}

// ── plot_dataframe (feature: polars) ─────────────────────────────────────────────────────────────

/// Build a [`Figure`](crate::Figure) from a Polars `DataFrame`: extract the
/// named x / y columns, dispatch a sensible mark (Line for numeric x, Bar
/// for categorical x), and optionally partition by a `color` column to emit
/// one mark per group with cycled palette colours and per-group legend
/// labels.
///
/// Used by the `plot!` macro's `DataFrame` arm — calling it directly is
/// fine, but the macro spelling reads more naturally for one-liners.
#[cfg(feature = "polars")]
#[must_use]
pub fn plot_dataframe(
    df: &polars::frame::DataFrame,
    x: &str,
    y: &str,
    color: Option<&str>,
) -> crate::Figure {
    use starsight_layer_1::primitives::Color;
    use starsight_layer_3::marks::{BarMark, LineMark, PointMark};

    let mut fig = crate::Figure::new(800, 600);
    let palette = group_palette();

    let x_is_categorical = is_string_column(df, x);

    if let Some(color_col) = color {
        // Group by the color column. For each unique value, partition the
        // x/y rows and emit one mark.
        let groups = extract_strings(df, color_col).unwrap_or_default();
        let mut unique: Vec<String> = Vec::new();
        for g in &groups {
            if !unique.iter().any(|u| u == g) {
                unique.push(g.clone());
            }
        }

        if x_is_categorical {
            // Categorical x: BarMark per group.
            let x_strings = extract_strings(df, x).unwrap_or_default();
            let y_values = extract_f64_with_nulls(df, y).unwrap_or_default();
            for (idx, value) in unique.iter().enumerate() {
                let xs: Vec<String> = x_strings
                    .iter()
                    .zip(&groups)
                    .filter(|(_, g)| *g == value)
                    .map(|(x, _)| x.clone())
                    .collect();
                let ys: Vec<f64> = y_values
                    .iter()
                    .zip(&groups)
                    .filter(|(_, g)| *g == value)
                    .map(|(y, _)| *y)
                    .collect();
                fig = fig.add(
                    BarMark::new(xs, ys)
                        .color(palette[idx % palette.len()])
                        .label(value),
                );
            }
        } else {
            // Numeric x: PointMark per group (more honest than Line for
            // grouped scatter).
            let x_values = extract_f64_with_nulls(df, x).unwrap_or_default();
            let y_values = extract_f64_with_nulls(df, y).unwrap_or_default();
            for (idx, value) in unique.iter().enumerate() {
                let xs: Vec<f64> = x_values
                    .iter()
                    .zip(&groups)
                    .filter(|(_, g)| *g == value)
                    .map(|(x, _)| *x)
                    .collect();
                let ys: Vec<f64> = y_values
                    .iter()
                    .zip(&groups)
                    .filter(|(_, g)| *g == value)
                    .map(|(y, _)| *y)
                    .collect();
                fig = fig.add(
                    PointMark::new(xs, ys)
                        .color(palette[idx % palette.len()])
                        .radius(5.0)
                        .label(value),
                );
            }
        }
    } else if x_is_categorical {
        let xs = extract_strings(df, x).unwrap_or_default();
        let ys = extract_f64_with_nulls(df, y).unwrap_or_default();
        fig = fig.add(BarMark::new(xs, ys).color(Color::BLUE));
    } else {
        let xs = extract_f64_with_nulls(df, x).unwrap_or_default();
        let ys = extract_f64_with_nulls(df, y).unwrap_or_default();
        fig = fig.add(LineMark::new(xs, ys).color(Color::BLUE));
    }
    fig
}

#[cfg(feature = "polars")]
fn is_string_column(df: &polars::frame::DataFrame, name: &str) -> bool {
    use polars::prelude::DataType;
    df.column(name)
        .is_ok_and(|c| matches!(c.dtype(), DataType::String) || c.dtype().is_categorical())
}

/// Six-color cycle used by the `plot!` macro's `color = "col"` grouping.
/// Matches the default `PieMark` palette so a chart that mixes the two reads
/// consistently.
#[cfg(feature = "polars")]
fn group_palette() -> [starsight_layer_1::primitives::Color; 6] {
    use starsight_layer_1::primitives::Color;
    [
        Color::from_hex(0x0033_77BB),
        Color::from_hex(0x00EE_7733),
        Color::from_hex(0x0033_AA66),
        Color::from_hex(0x00CC_3366),
        Color::from_hex(0x00AA_44AA),
        Color::from_hex(0x0099_AABB),
    ]
}

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
