//! Re-exports of legend and colorbar types.
//!
//! `Colorbar` shipped in 0.3.0 and lives in layer-5 (the renderer assembles
//! it from the figure's marks via `Mark::colormap_legend`). The standalone
//! `Legend` / `LegendItem` types remain a 0.4.0+ surface — for now legend
//! rendering is internal to `render_legend` in layer-5.

pub use crate::common::Colorbar;
