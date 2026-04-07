//! Re-exports of layer-1 backends.
//!
//! Each backend lives in its own sub-module of `starsight_layer_1::backends`.
//! Re-exporting them flat here lets users write `starsight::backends::SkiaBackend`.

pub use crate::background::backends::DrawBackend;
pub use crate::background::backends::rasters::SkiaBackend;
pub use crate::background::backends::vectors::SvgBackend;
