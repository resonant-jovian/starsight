//! `assets/hero/starsight-hero-{light,dark}.{svg,png}` — composite hero, paired variants.
//!
//! Layout (880 × ~870):
//! - rounded outer card (rx=12, 1px border)
//! - top strip (156 px): eclipse + wordmark + tagline + meta
//! - 3×3 grid of inlined theme-matched example SVGs
//!
//! Dual format: SVG is the canonical authored output (vector, scales freely);
//! a 2× retina PNG is rasterized alongside and is what the README references —
//! the PNG saves the README from inlining 5 MB of example SVG markup.
//!
//! Light variant uses `<name>.svg`; dark variant uses `<name>_dark.svg` from
//! the example output directories. Dark siblings come from re-running the
//! examples with `STARSIGHT_THEME=dark STARSIGHT_FORMAT=svg`.

use anyhow::{Result, anyhow};
use std::path::Path;

use super::eclipse;
use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::png;
use super::svg::{header, inline, write_atomic};

const W: u32 = 880;
const PAD: u32 = 24;
const TOP_H: u32 = 156;
const GUTTER: u32 = 10;
const COLS: u32 = 3;
const ROWS: u32 = 3;
const RADIUS: f32 = 12.0;
const PNG_SCALE: f32 = 2.0;

const HERO_BASES: &[&str] = &[
    "examples/basics/line_chart",
    "examples/basics/scatter",
    "examples/basics/bar_chart",
    "examples/basics/histogram",
    "examples/scientific/contour_fields",
    "examples/scientific/nightingale",
    "examples/scientific/candlestick",
    "examples/scientific/radar_spider",
    "examples/scientific/lorenz_line",
];

/// How example thumbs are embedded in a cell. `InlineSvg` keeps the SVG
/// composite vector (browsers handle the subpixel scaling). `EmbedExamplePng`
/// references the example's pre-rendered PNG sibling — at native resolution
/// strokes survive the down-sample to cell size, which 2× rasterization of an
/// inlined SVG cannot guarantee for thin lines.
#[derive(Copy, Clone)]
enum CellMode {
    InlineSvg,
    EmbedExamplePng,
}

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let meta = read_meta(root)?;
    let svg = compose(root, &meta, theme, CellMode::InlineSvg)?;
    let out = root.join(format!("assets/hero/starsight-hero-{}.svg", theme.suffix()));
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());

    let svg_for_png = compose(root, &meta, theme, CellMode::EmbedExamplePng)?;
    let pix = png::rasterize_at_scale(&svg_for_png, PNG_SCALE, root)?;
    let png_out = root.join(format!("assets/hero/starsight-hero-{}.png", theme.suffix()));
    png::write_png_atomic(&pix, &png_out)?;
    println!(
        "wrote {} ({} bytes)",
        png_out.display(),
        std::fs::metadata(&png_out)?.len()
    );
    Ok(())
}

struct Meta {
    version: String,
    edition: String,
    msrv: String,
    license: String,
}

fn read_meta(root: &Path) -> Result<Meta> {
    let cargo = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let toml: toml::Value = toml::from_str(&cargo)?;
    let pkg = toml
        .get("workspace")
        .and_then(|w| w.get("package"))
        .ok_or_else(|| anyhow!("no [workspace.package]"))?;
    let s = |k: &str, d: &str| pkg.get(k).and_then(|v| v.as_str()).unwrap_or(d).to_string();
    Ok(Meta {
        version: s("version", "0.0.0"),
        edition: s("edition", "2024"),
        msrv: s("rust-version", "1.89"),
        license: s("license", "GPL-3.0-only"),
    })
}

