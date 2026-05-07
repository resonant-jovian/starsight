//! Output / export formats status matrix.

use super::{Matrix, Row, Status};

const ROWS: &[Row<'_>] = &[
    Row { name: ".png raster", status: Status::Working, version: "0.1" },
    Row { name: ".svg text (via SvgBackend)", status: Status::Working, version: "0.1" },
    Row { name: ".jpeg raster", status: Status::Working, version: "0.2" },
    Row { name: "Raw RGBA buffer", status: Status::Working, version: "0.2" },
    Row { name: ".pdf vector (via krilla)", status: Status::Planned, version: "0.10" },
    Row { name: ".gif (animation pipeline)", status: Status::Planned, version: "0.7" },
    Row { name: ".html interactive (WebGPU + DOM)", status: Status::Planned, version: "0.10" },
    Row { name: ".wasm browser embed", status: Status::Planned, version: "0.10" },
    Row { name: "tty pixel (Kitty/Sixel/iTerm2)", status: Status::Planned, version: "0.8" },
    Row { name: "tty cell (half-block/Braille)", status: Status::Planned, version: "0.8" },
];

pub fn matrix() -> Matrix<'static> {
    Matrix {
        stem: "output",
        title: "starsight output formats — current and planned",
        rows: ROWS,
        footnote: Some("Figure::save dispatches on file extension; non-file outputs (window, tty, browser) use dedicated backends."),
    }
}
