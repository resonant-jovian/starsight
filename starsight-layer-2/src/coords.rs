//! Coordinate systems: convert data values to pixel positions.
//!
//! [`CartesianCoord`] bundles an x-axis, a y-axis, and the plot-area rectangle.
//! `data_to_pixel` maps a `(data_x, data_y)` pair to screen coordinates,
//! inverting the y-axis (data y up → pixel y down).

#![allow(clippy::cast_possible_truncation)]

use crate::axes::Axis;
use crate::scales::Scale;
use starsight_layer_1::primitives::{Point, Rect};

// ── CartesianCoord ───────────────────────────────────────────────────────────────────────────────

/// 2D Cartesian coordinate system: x-axis, y-axis, and a pixel-space plot rect.
pub struct CartesianCoord {
    /// X-axis (horizontal).
    pub x_axis: Axis,
    /// Y-axis (vertical).
    pub y_axis: Axis,
    /// Pixel rectangle the data is mapped into.
    pub plot_area: Rect,
}

impl CartesianCoord {
    /// Map a single data-space x value to a pixel-space x coordinate.
    #[must_use]
    pub fn map_x(&self, x: f64) -> f64 {
        let nx = self.x_axis.scale.map(x);
        f64::from(self.plot_area.left) + nx * f64::from(self.plot_area.width())
    }

    /// Map a single data-space y value to a pixel-space y coordinate. The y axis
    /// is inverted: a larger data-y becomes a smaller pixel-y (closer to the top).
    #[must_use]
    pub fn map_y(&self, y: f64) -> f64 {
        let ny = self.y_axis.scale.map(y);
        f64::from(self.plot_area.bottom) - ny * f64::from(self.plot_area.height())
    }

    /// Map a data-space point to a pixel-space `Point`.
    #[must_use]
    pub fn data_to_pixel(&self, x: f64, y: f64) -> Point {
        Point::new(self.map_x(x) as f32, self.map_y(y) as f32)
    }
}

// ── PolarCoord ───────────────────────────────────────────────────────────────────────────────────
// TODO(0.4.0): pub struct PolarCoord { pub center: Point, pub radius: f32, pub theta_axis: Axis, pub r_axis: Axis }
