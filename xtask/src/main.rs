//! `cargo xtask` — workspace-local build automation.
//!
//! Sub-commands:
//! - `cargo xtask gallery`   — render every example to `target/gallery/`.
//! - `cargo xtask showcase`  — symlink every example PNG into `showcase/`.
//! - `cargo xtask benches`   — run the benchmarks suite (stub).
//! - `cargo xtask snapshots` — manage `insta` snapshot files (stub).

mod benches;
mod gallery;
mod showcase;
mod snapshots;

use clap::Parser;

#[derive(Parser)]
#[command(name = "xtask", about = "Workspace-local build automation")]
enum Cmd {
    /// Run every example and aggregate outputs in target/gallery/.
    Gallery,
    /// Symlink every example PNG into a flat `showcase/` directory.
    Showcase,
}

fn main() -> anyhow::Result<()> {
    match Cmd::parse() {
        Cmd::Gallery => gallery::run(),
        Cmd::Showcase => showcase::run(),
    }
}
