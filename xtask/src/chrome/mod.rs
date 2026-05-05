//! `cargo xtask chrome` — regenerate documentation chrome assets.
//!
//! Six paired chrome assets feed the README (12 files total — light + dark each):
//!
//! | asset | live? |
//! |-------|-------|
//! | `assets/status/panel-{light,dark}.svg`     | yes (crates.io API)             |
//! | `assets/hero/starsight-hero-{light,dark}.png` | meta strip from `Cargo.toml` + theme-matched example thumbs |
//! | `assets/roadmap-{light,dark}.svg`          | "current" derived from version  |
//! | `assets/architecture-{light,dark}.svg`     | static                          |
//! | `assets/gallery-{light,dark}.png`          | static (theme-matched thumbs)   |
//! | `assets/wordmark-{light,dark}.svg`         | static                          |
//!
//! `--live` regenerates only status panel, hero, roadmap. Default = all six.
//! For hero/gallery, dark thumbnails come from `<name>_dark.png` siblings
//! produced by re-running examples with `STARSIGHT_THEME=dark`.

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

pub use palette::Theme;

#[derive(Args)]
pub struct ChromeArgs {
    /// Only regenerate live-data assets (status panel, hero, roadmap).
    #[arg(long)]
    pub live: bool,
    /// Regenerate a single named asset and exit.
    #[arg(long)]
    pub asset: Option<Asset>,
    /// Skip the dark-example pre-render step (assumes `_dark.png` siblings exist).
    #[arg(long)]
    pub skip_examples: bool,
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
    std::fs::create_dir_all(root.join("assets/hero"))?;
    std::fs::create_dir_all(root.join("assets/status"))?;

    if !args.skip_examples {
        ensure_dark_examples(&root)?;
    }

    if let Some(asset) = args.asset {
        for theme in Theme::ALL {
            regen_one(asset, &root, theme)?;
        }
        return Ok(());
    }

    // Live-data assets always run.
    let stats = crates_io::fetch().ok(); // single fetch, shared across themes
    for theme in Theme::ALL {
        if let Some(s) = stats.as_ref() {
            status_panel::regen(&root, theme, s)?;
        } else {
            eprintln!("status_panel ({}): skipping — crates.io fetch failed", theme.suffix());
        }
        hero::regen(&root, theme)?;
        roadmap::regen(&root, theme)?;
    }

    if !args.live {
        for theme in Theme::ALL {
            architecture::regen(&root, theme)?;
            gallery::regen(&root, theme)?;
            wordmark::regen(&root, theme)?;
        }
    }
    Ok(())
}

fn regen_one(asset: Asset, root: &Path, theme: Theme) -> Result<()> {
    match asset {
        Asset::StatusPanel => {
            let stats = crates_io::fetch()?;
            status_panel::regen(root, theme, &stats)
        }
        Asset::Hero => hero::regen(root, theme),
        Asset::Roadmap => roadmap::regen(root, theme),
        Asset::Architecture => architecture::regen(root, theme),
        Asset::Gallery => gallery::regen(root, theme),
        Asset::Wordmark => wordmark::regen(root, theme),
    }
}

/// Ensure `<name>_dark.png` siblings exist for every example used in the
/// hero / gallery composites. Re-runs each example binary once with
/// `STARSIGHT_THEME=dark` if its dark sibling is missing or older than the
/// `.rs` source. Skips the build when binaries are already up to date.
fn ensure_dark_examples(root: &Path) -> Result<()> {
    use std::process::Command;

    let needed: &[(&str, &str)] = &[
        // (group/name, full path stem under examples/)
        ("basics", "line_chart"),
        ("basics", "scatter"),
        ("basics", "bar_chart"),
        ("basics", "histogram"),
        ("basics", "heatmap"),
        ("basics", "bubble_scatter"),
        ("basics", "movie_heatmap"),
        ("scientific", "contour_fields"),
        ("scientific", "nightingale"),
        ("scientific", "candlestick"),
        ("scientific", "radar_spider"),
        ("scientific", "lorenz_line"),
        ("scientific", "gauge"),
        ("scientific", "wind_rose"),
        ("scientific", "polar_calendar"),
        ("scientific", "kruskal_szekeres_line"),
        ("scientific", "laser_plasma"),
        ("scientific", "error_bars"),
    ];

    let mut todo: Vec<&str> = Vec::new();
    for (group, name) in needed {
        let dark = root.join(format!("examples/{group}/{name}_dark.png"));
        let rs = root.join(format!("examples/{group}/{name}.rs"));
        if !dark.exists() {
            todo.push(*name);
            continue;
        }
        if let (Ok(d), Ok(s)) = (std::fs::metadata(&dark), std::fs::metadata(&rs)) {
            if let (Ok(dt), Ok(st)) = (d.modified(), s.modified()) {
                if st > dt {
                    todo.push(*name);
                }
            }
        }
    }

    if todo.is_empty() {
        return Ok(());
    }
    println!("regenerating {} dark example PNG(s)", todo.len());

    // Build the chrome subset once.
    let manifest = root.join("examples/Cargo.toml");
    let mut build = Command::new("cargo");
    build
        .args([
            "build",
            "--release",
            "--examples",
            "--all-features",
            "--manifest-path",
        ])
        .arg(&manifest)
        .current_dir(root);
    let status = build.status()?;
    if !status.success() {
        anyhow::bail!("cargo build --examples failed for dark regen");
    }

    let bin_dir = root.join("target/release/examples");
    for name in &todo {
        let bin = bin_dir.join(name);
        if !bin.exists() {
            eprintln!("  skip {name} (binary missing at {})", bin.display());
            continue;
        }
        let out = Command::new(&bin)
            .env("STARSIGHT_THEME", "dark")
            .current_dir(root)
            .output()?;
        if !out.status.success() {
            anyhow::bail!(
                "example {name} (dark) failed:\n{}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
        println!("  rendered {name}_dark.png");
    }
    Ok(())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is a member of the workspace")
        .to_path_buf()
}
