//! Layer 2 — scales, ticks, axes, and coordinate systems.
//!
//! Bridges raw data values and the pixel-space rendering primitives in
//! [`starsight_layer_1`]. Each module is single-purpose:
//!
//! - [`scales`]: map data domain → normalized `[0, 1]`.
//! - [`ticks`]: choose nice tick positions (Wilkinson Extended algorithm).
//! - [`axes`]: bundle a scale with ticks and labels.
//! - [`coords`]: convert data values to pixel positions.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::many_single_char_names
)]

pub mod axes;
pub mod coords;
pub mod scales;
pub mod ticks;
