use crate::scale::Scale;
use starsight_layer_1::primitives::geom::{Point, Rect};
use crate::axis::Axis;

pub struct CartesianCoord {
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub plot_area: Rect,
}

impl CartesianCoord {
    /// Map a single data-space x value to a pixel-space x coordinate.
    pub fn map_x(&self, x: f64) -> f64 {
        let nx = self.x_axis.scale.map(x);
        self.plot_area.left as f64 + nx * self.plot_area.width() as f64
    }

    /// Map a single data-space y value to a pixel-space y coordinate.
    pub fn map_y(&self, y: f64) -> f64 {
        let ny = self.y_axis.scale.map(y);
        self.plot_area.bottom as f64 - ny * self.plot_area.height() as f64
    }

    pub fn data_to_pixel(&self, x: f64, y: f64) -> Point {
        Point::new(self.map_x(x) as f32, self.map_y(y) as f32)
    }
}