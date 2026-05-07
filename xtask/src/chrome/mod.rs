//! `cargo xtask chrome` — regenerate documentation chrome assets.
//!
//! Live-data assets (rebuilt by `--live` and the daily chrome cron):
//!
//! | asset | live input |
//! |-------|------------|
//! | `assets/status/panel-{light,dark}.svg`        | crates.io API (downloads, dependents, updated_at) |
//! | `assets/hero/starsight-hero-{light,dark}.png` | meta strip from `Cargo.toml` + theme-matched example thumbs |
//! | `assets/roadmap-{light,dark}.svg`             | "current" milestone derived from version |
//! | `assets/buttons/<name>-{light,dark}.svg`      | version + license from `Cargo.toml`, coverage % from `assets/status/coverage.json` |
//!
//! Static-ish assets (rebuilt only on default runs): architecture, gallery,
//! wordmark, lorenz_card, social_card, tables, pipeline, matrices,
//! coming_from, comparison_matrix.
//!
//! For hero/gallery, dark thumbnails come from `<name>_dark.png` siblings
//! produced by re-running examples with `STARSIGHT_THEME=dark`.

mod architecture;
mod buttons;
mod coming_from;
mod comparison_matrix;
mod crates_io;
mod eclipse;
mod gallery;
mod hero;
mod lorenz_card;
mod matrices;
mod palette;
mod pipeline;
mod roadmap;
mod social_card;
mod status_panel;
mod svg;
mod svgo;
mod tables;
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
    Tables,
    Pipeline,
    Matrices,
    ComingFrom,
    Comparison,
    Buttons,
}

pub fn run(args: ChromeArgs) -> Result<()> {
    let root = repo_root();
    std::fs::create_dir_all(root.join("assets/hero"))?;
    std::fs::create_dir_all(root.join("assets/status"))?;

    if !args.skip_examples {
        ensure_example_outputs(&root)?;
        // Optimize examples first so the composites inline already-minified
        // copies. Skipping if svgo is unavailable (warns once).
        svgo::optimize_chrome_examples(&root)?;
    }

    if let Some(asset) = args.asset {
        for theme in Theme::ALL {
            regen_one(asset, &root, theme)?;
        }
        svgo::optimize_chrome_assets(&root)?;
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
        // Buttons read live data (coverage.json + Cargo.toml) so they belong
        // in the daily live cron — otherwise the codecov button drifts away
        // from the latest llvm-cov number.
        buttons::regen_all(&root, theme)?;
    }

    if !args.live {
        for theme in Theme::ALL {
            architecture::regen(&root, theme)?;
            gallery::regen(&root, theme)?;
            wordmark::regen(&root, theme)?;
            lorenz_card::regen(&root, theme)?;
            social_card::regen(&root, theme)?;
            tables::regen_all(&root, theme)?;
            pipeline::regen(&root, theme)?;
            matrices::regen_all(&root, theme)?;
            coming_from::regen(&root, theme)?;
            comparison_matrix::regen(&root, theme)?;
        }
    }

    svgo::optimize_chrome_assets(&root)?;
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
        Asset::Tables => tables::regen_all(root, theme),
        Asset::Pipeline => pipeline::regen(root, theme),
        Asset::Matrices => matrices::regen_all(root, theme),
        Asset::ComingFrom => coming_from::regen(root, theme),
        Asset::Comparison => comparison_matrix::regen(root, theme),
        Asset::Buttons => buttons::regen_all(root, theme),
    }
}

/// Examples used by hero / gallery / lorenz composites; (group, name) under `examples/`.
/// Hero set: 9 in `HERO_BASES` (basics + scientific essentials).
/// Gallery set: 9 in `gallery::GALLERY` (deeper scientific + composition).
pub(crate) const CHROME_EXAMPLES: &[(&str, &str)] = &[
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

/// Ensure all (theme × format) example outputs exist and are current for every
/// example used by the chrome composites.
///
/// Freshness check compares the output file's mtime against the example
/// binary's mtime. Cargo's incremental build re-touches the binary whenever
/// any of its dependencies change — including backend-side fixes in
/// `starsight-layer-*` that don't touch the example's own `.rs` source.
/// Comparing against the binary catches those (the original `.rs`-only check
/// missed `starsight-2ja`'s SVG opacity regression because the example
/// source hadn't moved).
fn ensure_example_outputs(root: &Path) -> Result<()> {
    use std::process::Command;

    struct Job {
        name: &'static str,
        suffix: &'static str,
        ext: &'static str,
        theme: Option<&'static str>,
        format: Option<&'static str>,
    }

    // Build the chrome example subset before computing freshness, so the
    // binary mtimes we compare against reflect the current source tree.
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

    let mut todo: Vec<Job> = Vec::new();
    for (group, name) in CHROME_EXAMPLES {
        let bin = bin_dir.join(name);
        for (theme, format, suffix, ext) in VARIANTS {
            let out = root.join(format!("examples/{group}/{name}{suffix}.{ext}"));
            let needs = if !out.exists() {
                true
            } else if let (Ok(o), Ok(b)) = (std::fs::metadata(&out), std::fs::metadata(&bin))
                && let (Ok(ot), Ok(bt)) = (o.modified(), b.modified())
            {
                bt > ot
            } else {
                // Binary missing — `cargo build` should have produced it. Force a
                // regen so the failure surfaces in the exec step below.
                true
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
