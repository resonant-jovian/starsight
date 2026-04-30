//! `cargo xtask svgs` — export every SVG snapshot to a viewable directory.
//!
//! Walks every `starsight-layer-N/tests/snapshots/` (N = 1..=7), strips the
//! YAML frontmatter that insta writes at the top of each `.snap` file, and
//! writes the SVG body to `.svg-preview/layer-N/<test>.svg`. Pending
//! snapshots (`.snap.new`) land alongside as `<test>.pending.svg`. Binary
//! metadata snapshots (whose body isn't SVG) are skipped.
//!
//! The output directory is rebuilt from scratch on every run and is added to
//! `.gitignore` once on first use.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

const PREVIEW_DIR: &str = ".svg-preview";
const GITIGNORE_LINE: &str = "/.svg-preview/";

pub fn run() -> Result<()> {
    let preview = Path::new(PREVIEW_DIR);
    if preview.exists() {
        fs::remove_dir_all(preview)
            .with_context(|| format!("clearing {}", preview.display()))?;
    }

    let mut accepted = 0_usize;
    let mut pending = 0_usize;
    let mut malformed: Vec<(PathBuf, &'static str)> = Vec::new();

    for n in 1..=7 {
        let layer_name = format!("layer-{n}");
        let snap_dir: PathBuf = format!("starsight-layer-{n}/tests/snapshots").into();
        if !snap_dir.is_dir() {
            continue;
        }

        for entry in WalkDir::new(&snap_dir).max_depth(1).into_iter().flatten() {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
                continue;
            };

            let (stem, suffix_label, is_pending) =
                if let Some(s) = file_name.strip_suffix(".snap.new") {
                    (s, ".pending.svg", true)
                } else if let Some(s) = file_name.strip_suffix(".snap") {
                    (s, ".svg", false)
                } else {
                    continue;
                };

            let Ok(raw) = fs::read_to_string(path) else {
                continue; // binary metadata files (paired with .snap.png) etc.
            };
            let body = match strip_frontmatter(&raw) {
                Ok(b) => b,
                Err(reason) => {
                    malformed.push((path.to_path_buf(), reason));
                    continue;
                }
            };
            if !body.trim_start().to_ascii_lowercase().starts_with("<svg") {
                continue;
            }

            let test_name = stem.strip_prefix("snapshot__").unwrap_or(stem);
            let out_dir = preview.join(&layer_name);
            fs::create_dir_all(&out_dir).with_context(|| format!("mkdir {}", out_dir.display()))?;
            let out = out_dir.join(format!("{test_name}{suffix_label}"));
            fs::write(&out, body).with_context(|| format!("writing {}", out.display()))?;

            if is_pending {
                pending += 1;
            } else {
                accepted += 1;
            }
        }
    }

    ensure_gitignore_entry().context("updating .gitignore")?;

    for (path, reason) in &malformed {
        eprintln!("skipping malformed: {}: {reason}", path.display());
    }
    println!(
        "Exported {accepted} SVG snapshots ({pending} pending) → {PREVIEW_DIR}/"
    );

    Ok(())
}

fn strip_frontmatter(s: &str) -> Result<&str, &'static str> {
    let rest = s.strip_prefix("---\n").ok_or("missing leading ---")?;
    let end = rest.find("\n---\n").ok_or("missing closing ---")?;
    Ok(&rest[end + 5..])
}

fn ensure_gitignore_entry() -> Result<()> {
    let path = Path::new(".gitignore");
    let existing = if path.exists() {
        fs::read_to_string(path)?
    } else {
        String::new()
    };
    if existing.lines().any(|l| l == GITIGNORE_LINE) {
        return Ok(());
    }
    let mut next = existing;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(GITIGNORE_LINE);
    next.push('\n');
    fs::write(path, next)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::strip_frontmatter;

    #[test]
    fn strips_standard_insta_header() {
        let input =
            "---\nsource: x.rs\nexpression: svg\n---\n<svg height=\"10\"/>\n";
        let body = strip_frontmatter(input).expect("valid header");
        assert!(body.starts_with("<svg"));
    }

    #[test]
    fn rejects_missing_leading_marker() {
        let input = "source: x.rs\n---\n<svg/>";
        let err = strip_frontmatter(input).expect_err("no leading ---");
        assert_eq!(err, "missing leading ---");
    }

    #[test]
    fn rejects_missing_closing_marker() {
        let input = "---\nsource: x.rs\nexpression: svg\n<svg/>";
        let err = strip_frontmatter(input).expect_err("no closing ---");
        assert_eq!(err, "missing closing ---");
    }
}
