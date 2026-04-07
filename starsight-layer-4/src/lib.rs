//! Layer 4 — layout, faceting, legends, colorbars.
//!
//! Layer 4 arranges multiple charts on a single figure. None of these types
//! render anything themselves: they compute panel positions and delegate to
//! the marks and backends in lower layers.
//!
//! Modules:
//! - [`layouts`]: grid and free panel layouts (stub).
//! - [`facets`]: split data by a variable, one panel per group (stub).
//! - [`legends`]: legend types (stub).
//! - [`colorbars`]: continuous color scales for heatmaps (stub).

pub mod colorbars;
pub mod facets;
pub mod layouts;
pub mod legends;
