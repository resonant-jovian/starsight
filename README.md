<div align="center">

<h1>starsight</h1>

**A unified scientific visualization crate for Rust — from zero-config one-liners to GPU-accelerated interactive 3D.**

[![Sponsor](https://img.shields.io/badge/Sponsor-resonant--jovian-ea4aaa?style=for-the-badge&logo=githubsponsors&logoColor=white)](https://github.com/sponsors/resonant-jovian)
[![Support on thanks.dev](https://img.shields.io/badge/thanks.dev-Support-green?style=for-the-badge&logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZmlsbD0id2hpdGUiIGQ9Ik0xMiAyMWMtNS41IDAtMTAtMy41LTEwLTkgMC00IDItNy41IDYtMTAgMS41IDIuNSAzIDQuNSA0IDQuNSAxLTEuNSAyLjUtMy41IDQtNC41IDQuNSAyLjUgNiA2IDYgMTAgMCA1LjUtNC41IDktMTAgOXoiLz48L3N2Zz4=)](https://thanks.dev/u/gh/resonant-jovian)

[![Crates.io](https://img.shields.io/crates/v/starsight?style=for-the-badge&logo=rust&logoColor=white&label=crates.io)](https://crates.io/crates/starsight)
[![docs.rs](https://img.shields.io/docsrs/starsight?style=for-the-badge&logo=docsdotrs&logoColor=white&label=docs.rs)](https://docs.rs/starsight)
[![Downloads](https://img.shields.io/crates/d/starsight?style=for-the-badge&logo=rust&logoColor=white&color=e6761b)](https://crates.io/crates/starsight)

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-3366cc?style=for-the-badge&logo=gnu&logoColor=white)](https://www.gnu.org/licenses/gpl-3.0)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-3366cc?style=for-the-badge&logo=rust&logoColor=white)](https://releases.rs/docs/1.85.0/)
[![Edition](https://img.shields.io/badge/Edition-2024-3366cc?style=for-the-badge&logo=rust&logoColor=white)](https://doc.rust-lang.org/edition-guide/)

[![CI](https://img.shields.io/github/actions/workflow/status/resonant-jovian/starsight/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=ci)](https://github.com/resonant-jovian/starsight/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/github/actions/workflow/status/resonant-jovian/starsight/coverage.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=coverage)](https://github.com/resonant-jovian/starsight/actions/workflows/coverage.yml)
[![Gallery](https://img.shields.io/github/actions/workflow/status/resonant-jovian/starsight/gallery.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=gallery)](https://github.com/resonant-jovian/starsight/actions/workflows/gallery.yml)
[![Release](https://img.shields.io/github/actions/workflow/status/resonant-jovian/starsight/release.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=release)](https://github.com/resonant-jovian/starsight/actions/workflows/release.yml)
[![Snapshots](https://img.shields.io/github/actions/workflow/status/resonant-jovian/starsight/snapshots.yml?style=for-the-badge&logo=githubactions&logoColor=white&label=snapshots)](https://github.com/resonant-jovian/starsight/actions/workflows/snapshots.yml)

</div>

> [!CAUTION]
> starsight is pre-release software. The API will be considered stable at 1.0.0.

---

Part of the [resonant-jovian](https://github.com/resonant-jovian) ecosystem alongside [prismatica](https://github.com/resonant-jovian/prismatica) (308 scientific colormaps), [chromata](https://github.com/resonant-jovian/chromata) (1,104 editor color themes), [caustic](https://github.com/resonant-jovian/caustic) (Vlasov-Poisson solver), and [phasma](https://github.com/resonant-jovian/phasma) (caustic TUI).

---

## Quickstart

> [!TIP]
> If the docs are insufficient, please tell me, I want people to use this after all!

```toml
[dependencies]
starsight = "0.1"
```

```rust
use starsight::prelude::*;

let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
let y = vec![2.3, 4.1, 3.8, 5.7, 4.9];
plot!(x, y).save("chart.png");
```

<!-- TBD: screenshot of output once rendering works -->

---

## Features

> [!IMPORTANT]
> Features that are wip or mvp are marked, don't use them in production.

<!-- TBD: fill in as features become functional -->

| Feature | Status | Description |
|---------|--------|-------------|
| CPU rendering (tiny-skia) | wip | Headless rasterization, PNG export |
| SVG export | wip | Scalable vector output |
| Line / Scatter / Bar charts | planned | Core 2D chart types |
| Grammar of graphics API | planned | Composable `Figure` builder |
| `plot!()` macro | planned | Zero-config one-liner |
| Polars DataFrame integration | planned | Column references by name |
| Statistical charts | planned | Violin, KDE, box plot, etc. |
| GPU rendering (wgpu) | planned | Native windows, WebGPU WASM |
| Terminal rendering | planned | Kitty / Sixel / Braille |
| 3D visualization | planned | Surface, scatter, isosurface |
| PDF export | planned | Publication-quality vector |
| Interactivity | planned | Hover, zoom, pan, selection |

---

## Usage

> [!NOTE]
> This is just what is in the `docs` main page.

<!-- TBD: copy from lib.rs doc comment once written -->

---

## Flags

> [!TIP]
> All listed flags are ready to be used.

| Flag | Default | Description |
|------|---------|-------------|
| `default` | yes | CPU rendering via tiny-skia, SVG, PNG export |
| `full` | no | All features enabled |
| `minimal` | no | Core types only, no rendering |
| `science` | no | `stats` + `contour` + `3d` + `pdf` |
| `dashboard` | no | `interactive` + `gpu` + `polars` |
| `terminal` | no | TUI rendering via ratatui |
| `web` | no | WASM + WebGPU browser target |
| `gpu` | no | wgpu + vello GPU rendering |
| `interactive` | no | winit + egui interactive windows |
| `polars` | no | Polars DataFrame acceptance |
| `ndarray` | no | ndarray acceptance |
| `arrow` | no | Apache Arrow RecordBatch acceptance |
| `3d` | no | 3D chart types via nalgebra |
| `pdf` | no | PDF export via krilla |
| `stats` | no | Statistical transforms via statrs |
| `contour` | no | Isoline generation |
| `geo` | no | Geospatial chart types |
| `resvg` | no | SVG-to-PNG rasterization |

---

## Versions

> [!TIP]
> Until `1.0.0` make sure to pin your version.

[Changelog for this project](CHANGELOG.md)

---

## Structure

> [!NOTE]
> Crate dependency graph:

```
starsight
├── starsight-figure
│   ├── starsight-layout
│   │   └── starsight-marks
│   │       └── starsight-core
│   └── starsight-marks
├── starsight-interact
│   └── starsight-figure
├── starsight-export
│   └── starsight-figure
├── starsight-gpu
│   └── starsight-core
└── starsight-derive
```

| Crate | Layer | Role |
|-------|-------|------|
| `starsight` | Facade | Re-exports everything, the only crate users depend on |
| `starsight-core` | 1-2 | Rendering backends, scales, axes, color, text, geometry primitives |
| `starsight-marks` | 3 | Geom/mark system, stat transforms, aesthetic mappings |
| `starsight-layout` | 4 | Grid layout, faceting, legends, colorbars |
| `starsight-figure` | 5 | Figure builder, `plot!()` macro, data acceptance |
| `starsight-interact` | 6 | Hover, zoom, pan, selection, streaming |
| `starsight-export` | 7 | Animation, PNG/SVG/PDF/HTML export, terminal output |
| `starsight-gpu` | 1 | wgpu + vello GPU backend (optional) |
| `starsight-derive` | -- | Proc macros (`#[starsight::recipe]`) |

---

## Contribution

[Contribution guidelines for this project](CONTRIBUTING.md)

---

## Roadmap

- [ ] 0.1.0 Foundation
  - [ ] Fix core-marks circular dependency (geometry primitives in core)
  - [ ] `DrawBackend` trait implementation (tiny-skia)
  - [ ] SVG backend
  - [ ] `LinearScale` with Wilkinson Extended tick generation
  - [ ] Axis rendering (X/Y with labels and ticks)
  - [ ] `LineMark` and `PointMark`
  - [ ] `Figure` builder, `plot!()` macro
  - [ ] First snapshot tests
- [ ] 0.2.0 Core charts (Bar, Area, Histogram, Heatmap)
- [ ] 0.3.0 Statistical charts (BoxPlot, Violin, KDE, Pie)
- [ ] 0.4.0 Layout (Grid, Faceting, Legends, PairPlot)
- [ ] 0.5.0 Scale infrastructure (Log, Symlog, DateTime, Band)
- [ ] 0.6.0 GPU + Interactivity (wgpu backend, hover/zoom/pan)
- [ ] 0.7.0 3D visualization (Surface, Scatter3D, Isosurface)
- [ ] 0.8.0 Terminal backend (Kitty/Sixel/Braille, ratatui widget)
- [ ] 0.9.0 Remaining chart types (all 66)
- [ ] 0.10.0 Export + WASM (PDF, HTML interactive, WebGPU)
- [ ] 0.11.0 Recipe system, ndarray/Arrow, API polish
- [ ] 0.12.0 Documentation, examples, gallery
- [ ] 1.0.0 Stable release

Full task-level roadmap with 637 checkboxes: [`.spec/STARSIGHT.md`](.spec/STARSIGHT.md)

---

## Citation

> [!IMPORTANT]
> Cite me :)

<!-- TBD: add CITATION.cff and BibTeX once published -->

```bibtex
@software{starsight,
  author  = {Sjögren, Albin},
  title   = {starsight: unified scientific visualization for Rust},
  url     = {https://github.com/resonant-jovian/starsight},
  license = {GPL-3.0-only}
}
```

---

## Minimum supported Rust version

Rust edition 2024, targeting **stable Rust 1.85+**. MSRV tracks latest stable minus two releases, consistent with wgpu and ratatui.

---

## Support

> [!NOTE]
> Everyone needs money.

If **starsight** is useful to your projects, or if you use it to earn money, consider supporting development via [GitHub Sponsors](https://github.com/sponsors/resonant-jovian) or [thanks.dev](https://thanks.dev/u/gh/resonant-jovian).

---

## License

> [!WARNING]
> This licence is intentional and I expect it to be followed and inherited.

This project is licensed under the [GNU General Public License v3.0](LICENSE).