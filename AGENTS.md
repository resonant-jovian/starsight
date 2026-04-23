# AGENTS.md — starsight

Quick reference for agents working in this repo.

## Build & Test

```bash
# Full workspace build
cargo build --workspace

# Run all tests
cargo test --workspace

# Run tests for a single layer (e.g., layer-3)
cargo test -p starsight-layer-3

# Run snapshot tests
cargo xtask snapshots         # update: pass --write to update
cargo xtask snapshots --check  # CI mode

# Lint order: fmt -> clippy -> typecheck -> test
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-features
cargo test --workspace
```

## Workspace Structure

- **starsight** (facade): re-exports layers 1-7
- **starsight-layer-1**: primitives, backends (SkiaBackend, SvgBackend), errors
- **starsight-layer-2**: scales, coords, axes, ticks
- **starsight-layer-3**: marks (LineMark, PointMark, BarMark, AreaMark)
- **starsight-layer-4**: layout (grid, legend)
- **starsight-layer-5**: Figure, plot! macro
- **starsight-layer-6**: interactivity (winit)
- **starsight-layer-7**: export (PDF, GIF, HTML, WASM)

## Module Paths (current)

- Errors: `crate::errors::{Result, StarsightError}` (not `crate::error`)
- Backends: `crate::backends::DrawBackend` (not `crate::backend`)
- Skia: `starsight_layer_1::backends::skia::SkiaBackend`
- Coords: `starsight_layer_2::coords::CartesianCoord`
- Paths: `crate::paths::{Path, PathCommand, PathStyle}`

## Key Conventions

- **No global state**: `plot!(x, y)` returns a Figure, no `plt.show()`
- **Builder pattern**: `Figure::new(800, 600).title("...").add(mark)`
- **NaN = gap**: LineMark treats NaN values as breaks in the line
- **Returns Result**: All public APIs return `Result<T>`, handle or propagate
- **MSRV 1.89**: pinned in workspace Cargo.toml, checked in CI
- **Edition 2024**: some Rust 2024 idioms, `std::path::Path` not `::std`

## Test Fixtures

- Snapshot tests in `starsight-layer-5/tests/snapshot.rs`
- Reference PNGs in `docs/screenshots/`
- SVG backend used for deterministic CI renders

## What Works Now (0.1.x)

- LineMark, PointMark, BarMark, AreaMark
- SkiaBackend, SvgBackend
- Figure + plot! macro
- Wilkinson Extended ticks (layer-2)
- 9 snapshot tests in layer-5

## Not Implemented (yet)

- Polars/DataFrame integration
- HeatmapMark, BoxPlotMark, ViolinMark, PieMark
- Feature flags: `interactive`, `polars`, `wasm`