//! Re-exports of layer-3 marks.
//!
//! `Mark` is the trait every visual element implements; the rest are concrete
//! mark types ready to be added to a [`Figure`](crate::figures::Figure).

pub use crate::components::marks::{
    ArcMark, AreaBaseline, AreaMark, AxisDir, BarMark, BarRenderContext, BoxPlotGroup, BoxPlotMark,
    CandlestickMark, ContourMark, ContourMode, DataExtent, ErrorBarMark, ErrorBarOrientation,
    HeatmapMark, HistogramMark, LegendGlyph, LineMark, Mark, Ohlc, Orientation, PieMark, PointMark,
    PolarBarMark, PolarRectMark, RadarMark, RugMark, StepMark, StepPosition, ViolinGroup,
    ViolinMark, ViolinScale,
};
