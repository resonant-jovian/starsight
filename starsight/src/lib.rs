pub use starsight_layer_1 as background;
pub use starsight_layer_2 as modifiers;
pub use starsight_layer_3 as components;
pub use starsight_layer_4 as composition;
pub use starsight_layer_5 as common;
pub use starsight_layer_6 as interactivity;
pub use starsight_layer_7 as export;

pub mod prelude;

/// Quick one-liner: `plot!(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]).save("out.png").unwrap();`
#[macro_export]
macro_rules! plot {
    ($x:expr, $y:expr $(,)?) => {{
        $crate::common::Figure::from_arrays(
            $x.into_iter().map(|&v| v as f64),
            $y.into_iter().map(|&v| v as f64),
        )
    }};
}
