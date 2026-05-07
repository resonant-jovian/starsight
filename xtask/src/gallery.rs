//! Gallery command: run every example, aggregate outputs in `target/gallery/`.
//!
//! Layout:
//! - `target/gallery/examples/<name>.png` — copies of `examples/<group>/<name>.png`,
//!   produced by running each `[[example]]` registered in `examples/Cargo.toml`.
//!   The PNG is assumed to live next to the `.rs` file; the source path comes
//!   from the manifest's `path = "..."` field, so the gallery follows whichever
//!   group sub-folder the example happens to live in.
//! - `target/gallery/snapshots/<name>.png` — mirrors of
//!   `starsight-layer-5/tests/snapshots/snapshot__snapshot_<name>-2.snap.png`,
//!   renamed to drop the insta prefix. Snapshot regen is a separate step
//!   (`INSTA_UPDATE=always cargo test -p starsight-layer-5 --test snapshot`);
//!   gallery only mirrors what's already on disk.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

pub fn run() -> Result<()> {
    let workspace_root = workspace_root()?;
    let examples = read_examples(&workspace_root)?;

    let gallery_examples = workspace_root.join("target/gallery/examples");
    let gallery_snapshots = workspace_root.join("target/gallery/snapshots");
    fs::create_dir_all(&gallery_examples)
        .with_context(|| format!("creating {}", gallery_examples.display()))?;
    fs::create_dir_all(&gallery_snapshots)
        .with_context(|| format!("creating {}", gallery_snapshots.display()))?;

    // Build all examples in a single cargo invocation so cargo's job server
    // parallelizes compilation and we pay the metadata + dep-graph check cost
    // once instead of per-example. After this returns, each example exists as a
    // standalone binary at `target/release/examples/<name>` and can be exec'd
    // directly — no cargo overhead per invocation. Fix for `starsight-qv7`.
    build_examples(&workspace_root)?;

    // Parallel execution: each example is a separate process, so run them
    // concurrently across the cores the host has. Cap at 32 to keep memory
    // bounded (each example holds ~100 MB of figure state mid-render).
    let workers = std::thread::available_parallelism()
        .map_or(2, std::num::NonZero::get)
        .min(examples.len().max(1))
        .clamp(2, 32);
    println!(
        "Running {} example(s) ({workers} workers) ...",
        examples.len()
    );

    let queue: Mutex<Vec<&ExampleEntry>> = Mutex::new(examples.iter().collect());
    let stats: Mutex<(usize, Vec<String>)> = Mutex::new((0, Vec::new()));
    let workspace_root_ref = &workspace_root;
    let gallery_examples_ref = &gallery_examples;

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
                    let Some(example) = next else { break };
                    let outcome =
                        process_example(workspace_root_ref, gallery_examples_ref, example);
                    let mut st = stats
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    match outcome {
                        Ok(()) => {
                            st.0 += 1;
                            println!("[OK]   {}", example.name);
                        }
                        Err(msg) => {
                            st.1.push(msg.clone());
                            println!("[FAIL] {}: {msg}", example.name);
                        }
                    }
                }
            });
        }
    });

    let (generated, failed) = stats
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

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

/// One `[[example]]` table from `examples/Cargo.toml`.
struct ExampleEntry {
    name: String,
    /// Workspace-relative path to the `.rs` source, e.g. `examples/basics/quickstart.rs`.
    rs_path: String,
}

impl ExampleEntry {
    /// Sibling `.png` of the source file: same directory, `.png` extension.
    fn png_path(&self) -> String {
        match self.rs_path.strip_suffix(".rs") {
            Some(stem) => format!("{stem}.png"),
            None => format!("{}.png", self.rs_path),
        }
    }
}

