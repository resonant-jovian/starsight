//! Layer 6 — interactivity: hover, zoom, pan, selection, streaming, windowing.
//!
//! Layer 6 adds runtime interaction to figures. None of it is required for
//! static export; everything here is feature-gated behind `interactive`.
//!
//! Modules:
//! - [`hovers`]: tooltips that follow the cursor (stub).
//! - [`zooms`]: wheel and box zoom (stub).
//! - [`pans`]: drag-to-pan (stub).
//! - [`selections`]: box and lasso selection (stub).
//! - [`streams`]: streaming data buffers (stub).
//! - [`windows`]: winit event loop integration (stub).

pub mod hovers;
pub mod pans;
pub mod selections;
pub mod streams;
pub mod windows;
pub mod zooms;
