//! `assets/prose/<stem>-{light,dark}.svg` — themed chrome cards that bake
//! prose paragraphs containing inline / display LaTeX math into a single
//! rounded-corner SVG per theme.
//!
//! Why a card composite: the README's worked-example prose ("the Lorenz
//! system, with σ=10, β=8/3, ρ=28 …") used to embed each math fragment as
//! an inline `<img src="assets/math/*.svg">` tag. Each `<img>` had a fixed
//! pixel `height` that interacted with the surrounding markdown font's
//! line-height differently on every viewer (crates.io vs github.com vs
//! GitHub mobile vs IDE preview), so math glyphs floated above or oversized
//! against the prose. Wrapping the whole paragraph in one fixed-pixel SVG
//! card insulates the prose+math layout from the surrounding markdown flow
//! the same way `status_panel`, `comparison_matrix`, and `architecture`
//! already do for their text-heavy chrome.
//!
//! Source format: each `assets/prose/<stem>.tex` is a complete LaTeX
//! paragraph body (prose + inline `$...$` + display `\[...\]` or
//! `\begin{equation*}...\end{equation*}`). The pipeline wraps the body in
//! a `minipage` of fixed `\textwidth`, runs `latex` to produce a DVI, then
//! `dvisvgm --no-fonts --exact-bbox` to convert the DVI into a single-file
//! SVG with glyph outlines as `<path>` elements (no font references). The
//! root `<svg>` gets `fill="currentColor"` so prose and math inherit the
//! surrounding text colour without a second render pass; `xtask` wraps the
//! inner SVG inside a palette-themed rounded-rect card frame and writes
//! per-theme outputs.
//!
//! Why glyph paths and not SVG `<text>`: the inline-`<img>` approach failed
//! because viewer fonts dictated math/text alignment. SVG `<text>` inside
//! the card would have the same problem — `<text>` is browser-rendered at
//! whatever face the user agent picks, while embedded math glyphs from
//! `dvisvgm` are fixed paths. Rendering everything (prose included) through
//! `latex` + `dvisvgm` gives one uniform glyph-path SVG with deterministic
//! kerning across every viewer.
//!
//! Live cron: prose-card rendering is **not** part of `--live` because the
//! cron Ubuntu runner doesn't carry a `TeXLive` install. Regen only fires on
//! a default `cargo xtask chrome` (or `--asset prose-card`), which is what
//! the contributor runs locally before a release. If `latex` or `dvisvgm`
//! is missing, regen warns once and skips so the rest of chrome still
//! builds.

use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use super::palette::{Theme, palette};
use super::svg::{header, write_atomic};

/// Outer card padding on all four sides, in SVG user units (= TeX points,
/// since `dvisvgm` emits the dvi at 1pt-per-unit by default). 32pt ≈ 11mm,
/// matching the gutter the other text-heavy chrome composites use around
/// their content.
const CARD_PAD: f32 = 32.0;

/// Corner radius of the card frame, matching `comparison_matrix` /
/// `lorenz_card` (`rx="12"`).
const CARD_RX: f32 = 12.0;

/// Stroke width on the card border, matching the rest of chrome.
const CARD_STROKE: f32 = 1.0;

/// Prose body width (set as `\textwidth` inside the LaTeX `minipage`). At
/// 1pt-per-unit dvisvgm scaling, 720pt of textwidth + 64pt of padding gives
/// an outer card around 784pt — sits between `status_panel` (880) and the
/// matrices/comparison cards (~1100), readable at content-width display.
/// Tuned in subtask E; bump if the Lorenz paragraph wraps awkwardly.
const TEX_TEXTWIDTH_PT: u32 = 720;

