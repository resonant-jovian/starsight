//! Comparison with sibling Rust crates and Python/R libraries.

use super::Table;

const HEADER: &[&str] = &[
    "library",
    "ecosystem",
    "strengths",
    "when to prefer starsight",
];
const COL_W: &[u32] = &[170, 100, 310, 292];
const COL_ALIGN: &[&str] = &["start", "start", "start", "start"];

const ROWS: &[&[&str]] = &[
    // Rust crates
    &[
        "plotters",
        "Rust",
        "mature, many backends, WASM-ready",
        "want typed marks; want to compose figures rather than imperatively draw",
    ],
    &[
        "plotly-rs",
        "Rust",
        "interactive HTML out of the box",
        "want static SVG/PNG; do not want a JS runtime",
    ],
    &[
        "charming",
        "Rust",
        "ECharts-quality visuals",
        "do not want to ship a JS engine (deno_core) at runtime",
    ],
    &[
        "plotters-iced / egui_plot",
        "Rust",
        "live in a GUI",
        "want headless rendering as the primary path",
    ],
    &[
        "poloto",
        "Rust",
        "small, no_std-friendly, terminal-first",
        "want polar / contour / candlestick + academic publication output",
    ],
    // Non-Rust
    &[
        "matplotlib",
        "Python",
        "vast scientific ecosystem; everyone knows it",
        "want a compiled-language pipeline; type-safe data flow; no GIL",
    ],
    &[
        "seaborn",
        "Python",
        "statistical defaults; tidy-data API",
        "want builder-style composition without a global pyplot",
    ],
    &[
        "ggplot2 (and plotnine)",
        "R / Python",
        "grammar of graphics canon",
        "want compile-time-checked figures + native multi-threading",
    ],
    &[
        "plotly.py / vega-altair",
        "Python",
        "interactive HTML, declarative spec",
        "want native rendering, no JS runtime, no browser dependency",
    ],
    &[
        "Mathematica / MATLAB",
        "proprietary",
        "scientific REPL with built-in plotting",
        "want a free, embeddable, redistributable library",
    ],
    &[
        "gnuplot",
        "C",
        "decades-old; canonical CLI plotter",
        "want type-safe data structures + modern Rust toolchain integration",
    ],
];

pub fn table() -> Table<'static> {
    Table {
        stem: "comparison",
        title: "starsight vs. sibling charting libraries (Rust + Python + others)",
        header: HEADER,
        rows: ROWS,
        col_widths: COL_W,
        col_align: COL_ALIGN,
        col_font: None,
    }
}
