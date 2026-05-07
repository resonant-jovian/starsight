//! `assets/pipeline-{light,dark}.svg` — what `plot!()` actually does.
//!
//! 8-stage horizontal flow (DATA → MARK → STATS → SCALE → LAYOUT → SCENE →
//! BACKEND → OUTPUT) with arrows between cards, plus a TRACE panel below
//! showing a worked `plot!(...).save("chart.png")` example.
//!
//! Pure SVG — `<text>` for everything, no rasterised content. Width 880 px
//! to match the rest of the chrome family.

use anyhow::Result;
use std::path::Path;

use super::palette::{MONO_FAMILY, Theme, palette};
use super::svg::{header, write_atomic};

const W: u32 = 880;
const PAD: u32 = 24;
const RADIUS: f32 = 12.0;

const STAGES: &[(&str, &[&str])] = &[
    ("DATA", &["Vec<f64>", "slices", "Polars", "ndarray"]),
    ("MARK", &["LineMark", "PointMark", "BarMark", "…"]),
    ("STATS", &["Bin", "KDE", "Boxplot"]),
    ("SCALE", &["Linear", "Log", "Band", "Wilkinson"]),
    ("LAYOUT", &["Figure", "Grid", "Facet", "Legend"]),
    ("SCENE", &["Path", "Text", "Group", "Clip"]),
    ("BACKEND", &["Skia", "SVG", "wgpu", "PDF", "Kitty"]),
    ("OUTPUT", &[".png", ".svg", "window", "tty"]),
];

const TRACE_CODE: &str =
    r#"plot!(&[1.0, 2.0, 3.0], &[10., 20., 15.]).save("chart.png")"#;
const TRACE_PILLS: &[&str] = &[
    "DATA",
    "MARK·LineMark",
    "SCALE·Linear",
    "LAYOUT·Figure 800×600",
    "SCENE",
    "BACKEND·Skia",
    "OUTPUT·png",
];

const STAGE_W: u32 = 96;
const STAGE_H: u32 = 124;
const STAGE_GAP: u32 = 4;
const ARROW_W: u32 = 10;

const EYEBROW_H: u32 = 32;
const TRACE_H: u32 = 100;
const TRACE_GAP: u32 = 14;

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let svg = render(theme);
    let out = root.join(format!("assets/pipeline-{}.svg", theme.suffix()));
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());
    Ok(())
}

fn render(theme: Theme) -> String {
    let p = palette(theme);
    let stages_n = STAGES.len() as u32;
    let row_w = stages_n * STAGE_W + (stages_n - 1) * (STAGE_GAP + ARROW_W + STAGE_GAP);
    // Centre the row inside the 880-px content area.
    let row_x = PAD + ((W - 2 * PAD).saturating_sub(row_w)) / 2;
    let row_y = PAD + EYEBROW_H;

    let h = PAD + EYEBROW_H + STAGE_H + TRACE_GAP + TRACE_H + PAD;

    let mut out = header(W, h, "starsight pipeline — plot!() trace", "starsight pipeline");

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

    // Eyebrow.
    out.push_str(&format!(
        r#"  <text x="{ex}" y="{ey}" font-family="{f}" font-size="11" fill="{c}">// pipeline · what plot!() actually does</text>
"#,
        ex = PAD,
        ey = PAD + 18,
        f = MONO_FAMILY,
        c = p.subtext,
    ));

    // Stage cards + arrows.
    let mut x_cursor = row_x;
    for (i, (tag, items)) in STAGES.iter().enumerate() {
        out.push_str(&render_stage(p, x_cursor, row_y, tag, items));
        x_cursor += STAGE_W;
        if i + 1 < STAGES.len() {
            // arrow between cards
            let ax = x_cursor + STAGE_GAP;
            let ay = row_y + STAGE_H / 2 + 4;
            out.push_str(&format!(
                r#"  <text x="{ax}" y="{ay}" font-family="{f}" font-size="14" fill="{c}" text-anchor="middle">→</text>
"#,
                ax = ax + ARROW_W / 2,
                f = MONO_FAMILY,
                c = p.muted,
            ));
            x_cursor += STAGE_GAP + ARROW_W + STAGE_GAP;
        }
    }

    // TRACE panel below stages.
    let trace_y = row_y + STAGE_H + TRACE_GAP;
    out.push_str(&render_trace(p, PAD, trace_y, W - 2 * PAD, TRACE_H));

    out.push_str("</svg>\n");
    out
}

fn render_stage(p: &super::palette::Palette, x: u32, y: u32, tag: &str, items: &[&str]) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="{bg}" stroke="{c}" stroke-width="1"/>
"#,
        w = STAGE_W,
        h = STAGE_H,
        bg = p.card,
        c = p.border,
    ));
    // Tag at top.
    s.push_str(&format!(
        r#"  <text x="{tx}" y="{ty}" font-family="{f}" font-weight="700" font-size="10" fill="{c}" letter-spacing="0.6">{tag}</text>
"#,
        tx = x + 10,
        ty = y + 18,
        f = MONO_FAMILY,
        c = p.text,
    ));
    // Items (mono, subtext) below.
    for (i, item) in items.iter().enumerate() {
        s.push_str(&format!(
            r#"  <text x="{tx}" y="{ty}" font-family="{f}" font-size="10" fill="{c}">{txt}</text>
"#,
            tx = x + 10,
            ty = y + 36 + (i as u32) * 16,
            f = MONO_FAMILY,
            c = p.subtext,
            txt = escape(item),
        ));
    }
    s
}

fn render_trace(p: &super::palette::Palette, x: u32, y: u32, w: u32, h: u32) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{w}" height="{h}" rx="4" fill="{bg}" stroke="{c}" stroke-width="1"/>
"#,
        bg = p.card,
        c = p.border,
    ));
    // TRACE tag.
    s.push_str(&format!(
        r#"  <text x="{tx}" y="{ty}" font-family="{f}" font-weight="700" font-size="10" fill="{c}" letter-spacing="0.8">TRACE</text>
"#,
        tx = x + 12,
        ty = y + 20,
        f = MONO_FAMILY,
        c = p.text,
    ));
    // The plot!() one-liner.
    s.push_str(&format!(
        r#"  <text x="{tx}" y="{ty}" font-family="{f}" font-size="12" fill="{c}">{code}</text>
"#,
        tx = x + 12,
        ty = y + 46,
        f = MONO_FAMILY,
        c = p.text,
        code = escape(TRACE_CODE),
    ));
    // Pills tracing each stage.
    let pill_y = y + 62;
    let mut pill_x = x + 12;
    for pill in TRACE_PILLS {
        let label = escape(pill);
        // Approx width: ~7 px/char + 16 padding.
        let pill_w = (pill.chars().count() as u32) * 7 + 16;
        s.push_str(&format!(
            r#"  <rect x="{px}" y="{py}" width="{pw}" height="20" rx="3" fill="{bg}" stroke="{c}" stroke-width="1"/>
"#,
            px = pill_x,
            py = pill_y,
            pw = pill_w,
            bg = p.bg,
            c = p.border,
        ));
        s.push_str(&format!(
            r#"  <text x="{tx}" y="{ty}" font-family="{f}" font-size="10" fill="{c}">{label}</text>
"#,
            tx = pill_x + 8,
            ty = pill_y + 14,
            f = MONO_FAMILY,
            c = p.subtext,
        ));
        pill_x += pill_w + 6;
    }
    s
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
