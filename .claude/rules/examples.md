---
paths:
  - "examples/**/*.rs"
  - "examples/**/*.png"
  - "showcase/**"
---

# Examples & showcase

Examples are grouped, with each `.rs` source co-located with its rendered `.png`.

## Groups

- `examples/basics/` — quickstart, scatter, line, bar, heatmap, histogram, bubble.
- `examples/composition/` — multi-mark figures (recipe, gallery composite, statistical, waterfall, pie, donut, box-plot, violin).
- `examples/data/` — data-source integrations (currently `polars_integration`).
- `examples/planned/` — placeholders for backend-blocked features (faceting, interactive, streaming). Outputs are static placeholder PNGs flagged in `README.md` as not-yet-implemented.
- `examples/scientific/` — domain-specific (planned breadth: physics, biology, ML).
- `examples/theming/` — theme demonstrations.

## Pipeline

- `cargo xtask gallery` — runs every example, writes outputs to `target/gallery/<group>_<name>.png`. After 262166d (`gallery build performance`) it builds the example binaries once and execs them per invocation, so a full gallery is fast.
- `cargo xtask showcase` — symlinks every gallery PNG into the flat `showcase/` directory at the repo root.
- The `.png` checked into each example dir is the canonical render — committed alongside the source. Keep them in sync: if you change the source, regenerate.

## Example shape

```rust
use starsight::prelude::*;

fn main() -> starsight::Result<()> {
    plot!(&xs, &ys).title("…").save("basics_line_chart.png")
}
```

The output filename should match the file name `<group>_<name>.png` so `cargo xtask gallery` aggregates correctly.

## What goes here vs `showcase/`

- **examples/**: source + canonical PNG, browsable per-group.
- **showcase/**: flat PNG directory (symlinks) for the README image gallery.
- **`.spec/SHOWCASE_INPUTS.md`**: input-data specs the showcase examples depend on.
