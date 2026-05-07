//! Per-category status matrices: marks, scales, backends, stats, layout, output, themes.
//!
//! Each matrix is an SVG: outer rounded card, eyebrow `// <category> · N working /
//! M planned`, thin progress bar, header row + body rows. Body row = status dot
//! (filled = working, hollow = planned, half = wip) + name + version pill.
//!
//! Outputs: `assets/matrices/<category>-{light,dark}.svg`. The README references
//! these via `<picture>` blocks inside `<details>` containers under a `## Status`
//! heading.

mod backends;
mod layout;
mod marks;
mod output;
mod scales;
mod stats;
mod themes;

use anyhow::Result;
use std::path::Path;

use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::svg::{header, write_atomic};

/// One row in a matrix.
#[derive(Copy, Clone)]
pub struct Row<'a> {
    pub name: &'a str,
    pub status: Status,
    pub version: &'a str,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Status {
    Working,
    Wip,
    Planned,
}

/// One matrix.
pub struct Matrix<'a> {
    pub stem: &'a str,
    pub title: &'a str,
    pub rows: &'a [Row<'a>],
    /// Optional tail blurb under the matrix; rendered as a single `<text>` line.
    pub footnote: Option<&'a str>,
}

const W: u32 = 920;
const PAD: u32 = 24;
const RADIUS: f32 = 12.0;
const HEADER_H: u32 = 36;
const ROW_H: u32 = 28;
const PROGRESS_H: u32 = 4;
const EYEBROW_H: u32 = 22;
const BAR_GAP: u32 = 10;
const FOOTNOTE_H: u32 = 24;
const FONT_SIZE: u32 = 13;

const NAME_X: u32 = PAD + 28; // status dot sits in the first 28 px
const VER_W: u32 = 80;
const STATUS_LABEL_W: u32 = 90;

pub fn regen_all(root: &Path, theme: Theme) -> Result<()> {
    let dir = root.join("assets/matrices");
    std::fs::create_dir_all(&dir)?;

    for matrix in [
        marks::matrix(),
        scales::matrix(),
        backends::matrix(),
        stats::matrix(),
        layout::matrix(),
        output::matrix(),
        themes::matrix(),
    ] {
        let svg = render_matrix(&matrix, theme);
        let out = dir.join(format!("{}-{}.svg", matrix.stem, theme.suffix()));
        write_atomic(&out, &svg)?;
        println!("wrote {} ({} bytes)", out.display(), svg.len());
    }
    Ok(())
}

