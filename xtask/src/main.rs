//! `cargo xtask` — workspace-local build automation.
//!
//! Sub-commands:
//! - `cargo xtask gallery` — render every example to `target/gallery/`.
//! - `cargo xtask benches` — run the benchmarks suite.
//! - `cargo xtask snapshots` — manage `insta` snapshot files.

mod benches;
mod gallery;
mod snapshots;

fn main() {
    // TODO(0.1.0): parse argv with `clap` and dispatch to gallery / benches / snapshots.
}
