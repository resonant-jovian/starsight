//! Layer 7 — animation and export.
//!
//! Layer 7 is the final layer before the user. It produces files (PDF, GIF,
//! HTML), drives animations, and pipes output into the terminal.
//!
//! Modules:
//! - [`animations`]: timeline + frame recording (stub).
//! - [`exports`]: `Export` trait + format dispatch (stub).
//! - [`prints`]: PDF via `krilla` (stub).
//! - [`webs`]: interactive HTML + WASM bridge (stub).
//! - [`gifs`]: animated GIF (stub).
//! - [`terminals`]: inline terminal export (stub).

pub mod animations;
pub mod exports;
pub mod gifs;
pub mod prints;
pub mod terminals;
pub mod webs;
