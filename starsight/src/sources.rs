//! Re-exports of layer-5 data-source types.

pub use crate::common::sources::{DataSource, SliceSource, VecSource};

#[cfg(feature = "polars")]
pub use crate::common::sources::{
    FrameSource, extract_f64, extract_f64_with_nulls, extract_strings, plot_dataframe,
};