fn compose(root: &Path, meta: &Meta, theme: Theme, cell_mode: CellMode) -> Result<String> {
    let p = palette(theme);
    let cell_w = (W - 2 * PAD - GUTTER * (COLS - 1)) / COLS;
    let cell_h = ((cell_w as f32) * 0.62) as u32;
    let grid_h = cell_h * ROWS + GUTTER * (ROWS - 1);
    let h = TOP_H + grid_h + 2 * PAD;

    let mut out = header(
        W,
        h,
        "starsight hero — eclipse mark, wordmark, tagline, and 9 example renders",
        "starsight hero",
    );

    // Outer rounded card with bg + 1px border. Vector, scales cleanly.
    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{w}" height="{hh}" rx="{r}" fill="{bg}" stroke="{s}" stroke-width="1"/>
"#,
        w = W - 1,
        hh = h - 1,
        r = RADIUS,
        bg = p.bg,
        s = p.border,
    ));

    // Top strip: eclipse + wordmark + tagline + meta.
    let ey = PAD + (TOP_H - 96) / 2;
    out.push_str(&format!(
        r#"  <g transform="translate({PAD},{ey}) scale(0.92)">
"#,
    ));
    out.push_str(&eclipse::svg_inner(p));
    out.push_str("  </g>\n");
    out.push_str(&format!(
        r#"  <text x="{wm_x}" y="{wm_y}" font-family="{sans}" font-weight="700" font-size="56" fill="{text}" letter-spacing="-1.5">starsight</text>
  <text x="{tag_x}" y="{tag_y}" font-family="{sans}" font-size="16" fill="{sub}">scientific visualization for Rust — typed, layered, eight backends</text>
  <text x="{meta_x}" y="{meta_y}" font-family="{mono}" font-size="11" fill="{muted}" text-anchor="end">v{ver}  ·  rust {msrv}  ·  edition {ed}  ·  {lic}</text>
"#,
        wm_x = PAD + 110,
        wm_y = PAD + 70,
        tag_x = PAD + 114,
        tag_y = PAD + 110,
        meta_x = W - PAD,
        meta_y = PAD + TOP_H - 14,
        sans = SANS,
        mono = MONO_FAMILY,
        text = p.text,
        sub = p.subtext,
        muted = p.muted,
        ver = meta.version,
        msrv = meta.msrv,
        ed = meta.edition,
        lic = meta.license,
    ));

    // Hairline rule under the top strip.
    out.push_str(&format!(
        r#"  <line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}" stroke="{c}" stroke-width="1"/>
"#,
        x1 = PAD,
        x2 = W - PAD,
        y = PAD + TOP_H - 1,
        c = p.border,
    ));

    // 3×3 grid of theme-matched example thumbs — vector inline for SVG output,
    // PNG `<image href>` for PNG output (preserves stroke widths through the
    // 2× rasterization that subpixel SVG strokes lose).
    let suffix = theme.example_suffix();
    for (i, base) in HERO_BASES.iter().enumerate() {
        let col = (i as u32) % COLS;
        let row = (i as u32) / COLS;
        let x0 = PAD + col * (cell_w + GUTTER);
        let y0 = PAD + TOP_H + row * (cell_h + GUTTER);
        let rel = format!("{base}{suffix}");
        out.push_str(&render_cell(
            root, &rel, x0, y0, cell_w, cell_h, theme, cell_mode,
        )?);
    }

    out.push_str("</svg>\n");
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn render_cell(
    root: &Path,
    rel: &str,
    x: u32,
    y: u32,
    cell_w: u32,
    cell_h: u32,
    theme: Theme,
    cell_mode: CellMode,
) -> Result<String> {
    let p = palette(theme);
    let mut s = String::new();
    // Cell background + 1px border.
    s.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{bg}" stroke="{c}" stroke-width="1"/>
"#,
        w = cell_w,
        h = cell_h,
        bg = p.card,
        c = p.border,
    ));

    match cell_mode {
        CellMode::InlineSvg => {
            let svg_path = root.join(format!("{rel}.svg"));
            if svg_path.exists() {
                let (inner, vb) = inline(&svg_path)?;
                // Nested <svg> with preserveAspectRatio letterboxes the chart into the cell.
                s.push_str(&format!(
                    r#"  <svg x="{x}" y="{y}" width="{cell_w}" height="{cell_h}" viewBox="{vb}" preserveAspectRatio="xMidYMid meet">{inner}</svg>
"#,
                ));
            } else {
                s.push_str(&missing(&svg_path, x, y, cell_w, cell_h, p.muted));
            }
        }
        CellMode::EmbedExamplePng => {
            let png_rel = format!("{rel}.png");
            let png_path = root.join(&png_rel);
            if png_path.exists() {
                // usvg resolves relative href against Options::resources_dir (= root).
                s.push_str(&format!(
                    r#"  <image x="{x}" y="{y}" width="{cell_w}" height="{cell_h}" href="{png_rel}" preserveAspectRatio="xMidYMid meet"/>
"#,
                ));
            } else {
                s.push_str(&missing(&png_path, x, y, cell_w, cell_h, p.muted));
            }
        }
    }
    Ok(s)
}

fn missing(path: &Path, x: u32, y: u32, cell_w: u32, cell_h: u32, muted: &str) -> String {
    format!(
        r#"  <text x="{tx}" y="{ty}" font-size="11" fill="{muted}" text-anchor="middle">{name} missing</text>
"#,
        tx = x + cell_w / 2,
        ty = y + cell_h / 2,
        name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
    )
}
