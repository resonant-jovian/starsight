//! Feature-flag table.

use super::{Family, Table};

const HEADER: &[&str] = &["flag", "what it adds"];
const COL_W: &[u32] = &[200, 672];
const COL_ALIGN: &[&str] = &["start", "start"];
const COL_FONT: &[Family] = &[Family::Mono, Family::Sans];

const ROWS: &[&[&str]] = &[
    &["polars", "accept polars::DataFrame columns directly"],
    &["ndarray", "accept ndarray::ArrayN views (planned 0.11)"],
    &["arrow", "accept arrow::RecordBatch (planned 0.11)"],
    &["gpu", "wgpu + vello GPU rendering (planned 0.6)"],
    &[
        "interactive",
        "winit + egui interactive windows (planned 0.6)",
    ],
    &[
        "terminal",
        "TUI via ratatui — Kitty / Sixel / iTerm2 / half-block / Braille (planned 0.8)",
    ],
    &["pdf", "PDF export via krilla (planned 0.10)"],
    &["web", "WASM + WebGPU browser target (planned 0.10)"],
    &["3d", "3D chart types via nalgebra (planned 0.9)"],
];

pub fn table() -> Table<'static> {
    Table {
        stem: "install",
        title: "starsight feature flags",
        header: HEADER,
        rows: ROWS,
        col_widths: COL_W,
        col_align: COL_ALIGN,
        col_font: Some(COL_FONT),
    }
}
