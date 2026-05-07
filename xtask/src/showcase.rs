//! Showcase command: build a flat `showcase/` directory of symlinks pointing
//! at every PNG under `examples/`. Names are prefixed with the source subdir
//! (e.g. `basics_histogram.png`) so the directory can be opened in an image
//! viewer as a single gallery without having to walk per-group folders.
//!
//! `showcase/` is gitignored — regenerate any time by re-running this command.
//! The destination is wiped and rebuilt on every run so removed/renamed source
//! files don't leave stale links.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn run() -> Result<()> {
    let workspace_root = workspace_root()?;
    let src_dir = workspace_root.join("examples");
    let out_dir = workspace_root.join("showcase");

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)
            .with_context(|| format!("removing existing {}", out_dir.display()))?;
    }
    fs::create_dir_all(&out_dir).with_context(|| format!("creating {}", out_dir.display()))?;

    let mut pngs = Vec::new();
    collect_pngs(&src_dir, &src_dir, &mut pngs)?;
    pngs.sort();

    for rel in &pngs {
        let flat = rel.replace('/', "_");
        let link_path = out_dir.join(&flat);
        // Relative target so showcase/ stays valid if the repo is moved.
        let target = PathBuf::from("..").join("examples").join(rel);
        symlink(&target, &link_path).with_context(|| {
            format!("symlinking {} -> {}", link_path.display(), target.display())
        })?;
    }

    println!("Linked {} PNGs into {}", pngs.len(), out_dir.display());
    Ok(())
}

/// Recursively collect `*.png` files under `dir`, returning paths relative to
/// `root` joined with forward slashes so the flatten-to-underscore step is
/// deterministic regardless of the host's path separator.
fn collect_pngs(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_pngs(root, &path, out)?;
        } else if file_type.is_file() && path.extension().and_then(|e| e.to_str()) == Some("png") {
            let rel = path
                .strip_prefix(root)
                .with_context(|| format!("{} is not under {}", path.display(), root.display()))?;
            let rel_str = rel
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect::<Vec<_>>()
                .join("/");
            out.push(rel_str);
        }
    }
    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    let manifest = std::env::var("CARGO_MANIFEST_DIR")
        .context("CARGO_MANIFEST_DIR not set; run via `cargo xtask`")?;
    PathBuf::from(manifest)
        .parent()
        .map(Path::to_path_buf)
        .context("xtask manifest has no parent directory")
}

#[cfg(unix)]
fn symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn symlink(target: &Path, link: &Path) -> std::io::Result<()> {
    // Requires Developer Mode or SeCreateSymbolicLinkPrivilege on Windows.
    std::os::windows::fs::symlink_file(target, link)
}
