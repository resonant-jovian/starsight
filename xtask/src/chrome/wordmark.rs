//! `assets/wordmark-light.svg` — eclipse mark + "starsight" lockup, monochrome.
//!
//! The eclipse is the brand mark (E03 · total). In the monochrome rebrand, the
//! corona is rendered in a subtle grey instead of the original orange, and the
//! disc stays near-black. Pure SVG paths — no fonts, no rasters.

use anyhow::Result;
use std::path::Path;

use super::eclipse;
use super::palette::{MONO, MONO_FAMILY, SANS};
use super::svg::{header, write_atomic};

const W: u32 = 720;
const H: u32 = 220;

pub fn regen(root: &Path) -> Result<()> {
    let svg = render();
    let out = root.join("assets/wordmark-light.svg");
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());
    Ok(())
}

fn render() -> String {
    let p = &MONO;
    let mut out = header(W, H, "starsight wordmark", "starsight");

    // Eclipse mark at (24, 60), size 100×100
    out.push_str(&format!(r#"  <g transform="translate(24,60)">
"#));
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
