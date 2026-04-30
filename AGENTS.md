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
- **starsight-layer-3**: marks + stats (LineMark, PointMark, BarMark, AreaMark, StepMark, HistogramMark, HeatmapMark, BoxPlotMark, ViolinMark, PieMark, CandlestickMark; Kde, BoxPlotStats, Bandwidth helpers)
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
- Reference PNGs in `examples/<group>/`, alongside each `.rs` source (run `cargo xtask gallery` to refresh them all)
- SVG backend used for deterministic CI renders

## What Works Now (0.3.x)

- LineMark, PointMark (per-point colors/radii/alpha), BarMark (vertical/horizontal/grouped/per-bar bases+colors+connectors), AreaMark (with baseline), HeatmapMark (Linear + Log color scale), HistogramMark (auto-binning), StepMark
- BoxPlotMark + BoxPlotGroup (five-number summary, whiskers, outliers, palette)
- ViolinMark + ViolinGroup + ViolinScale (KDE-driven density, optional inner box overlay, split mode, Area/Count/Width normalization)
- PieMark + PieLabelMode (solid pie / donut via inner_radius, percent / value label modes, 6-color default palette)
- CandlestickMark + Ohlc (financial OHLC bars with up/down body color dispatch)
- Polars data integration (`polars` feature) — FrameSource + extract helpers + `plot!(df, x="col", y="col", color="col")`
- BandScale for categorical x-axis; `infer_chart_kind` chart-type inference
- `Color::cycle_next` (Tableau10 default palette); LegendGlyph dispatch (line / point / bar / area glyphs)
- Title and axis-label rendering; LayoutFonts shared between layout and render passes
- SkiaBackend, SvgBackend (with opacity)
- Figure + plot! macro (DataFrame arm gated on the `polars` feature)
- Wilkinson Extended ticks (layer-2)
- Stat infrastructure: BoxPlotStats, Kde (Gaussian + Silverman/Scott/Manual bandwidth), percentile, std_dev
- Snapshot tests in layer-5

## Not Implemented (yet)

- Layout: GridLayout, faceting, legends, colorbars (0.4.0)
- LogScale, SymLogScale, DateTimeScale (0.5.0)
- GPU + interactivity: wgpu, hover/zoom/pan (0.6.0)
- Animation, GIF (0.7.0)
- Terminal backends (0.8.0)
- 3D: Surface3D, Scatter3D (0.9.0)
- PDF (krilla), interactive HTML, WASM (0.10.0)
- ndarray / Arrow input (0.11.0; Polars landed early in 0.3.0)

<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
