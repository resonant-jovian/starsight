//! "What works at 0.3.0" capability matrix.

use super::Table;

const HEADER: &[&str] = &["capability", "available", "added in"];
const COL_W: &[u32] = &[622, 170, 80];
const COL_ALIGN: &[&str] = &["start", "start", "end"];

const ROWS: &[&[&str]] = &[
    &[
        "LineMark, PointMark, Figure, plot!, SVG + tiny-skia backends, Wilkinson ticks",
        "shipped",
        "0.1",
    ],
    &[
        "BarMark (vert/horiz/grouped/stacked), AreaMark (NaN-gap), HistogramMark, HeatmapMark",
        "shipped",
        "0.2",
    ],
    &[
        "BoxPlotMark, ViolinMark + Kde, PieMark / donut, CandlestickMark",
        "shipped",
        "0.3",
    ],
    &[
        "PolarCoord, ArcMark (Nightingale, Gauge, Sunburst), PolarBarMark, PolarRectMark, RadarMark",
        "shipped",
        "0.3",
    ],
    &[
        "ContourMark + marching-squares, ErrorBarMark, RugMark, auto-attached Colorbar, MultiPanelFigure",
        "shipped",
        "0.3",
    ],
    &[
        "Polars DataFrame integration",
        "shipped (polars feature)",
        "0.3",
    ],
    &["LogScale, SqrtScale, CategoricalScale", "shipped", "0.3"],
    &[
        "FacetWrap, shared axes across panels, polar-aware legend placement, contour filled bands",
        "planned",
        "0.4",
    ],
    &["SymLogScale, DateTimeScale, BandScale", "planned", "0.5"],
    &[
        "GPU + interactivity (wgpu, hover / zoom / pan)",
        "planned",
        "0.6",
    ],
    &["Animation, GIF, frame recording", "planned", "0.7"],
    &[
        "Terminal backend (Kitty / Sixel / iTerm2 / half-block / Braille)",
        "planned",
        "0.8",
    ],
    &[
        "3D marks (Surface3D, Scatter3D, isosurface)",
        "planned",
        "0.9",
    ],
    &["PDF (krilla), interactive HTML, WebGPU", "planned", "0.10"],
    &["ndarray / Arrow data acceptance", "planned", "0.11"],
];

pub fn table() -> Table<'static> {
    Table {
        stem: "capabilities",
        title: "starsight 0.3.0 capability matrix",
        header: HEADER,
        rows: ROWS,
        col_widths: COL_W,
        col_align: COL_ALIGN,
        col_font: None,
    }
}
