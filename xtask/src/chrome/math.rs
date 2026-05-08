//! `assets/math/<name>.svg` — pre-rendered LaTeX math expressions.
//!
//! Why pre-render: GitHub-Flavoured Markdown's LaTeX math extension renders
//! `$inline$` and `$$display$$` natively on github.com web, but **crates.io**
//! and the **GitHub mobile app** render those expressions as raw dollar-
//! delimited text. Pre-rendering each expression to an SVG and embedding it
//! via `<img>` makes the README readable on every surface.
//!
//! Source format: each `assets/math/<name>.tex` contains a single LaTeX math
//! expression, including the math-mode delimiters (`$...$` for inline,
//! `$$...$$` or `\[...\]` for display). The file content is wrapped in a
//! `standalone` document and rendered to a tightly-cropped DVI via `latex`,
//! then converted to a single-file SVG with embedded glyph outlines via
//! `dvisvgm --no-fonts`. Output paths get `fill="currentColor"` injected on
//! the root `<svg>` element so the math inherits the surrounding text colour
//! — light / dark theming works automatically without rendering twice.
//!
//! Live cron: math rendering is **not** part of `--live` because the cron
//! Ubuntu runner doesn't carry a TeXLive install. Math regen only fires on
//! a default `cargo xtask chrome` (or `--asset math`), which is what the
//! contributor runs locally before a release. If `latex` or `dvisvgm` is
//! missing, regen warns once and skips so the rest of chrome still builds.

use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

// `article` + `preview` instead of `standalone`: standalone's auto-detection
// gathers `\begin{document}` content into a single `\hbox`, which suppresses
// display math (`\dot`, `\sum`, ... raise "allowed only in math mode" errors
// because `\hbox` doesn't establish display math). The `preview` package with
// explicit `\PreviewEnvironment` directives handles `equation*` / `equation`
// / `align*` correctly. Inline math wrapped in `$...$` survives either path,
// so the same preamble works for inline and display sources alike.
const PREAMBLE: &str = r"\documentclass{article}
\usepackage[active,tightpage]{preview}
\usepackage{amsmath,amssymb,amsfonts}
\PreviewEnvironment{preview}
\PreviewEnvironment{equation*}
\PreviewEnvironment{equation}
\PreviewEnvironment{align*}
\PreviewEnvironment{align}
\begin{document}
\begin{preview}
";
const POSTAMBLE: &str = "\n\\end{preview}\n\\end{document}\n";

/// Render every `assets/math/*.tex` source to a sibling `.svg` via
/// `latex` + `dvisvgm`. Skips with a one-time warning if either tool is
/// missing — the rest of chrome regen continues so contributors without a
/// TeXLive install can still rebuild buttons / hero / etc.
pub fn regen_all(root: &Path) -> Result<()> {
    let dir = root.join("assets/math");
    if !dir.exists() {
        return Ok(());
    }
    if !tools_present() {
        return Ok(());
    }

    let mut sources: Vec<PathBuf> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("tex"))
        .collect();
    sources.sort();

    if sources.is_empty() {
        return Ok(());
    }

    for tex in &sources {
        render_one(tex).with_context(|| format!("render math {}", tex.display()))?;
    }
    Ok(())
}

