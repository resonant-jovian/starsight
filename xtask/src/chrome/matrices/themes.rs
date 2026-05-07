//! Themes & colormaps status matrix.
//!
//! starsight pulls colour identity from two sister crates: `chromata` (1,104
//! editor / terminal themes as compile-time constants) and `prismatica`
//! (260+ perceptually uniform colormaps). The matrix surfaces both the
//! integration points and the upstream content sizes.

use super::{Matrix, Row, Status};

const ROWS: &[Row<'_>] = &[
    Row {
        name: "chromata: 1,104 editor / terminal themes",
        status: Status::Working,
        version: "shipped",
    },
    Row {
        name: "prismatica: 260+ colormaps (Viridis/Inferno/Plasma/…)",
        status: Status::Working,
        version: "shipped",
    },
    Row {
        name: "Theme integration (Figure::theme)",
        status: Status::Working,
        version: "0.1",
    },
    Row {
        name: "Light + dark default themes",
        status: Status::Working,
        version: "0.1",
    },
    Row {
        name: "Colormap integration (HeatmapMark, ContourMark)",
        status: Status::Working,
        version: "0.2",
    },
    Row {
        name: "Per-mark color_by(&groups)",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "Auto-attached Colorbar on continuous color",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "User-defined chromata::Theme",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "Theme dark-mode helpers (env-driven)",
        status: Status::Working,
        version: "0.3",
    },
    Row {
        name: "Diverging/cyclic colormap helpers",
        status: Status::Planned,
        version: "0.5",
    },
    Row {
        name: "Colorblind-safety annotations",
        status: Status::Planned,
        version: "post-1.0",
    },
];

pub fn matrix() -> Matrix<'static> {
    Matrix {
        stem: "themes",
        title: "starsight themes & colormaps — current and planned",
        rows: ROWS,
        footnote: Some(
            "All themes and colormaps are compile-time constants — no runtime parsing or asset loading.",
        ),
    }
}