fn render_matrix(matrix: &Matrix, theme: Theme) -> String {
    let p = palette(theme);

    let working = matrix
        .rows
        .iter()
        .filter(|r| r.status == Status::Working)
        .count();
    let wip = matrix
        .rows
        .iter()
        .filter(|r| r.status == Status::Wip)
        .count();
    let planned = matrix
        .rows
        .iter()
        .filter(|r| r.status == Status::Planned)
        .count();
    let total = matrix.rows.len();

    let body_h = HEADER_H + ROW_H * (matrix.rows.len() as u32);
    let footnote_h = if matrix.footnote.is_some() {
        FOOTNOTE_H
    } else {
        0
    };
    let h = PAD + EYEBROW_H + PROGRESS_H + BAR_GAP + body_h + footnote_h + PAD;

    let mut out = header(W, h, matrix.title, matrix.title);

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
    let eyebrow = format!(
        "// {} · {} working / {} planned{}",
        matrix.stem,
        working + wip,
        planned,
        if total == 0 {
            String::new()
        } else {
            format!(" · {total} total")
        },
    );
    out.push_str(&format!(
        r#"  <text x="{x}" y="{y}" font-family="{f}" font-size="11" fill="{c}">{txt}</text>
"#,
        x = PAD,
        y = PAD + 14,
        f = MONO_FAMILY,
        c = p.subtext,
        txt = escape(&eyebrow),
    ));

    // Progress bar (working+wip / total).
    let bar_y = PAD + EYEBROW_H + 2;
    let bar_w = W - 2 * PAD;
    out.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{w}" height="{ph}" rx="2" fill="{bg}" stroke="{s}" stroke-width="0.5"/>
"#,
        x = PAD,
        y = bar_y,
        w = bar_w,
        ph = PROGRESS_H,
        bg = p.rule,
        s = p.border,
    ));
    if total > 0 {
        let fill_w = (bar_w as f32) * ((working + wip) as f32) / (total as f32);
        out.push_str(&format!(
            r#"  <rect x="{x}" y="{y}" width="{w:.1}" height="{ph}" rx="2" fill="{c}"/>
"#,
            x = PAD,
            y = bar_y,
            w = fill_w,
            ph = PROGRESS_H,
            c = p.text,
        ));
    }

    // Header row.
    let body_y = bar_y + PROGRESS_H + BAR_GAP;
    out.push_str(&format!(
        r#"  <text x="{x}" y="{y}" font-family="{f}" font-weight="700" font-size="{sz}" fill="{c}">name</text>
"#,
        x = NAME_X,
        y = body_y + 22,
        f = SANS,
        sz = FONT_SIZE,
        c = p.text,
    ));
    out.push_str(&format!(
        r#"  <text x="{x}" y="{y}" font-family="{f}" font-weight="700" font-size="{sz}" fill="{c}">status</text>
"#,
        x = W - PAD - VER_W - 12 - STATUS_LABEL_W,
        y = body_y + 22,
        f = SANS,
        sz = FONT_SIZE,
        c = p.text,
    ));
    out.push_str(&format!(
        r#"  <text x="{x}" y="{y}" font-family="{f}" font-weight="700" font-size="{sz}" fill="{c}" text-anchor="end">added in</text>
"#,
        x = W - PAD - 8,
        y = body_y + 22,
        f = SANS,
        sz = FONT_SIZE,
        c = p.text,
    ));
    out.push_str(&format!(
        r#"  <line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}" stroke="{c}" stroke-width="1"/>
"#,
        x1 = PAD,
        x2 = W - PAD,
        y = body_y + HEADER_H,
        c = p.border,
    ));

    // Body rows.
    for (i, row) in matrix.rows.iter().enumerate() {
        let y = body_y + HEADER_H + ROW_H * (i as u32);
        if i % 2 == 1 {
            out.push_str(&format!(
                r#"  <rect x="{x}" y="{y}" width="{w}" height="{rh}" fill="{c}"/>
"#,
                x = PAD,
                w = W - 2 * PAD,
                rh = ROW_H,
                c = p.card,
            ));
        }
        out.push_str(&render_row(p, y, row));
    }

    // Footnote (one line, mono).
    if let Some(note) = matrix.footnote {
        let fn_y = body_y + HEADER_H + ROW_H * (matrix.rows.len() as u32) + 18;
        out.push_str(&format!(
            r#"  <text x="{x}" y="{y}" font-family="{f}" font-size="11" fill="{c}">{txt}</text>
"#,
            x = PAD,
            y = fn_y,
            f = MONO_FAMILY,
            c = p.subtext,
            txt = escape(note),
        ));
    }

    out.push_str("</svg>\n");
    out
}

fn render_row(p: &super::palette::Palette, y: u32, row: &Row) -> String {
    let mut s = String::new();

    // Status dot at left of row.
    let dot_cx = PAD + 12;
    let dot_cy = y + 14;
    match row.status {
        Status::Working => {
            s.push_str(&format!(
                r#"  <circle cx="{cx}" cy="{cy}" r="5" fill="{c}"/>
"#,
                cx = dot_cx,
                cy = dot_cy,
                c = p.text,
            ));
        }
        Status::Wip => {
            s.push_str(&format!(
                r#"  <circle cx="{cx}" cy="{cy}" r="5" fill="{c}" stroke="{s}" stroke-width="1"/>
"#,
                cx = dot_cx,
                cy = dot_cy,
                c = p.muted,
                s = p.text,
            ));
        }
        Status::Planned => {
            s.push_str(&format!(
                r#"  <circle cx="{cx}" cy="{cy}" r="4.5" fill="{bg}" stroke="{s}" stroke-width="1"/>
"#,
                cx = dot_cx,
                cy = dot_cy,
                bg = p.bg,
                s = p.muted,
            ));
        }
    }

    // Name.
    s.push_str(&format!(
        r#"  <text x="{x}" y="{ty}" font-family="{f}" font-size="{sz}" fill="{c}">{txt}</text>
"#,
        x = NAME_X,
        ty = y + 19,
        f = SANS,
        sz = FONT_SIZE,
        c = match row.status {
            Status::Planned => p.subtext,
            _ => p.text,
        },
        txt = escape(row.name),
    ));

    // Status label.
    let status_label = match row.status {
        Status::Working => "working",
        Status::Wip => "wip",
        Status::Planned => "planned",
    };
    s.push_str(&format!(
        r#"  <text x="{x}" y="{ty}" font-family="{f}" font-size="11" fill="{c}">{txt}</text>
"#,
        x = W - PAD - VER_W - 12 - STATUS_LABEL_W,
        ty = y + 19,
        f = MONO_FAMILY,
        c = p.subtext,
        txt = status_label,
    ));

    // Version pill.
    s.push_str(&format!(
        r#"  <text x="{x}" y="{ty}" font-family="{f}" font-size="11" fill="{c}" text-anchor="end">{txt}</text>
"#,
        x = W - PAD - 8,
        ty = y + 19,
        f = MONO_FAMILY,
        c = p.subtext,
        txt = escape(row.version),
    ));

    s
}

pub(super) fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