/// Render a single `<name>.tex` source to a sibling `<name>.svg`. The TeX
/// content is wrapped in a `standalone` preamble; `latex` produces a DVI in
/// a per-job temp directory; `dvisvgm --no-fonts` converts the DVI to a
/// single-file SVG (paths instead of font references); finally the root
/// `<svg>` element gets `fill="currentColor"` so the math inherits the
/// surrounding text colour for light / dark theming.
fn render_one(tex_path: &Path) -> Result<()> {
    let stem = tex_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("non-utf8 stem: {}", tex_path.display()))?;
    let tex_body = std::fs::read_to_string(tex_path)?;

    let workdir = tempdir(stem)?;
    let job = workdir.join(format!("{stem}.tex"));
    std::fs::write(&job, format!("{PREAMBLE}{tex_body}{POSTAMBLE}"))?;

    // `latex` is the DVI-producing engine (vs. `pdflatex` which goes straight
    // to PDF). dvisvgm consumes DVI directly. -interaction=batchmode silences
    // the prompt-on-error behaviour that would otherwise hang headless runs.
    let dvi = workdir.join(format!("{stem}.dvi"));
    let latex_out = Command::new("latex")
        .args([
            "-interaction=batchmode",
            "-halt-on-error",
            "-output-directory",
        ])
        .arg(&workdir)
        .arg(&job)
        .output()?;
    if !latex_out.status.success() || !dvi.exists() {
        return Err(anyhow!(
            "latex failed for {}: {}",
            tex_path.display(),
            String::from_utf8_lossy(&latex_out.stdout)
        ));
    }

    let svg_out_path = tex_path.with_extension("svg");
    // --no-fonts → glyph outlines as <path> instead of <font> references, so
    // the SVG is self-contained and any markdown viewer renders it.
    // --exact-bbox → tight crop on the actual ink, not the LaTeX page.
    // -o (short form) takes a literal path; the long form `--output=PATTERN`
    // treats the value as a printf-style pattern that breaks on filenames
    // containing `-` followed by a digit.
    let dvisvgm_out = Command::new("dvisvgm")
        .args(["--no-fonts", "--exact-bbox", "-o"])
        .arg(&svg_out_path)
        .arg(&dvi)
        .output()?;
    if !dvisvgm_out.status.success() {
        return Err(anyhow!(
            "dvisvgm failed for {}: {}",
            tex_path.display(),
            String::from_utf8_lossy(&dvisvgm_out.stderr)
        ));
    }

    inject_current_color(&svg_out_path)?;
    println!(
        "wrote {} ({} bytes)",
        svg_out_path.display(),
        std::fs::metadata(&svg_out_path)?.len()
    );
    Ok(())
}



/// Add `fill="currentColor"` to the root `<svg>` element so the math inherits
/// the surrounding text colour — the same SVG renders dark text on light
/// backgrounds and light text on dark, without a second render pass.
///
/// dvisvgm output puts the root attributes in a single space-separated
/// `<svg ...>` tag near the top of the file; we insert the attribute right
/// after the element name. Safe even if the file already has one — we check
/// first.
fn inject_current_color(path: &Path) -> Result<()> {
    let svg = std::fs::read_to_string(path)?;
    if svg.contains("fill=\"currentColor\"") {
        return Ok(());
    }
    let patched = svg.replacen("<svg ", "<svg fill=\"currentColor\" ", 1);
    std::fs::write(path, patched)?;
    Ok(())
}

/// `latex` and `dvisvgm` aren't a hard dependency for chrome regen; warn once
/// and skip if either is absent so contributors without a TeXLive install can
/// still rebuild the rest of the chrome assets.
fn tools_present() -> bool {
    static ANNOUNCED: OnceLock<bool> = OnceLock::new();
    let ok = which("latex") && which("dvisvgm");
    if !ok {
        let _ = ANNOUNCED.get_or_init(|| {
            eprintln!(
                "math: skipping (need `latex` + `dvisvgm`; see assets/math/README \
                 for the rendering recipe)"
            );
            true
        });
    }
    ok
}

fn which(bin: &str) -> bool {
    Command::new(bin)
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Make a per-job temp dir under the system temp root. Mirrors the pattern
/// `tempfile::tempdir()` would give us, but we avoid pulling another dep in
/// for one short-lived directory per equation.
fn tempdir(stem: &str) -> Result<PathBuf> {
    let base = std::env::temp_dir().join(format!("starsight-math-{}-{}", stem, std::process::id()));
    if base.exists() {
        std::fs::remove_dir_all(&base)?;
    }
    std::fs::create_dir_all(&base)?;
    Ok(base)
}
