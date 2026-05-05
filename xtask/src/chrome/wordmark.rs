//! `assets/wordmark-light.svg` — eclipse mark + "starsight" lockup, monochrome.
//!
//! The eclipse is the brand mark (E03 · total). In the monochrome rebrand, the
//! corona is rendered in a subtle grey instead of the original orange, and the
//! disc stays near-black. Pure SVG paths — no fonts, no rasters.

use anyhow::Result;
use std::path::Path;

use super::eclipse;
use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::svg::{header, write_atomic};

const W: u32 = 720;
const H: u32 = 220;

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let svg = render(theme);
    let out = root.join(format!("assets/wordmark-{}.svg", theme.suffix()));
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());
    Ok(())
}

fn render(theme: Theme) -> String {
    let p = palette(theme);
    let mut out = header(W, H, "starsight wordmark", "starsight");

    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{w}" height="{h}" rx="8" fill="{c}" stroke="{s}" stroke-width="1"/>
"#,
        w = W - 1,
        h = H - 1,
        c = p.card,
        s = p.border
    ));

    // Eclipse mark at (24, 60), size 100×100
    out.push_str(&format!(
        r#"  <g transform="translate(24,60)">
"#
    ));
    out.push_str(&eclipse::svg_inner(p));
    out.push_str("  </g>\n");

    // Wordmark text
    out.push_str(&format!(
        r#"  <text x="170" y="156" font-family="{f}" font-weight="700" font-size="104" fill="{c}" letter-spacing="-3">starsight</text>
"#,
        f = SANS,
        c = p.text
    ));
    out.push_str(&format!(
        r#"  <text x="172" y="190" font-family="{f}" font-size="14" fill="{c}">scientific visualization · rust crate</text>
"#,
        f = MONO_FAMILY,
        c = p.muted
    ));
    out.push_str("</svg>\n");
    out
}
