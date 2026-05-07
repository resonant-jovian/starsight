//! `assets/lorenz-{light,dark}.{svg,png}` — bordered "worked example" card.
//!
//! Wraps the `examples/scientific/lorenz_line{,_dark}.svg` chart in a chrome
//! card (rounded rect, 1px border) so it sits visually next to the hero,
//! gallery, status, etc. assets in the README. Inline-embeds the example SVG
//! verbatim — `<img src=...>` SVG sandboxing forbids external `<image href>`
//! references. Dual format: SVG is canonical; a 2× retina PNG is rasterized
//! alongside and is what the README references.

use anyhow::{Context, Result};
use std::path::Path;

use super::palette::{Theme, palette};
use super::png;
use super::svg::{header, inline, write_atomic};

const W: u32 = 880;
const PAD: u32 = 16;
const RADIUS: f32 = 12.0;
const PNG_SCALE: f32 = 2.0;

/// Aspect-ratio used to size the card height when the example SVG is missing
/// (`lorenz_line` normally renders at 1000×600 ≈ 1.667).
const FALLBACK_RATIO: f32 = 1000.0 / 600.0;

/// How the Lorenz example is embedded. `InlineSvg` keeps the SVG composite
/// vector. `EmbedExamplePng` references the example's pre-rendered PNG so
/// strokes survive the down-sample to card width — 2× rasterization of an
/// inlined SVG cannot guarantee that for thin lines.
#[derive(Copy, Clone)]
enum CellMode {
    InlineSvg,
    EmbedExamplePng,
}

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let svg = compose(root, theme, CellMode::InlineSvg)?;
    let out = root.join(format!("assets/lorenz-{}.svg", theme.suffix()));
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());

    let svg_for_png = compose(root, theme, CellMode::EmbedExamplePng)?;
    let pix = png::rasterize_at_scale(&svg_for_png, PNG_SCALE, root)?;
    let png_out = root.join(format!("assets/lorenz-{}.png", theme.suffix()));
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
    let suffix = theme.example_suffix();
    let svg_src = root.join(format!("examples/scientific/lorenz_line{suffix}.svg"));
    let png_rel = format!("examples/scientific/lorenz_line{suffix}.png");
    let png_src = root.join(&png_rel);

    let inner_w = W - 2 * PAD;
    // Card height comes from the example's aspect ratio; both modes need the
    // same height so the surrounding chrome (rect, padding) stays identical.
    let (vb_for_ratio, has_svg) = if svg_src.exists() {
        let (_, vb) =
            inline(&svg_src).with_context(|| format!("inlining {}", svg_src.display()))?;
        (Some(vb), true)
    } else {
        (None, false)
    };
    let ratio = vb_for_ratio
        .as_deref()
        .and_then(parse_ratio)
        .unwrap_or(FALLBACK_RATIO);
    let inner_h = ((inner_w as f32) / ratio).round() as u32;
    let h = inner_h + 2 * PAD;

    let mut out = header(W, h, "starsight Lorenz worked example", "Lorenz attractor");
    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{w}" height="{hh}" rx="{r}" fill="{bg}" stroke="{s}" stroke-width="1"/>
"#,
        w = W - 1,
        hh = h - 1,
        r = RADIUS,
        bg = p.card,
        s = p.border,
    ));

    match cell_mode {
        CellMode::InlineSvg => {
            if has_svg {
                let (inner, vb) =
                    inline(&svg_src).with_context(|| format!("inlining {}", svg_src.display()))?;
                out.push_str(&format!(
                    r#"  <svg x="{PAD}" y="{PAD}" width="{inner_w}" height="{inner_h}" viewBox="{vb}" preserveAspectRatio="xMidYMid meet">{inner}</svg>
"#,
                ));
            } else {
                out.push_str(&missing(&svg_src, inner_w, inner_h, p.muted));
            }
        }
        CellMode::EmbedExamplePng => {
            if png_src.exists() {
                // usvg resolves relative href against Options::resources_dir (= root).
                out.push_str(&format!(
                    r#"  <image x="{PAD}" y="{PAD}" width="{inner_w}" height="{inner_h}" href="{png_rel}" preserveAspectRatio="xMidYMid meet"/>
"#,
                ));
            } else {
                out.push_str(&missing(&png_src, inner_w, inner_h, p.muted));
            }
        }
    }
    out.push_str("</svg>\n");
    Ok(out)
}

fn missing(src: &Path, inner_w: u32, inner_h: u32, muted: &str) -> String {
    format!(
        r#"  <text x="{x}" y="{y}" font-size="12" fill="{muted}" text-anchor="middle">{display} missing — run `cargo xtask chrome`</text>
"#,
        x = PAD + inner_w / 2,
        y = PAD + inner_h / 2,
        display = src.display(),
    )
}

fn parse_ratio(view_box: &str) -> Option<f32> {
    let mut parts = view_box.split_ascii_whitespace();
    let _x = parts.next()?;
    let _y = parts.next()?;
    let w: f32 = parts.next()?.parse().ok()?;
    let h: f32 = parts.next()?.parse().ok()?;
    if h == 0.0 {
        return None;
    }
    Some(w / h)
}
