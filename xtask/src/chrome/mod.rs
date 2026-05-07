//! `cargo xtask chrome` — regenerate documentation chrome assets.
//!
//! Live-data assets (rebuilt by `--live` and the daily chrome cron):
//!
//! | asset | live input |
//! |-------|------------|
//! | `assets/status/panel-{light,dark}.svg`              | crates.io API (downloads, dependents, `updated_at`) |
//! | `assets/hero/starsight-hero-{light,dark}.{svg,png}` | meta strip from `Cargo.toml` + theme-matched example thumbs |
//! | `assets/roadmap-{light,dark}.svg`                   | "current" milestone derived from version |
//! | `assets/buttons/<name>-{light,dark}.svg`            | version + license from `Cargo.toml`, coverage % from `assets/status/coverage.json` |
//!
//! Static-ish assets (rebuilt only on default runs): architecture, `gallery`
//! (dual SVG+PNG), wordmark, `lorenz_card` (dual SVG+PNG), `social_card`
//! (dual SVG+PNG), tables, pipeline, matrices, `coming_from`,
//! `comparison_matrix`.
//!
//! Format rule: composites that contain plotted data (`hero`, `gallery`,
//! `lorenz_card`) ship dual-format — PNG canonical for the README, SVG kept
//! alongside for vector consumers — at 2× retina scale. Diagrams, badges,
//! status, and matrices stay SVG-only.
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
mod png;
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
    /// Skip the trailing svgo optimization pass — useful for fast local iteration.
    /// CI continues to optimize.
    #[arg(long)]
    pub no_svgo: bool,
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
        if !args.no_svgo {
            svgo::optimize_chrome_examples(&root)?;
        }
    }

    if let Some(asset) = args.asset {
        let mut written: Vec<PathBuf> = Vec::new();
        for theme in Theme::ALL {
            regen_one(asset, &root, theme)?;
            written.extend(asset_svg_outputs(asset, &root, theme));
        }
        if !args.no_svgo {
            // Scope svgo to just the file(s) we wrote — running over all 36
            // composites here is the bulk of `--asset X` wall time.
            svgo::optimize_paths(&written, "scoped")?;
        }
        return Ok(());
    }

    // Live-data assets always run. The light + dark themes are independent —
    // run them in parallel so hero's PNG rasterization (the biggest single
    // composite cost) overlaps. Buttons read live data; the live cron uses
    // the same path so the codecov button doesn't lag llvm-cov by a day.
    let stats = crates_io::fetch().ok(); // single fetch, shared across themes
    std::thread::scope(|s| -> Result<()> {
        let stats_ref = stats.as_ref();
        let root_ref = &root;
        let mut handles = Vec::new();
        for theme in Theme::ALL {
            // Theme is Copy; rebinding to make the move into `s.spawn` explicit.
            handles.push(s.spawn(move || -> Result<()> {
                if let Some(stats) = stats_ref {
                    status_panel::regen(root_ref, theme, stats)?;
                } else {
                    eprintln!(
                        "status_panel ({}): skipping — crates.io fetch failed",
                        theme.suffix()
                    );
                }
                hero::regen(root_ref, theme)?;
                roadmap::regen(root_ref, theme)?;
                buttons::regen_all(root_ref, theme)?;
                Ok(())
            }));
        }
        join_handles(handles, "live regen")
    })?;

    if !args.live {
        std::thread::scope(|s| -> Result<()> {
            let root_ref = &root;
            let mut handles = Vec::new();
            for theme in Theme::ALL {
                // Theme is Copy; rebinding to make the move into `s.spawn` explicit.
                handles.push(s.spawn(move || -> Result<()> {
                    architecture::regen(root_ref, theme)?;
                    gallery::regen(root_ref, theme)?;
                    wordmark::regen(root_ref, theme)?;
                    lorenz_card::regen(root_ref, theme)?;
                    social_card::regen(root_ref, theme)?;
                    tables::regen_all(root_ref, theme)?;
                    pipeline::regen(root_ref, theme)?;
                    matrices::regen_all(root_ref, theme)?;
                    coming_from::regen(root_ref, theme)?;
                    comparison_matrix::regen(root_ref, theme)?;
                    Ok(())
                }));
            }
            join_handles(handles, "static regen")
        })?;
    }

    if !args.no_svgo {
        svgo::optimize_chrome_assets(&root)?;
    }
    Ok(())
}

/// Join a list of scoped thread handles, surfacing the first error and reporting
/// any panic with the section label.
fn join_handles<'scope>(
    handles: Vec<std::thread::ScopedJoinHandle<'scope, Result<()>>>,
    label: &str,
) -> Result<()> {
    let mut first_err: Option<anyhow::Error> = None;
    for h in handles {
        match h.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                if first_err.is_none() {
                    first_err = Some(e);
                }
            }
            Err(_) => {
                if first_err.is_none() {
                    first_err = Some(anyhow::anyhow!("{label}: worker panicked"));
                }
            }
        }
    }
    if let Some(e) = first_err {
        return Err(e);
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
        Asset::Tables => tables::regen_all(root, theme),
        Asset::Pipeline => pipeline::regen(root, theme),
        Asset::Matrices => matrices::regen_all(root, theme),
        Asset::ComingFrom => coming_from::regen(root, theme),
        Asset::Comparison => comparison_matrix::regen(root, theme),
        Asset::Buttons => buttons::regen_all(root, theme),
    }
}

