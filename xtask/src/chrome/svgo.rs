//! SVG optimization via [`svgo`](https://github.com/svg/svgo).
//!
//! Runs `npx --yes svgo --multipass --quiet <file>` over the chrome example
//! SVGs and the composite chrome assets. svgo collapses redundant whitespace,
//! rounds coordinate precision, drops default attribute values, and merges
//! identical paths — typically halving the heaviest example outputs.
//!
//! Per-file invocations are parallelized with `std::thread::scope` because the
//! npx + node startup cost dominates: serialized over ~36 composite SVGs the
//! pass was the largest single contributor to `cargo xtask chrome` wall time.
//! Workers count is `available_parallelism()` clamped to `[2, 8]`. Per-file
//! resilience is preserved — svgo's hard cap on attribute-value length blows
//! up on the Lorenz path's ten-thousands-of-points `d` attribute, so a batch
//! mode would abort the whole pass; we tolerate per-file failures and leave
//! the offending source unchanged.
//!
//! If `npx` (or `node`) is not on `PATH`, this becomes a no-op with a single
//! warning so the chrome regen still completes locally without npm. CI runs
//! the full pipeline because the GitHub-hosted runners ship Node by default.
//! `cargo xtask chrome --no-svgo` skips this pass entirely for fast local
//! iteration.

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

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
        "assets/coming-from-light.svg",
        "assets/coming-from-dark.svg",
        "assets/comparison-light.svg",
        "assets/comparison-dark.svg",
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

    for stem in ["install"] {
        for theme in ["light", "dark"] {
            let p = root.join(format!("assets/tables/{stem}-{theme}.svg"));
            if p.exists() {
                paths.push(p);
            }
        }
    }
    for stem in [
        "marks", "scales", "backends", "stats", "layout", "output", "themes",
    ] {
        for theme in ["light", "dark"] {
            let p = root.join(format!("assets/matrices/{stem}-{theme}.svg"));
            if p.exists() {
                paths.push(p);
            }
        }
    }
    for stem in ["crates", "docs", "codecov", "ci", "license"] {
        for theme in ["light", "dark"] {
            let p = root.join(format!("assets/buttons/{stem}-{theme}.svg"));
            if p.exists() {
                paths.push(p);
            }
        }
    }
    optimize(&paths, "chrome assets")
}

/// Optimize a caller-supplied set of SVG paths. Used by `--asset X` runs to
/// avoid re-optimizing all 36 composites when only one (or a small subset) was
/// rewritten. Non-existent paths are silently skipped.
pub fn optimize_paths(paths: &[PathBuf], label: &str) -> Result<()> {
    let live: Vec<PathBuf> = paths.iter().filter(|p| p.exists()).cloned().collect();
    optimize(&live, label)
}

fn optimize(paths: &[PathBuf], label: &str) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let Some(npx) = which_npx() else {
        eprintln!("svgo: skipping {label} — npx not found on PATH");
        return Ok(());
    };

    let started = std::time::Instant::now();
    let total_before = AtomicU64::new(0);
    let total_after = AtomicU64::new(0);
    let ok_count = AtomicUsize::new(0);
    let skip_count = AtomicUsize::new(0);
    let stderr_lock = Mutex::new(());

    // Workers pop the next path off a shared queue. Parallelism is bounded by
    // available cores (worker pool), by the queue length (early workers exit
    // when there's no more work), and by an absolute cap of 32 — beyond that,
    // disk + npx/node spawn contention erodes returns even on big-core boxes.
    let workers = std::thread::available_parallelism()
        .map_or(2, std::num::NonZero::get)
        .min(paths.len().max(1))
        .clamp(2, 32);
    let queue: Mutex<Vec<&PathBuf>> = Mutex::new(paths.iter().collect());

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
                    let Some(p) = next else { break };
                    process_one(
                        &npx,
                        p,
                        &total_before,
                        &total_after,
                        &ok_count,
                        &skip_count,
                        &stderr_lock,
                    );
                }
            });
        }
    });

    let total_before = total_before.load(Ordering::Relaxed);
    let total_after = total_after.load(Ordering::Relaxed);
    if total_before > 0 {
        let saved = total_before.saturating_sub(total_after);
        let pct = (saved as f64 / total_before as f64) * 100.0;
        let ms = started.elapsed().as_millis();
        println!(
            "svgo {label}: {} ok, {} skipped, {total_before} → {total_after} bytes (-{pct:.1}%) in {ms} ms ({workers} workers)",
            ok_count.load(Ordering::Relaxed),
            skip_count.load(Ordering::Relaxed),
        );
    }
    Ok(())
}

fn process_one(
    npx: &Path,
    p: &Path,
    total_before: &AtomicU64,
    total_after: &AtomicU64,
    ok: &AtomicUsize,
    skip: &AtomicUsize,
    stderr_lock: &Mutex<()>,
) {
    let before = std::fs::metadata(p).ok().map_or(0, |m| m.len());
    let mut cmd = Command::new(npx);
    cmd.arg("--yes")
        .arg("svgo")
        .arg("--multipass")
        .arg("--quiet")
        .arg(p);
    let out = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            let _g = stderr_lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            eprintln!("svgo: spawn failed for {}: {e}", p.display());
            skip.fetch_add(1, Ordering::Relaxed);
            return;
        }
    };
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let first = stderr.lines().next().unwrap_or("").trim().to_string();
        let _g = stderr_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        eprintln!(
            "svgo: skip {} ({first})",
            p.file_name().and_then(|s| s.to_str()).unwrap_or("?")
        );
        skip.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let after = std::fs::metadata(p).ok().map_or(0, |m| m.len());
    total_before.fetch_add(before, Ordering::Relaxed);
    total_after.fetch_add(after, Ordering::Relaxed);
    ok.fetch_add(1, Ordering::Relaxed);
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
