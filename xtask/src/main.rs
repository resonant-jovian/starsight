//! `cargo xtask` — workspace-local build automation.
//!
//! Sub-commands:
//! - `cargo xtask gallery`             — render every example to `target/gallery/`.
//! - `cargo xtask showcase`            — symlink every example PNG into `showcase/`.
//! - `cargo xtask svgs`                — export SVG snapshots to a viewable `.svg-preview/`.
//! - `cargo xtask snapshots review`    — open `cargo insta review`.
//! - `cargo xtask snapshots accept`    — accept all pending snapshots.
//! - `cargo xtask snapshots prune`     — delete orphan `.snap` files.
//! - `cargo xtask benches`             — run the benchmarks suite (stub).

mod benches;
mod gallery;
mod showcase;
mod snapshots;
mod svgs;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask", about = "Workspace-local build automation")]
enum Cmd {
    /// Run every example and aggregate outputs in target/gallery/.
    Gallery,
    /// Symlink every example PNG into a flat `showcase/` directory.
    Showcase,
    /// Export SVG snapshots into a viewable `.svg-preview/` directory.
    Svgs,
    /// Insta snapshot management (review / accept / prune).
    Snapshots {
        #[command(subcommand)]
        action: SnapshotsAction,
    },
}

#[derive(Subcommand)]
enum SnapshotsAction {
    /// Open `cargo insta review` to inspect pending snapshots.
    Review,
    /// Run `cargo insta accept` to accept all pending snapshots.
    Accept,
    /// Delete orphan `.snap` files (snapshots with no matching test).
    Prune,
}

fn main() -> anyhow::Result<()> {
    match Cmd::parse() {
        Cmd::Gallery => gallery::run(),
        Cmd::Showcase => showcase::run(),
        Cmd::Svgs => svgs::run(),
        Cmd::Snapshots { action } => match action {
            SnapshotsAction::Review => snapshots::review(),
            SnapshotsAction::Accept => snapshots::accept(),
            SnapshotsAction::Prune => snapshots::prune(),
        },
    }
}
