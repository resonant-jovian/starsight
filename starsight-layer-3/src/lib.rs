//! Layer 3 — marks, statistics, aesthetics, position adjustments.
//!
//! Layer 3 is the visual vocabulary of the library. Marks are the geometric
//! shapes that read data and draw it. Statistics are data transforms (binning,
//! KDE, regression, ...). Aesthetics map data columns to visual properties.
//! Position adjustments resolve overlapping marks (stack, dodge, jitter).
//!
//! Modules:
//! - [`marks`]: `Mark` trait + `LineMark`, `PointMark`, `BarMark`, ...
//! - [`statistics`]: data transforms (stub).
//! - [`aesthetics`]: aesthetic mapping types (stub).
//! - [`positions`]: position adjustments (stub).

pub mod aesthetics;
pub mod marks;
pub mod positions;
pub mod statistics;
