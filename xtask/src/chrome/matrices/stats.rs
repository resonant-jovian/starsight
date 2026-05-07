//! Statistics status matrix.

use super::{Matrix, Row, Status};

const ROWS: &[Row<'_>] = &[
    Row {
        name: "Wilkinson Extended ticks",
        status: Status::Working,
        version: "0.1",
    },
    Row {
        name: "HistogramMark::method (Count/Width/FreedmanDiaconis)",
        status: Status::Working,
        version: "0.2",
    },
    Row {
        name: "Kde + Bandwidth (Silverman/Scott) + Kernel",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "BoxPlot summary statistics (Tukey fences)",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "ViolinMark (KDE-based)",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "Auto-attached Colorbar",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "RegressionMark / smoothing (LOESS, polynomial)",
        status: Status::Planned,
        version: "0.5",
    },
    Row {
        name: "Spectral analysis helpers",
        status: Status::Planned,
        version: "post-1.0",
    },
];

pub fn matrix() -> Matrix<'static> {
    Matrix {
        stem: "stats",
        title: "starsight statistics — current and planned",
        rows: ROWS,
        footnote: Some(
            "Stats are exposed as composable types — Kde, Bandwidth, Kernel, etc. — independent of the marks that consume them.",
        ),
    }
}
