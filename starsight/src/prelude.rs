//! Prelude: a single glob import for the most-used types.
//!
//! ```no_run
//! use starsight::prelude::*;
//! ```

pub use crate::background::errors::{Result, StarsightError};
pub use crate::background::primitives::{Color, ColorAlpha, Point, Rect, Size, Transform, Vec2};
pub use crate::common::Figure;
pub use crate::components::marks::{BarMark, HistogramMark, LineMark, Mark, PointMark, StepMark};
pub use crate::plot;
