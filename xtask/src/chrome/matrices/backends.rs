//! Backends status matrix.

use super::{Matrix, Row, Status};

const ROWS: &[Row<'_>] = &[
    Row { name: "SkiaBackend (CPU raster · .png/.jpeg)", status: Status::Working, version: "0.1" },
    Row { name: "SvgBackend (vector · .svg)", status: Status::Working, version: "0.1" },
    Row { name: "WgpuBackend (GPU surface)", status: Status::Planned, version: "0.6" },
    Row { name: "KrillaBackend (vector · .pdf)", status: Status::Planned, version: "0.10" },
    Row { name: "WasmBackend (browser canvas)", status: Status::Planned, version: "0.10" },
    Row { name: "Kitty terminal protocol", status: Status::Planned, version: "0.8" },
    Row { name: "Sixel terminal protocol", status: Status::Planned, version: "0.8" },
    Row { name: "iTerm2 terminal protocol", status: Status::Planned, version: "0.8" },
    Row { name: "half-block terminal cells", status: Status::Planned, version: "0.8" },
    Row { name: "Braille terminal cells", status: Status::Planned, version: "0.8" },
];

pub fn matrix() -> Matrix<'static> {
    Matrix {
        stem: "backends",
        title: "starsight rendering backends — current and planned",
        rows: ROWS,
        footnote: Some("DrawBackend is the only trait marks see; new backends slot in without touching marks."),
    }
}
