//! Layer 1 — primitives, error types, drawing primitives, and rendering backends.
//!
//! This is the foundation layer. Every other layer depends on it.
//!
//! Modules:
//! - [`errors`]: `StarsightError` enum and `Result` alias.
//! - [`primitives`]: `Color`, `ColorAlpha`, `Point`, `Vec2`, `Rect`, `Size`, `Transform`.
//! - [`paths`]: `Path`, `PathCommand`, `PathStyle`, `LineCap`, `LineJoin`.
//! - [`scenes`]: `Scene`, `SceneNode` (stub — 0.5.0).
//! - [`backends`]: `DrawBackend` trait + raster/vector/print/gpu/terminal implementations.

pub mod backends;
pub mod errors;
pub mod paths;
pub mod primitives;
pub mod scenes;
