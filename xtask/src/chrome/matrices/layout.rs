//! Layout & composition status matrix.

use super::{Matrix, Row, Status};

const ROWS: &[Row<'_>] = &[
    Row { name: "Figure (single-panel)", status: Status::Working, version: "0.1" },
    Row { name: "MultiPanelFigure (rows × cols grid)", status: Status::Working, version: "0.3" },
    Row { name: "Legend default placement", status: Status::Working, version: "0.1" },
    Row { name: "Legend least-overlap fallback (count → area → TR>TL>BR>BL)", status: Status::Working, version: "0.3" },
    Row { name: "LegendPosition::Inside (corner-anchored)", status: Status::Working, version: "0.3" },
    Row { name: "LegendPosition::Outside (Edge slot)", status: Status::Working, version: "0.3" },
    Row { name: "Colorbar (auto-attached on continuous color)", status: Status::Working, version: "0.3" },
    Row { name: "GridLayout / facet placeholder", status: Status::Planned, version: "0.4" },
    Row { name: "FacetWrap", status: Status::Planned, version: "0.4" },
    Row { name: "Shared axes across panels", status: Status::Planned, version: "0.4" },
    Row { name: "Per-panel title + uniform tick count", status: Status::Planned, version: "0.4" },
    Row { name: "Polar-aware legend placement", status: Status::Planned, version: "0.4" },
    Row { name: "Contour filled bands", status: Status::Planned, version: "0.4" },
];

pub fn matrix() -> Matrix<'static> {
    Matrix {
        stem: "layout",
        title: "starsight layout & composition — current and planned",
        rows: ROWS,
        footnote: Some("Layout is the bridge between marks and the canvas — it owns axes, legends, and colorbars."),
    }
}
