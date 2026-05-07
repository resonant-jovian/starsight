//! Scales status matrix.

use super::{Matrix, Row, Status};

const ROWS: &[Row<'_>] = &[
    Row {
        name: "LinearScale",
        status: Status::Working,
        version: "0.1",
    },
    Row {
        name: "Wilkinson Extended ticks",
        status: Status::Working,
        version: "0.1",
    },
    Row {
        name: "LogScale",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "SqrtScale",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "CategoricalScale",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "PolarCoord",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "BandScale",
        status: Status::Planned,
        version: "0.5",
    },
    Row {
        name: "SymLogScale",
        status: Status::Planned,
        version: "0.5",
    },
    Row {
        name: "DateTimeScale",
        status: Status::Planned,
        version: "0.5",
    },
    Row {
        name: "TimedeltaScale",
        status: Status::Planned,
        version: "0.5",
    },
];

pub fn matrix() -> Matrix<'static> {
    Matrix {
        stem: "scales",
        title: "starsight scales — current and planned",
        rows: ROWS,
        footnote: Some(
            "Each Scale type maps domain → range; tick generation is composable per-scale.",
        ),
    }
}
