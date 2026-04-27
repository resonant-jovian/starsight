//! Gallery command: run every example, aggregate outputs in `target/gallery/`.
//!
//! Layout:
//! - `target/gallery/examples/<name>.png` — copies of `examples/showcases/<name>.png`,
//!   produced by running each `[[example]]` registered in `examples/Cargo.toml`.
//! - `target/gallery/snapshots/<name>.png` — mirrors of
//!   `starsight-layer-5/tests/snapshots/snapshot__snapshot_<name>-2.snap.png`,
//!   renamed to drop the insta prefix. Snapshot regen is a separate step
//!   (`INSTA_UPDATE=always cargo test -p starsight-layer-5 --test snapshot`);
//!   gallery only mirrors what's already on disk.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run() -> Result<()> {
    let workspace_root = workspace_root()?;
    let names = read_example_names(&workspace_root)?;

    let gallery_examples = workspace_root.join("target/gallery/examples");
    let gallery_snapshots = workspace_root.join("target/gallery/snapshots");
    fs::create_dir_all(&gallery_examples)
        .with_context(|| format!("creating {}", gallery_examples.display()))?;
    fs::create_dir_all(&gallery_snapshots)
        .with_context(|| format!("creating {}", gallery_snapshots.display()))?;

    let mut generated = 0usize;
    let mut failed: Vec<String> = Vec::new();
    for name in &names {
        match run_example(&workspace_root, name) {
            Ok(()) => {
                let src = workspace_root.join(format!("examples/showcases/{name}.png"));
                let dst = gallery_examples.join(format!("{name}.png"));
                if src.exists() {
                    fs::copy(&src, &dst).with_context(|| {
                        format!("copying {} -> {}", src.display(), dst.display())
                    })?;
                    generated += 1;
                    println!("[OK]   {name}");
                } else {
                    failed.push(format!("{name} (ran but produced no PNG at {})", src.display()));
                    println!("[WARN] {name} ran but did not write {}", src.display());
                }
            }
            Err(e) => {
                failed.push(format!("{name} ({e})"));
                println!("[FAIL] {name}: {e}");
            }
        }
    }

    let snapshots_src = workspace_root.join("starsight-layer-5/tests/snapshots");
    let snapshot_count = mirror_snapshots(&snapshots_src, &gallery_snapshots)?;

    println!();
    println!(
        "{generated} examples generated, {snapshot_count} snapshots aggregated, {} failures",
        failed.len()
    );
    if !failed.is_empty() {
        for f in &failed {
            println!("  - {f}");
        }
        anyhow::bail!("{} example(s) failed", failed.len());
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set; run via `cargo xtask`")?;
    let xtask_dir = PathBuf::from(manifest);
    xtask_dir
        .parent()
        .map(Path::to_path_buf)
        .context("xtask manifest has no parent directory")
}

/// Parse `examples/Cargo.toml` for the `name` of each `[[example]]` table.
fn read_example_names(workspace_root: &Path) -> Result<Vec<String>> {
    let path = workspace_root.join("examples/Cargo.toml");
    let text = fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;

    let mut names = Vec::new();
    let mut in_example = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[[example]]" {
            in_example = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_example = false;
            continue;
        }
        if !in_example {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name") {
            let value = rest
                .trim_start_matches([' ', '=', '\t'])
                .trim()
                .trim_matches('"');
            if !value.is_empty() {
                names.push(value.to_string());
            }
        }
    }
    Ok(names)
}

fn run_example(workspace_root: &Path, name: &str) -> Result<()> {
    let manifest = workspace_root.join("examples/Cargo.toml");
    let status = Command::new("cargo")
        .args([
            "run",
            "--release",
            "--example",
            name,
            "--manifest-path",
        ])
        .arg(&manifest)
        .current_dir(workspace_root)
        .status()
        .with_context(|| format!("spawning cargo run --example {name}"))?;
    if !status.success() {
        anyhow::bail!("cargo run --example {name} exited with {status}");
    }
    Ok(())
}

/// Mirror each `snapshot__snapshot_<name>-2.snap.png` (PNG variant) into the
/// gallery dir as `<name>.png`. Falls back to `snapshot__snapshot_<name>.snap.png`
/// for tests that only have a single PNG variant.
fn mirror_snapshots(src_dir: &Path, dst_dir: &Path) -> Result<usize> {
    if !src_dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in fs::read_dir(src_dir)
        .with_context(|| format!("reading {}", src_dir.display()))?
    {
        let entry = entry?;
        let file_name = entry.file_name();
        let s = match file_name.to_str() {
            Some(s) => s,
            None => continue,
        };
        if !s.ends_with(".snap.png") {
            continue;
        }
        // snapshot__snapshot_<name>-2.snap.png  or  snapshot__snapshot_<name>.snap.png
        let stem = s
            .strip_suffix(".snap.png")
            .and_then(|x| x.strip_prefix("snapshot__snapshot_"));
        let stem = match stem {
            Some(s) => s,
            None => continue,
        };
        let name = stem.strip_suffix("-2").unwrap_or(stem);
        let dst = dst_dir.join(format!("{name}.png"));
        fs::copy(entry.path(), &dst)
            .with_context(|| format!("copying snapshot {s} -> {}", dst.display()))?;
        count += 1;
    }
    Ok(count)
}
