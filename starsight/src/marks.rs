//! Re-exports of layer-3 marks.
//!
//! `Mark` is the trait every visual element implements; the rest are concrete
//! mark types ready to be added to a [`Figure`](crate::figures::Figure).

pub use crate::components::marks::{
    AreaBaseline, AreaMark, BarMark, BarRenderContext, DataExtent, HeatmapMark, HistogramMark,
    LegendGlyph, LineMark, Mark, Orientation, PointMark, StepMark, StepPosition,
};