/// SVG outputs an `--asset X` run produces (per theme) — used to scope the
/// trailing svgo pass. PNG siblings (`hero` / `gallery` / `lorenz_card` /
/// `social_card`) are excluded since svgo only operates on SVG.
fn asset_svg_outputs(asset: Asset, root: &Path, theme: Theme) -> Vec<PathBuf> {
    let s = theme.suffix();
    match asset {
        Asset::StatusPanel => vec![root.join(format!("assets/status/panel-{s}.svg"))],
        Asset::Hero => vec![root.join(format!("assets/hero/starsight-hero-{s}.svg"))],
        Asset::Roadmap => vec![root.join(format!("assets/roadmap-{s}.svg"))],
        Asset::Architecture => vec![root.join(format!("assets/architecture-{s}.svg"))],
        Asset::Gallery => vec![root.join(format!("assets/gallery-{s}.svg"))],
        Asset::Wordmark => vec![root.join(format!("assets/wordmark-{s}.svg"))],
        Asset::LorenzCard => vec![root.join(format!("assets/lorenz-{s}.svg"))],
        Asset::SocialCard => vec![root.join(format!("assets/social/card-{s}.svg"))],
        Asset::Tables => vec![root.join(format!("assets/tables/install-{s}.svg"))],
        Asset::Pipeline => vec![root.join(format!("assets/pipeline-{s}.svg"))],
        Asset::Matrices => [
            "marks", "scales", "backends", "stats", "layout", "output", "themes",
        ]
        .iter()
        .map(|stem| root.join(format!("assets/matrices/{stem}-{s}.svg")))
        .collect(),
        Asset::ComingFrom => vec![root.join(format!("assets/coming-from-{s}.svg"))],
        Asset::Comparison => vec![root.join(format!("assets/comparison-{s}.svg"))],
        Asset::Buttons => ["crates", "docs", "codecov", "ci", "license"]
            .iter()
            .map(|stem| root.join(format!("assets/buttons/{stem}-{s}.svg")))
            .collect(),
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
struct ExampleJob {
    name: &'static str,
    suffix: &'static str,
    ext: &'static str,
    theme: Option<&'static str>,
    format: Option<&'static str>,
}

fn ensure_example_outputs(root: &Path) -> Result<()> {
    use std::process::Command;
    use std::sync::Mutex;

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

    let mut todo: Vec<ExampleJob> = Vec::new();
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
                todo.push(ExampleJob {
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

    // Run example binaries in parallel — each is a separate process, so this
    // saturates whatever cores are available. On a 24-thread box the 68-job
    // sweep that used to take ~17 min serially drops to roughly job-count
    // divided by worker-count × per-job cost. Cap at 32 to keep memory under
    // control (each example holds ~100 MB of figure state mid-render).
    let workers = std::thread::available_parallelism()
        .map_or(2, std::num::NonZero::get)
        .min(todo.len())
        .clamp(2, 32);
    println!(
        "regenerating {} example output(s) ({workers} workers)",
        todo.len()
    );

    let queue: Mutex<Vec<&ExampleJob>> = Mutex::new(todo.iter().collect());
    let errors: Mutex<Vec<String>> = Mutex::new(Vec::new());
    let bin_dir_ref = &bin_dir;

    std::thread::scope(|s| {
        for _ in 0..workers {
            s.spawn(|| {
                loop {
                    let next = {
                        let mut q = queue
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        q.pop()
                    };
                    let Some(job) = next else { break };
                    if let Err(msg) = run_example_job(root, bin_dir_ref, job) {
                        let mut errs = errors
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        errs.push(msg);
                    }
                }
            });
        }
    });

    let errs = errors
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(first) = errs.into_iter().next() {
        anyhow::bail!("{first}");
    }
    Ok(())
}

fn run_example_job(
    root: &Path,
    bin_dir: &Path,
    job: &ExampleJob,
) -> std::result::Result<(), String> {
    use std::process::Command;
    let bin = bin_dir.join(job.name);
    if !bin.exists() {
        eprintln!("  skip {} (binary missing at {})", job.name, bin.display());
        return Ok(());
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
    let out = cmd
        .output()
        .map_err(|e| format!("spawn {}: {e}", job.name))?;
    if !out.status.success() {
        return Err(format!(
            "example {} (theme={:?}, format={:?}) failed:\n{}",
            job.name,
            job.theme.unwrap_or("light"),
            job.format.unwrap_or("png"),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    println!("  rendered {}{}.{}", job.name, job.suffix, job.ext);
    Ok(())
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask is a member of the workspace")
        .to_path_buf()
}
