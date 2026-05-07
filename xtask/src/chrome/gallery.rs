//! `assets/gallery-{light,dark}.{svg,png}` — showcase composite, paired variants.
//!
//! Outer card (rounded, 1px border) → eyebrow strip → 3×3 grid of captioned
//! cells. Each cell inlines its theme-matched example SVG and draws a caption
//! beneath. Dual format: SVG is canonical (vector); a 2× retina PNG is
//! rasterized alongside and is what the README references — saves the README
//! from inlining 8 MB of example SVG markup.

use anyhow::Result;
use std::path::Path;

use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::png;
use super::svg::{header, inline, write_atomic};

const W: u32 = 880;
const PAD: u32 = 24;
const GUTTER: u32 = 10;
const COLS: u32 = 3;
const ROWS: u32 = 3;
const EYEBROW_H: u32 = 36;
const CAP_H: u32 = 30;
const RADIUS: f32 = 12.0;
const PNG_SCALE: f32 = 2.0;

const GALLERY: &[(&str, &str)] = &[
    ("examples/scientific/contour_fields", "contour fields"),
    (
        "examples/scientific/kruskal_szekeres_line",
        "kruskal–szekeres",
    ),
    ("examples/scientific/laser_plasma", "laser plasma · contour"),
    ("examples/scientific/reciprocal_space", "reciprocal space"),
    ("examples/theming/custom_colormap", "custom colormap"),
    (
        "examples/scientific/bollinger_candlestick",
        "bollinger · candlestick",
    ),
    (
        "examples/composition/distribution_dashboard",
        "distribution dashboard",
    ),
    ("examples/basics/bubble_scatter", "bubble · scatter"),
    ("examples/composition/statistical", "statistical"),
];

/// How example thumbs are embedded in a cell. `InlineSvg` keeps the SVG
/// composite vector. `EmbedExamplePng` references the example's pre-rendered
/// PNG sibling so strokes survive the down-sample to cell size — 2×
/// rasterization of an inlined SVG cannot guarantee that for thin lines.
#[derive(Copy, Clone)]
enum CellMode {
    InlineSvg,
    EmbedExamplePng,
}

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let svg = compose(root, theme, CellMode::InlineSvg)?;
    let out = root.join(format!("assets/gallery-{}.svg", theme.suffix()));
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());

    let svg_for_png = compose(root, theme, CellMode::EmbedExamplePng)?;
    let pix = png::rasterize_at_scale(&svg_for_png, PNG_SCALE, root)?;
    let png_out = root.join(format!("assets/gallery-{}.png", theme.suffix()));
    png::write_png_atomic(&pix, &png_out)?;
    println!(
        "wrote {} ({} bytes)",
        png_out.display(),
        std::fs::metadata(&png_out)?.len()
    );
    Ok(())
}

fn compose(root: &Path, theme: Theme, cell_mode: CellMode) -> Result<String> {
    let p = palette(theme);
    let cell_w = (W - 2 * PAD - GUTTER * (COLS - 1)) / COLS;
    let cell_img_h = ((cell_w as f32) * 0.62) as u32;
    let cell_h = cell_img_h + CAP_H;
    let grid_h = cell_h * ROWS + GUTTER * (ROWS - 1);
    let h = EYEBROW_H + grid_h + 2 * PAD;

    let mut out = header(W, h, "starsight showcase composite", "starsight showcase");

    // Outer rounded card.
    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{w}" height="{hh}" rx="{r}" fill="{bg}" stroke="{s}" stroke-width="1"/>
"#,
        w = W - 1,
        hh = h - 1,
        r = RADIUS,
        bg = p.bg,
        s = p.border,
    ));

    // Eyebrow strip.
    out.push_str(&format!(
        r#"  <text x="{x}" y="{y}" font-family="{f}" font-size="12" fill="{c}" letter-spacing="0.6">// showcase  ·  9 of 38 examples  ·  source under examples/</text>
"#,
        x = PAD,
        y = PAD + 22,
        f = MONO_FAMILY,
        c = p.muted,
    ));

    let suffix = theme.example_suffix();
    for (i, (base, caption)) in GALLERY.iter().enumerate() {
        let col = (i as u32) % COLS;
        let row = (i as u32) / COLS;
        let x0 = PAD + col * (cell_w + GUTTER);
        let y0 = PAD + EYEBROW_H + row * (cell_h + GUTTER);

        // Cell background + 1px border (image area).
        out.push_str(&format!(
            r#"  <rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{bg}" stroke="{c}" stroke-width="1"/>
"#,
            x = x0,
            y = y0,
            w = cell_w,
            h = cell_img_h,
            bg = p.card,
            c = p.border,
        ));

        let rel = format!("{base}{suffix}");
        match cell_mode {
            CellMode::InlineSvg => {
                let path = root.join(format!("{rel}.svg"));
                if path.exists() {
                    let (inner, vb) = inline(&path)?;
                    out.push_str(&format!(
                        r#"  <svg x="{x0}" y="{y0}" width="{cell_w}" height="{cell_img_h}" viewBox="{vb}" preserveAspectRatio="xMidYMid meet">{inner}</svg>
"#,
                    ));
                } else {
                    out.push_str(&format!(
                        r#"  <text x="{x}" y="{y}" font-size="11" fill="{c}" text-anchor="middle">missing</text>
"#,
                        x = x0 + cell_w / 2,
                        y = y0 + cell_img_h / 2,
                        c = p.muted,
                    ));
                }
            }
            CellMode::EmbedExamplePng => {
                let png_rel = format!("{rel}.png");
                let path = root.join(&png_rel);
                if path.exists() {
                    // usvg resolves relative href against Options::resources_dir (= root).
                    out.push_str(&format!(
                        r#"  <image x="{x0}" y="{y0}" width="{cell_w}" height="{cell_img_h}" href="{png_rel}" preserveAspectRatio="xMidYMid meet"/>
"#,
                    ));
                } else {
                    out.push_str(&format!(
                        r#"  <text x="{x}" y="{y}" font-size="11" fill="{c}" text-anchor="middle">missing</text>
"#,
                        x = x0 + cell_w / 2,
                        y = y0 + cell_img_h / 2,
                        c = p.muted,
                    ));
                }
            }
        }

        // Caption beneath the cell.
        out.push_str(&format!(
            r#"  <text x="{x}" y="{y}" font-family="{f}" font-weight="700" font-size="12" fill="{c}" text-anchor="middle">{caption}</text>
"#,
            x = x0 + cell_w / 2,
            y = y0 + cell_img_h + 20,
            f = SANS,
            c = p.text,
            caption = caption,
        ));
    }

    out.push_str("</svg>\n");
    Ok(out)
}
