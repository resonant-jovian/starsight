---
paths:
  - "starsight-layer-*/tests/snapshot.rs"
  - "**/snapshots/**"
---

# Snapshot tests (insta)

Every layer has a `tests/snapshot.rs` file. **Layers 1 and 5 are populated today** (the natural fits — layer-1 backends are byte-exact at the primitive level, layer-5 is the full Figure pipeline). Layers 2, 3, 4, 6, 7 have empty placeholder files that will fill out as their surfaces stabilize. Add tests to a layer's `snapshot.rs` whenever a behavior in that layer becomes worth pinning visually or byte-exactly.

## PNG vs SVG — pick by determinism

- **SVG (`insta::assert_snapshot!(svg)`)** when the test exercises any path that touches **font layout / cosmic-text** (titles, axis labels, legend text). Used by layer-5's full-pipeline snapshots and layer-1's `svg_blue_rect`. SVG keeps `<text>` unrasterized so it's byte-exact across OS/font setups.
- **PNG (`insta::assert_binary_snapshot!(".png", bytes)`)** only when the test is **backend-pure** — no text, no font, just primitives. Layer-1's `blue_rect_on_white` is the canonical example.
- Default to SVG. Reach for PNG only when there's a specific reason (e.g. you're testing the raster encoder itself).

## Local workflow

```bash
# Run all snapshot tests across layers
cargo test --workspace --test snapshot

# Or one layer
cargo test -p starsight-layer-1 --test snapshot
cargo test -p starsight-layer-5 --test snapshot

# Update on the spot
INSTA_UPDATE=always cargo test --workspace --test snapshot

# Or accept after a normal run
cargo insta accept

# CI invariant — fails on any pending snapshot or orphaned .snap
cargo insta test --workspace --check --unreferenced reject
```

`cargo xtask snapshots` is a stub — don't use it.

## Adding a test

- Use realistic, deterministic data. Layer-5 uses domain helpers like `damped_cosine(n)` (physics / weather / statistics). Match that style.
- Test name: `snapshot_<thing>_<variant>` — e.g. `snapshot_violin_split`, `blue_rect_on_white`.
- Render at a non-default size when it helps reveal layout: `Figure::new(1200, 800)` is common in layer-5.
- Run with `INSTA_UPDATE=always`, eyeball the resulting snapshot for sanity, then commit both the test code and the snapshot file.

## Removing or renaming

`cargo insta test --workspace --check --unreferenced reject` is the orphan-detector. Run it after any rename or removal — orphan `.snap` files fail CI.

## Commit 946d278

`fix(snapshots): drop font-rasterized PNG asserts from layer-5 tests` — a previous attempt put PNG asserts on the full pipeline; they were removed because font rasterization is non-deterministic across hosts. The lesson encoded above (PNG only when backend-pure) comes from that.
