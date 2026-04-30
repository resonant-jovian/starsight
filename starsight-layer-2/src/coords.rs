//! Coordinate systems: convert data values to pixel positions.
//!
//! The [`Coord`] trait abstracts over the concrete coordinate systems that a
//! mark may render against — currently [`CartesianCoord`], with `PolarCoord`
//! arriving in 0.3.0. Marks accept `&dyn Coord` and downcast to the concrete
//! variant they support via [`Coord::as_any`]; cross-coord rendering is opt-in,
//! not implicit.

#![allow(clippy::cast_possible_truncation)]

use crate::axes::Axis;
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

/// 2D polar coordinate system: angular axis, radial axis, and a pixel-space
/// disk inscribed in `plot_area`.
///
/// Conventions:
/// - Theta = 0 points straight up (12 o'clock). Increasing theta sweeps
///   clockwise — the compass convention used by Nightingale, gauges, and wind
///   roses. Mathematical (CCW from east) plots can flip the data through
///   their `theta_axis` scale.
/// - Radius increases outward from `center`. `r_axis` maps data space to the
///   pixel range `0..radius`.
/// - `plot_area` is the bounding rect; `center` and `radius` default to the
///   inscribed circle but can be set independently to inset / offset the
///   polar plot inside a larger figure rect.
pub struct PolarCoord {
    /// Angular (theta) axis. The scale's normalized output is multiplied by
    /// `2π` to obtain the rendered angle.
    pub theta_axis: Axis,
    /// Radial (r) axis. The scale's normalized output is multiplied by
    /// `radius` to obtain the rendered pixel distance from `center`.
    pub r_axis: Axis,
    /// Pixel rectangle the disk is inscribed in.
    pub plot_area: Rect,
    /// Pixel-space center of the polar disk.
    pub center: Point,
    /// Pixel-space outer radius of the polar disk.
    pub radius: f32,
}

impl PolarCoord {
    /// Build a polar coord whose disk is inscribed in `plot_area` (centered,
    /// radius = `min(width, height) / 2`).
    #[must_use]
    pub fn inscribed(theta_axis: Axis, r_axis: Axis, plot_area: Rect) -> Self {
        let cx = (plot_area.left + plot_area.right) * 0.5;
        let cy = (plot_area.top + plot_area.bottom) * 0.5;
        let radius = plot_area.width().min(plot_area.height()) * 0.5;
        Self {
            theta_axis,
            r_axis,
            plot_area,
            center: Point::new(cx, cy),
            radius,
        }
    }

    /// Override the disk center independently of `plot_area`.
    #[must_use]
    pub fn with_center(mut self, center: Point) -> Self {
        self.center = center;
        self
    }

    /// Override the outer radius independently of `plot_area`.
    #[must_use]
    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Map a data-space `(theta, r)` pair to a pixel-space `Point`.
    ///
    /// `theta` is normalized through `theta_axis.scale` then multiplied by
    /// `2π`; `r` is normalized through `r_axis.scale` then multiplied by
    /// `radius`. Compass convention: theta = 0 is up, increasing clockwise.
    #[must_use]
    pub fn data_to_pixel(&self, theta: f64, r: f64) -> Point {
        let theta_norm = self.theta_axis.scale.map(theta);
        let r_norm = self.r_axis.scale.map(r);
        let angle = theta_norm * std::f64::consts::TAU;
        let pixel_r = r_norm * f64::from(self.radius);
        let x = f64::from(self.center.x) + pixel_r * angle.sin();
        let y = f64::from(self.center.y) - pixel_r * angle.cos();
        Point::new(x as f32, y as f32)
    }
}

impl Coord for PolarCoord {
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

#[cfg(test)]
mod tests {
    use super::{Coord, PolarCoord};
    use crate::axes::Axis;
    use crate::scales::LinearScale;
    use starsight_layer_1::primitives::Rect;

    fn unit_polar() -> PolarCoord {
        let theta = Axis {
            scale: Box::new(LinearScale {
                domain_min: 0.0,
                domain_max: 1.0,
            }),
            label: None,
            tick_positions: vec![],
            tick_labels: vec![],
        };
        let r = Axis {
            scale: Box::new(LinearScale {
                domain_min: 0.0,
                domain_max: 1.0,
            }),
            label: None,
            tick_positions: vec![],
            tick_labels: vec![],
        };
        // 200x200 plot area centered at (100, 100), radius 100
        PolarCoord::inscribed(theta, r, Rect::new(0.0, 0.0, 200.0, 200.0))
    }

    #[test]
    fn polar_inscribed_centers_in_plot_area() {
        let p = unit_polar();
        assert!((p.center.x - 100.0).abs() < 1e-4);
        assert!((p.center.y - 100.0).abs() < 1e-4);
        assert!((p.radius - 100.0).abs() < 1e-4);
    }

    #[test]
    fn polar_theta_zero_points_up() {
        // theta = 0, r = 1 -> directly above center
        let p = unit_polar();
        let pt = p.data_to_pixel(0.0, 1.0);
        assert!((pt.x - 100.0).abs() < 1e-3);
        assert!((pt.y - 0.0).abs() < 1e-3);
    }

    #[test]
    fn polar_quarter_turn_points_right() {
        // theta = 0.25 (90°), r = 1 -> directly right of center
        let p = unit_polar();
        let pt = p.data_to_pixel(0.25, 1.0);
        assert!((pt.x - 200.0).abs() < 1e-3);
        assert!((pt.y - 100.0).abs() < 1e-3);
    }

    #[test]
    fn polar_zero_radius_returns_center() {
        let p = unit_polar();
        let pt = p.data_to_pixel(0.5, 0.0);
        assert!((pt.x - 100.0).abs() < 1e-3);
        assert!((pt.y - 100.0).abs() < 1e-3);
    }

    #[test]
    fn polar_dispatches_through_coord_trait() {
        let p = unit_polar();
        let dyn_coord: &dyn Coord = &p;
        assert_eq!(dyn_coord.plot_area(), p.plot_area);
        let pt_via_trait = dyn_coord.data_to_pixel(0.0, 1.0);
        let pt_direct = p.data_to_pixel(0.0, 1.0);
        assert_eq!(pt_via_trait, pt_direct);
        // downcast back to PolarCoord
        assert!(dyn_coord.as_any().is::<PolarCoord>());
    }
}
