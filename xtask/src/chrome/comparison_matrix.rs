//! `assets/comparison-{light,dark}.svg` — feature × crate matrix vs. siblings.
//!
//! Rows = features, columns = crates. starsight column is visually
//! distinguished (faint card-coloured background fill). Cells: filled `●` if
//! the feature is present, light `·` if absent. Footer prose summarises the
//! pre-1.0 bet.

use anyhow::Result;
use std::path::Path;

use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::svg::{header, write_atomic};

const W: u32 = 960;
const PAD: u32 = 24;
const RADIUS: f32 = 12.0;

const HEADER_H: u32 = 50;
const ROW_H: u32 = 28;
const FOOTER_H: u32 = 64;
const EYEBROW_H: u32 = 24;

const FEATURE_COL_W: u32 = 240;

const FEATURES: &[&str] = &[
    "CPU raster",
    "SVG export",
    "PDF export",
    "GPU rendering",
    "Terminal",
    "3D charts",
    "Polars / DataFrame",
    "WASM / browser",
    "Themes & colormaps",
    "Interactive (zoom/pan)",
    "Animation",
    "Pure-Rust core",
    "Statistical marks",
    "Faceting",
];

struct Crate {
    name: &'static str,
    note: &'static str,
    /// Feature presence — must align with FEATURES.
    vals: [u8; 14],
}

const CRATES: &[Crate] = &[
    Crate {
        name: "starsight",
        note: "unified · pre-1.0",
        //         CPU SVG PDF GPU TTY 3D  Pol Wasm Theme Int Anim PureR Stats Facet
        vals: [1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0],
    },
    Crate {
        name: "plotters",
        note: "mature · syn-tree",
        vals: [1, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0],
    },
    Crate {
        name: "plotly-rs",
        note: "json → js bridge",
        vals: [0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1],
    },
    Crate {
        name: "charming",
        note: "echarts wrapper",
        vals: [0, 1, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0],
    },
    Crate {
        name: "poloto",
        note: "svg-only · simple",
        vals: [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    },
    Crate {
        name: "matplotlib",
        note: "python · canonical",
        vals: [1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1],
    },
    Crate {
        name: "ggplot2",
        note: "R · grammar of graphics",
        vals: [1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1],
    },
    Crate {
        name: "plotly.py",
        note: "python · interactive html",
        vals: [0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1],
    },
    Crate {
        name: "gnuplot",
        note: "C · CLI",
        vals: [1, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0],
    },
];

const FOOTER: &str = "starsight is the pre-1.0 newcomer. The bet: one crate covering CPU + GPU + terminal + PDF \
     with a grammar-of-graphics builder and shared themes/colormaps via chromata + prismatica.";

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let svg = render(theme);
    let out = root.join(format!("assets/comparison-{}.svg", theme.suffix()));
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());
    Ok(())
}

fn render(theme: Theme) -> String {
    let p = palette(theme);

    let crates_n = CRATES.len() as u32;
    let crate_col_w = (W - 2 * PAD - FEATURE_COL_W) / crates_n;
    let body_h = HEADER_H + ROW_H * (FEATURES.len() as u32);
    let h = PAD + EYEBROW_H + body_h + FOOTER_H + PAD;

    let mut out = header(
        W,
        h,
        "starsight vs. sibling charting libraries",
        "starsight comparison",
    );

    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{w}" height="{hh}" rx="{r}" fill="{bg}" stroke="{s}" stroke-width="1"/>
"#,
        w = W - 1,
        hh = h - 1,
        r = RADIUS,
        bg = p.bg,
        s = p.border,
    ));

    out.push_str(&format!(
        r#"  <text x="{x}" y="{y}" font-family="{f}" font-size="11" fill="{c}">// vs. siblings · rust + non-rust plotting · as of 0.3</text>
"#,
        x = PAD,
        y = PAD + 16,
        f = MONO_FAMILY,
        c = p.subtext,
    ));

    let body_x = PAD;
    let body_y = PAD + EYEBROW_H + 4;

    // starsight column highlight (full body height).
    let starsight_idx = CRATES
        .iter()
        .position(|c| c.name == "starsight")
        .unwrap_or(0) as u32;
    let starsight_x = body_x + FEATURE_COL_W + starsight_idx * crate_col_w;
    out.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{c}"/>
