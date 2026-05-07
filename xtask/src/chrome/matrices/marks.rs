//! Marks status matrix.

use super::{Matrix, Row, Status};

const ROWS: &[Row<'_>] = &[
    Row { name: "LineMark", status: Status::Working, version: "0.1" },
    Row { name: "PointMark", status: Status::Working, version: "0.1" },
    Row { name: "BarMark (vert/horiz/grouped/stacked)", status: Status::Working, version: "0.2" },
    Row { name: "AreaMark (NaN-gap)", status: Status::Working, version: "0.2" },
    Row { name: "HistogramMark", status: Status::Working, version: "0.2" },
    Row { name: "HeatmapMark", status: Status::Working, version: "0.2" },
    Row { name: "BoxPlotMark", status: Status::Working, version: "0.3" },
    Row { name: "ViolinMark + Kde", status: Status::Working, version: "0.3" },
    Row { name: "PieMark / donut", status: Status::Working, version: "0.3" },
    Row { name: "CandlestickMark", status: Status::Working, version: "0.3" },
    Row { name: "ContourMark + marching squares", status: Status::Working, version: "0.3" },
    Row { name: "ErrorBarMark", status: Status::Working, version: "0.3" },
    Row { name: "RugMark", status: Status::Working, version: "0.3" },
    Row { name: "ArcMark (Nightingale, Gauge, Sunburst)", status: Status::Working, version: "0.3" },
    Row { name: "PolarBarMark (wind rose)", status: Status::Working, version: "0.3" },
    Row { name: "PolarRectMark (polar calendar)", status: Status::Working, version: "0.3" },
    Row { name: "RadarMark (spider)", status: Status::Working, version: "0.3" },
    Row { name: "StepMark", status: Status::Working, version: "0.3" },
    Row { name: "RegressionMark / smoothing", status: Status::Planned, version: "0.5" },
    Row { name: "Surface3D", status: Status::Planned, version: "0.9" },
    Row { name: "Scatter3D", status: Status::Planned, version: "0.9" },
    Row { name: "Isosurface", status: Status::Planned, version: "0.9" },
];

pub fn matrix() -> Matrix<'static> {
    Matrix {
        stem: "marks",
        title: "starsight marks — current and planned",
        rows: ROWS,
        footnote: Some("All marks implement the same trait; new ones slot in by adding one impl + a snapshot."),
    }
}
