//! `cargo xtask chrome` — regenerate documentation chrome assets.
//!
//! Six chrome assets feed the README:
//!
//! | asset | live? | regenerator |
//! |-------|-------|-------------|
//! | `assets/status/panel-light.svg`     | yes (crates.io API)             | [`status_panel`] |
//! | `assets/hero/starsight-hero.png`    | meta strip from `Cargo.toml`    | [`hero`] |
//! | `assets/roadmap-light.svg`          | "current" derived from version  | [`roadmap`] |
//! | `assets/architecture-light.svg`     | static                          | [`architecture`] |
//! | `assets/gallery-light.png`          | static (rebuilds when examples regenerate) | [`gallery`] |
//! | `assets/wordmark-light.svg`         | static                          | [`wordmark`] |
//!
//! `--live` regenerates only the three live-data assets — used by the daily CI
//! workflow. Default regenerates everything.

mod architecture;
mod crates_io;
mod eclipse;
mod gallery;
mod hero;
mod palette;
mod roadmap;
mod status_panel;
mod svg;
mod wordmark;

use anyhow::Result;
use clap::{Args, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct ChromeArgs {
    /// Only regenerate live-data assets (status panel, hero, roadmap).
    #[arg(long)]
    pub live: bool,
    /// Regenerate a single named asset and exit.
    #[arg(long)]
    pub asset: Option<Asset>,
}

#[derive(Copy, Clone, ValueEnum)]
pub enum Asset {
    StatusPanel,
    Hero,
    Roadmap,
    Architecture,
    Gallery,
    Wordmark,
}

pub fn run(args: ChromeArgs) -> Result<()> {
    let root = repo_root();
    let assets = root.join("assets");
    std::fs::create_dir_all(assets.join("hero"))?;
    std::fs::create_dir_all(assets.join("status"))?;

    if let Some(asset) = args.asset {
        return regen_one(asset, &root);
    }

    // Live-data assets always run.
    status_panel::regen(&root)?;
    hero::regen(&root)?;
    roadmap::regen(&root)?;

    if !args.live {
        architecture::regen(&root)?;
        gallery::regen(&root)?;
        wordmark::regen(&root)?;
    }
    Ok(())
}

fn regen_one(asset: Asset, root: &Path) -> Result<()> {
    match asset {
        Asset::StatusPanel => status_panel::regen(root),
        Asset::Hero => hero::regen(root),
        Asset::Roadmap => roadmap::regen(root),
        Asset::Architecture => architecture::regen(root),
        Asset::Gallery => gallery::regen(root),
        Asset::Wordmark => wordmark::regen(root),
    }
}

fn repo_root() -> PathBuf {
    // xtask runs from the workspace root via `cargo run -p xtask`.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is a member of the workspace")
        .to_path_buf()
}
