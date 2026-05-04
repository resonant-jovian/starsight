//! Layer 5 — high-level API: figures, rendering helpers, data acceptance.
//!
//! Layer 5 is what users interact with directly. The [`figures::Figure`] builder
//! is the entry point; the `starsight::plot!` macro in the facade
//! forwards through `Figure::from_arrays`.
//!
//! Modules:
//! - [`figures`]: the `Figure` builder.
//! - [`layout`]: modular margin / slot composition for chrome.
//! - [`renders`]: rendering helpers (axes, background, legend).
//! - [`inferences`]: auto-pick a chart type from data shape (stub).
//! - [`sources`]: data acceptance from slices, `DataFrames`, ndarray, Arrow (stub).

pub mod colorbar;
pub mod figures;
pub mod inferences;
pub mod layout;
pub mod renders;
pub mod sources;

pub use colorbar::Colorbar;
pub use figures::{Figure, MultiPanelFigure};