/// LaTeX preamble + `minipage` open. The body of `assets/prose/<stem>.tex`
/// is concatenated between this and `POSTAMBLE`.
///
/// Choices baked in:
/// - `article` + `preview` (not `standalone`): `standalone`'s
///   auto-detection wraps content in a single `\hbox`, which suppresses
///   display math (`\dot`, `\sum`, ...) with "allowed only in math mode"
///   errors. The `preview` package with an explicit `\PreviewEnvironment`
///   directive targets the `minipage` instead, so the entire paragraph
///   becomes one preview image with display math intact.
/// - `tgheros` + `\renewcommand{\familydefault}{\sfdefault}` so prose
///   typesets in TeX Gyre Heros (a Helvetica-equivalent sans face). This
///   keeps the card visually aligned with the rest of starsight's chrome,
///   which uses `&quot;DejaVu Sans&quot;` everywhere else. Math stays in
///   Latin Modern Math via `amsmath`.
/// - `amsmath, amssymb, amsfonts` for the operators and Greek that appear
///   in the Lorenz system (`\dot`, `\sigma`, `\rho`, `\in`).
fn build_doc(body: &str) -> String {
    let textwidth = TEX_TEXTWIDTH_PT;
    format!(
        r"\documentclass{{article}}
\usepackage[T1]{{fontenc}}
\usepackage[utf8]{{inputenc}}
\usepackage{{amsmath,amssymb,amsfonts}}
\usepackage{{tgheros}}
\renewcommand{{\familydefault}}{{\sfdefault}}
\usepackage[active,tightpage]{{preview}}
\setlength\PreviewBorder{{2pt}}
\PreviewEnvironment{{minipage}}
\begin{{document}}
\begin{{minipage}}{{{textwidth}pt}}
\noindent
{body}
\end{{minipage}}
\end{{document}}
"
    )
}

/// Regenerate every `assets/prose/<stem>.tex` source into a themed
/// `assets/prose/<stem>-{theme}.svg` card.
///
/// Skips silently if `latex` / `dvisvgm` aren't installed (warns once via
/// `tools_present()`). Outputs are SVG-only — prose cards are pure
/// typography, not plotted data, so the dual-format PNG+SVG rule from
/// `chrome::mod` doesn't apply.
pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let dir = root.join("assets/prose");
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
        render_one(tex, theme).with_context(|| format!("render prose-card {}", tex.display()))?;
    }
    Ok(())
}

/// Render a single `<name>.tex` source to `<name>-{theme}.svg`.
///
/// 1. Wrap the body in `build_doc()` (preamble + `minipage` + postamble).
/// 2. Run `latex` to produce a DVI in a per-job temp directory.
/// 3. Run `dvisvgm --no-fonts --exact-bbox` to convert DVI to a glyph-path
///    SVG (paths instead of font references — viewer-font-independent).
/// 4. Wrap the inner SVG inside a palette-themed rounded-rect card frame
///    and write atomically to `assets/prose/<name>-{theme}.svg`.
fn render_one(tex_path: &Path, theme: Theme) -> Result<()> {
    let stem = tex_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("non-utf8 stem: {}", tex_path.display()))?;
    let body = std::fs::read_to_string(tex_path)?;
    let inner_svg =
        render_inner(&body).with_context(|| format!("render_inner for {}", tex_path.display()))?;
    let card = wrap_card(&inner_svg, theme, stem)
        .with_context(|| format!("wrap_card for {}", tex_path.display()))?;

    let out_dir = tex_path
        .parent()
        .ok_or_else(|| anyhow!("no parent dir for {}", tex_path.display()))?;
    let out = out_dir.join(format!("{stem}-{}.svg", theme.suffix()));
    write_atomic(&out, &card)?;
    println!("wrote {} ({} bytes)", out.display(), card.len());
    Ok(())
}

