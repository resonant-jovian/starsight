use crate::scale::Scale;
use starsight_layer_1::primitives::geom::{Point, Rect};
use crate::axis::Axis;

pub struct CartesianCoord {
    pub x_axis: Axis,
    pub y_axis: Axis,
    pub plot_area: Rect,
}

impl CartesianCoord {
    pub fn data_to_pixel(&self, x: f64, y: f64) -> Point {
        let nx = self.x_axis.scale.map(x);
        let ny = self.y_axis.scale.map(y);
        Point::new(
            self.plot_area.left + nx as f32 * self.plot_area.width(),
            self.plot_area.bottom - ny as f32 * self.plot_area.height(),
        )
    }
}