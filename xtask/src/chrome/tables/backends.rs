//! Backends matrix.

use super::{Family, Table};

const HEADER: &[&str] = &[
    "backend",
    "output",
    "dependencies",
    "feature flag",
    "status",
];
const COL_W: &[u32] = &[160, 230, 200, 130, 112];
const COL_ALIGN: &[&str] = &["start", "start", "start", "start", "end"];
const COL_FONT: &[Family] = &[
    Family::Mono,
    Family::Sans,
    Family::Sans,
    Family::Mono,
    Family::Sans,
];

const ROWS: &[&[&str]] = &[
    &[
        "SkiaBackend",
        ".png / .jpeg / raw RGBA",
        "tiny-skia",
        "default",
        "stable",
    ],
    &["SvgBackend", ".svg text", "none", "default", "stable"],
    &[
        "WgpuBackend",
        "GPU surface, headless or windowed",
        "wgpu, vello",
        "gpu",
        "planned 0.6",
    ],
    &[
        "RatatuiBackend",
        "TUI cells (Kitty / Sixel / iTerm2 / half-block / Braille)",
        "ratatui",
        "terminal",
        "planned 0.8",
    ],
    &["KrillaBackend", ".pdf", "krilla", "pdf", "planned 0.10"],
    &[
        "WasmBackend",
        "<canvas> in browser",
        "wasm-bindgen, web-sys",
        "web",
        "planned 0.10",
    ],
];

pub fn table() -> Table<'static> {
    Table {
        stem: "backends",
        title: "starsight rendering backends",
        header: HEADER,
        rows: ROWS,
        col_widths: COL_W,
        col_align: COL_ALIGN,
        col_font: Some(COL_FONT),
    }
}
