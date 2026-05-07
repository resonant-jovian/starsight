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
mod lorenz_card;
mod palette;
mod roadmap;
mod social_card;
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
    LorenzCard,
    SocialCard,
}

pub fn run(args: ChromeArgs) -> Result<()> {
    let root = repo_root();
    std::fs::create_dir_all(root.join("assets/hero"))?;
    std::fs::create_dir_all(root.join("assets/status"))?;

    if !args.skip_examples {
        ensure_example_outputs(&root)?;
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
            eprintln!(
                "status_panel ({}): skipping — crates.io fetch failed",
                theme.suffix()
            );
        }
        hero::regen(&root, theme)?;
        roadmap::regen(&root, theme)?;
    }

    if !args.live {
        for theme in Theme::ALL {
            architecture::regen(&root, theme)?;
            gallery::regen(&root, theme)?;
            wordmark::regen(&root, theme)?;
            lorenz_card::regen(&root, theme)?;
            social_card::regen(&root, theme)?;
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
        Asset::LorenzCard => lorenz_card::regen(root, theme),
        Asset::SocialCard => social_card::regen(root, theme),
    }
}

/// Examples used by hero / gallery / lorenz composites; (group, name) under `examples/`.
/// Hero set: 9 in `HERO_BASES` (basics + scientific essentials).
/// Gallery set: 9 in `gallery::GALLERY` (deeper scientific + composition).
const CHROME_EXAMPLES: &[(&str, &str)] = &[
    // Hero (basics row)
    ("basics", "line_chart"),
    ("basics", "scatter"),
    ("basics", "bar_chart"),
    ("basics", "histogram"),
    // Hero (scientific row)
    ("scientific", "nightingale"),
    ("scientific", "candlestick"),
    ("scientific", "radar_spider"),
    // Hero + gallery overlap
    ("scientific", "contour_fields"),
    ("scientific", "lorenz_line"),
    ("scientific", "kruskal_szekeres_line"),
    ("scientific", "laser_plasma"),
    ("basics", "bubble_scatter"),
    // Gallery-only
    ("scientific", "reciprocal_space"),
    ("scientific", "bollinger_candlestick"),
    ("theming", "custom_colormap"),
    ("composition", "distribution_dashboard"),
    ("composition", "statistical"),
];

/// Theme + format combinations the chrome composites consume. The light/png cell is
/// covered by `cargo xtask gallery`, but we still re-run if it's stale to keep the
/// matrix consistent.
const VARIANTS: &[(Option<&str>, Option<&str>, &str, &str)] = &[
    // (STARSIGHT_THEME, STARSIGHT_FORMAT, suffix, ext)
    (None, None, "", "png"),
    (Some("dark"), None, "_dark", "png"),
    (None, Some("svg"), "", "svg"),
    (Some("dark"), Some("svg"), "_dark", "svg"),
];

/// Ensure all (theme × format) example outputs exist for every example used by the
/// chrome composites. Re-runs each example binary as needed when an output file
/// is missing or older than its `.rs` source.
fn ensure_example_outputs(root: &Path) -> Result<()> {
    use std::process::Command;

    struct Job {
        name: &'static str,
        suffix: &'static str,
        ext: &'static str,
        theme: Option<&'static str>,
        format: Option<&'static str>,
    }

    let mut todo: Vec<Job> = Vec::new();
    for (group, name) in CHROME_EXAMPLES {
        let rs = root.join(format!("examples/{group}/{name}.rs"));
        for (theme, format, suffix, ext) in VARIANTS {
            let out = root.join(format!("examples/{group}/{name}{suffix}.{ext}"));
            let needs = if !out.exists() {
                true
            } else if let (Ok(d), Ok(s)) = (std::fs::metadata(&out), std::fs::metadata(&rs))
                && let (Ok(dt), Ok(st)) = (d.modified(), s.modified())
            {
                st > dt
            } else {
                false
            };
            if needs {
                todo.push(Job {
                    name,
                    suffix,
                    ext,
                    theme: *theme,
                    format: *format,
                });
            }
        }
    }

    if todo.is_empty() {
        return Ok(());
    }
    println!("regenerating {} example output(s)", todo.len());

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
        anyhow::bail!("cargo build --examples failed for chrome regen");
    }

    let bin_dir = root.join("target/release/examples");
    for job in &todo {
        let bin = bin_dir.join(job.name);
        if !bin.exists() {
            eprintln!("  skip {} (binary missing at {})", job.name, bin.display());
            continue;
        }
        let mut cmd = Command::new(&bin);
        cmd.current_dir(root);
        if let Some(t) = job.theme {
            cmd.env("STARSIGHT_THEME", t);
        } else {
            cmd.env_remove("STARSIGHT_THEME");
        }
        if let Some(f) = job.format {
            cmd.env("STARSIGHT_FORMAT", f);
        } else {
            cmd.env_remove("STARSIGHT_FORMAT");
        }
        let out = cmd.output()?;
        if !out.status.success() {
            anyhow::bail!(
                "example {} (theme={:?}, format={:?}) failed:\n{}",
                job.name,
                job.theme.unwrap_or("light"),
                job.format.unwrap_or("png"),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        println!("  rendered {}{}.{}", job.name, job.suffix, job.ext);
    }
    Ok(())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is a member of the workspace")
        .to_path_buf()
}