/// Parse `examples/Cargo.toml` for every `[[example]]` table, capturing both
/// `name` and `path` so the gallery can locate each generated PNG.
fn read_examples(workspace_root: &Path) -> Result<Vec<ExampleEntry>> {
    let manifest_path = workspace_root.join("examples/Cargo.toml");
    let text = fs::read_to_string(&manifest_path)
        .with_context(|| format!("reading {}", manifest_path.display()))?;

    let mut entries = Vec::new();
    let mut in_example = false;
    let mut current_name: Option<String> = None;
    let mut current_path: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[[example]]" {
            push_if_complete(&mut entries, &mut current_name, &mut current_path);
            in_example = true;
            continue;
        }
        if trimmed.starts_with('[') {
            push_if_complete(&mut entries, &mut current_name, &mut current_path);
            in_example = false;
            continue;
        }
        if !in_example {
            continue;
        }
        if let Some(value) = parse_kv(trimmed, "name") {
            current_name = Some(value);
        } else if let Some(value) = parse_kv(trimmed, "path") {
            current_path = Some(value);
        }
    }
    push_if_complete(&mut entries, &mut current_name, &mut current_path);
    Ok(entries)
}

fn parse_kv(line: &str, key: &str) -> Option<String> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    let value = rhs.trim().trim_matches('"');
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn push_if_complete(
    entries: &mut Vec<ExampleEntry>,
    name: &mut Option<String>,
    path: &mut Option<String>,
) {
    if let (Some(name), Some(path)) = (name.take(), path.take()) {
        entries.push(ExampleEntry {
            name,
            rs_path: format!("examples/{path}"),
        });
    }
}

/// Compile every `[[example]]` registered in `examples/Cargo.toml` once.
/// Cargo's job server parallelizes the work; the resulting binaries live at
/// `target/release/examples/<name>` and are then exec'd directly by
/// [`run_example`].
fn build_examples(workspace_root: &Path) -> Result<()> {
    let manifest = workspace_root.join("examples/Cargo.toml");
    println!("Building examples (release) ...");
    // --all-features so feature-gated examples (e.g. polars_integration with
    // required-features = ["starsight/polars"]) actually build, otherwise they
    // are silently skipped and the run loop fails on the missing binary.
    let status = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--examples",
            "--all-features",
            "--manifest-path",
        ])
        .arg(&manifest)
        .current_dir(workspace_root)
        .status()
        .context("spawning cargo build --examples")?;
    if !status.success() {
        anyhow::bail!("cargo build --examples exited with {status}");
    }
    Ok(())
}

fn run_example(workspace_root: &Path, name: &str) -> Result<()> {
    let bin = workspace_root.join("target/release/examples").join(name);
    // `output()` instead of `status()` — collecting the streams keeps parallel
    // workers from interleaving each example's stdout/stderr with the gallery
    // driver's progress lines.
    let out = Command::new(&bin)
        .current_dir(workspace_root)
        .output()
        .with_context(|| format!("spawning {}", bin.display()))?;
    if !out.status.success() {
        anyhow::bail!(
            "{} exited with {}: {}",
            bin.display(),
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    Ok(())
}

fn process_example(
    workspace_root: &Path,
    gallery_examples: &Path,
    example: &ExampleEntry,
) -> std::result::Result<(), String> {
    let name = &example.name;
    if let Err(e) = run_example(workspace_root, name) {
        return Err(format!("{e}"));
    }
    let src = workspace_root.join(example.png_path());
    let dst = gallery_examples.join(format!("{name}.png"));
    if !src.exists() {
        return Err(format!("ran but produced no PNG at {}", src.display()));
    }
    fs::copy(&src, &dst)
        .map_err(|e| format!("copying {} -> {}: {e}", src.display(), dst.display()))?;
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
    for entry in fs::read_dir(src_dir).with_context(|| format!("reading {}", src_dir.display()))? {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(s) = file_name.to_str() else {
            continue;
        };
        if !s.ends_with(".snap.png") {
            continue;
        }
        // snapshot__snapshot_<name>-2.snap.png  or  snapshot__snapshot_<name>.snap.png
        let stem = s
            .strip_suffix(".snap.png")
            .and_then(|x| x.strip_prefix("snapshot__snapshot_"));
        let Some(stem) = stem else {
            continue;
        };
        let name = stem.strip_suffix("-2").unwrap_or(stem);
        let dst = dst_dir.join(format!("{name}.png"));
        fs::copy(entry.path(), &dst)
            .with_context(|| format!("copying snapshot {s} -> {}", dst.display()))?;
        count += 1;
    }
    Ok(count)
}
