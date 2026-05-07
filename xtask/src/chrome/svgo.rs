//! SVG optimization via [`svgo`](https://github.com/svg/svgo).
//!
//! Runs `npx --yes svgo --multipass --quiet <files…>` over the chrome example
//! SVGs and the composite chrome assets. svgo collapses redundant whitespace,
//! rounds coordinate precision, drops default attribute values, and merges
//! identical paths — typically halving the heaviest example outputs.
//!
//! If `npx` (or `node`) is not on `PATH`, this becomes a no-op with a single
//! warning so the chrome regen still completes locally without npm. CI runs
//! the full pipeline because the GitHub-hosted runners ship Node by default.

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::CHROME_EXAMPLES;

/// In-place optimize every `examples/<group>/<name>{,_dark}.svg` referenced by
/// the chrome composites. Examples not in `CHROME_EXAMPLES` are left untouched.
pub fn optimize_chrome_examples(root: &Path) -> Result<()> {
    let mut paths: Vec<PathBuf> = Vec::with_capacity(CHROME_EXAMPLES.len() * 2);
    for (group, name) in CHROME_EXAMPLES {
        for suffix in ["", "_dark"] {
            let p = root.join(format!("examples/{group}/{name}{suffix}.svg"));
            if p.exists() {
                paths.push(p);
            }
        }
    }
    optimize(&paths, "chrome examples")
}

/// In-place optimize every chrome composite SVG written under `assets/`.
pub fn optimize_chrome_assets(root: &Path) -> Result<()> {
    let mut paths: Vec<PathBuf> = [
        "assets/architecture-light.svg",
        "assets/architecture-dark.svg",
        "assets/gallery-light.svg",
        "assets/gallery-dark.svg",
        "assets/lorenz-light.svg",
        "assets/lorenz-dark.svg",
        "assets/pipeline-light.svg",
        "assets/pipeline-dark.svg",
        "assets/roadmap-light.svg",
        "assets/roadmap-dark.svg",
        "assets/wordmark-light.svg",
        "assets/wordmark-dark.svg",
        "assets/hero/starsight-hero-light.svg",
        "assets/hero/starsight-hero-dark.svg",
        "assets/social/card-light.svg",
        "assets/social/card-dark.svg",
        "assets/status/panel-light.svg",
        "assets/status/panel-dark.svg",
    ]
    .iter()
    .map(|c| root.join(c))
    .filter(|p| p.exists())
    .collect();

    for stem in [
        "install",
        "capabilities",
        "backends",
        "translation",
        "comparison",
    ] {
        for theme in ["light", "dark"] {
            let p = root.join(format!("assets/tables/{stem}-{theme}.svg"));
            if p.exists() {
                paths.push(p);
            }
        }
    }
    optimize(&paths, "chrome assets")
}

fn optimize(paths: &[PathBuf], label: &str) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let Some(npx) = which_npx() else {
        eprintln!("svgo: skipping {label} — npx not found on PATH");
        return Ok(());
    };

    let mut total_before: u64 = 0;
    let mut total_after: u64 = 0;
    let mut ok_count = 0_usize;
    let mut skip_count = 0_usize;

    // Process files one-at-a-time. SVGO has a hard cap on attribute-value length
    // that some heavy chart paths blow through (Lorenz has tens of thousands of
    // points concatenated into one `d` attribute); a batch error would abort the
    // whole run, so we tolerate per-file failures and leave the offending source
    // unchanged.
    for p in paths {
        let before = std::fs::metadata(p).ok().map_or(0, |m| m.len());
        let mut cmd = Command::new(&npx);
        cmd.arg("--yes")
            .arg("svgo")
            .arg("--multipass")
            .arg("--quiet")
            .arg(p);
        let out = cmd.output()?;
        if !out.status.success() {
            skip_count += 1;
            let stderr = String::from_utf8_lossy(&out.stderr);
            let first = stderr.lines().next().unwrap_or("").trim();
            eprintln!(
                "svgo: skip {} ({first})",
                p.file_name().and_then(|s| s.to_str()).unwrap_or("?")
            );
            continue;
        }
        let after = std::fs::metadata(p).ok().map_or(0, |m| m.len());
        total_before += before;
        total_after += after;
        ok_count += 1;
    }

    if total_before > 0 {
        let saved = total_before.saturating_sub(total_after);
        let pct = (saved as f64 / total_before as f64) * 100.0;
        println!(
            "svgo {label}: {ok_count} ok, {skip_count} skipped, {total_before} → {total_after} bytes (-{pct:.1}%)",
        );
    }
    Ok(())
}

fn which_npx() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        for name in ["npx", "npx.cmd"] {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}
