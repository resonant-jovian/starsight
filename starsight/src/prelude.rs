//! Prelude: a single glob import for the most-used types.
//!
//! ```no_run
//! use starsight::prelude::*;
//! ```

pub use crate::background::errors::{Result, StarsightError};
pub use crate::background::primitives::{Color, ColorAlpha, Point, Rect, Size, Transform, Vec2};
pub use crate::colormap::{Colormap, DEFAULT};
pub use crate::common::inferences::ChartKind;
pub use crate::common::{Figure, MultiPanelFigure};
pub use crate::components::marks::{
    ArcMark, BarMark, BoxPlotGroup, BoxPlotMark, CandlestickMark, ContourMark, ContourMode,
    HistogramMark, LineMark, Mark, Ohlc, Orientation, PieMark, PointMark, RadarMark, StepMark,
    ViolinGroup, ViolinMark, ViolinScale,
};
pub use crate::plot;
pub use crate::theme::{DEFAULT_DARK, DEFAULT_LIGHT, Theme};
