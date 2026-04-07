use crate::axis::Axis;
use crate::scale::Scale;
use starsight_layer_1::primitives::geom::{Point, Rect};

pub struct CartesianCoord {
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub plot_area: Rect,
}

impl CartesianCoord {
    /// Map a single data-space x value to a pixel-space x coordinate.
    #[must_use]
    pub fn map_x(&self, x: f64) -> f64 {
        let nx = self.x_axis.scale.map(x);
        f64::from(self.plot_area.left) + nx * f64::from(self.plot_area.width())
    }

    /// Map a single data-space y value to a pixel-space y coordinate.
    #[must_use]
    pub fn map_y(&self, y: f64) -> f64 {
        let ny = self.y_axis.scale.map(y);
        f64::from(self.plot_area.bottom) - ny * f64::from(self.plot_area.height())
    }

    #[must_use]
    pub fn data_to_pixel(&self, x: f64, y: f64) -> Point {
        Point::new(self.map_x(x) as f32, self.map_y(y) as f32)
    }
}