"#,
        x = starsight_x,
        y = body_y,
        w = crate_col_w,
        h = body_h,
        c = p.card,
    ));

    // Header row.
    // feature label.
    out.push_str(&format!(
        r#"  <text x="{x}" y="{y}" font-family="{f}" font-size="11" fill="{c}" letter-spacing="0.6">FEATURE</text>
"#,
        x = body_x + 6,
        y = body_y + 32,
        f = MONO_FAMILY,
        c = p.muted,
    ));
    // crate names + notes.
    for (i, c) in CRATES.iter().enumerate() {
        let cx = body_x + FEATURE_COL_W + (i as u32) * crate_col_w + crate_col_w / 2;
        let is_us = c.name == "starsight";
        out.push_str(&format!(
            r#"  <text x="{x}" y="{y}" font-family="{f}" font-weight="{fw}" font-size="12" fill="{cc}" text-anchor="middle">{name}</text>
"#,
            x = cx,
            y = body_y + 22,
            f = SANS,
            fw = if is_us { "700" } else { "600" },
            cc = p.text,
            name = escape(c.name),
        ));
        out.push_str(&format!(
            r#"  <text x="{x}" y="{y}" font-family="{f}" font-size="9" fill="{cc}" text-anchor="middle">{note}</text>
"#,
            x = cx,
            y = body_y + 38,
            f = MONO_FAMILY,
            cc = p.muted,
            note = escape(c.note),
        ));
    }

    // Header underline.
    out.push_str(&format!(
        r#"  <line x1="{x1}" y1="{ly}" x2="{x2}" y2="{ly}" stroke="{c}" stroke-width="1"/>
"#,
        x1 = body_x,
        x2 = body_x + FEATURE_COL_W + crates_n * crate_col_w,
        ly = body_y + HEADER_H,
        c = p.border,
    ));

    // Body rows.
    for (fi, feat) in FEATURES.iter().enumerate() {
        let row_y = body_y + HEADER_H + ROW_H * (fi as u32);
        if fi % 2 == 1 {
            out.push_str(&format!(
                r#"  <rect x="{x}" y="{y}" width="{w}" height="{rh}" fill="{c}" opacity="0.6"/>
"#,
                x = body_x,
                y = row_y,
                w = FEATURE_COL_W,
                rh = ROW_H,
                c = p.card,
            ));
        }
        out.push_str(&format!(
            r#"  <text x="{x}" y="{y}" font-family="{f}" font-size="12" fill="{c}">{feat}</text>
"#,
            x = body_x + 6,
            y = row_y + 19,
            f = SANS,
            c = p.text,
            feat = escape(feat),
        ));

        for (ci, crat) in CRATES.iter().enumerate() {
            let cx = body_x + FEATURE_COL_W + (ci as u32) * crate_col_w + crate_col_w / 2;
            let cy = row_y + 19;
            let val = crat.vals[fi];
            let glyph = if val == 1 { "●" } else { "·" };
            out.push_str(&format!(
                r#"  <text x="{x}" y="{y}" font-family="{f}" font-size="14" fill="{c}" text-anchor="middle">{g}</text>
"#,
                x = cx,
                y = cy,
                f = SANS,
                c = if val == 1 { p.text } else { p.muted },
                g = glyph,
            ));
        }
    }

    // Footer prose (mono, subtext, wrapped manually into ~3 lines).
    let footer_y = body_y + body_h + 18;
    let lines = wrap(FOOTER, 100);
    for (i, line) in lines.iter().enumerate() {
        out.push_str(&format!(
            r#"  <text x="{x}" y="{y}" font-family="{f}" font-size="11" fill="{c}">{line}</text>
"#,
            x = PAD,
            y = footer_y + (i as u32) * 16,
            f = MONO_FAMILY,
            c = p.subtext,
            line = escape(line),
        ));
    }

    out.push_str("</svg>\n");
    out
}

/// Greedy word-wrap into lines of at most `max_chars` characters.
fn wrap(s: &str, max_chars: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        if cur.is_empty() {
            cur.push_str(word);
        } else if cur.chars().count() + 1 + word.chars().count() <= max_chars {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(std::mem::take(&mut cur));
            cur.push_str(word);
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
