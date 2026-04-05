pub use starsight_layer_1::backend::skia::SkiaBackend;
pub use starsight_layer_1::backend::DrawBackend;
pub use starsight_layer_1::error::{Result, StarsightError};
pub use starsight_layer_1::primitives::color::Color;
pub use starsight_layer_1::primitives::geom::{Point, Rect, Size, Vec2};
pub use starsight_layer_3::mark::{LineMark, Mark, PointMark};
pub use starsight_layer_5::Figure;

pub mod prelude;

/// Quick one-liner: `plot!(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]).save("out.png").unwrap();`
#[macro_export]
macro_rules! plot {
    ($x:expr, $y:expr $(,)?) => {{
        $crate::Figure::from_arrays(
            $x.into_iter().map(|&v| v as f64),
            $y.into_iter().map(|&v| v as f64),
        )
    }};
}