/// Run `latex` + `dvisvgm` on the given LaTeX body and return the resulting
/// SVG content as a string. The dvisvgm root `<svg>` is stripped by
/// `parse_inner` later in `wrap_card`; the inner `<svg>` we emit there
/// carries `fill="currentColor"` directly so glyph paths cascade-inherit
/// the palette text colour.
///
/// The body is wrapped via `build_doc()`. Errors propagate from each
/// subprocess with the captured stdout/stderr included for diagnosis.
fn render_inner(body: &str) -> Result<String> {
    let workdir = tempdir("prose-card")?;
    let job = workdir.join("doc.tex");
    std::fs::write(&job, build_doc(body))?;

    // `latex` is the DVI-producing engine (vs. `pdflatex` which goes straight
    // to PDF). `dvisvgm` consumes DVI directly. `-interaction=batchmode`
    // silences the prompt-on-error behaviour that would otherwise hang
    // headless runs.
    let dvi = workdir.join("doc.dvi");
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
            "latex failed: {}\n{}",
            String::from_utf8_lossy(&latex_out.stdout),
            String::from_utf8_lossy(&latex_out.stderr)
        ));
    }

    let svg_out_path = workdir.join("doc.svg");
    // `--no-fonts` → glyph outlines as `<path>` instead of `<font>` refs, so
    // the SVG is self-contained and viewer-font-independent.
    // `--exact-bbox` → tight crop on the actual ink, not the LaTeX page.
    // `-o` (short form) takes a literal path; the long form `--output=PATTERN`
    // treats the value as a printf-style pattern that breaks on filenames
    // containing `-` followed by a digit.
    let dvisvgm_out = Command::new("dvisvgm")
        .args(["--no-fonts", "--exact-bbox", "-o"])
        .arg(&svg_out_path)
        .arg(&dvi)
        .output()?;
    if !dvisvgm_out.status.success() {
        return Err(anyhow!(
            "dvisvgm failed: {}",
            String::from_utf8_lossy(&dvisvgm_out.stderr)
        ));
    }

    Ok(std::fs::read_to_string(&svg_out_path)?)
}

/// Wrap an inner `dvisvgm` SVG in a palette-themed rounded-rect card frame.
///
/// Reads the inner SVG's `viewBox` to size the outer canvas, embeds the
/// inner SVG (sans XML declaration / DOCTYPE) inside a `<g>` translated by
/// `CARD_PAD`, and emits the standard chrome `<svg>` envelope via
/// `chrome::svg::header`. The outer `<svg>` `color` attribute is set to the
/// palette text colour so the inner `currentColor` glyphs render in the
/// right shade for `theme`.
fn wrap_card(inner_svg: &str, theme: Theme, stem: &str) -> Result<String> {
    let p = palette(theme);
    let (inner_body, view_box) = parse_inner(inner_svg)?;
    let (inner_w, inner_h) =
        parse_view_box(&view_box).ok_or_else(|| anyhow!("could not parse viewBox '{view_box}'"))?;

    let outer_w = (inner_w + 2.0 * CARD_PAD).round() as u32;
    let outer_h = (inner_h + 2.0 * CARD_PAD).round() as u32;

    // `header` gives us the `<svg>` envelope; we then push the card rect
    // (palette card-fill + border) and a `<g>` that translates the inner
    // dvisvgm body by `CARD_PAD` so it lands inside the rounded gutter.
    let aria = format!("starsight prose card · {stem} · {}", theme.suffix());
    let title = format!("starsight prose card — {stem}");
    let mut out = header(outer_w, outer_h, &aria, &title);

    // Card frame. Inset by 0.5 so the 1px stroke sits cleanly on the pixel
    // grid (same trick `status_panel.rs` uses).
    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{w}" height="{h}" rx="{rx}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}"/>
"#,
        w = outer_w - 1,
        h = outer_h - 1,
        rx = CARD_RX,
        fill = p.card,
        stroke = p.border,
        sw = CARD_STROKE,
    ));

    // Embed the inner dvisvgm body inside a translated group with the
    // theme's text colour so `currentColor` paths pick up the right shade.
    // The inner `<svg>` re-declares the `xlink` namespace because dvisvgm
    // emits `<use xlink:href="#g0-..." />` glyph references; without the
    // namespace declaration on a nested `<svg>` boundary, strict XML
    // parsers (rsvg-convert, librsvg, usvg) reject the document with
    // "Namespace prefix xlink for href on use is not defined".
    out.push_str(&format!(
        r#"  <g transform="translate({pad} {pad})" color="{color}">
    <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="{vb}" width="{iw}" height="{ih}" overflow="visible" fill="currentColor">
{body}
    </svg>
  </g>
"#,
        pad = CARD_PAD,
        color = p.text,
        vb = view_box,
        iw = inner_w,
        ih = inner_h,
        body = inner_body,
    ));

    out.push_str("</svg>\n");
    Ok(out)
}

