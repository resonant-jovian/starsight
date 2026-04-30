//! Coordinate systems: convert data values to pixel positions.
//!
//! The [`Coord`] trait abstracts over the concrete coordinate systems that a
//! mark may render against — currently [`CartesianCoord`], with `PolarCoord`
//! arriving in 0.3.0. Marks accept `&dyn Coord` and downcast to the concrete
//! variant they support via [`Coord::as_any`]; cross-coord rendering is opt-in,
//! not implicit.

#![allow(clippy::cast_possible_truncation)]

use crate::axes::Axis;
use crate::scales::Scale;
use starsight_layer_1::primitives::{Point, Rect};
use std::any::Any;

// ── Coord ────────────────────────────────────────────────────────────────────────────────────────

/// Coordinate system contract every mark renders against.
///
/// Object-safe so marks can be stored as `Box<dyn Mark>` and dispatched via
/// `&dyn Coord`. The trait exposes the common surface shared by every coord
/// system (plot rect + data→pixel mapping); coord-specific accessors like
/// `CartesianCoord::x_axis` are reached via [`Coord::as_any`] downcasting.
pub trait Coord: Any {
    /// The pixel-space rectangle this coord maps data into.
    fn plot_area(&self) -> Rect;
    /// Map a data-space `(x, y)` pair to a pixel-space [`Point`].
    fn data_to_pixel(&self, x: f64, y: f64) -> Point;
    /// Erased self-reference used to downcast to a concrete coord type.
    fn as_any(&self) -> &dyn Any;
}

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

impl Coord for CartesianCoord {
    fn plot_area(&self) -> Rect {
        self.plot_area
    }
    fn data_to_pixel(&self, x: f64, y: f64) -> Point {
        Self::data_to_pixel(self, x, y)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// ── PolarCoord ───────────────────────────────────────────────────────────────────────────────────
// TODO(A.2): pub struct PolarCoord { pub center: Point, pub radius: f32, pub theta_axis: Axis, pub r_axis: Axis }