/// Strip XML declaration / DOCTYPE / outer `<svg>` tags from a `dvisvgm`
/// SVG and return `(inner_xml, view_box)` for nested embedding.
///
/// The shared `chrome::svg::inline` reads from a path; we already have the
/// body in memory, so this is a thin parallel that operates on `&str`.
fn parse_inner(raw: &str) -> Result<(String, String)> {
    let body = strip_decl(raw);

    let open_start = body
        .find("<svg")
        .ok_or_else(|| anyhow!("no <svg root in dvisvgm output"))?;
    let open_end = body[open_start..]
        .find('>')
        .map(|i| open_start + i + 1)
        .ok_or_else(|| anyhow!("unterminated <svg tag in dvisvgm output"))?;
    let open_tag = &body[open_start..open_end];

    let view_box = extract_attr(open_tag, "viewBox").unwrap_or_else(|| {
        let w = extract_attr(open_tag, "width").unwrap_or_else(|| "1000".into());
        let h = extract_attr(open_tag, "height").unwrap_or_else(|| "600".into());
        let trim = |s: &str| {
            s.trim_end_matches("pt")
                .trim_end_matches("px")
                .trim()
                .to_string()
        };
        format!("0 0 {} {}", trim(&w), trim(&h))
    });

    let close_idx = body
        .rfind("</svg>")
        .ok_or_else(|| anyhow!("no </svg> in dvisvgm output"))?;
    let inner = body[open_end..close_idx].to_string();
    Ok((inner, view_box))
}

fn strip_decl(s: &str) -> &str {
    let mut rest = s.trim_start();
    if let Some(after) = rest.strip_prefix("<?xml")
        && let Some(end) = after.find("?>")
    {
        rest = after[end + 2..].trim_start();
    }
    if rest.starts_with("<!DOCTYPE")
        && let Some(end) = rest.find('>')
    {
        rest = rest[end + 1..].trim_start();
    }
    rest
}

/// `dvisvgm` writes attributes with single quotes (`width='716pt'`) while the
/// rest of the chrome composites use double quotes. Search for either so the
/// parser handles both styles.
fn extract_attr(tag: &str, name: &str) -> Option<String> {
    for q in ['"', '\''] {
        let needle = format!("{name}={q}");
        if let Some(start_offset) = tag.find(&needle) {
            let start = start_offset + needle.len();
            if let Some(end_offset) = tag[start..].find(q) {
                return Some(tag[start..start + end_offset].to_string());
            }
        }
    }
    None
}

/// Parse a `viewBox` attribute value `"x y w h"` into `(width, height)`.
fn parse_view_box(vb: &str) -> Option<(f32, f32)> {
    let parts: Vec<&str> = vb.split_whitespace().collect();
    if parts.len() != 4 {
        return None;
    }
    let w: f32 = parts[2].parse().ok()?;
    let h: f32 = parts[3].parse().ok()?;
    Some((w, h))
}

/// `latex` and `dvisvgm` aren't a hard dependency for chrome regen; warn once
/// and skip if either is absent so contributors without a `TeXLive` install can
/// still rebuild the rest of the chrome assets.
fn tools_present() -> bool {
    static ANNOUNCED: OnceLock<bool> = OnceLock::new();
    let ok = which("latex") && which("dvisvgm");
    if !ok {
        let _ = ANNOUNCED.get_or_init(|| {
            eprintln!(
                "prose-card: skipping (need `latex` + `dvisvgm`; see assets/prose/README \
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
/// for one short-lived directory per card.
fn tempdir(stem: &str) -> Result<PathBuf> {
    let base =
        std::env::temp_dir().join(format!("starsight-prose-{}-{}", stem, std::process::id()));
    if base.exists() {
        std::fs::remove_dir_all(&base)?;
    }
    std::fs::create_dir_all(&base)?;
    Ok(base)
}
