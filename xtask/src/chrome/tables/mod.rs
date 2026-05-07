//! Pure-SVG renderings of the README's GFM tables — flag/feature, capabilities,
//! backends, language translation, and library comparison.
//!
//! Each table is an SVG composed from a shared primitive (`render_table`) so
//! the styling, theming, and bordered card chrome stays consistent across the
//! README. Cells are single-line `<text>` elements (rich markdown — bold,
//! code spans, links — is stripped at the data layer).
//!
//! Outputs: `assets/tables/<name>-{light,dark}.svg`.

mod backends;
mod capabilities;
mod comparison;
mod install;
mod translation;

use anyhow::Result;
use std::path::Path;

use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::svg::{header, write_atomic};

/// One table rendered into an SVG.
pub struct Table<'a> {
    /// File-name stem, e.g. `"install"` → `assets/tables/install-{light,dark}.svg`.
    pub stem: &'a str,
    /// `<title>` content for screen readers.
    pub title: &'a str,
    /// Header cell strings.
    pub header: &'a [&'a str],
    /// Body rows; each row must have `header.len()` cells.
    pub rows: &'a [&'a [&'a str]],
    /// Per-column width in SVG units (must sum to `WIDTH - 2 * PAD`).
    pub col_widths: &'a [u32],
    /// Per-column horizontal alignment (`"start"` or `"end"`).
    pub col_align: &'a [&'a str],
    /// Per-cell font family for body cells, indexed by column.
    /// `None` → SANS for everything.
    pub col_font: Option<&'a [Family]>,
}

#[derive(Copy, Clone)]
pub enum Family {
    Sans,
    Mono,
}

const WIDTH: u32 = 880;
const PAD: u32 = 24;
const HEADER_H: u32 = 36;
const ROW_H: u32 = 30;
const RADIUS: f32 = 12.0;
const FONT_SIZE: u32 = 13;
const HEADER_FONT_SIZE: u32 = 13;

pub fn regen_all(root: &Path, theme: Theme) -> Result<()> {
    let dir = root.join("assets/tables");
    std::fs::create_dir_all(&dir)?;

    for table in [
        install::table(),
        capabilities::table(),
        backends::table(),
        translation::table(),
        comparison::table(),
    ] {
        let svg = render_table(&table, theme);
        let out = dir.join(format!("{}-{}.svg", table.stem, theme.suffix()));
        write_atomic(&out, &svg)?;
        println!("wrote {} ({} bytes)", out.display(), svg.len());
    }
    Ok(())
}

fn render_table(table: &Table, theme: Theme) -> String {
    let p = palette(theme);
    assert_eq!(table.col_widths.len(), table.header.len());
    assert_eq!(table.col_align.len(), table.header.len());
    if let Some(fonts) = table.col_font {
        assert_eq!(fonts.len(), table.header.len());
    }
    let total_cols_w: u32 = table.col_widths.iter().sum();
    assert!(total_cols_w + 2 * PAD <= WIDTH, "table {} columns overflow", table.stem);

    let body_h = HEADER_H + ROW_H * (table.rows.len() as u32);
    let h = body_h + 2 * PAD;

    let mut out = header(WIDTH, h, table.title, table.title);

    // Outer rounded card.
    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{w}" height="{hh}" rx="{r}" fill="{bg}" stroke="{s}" stroke-width="1"/>
"#,
        w = WIDTH - 1,
        hh = h - 1,
        r = RADIUS,
        bg = p.bg,
        s = p.border,
    ));

    let body_x = PAD;
    let body_y = PAD;

    // Header row background.
    out.push_str(&format!(
        r#"  <rect x="{x}" y="{y}" width="{w}" height="{hh}" fill="{c}"/>
"#,
        x = body_x,
        y = body_y,
        w = WIDTH - 2 * PAD,
        hh = HEADER_H,
        c = p.card,
    ));

    // Header cell text + col separators.
    let mut x_cursor = body_x;
    for (i, &label) in table.header.iter().enumerate() {
        let col_w = table.col_widths[i];
        let align = table.col_align[i];
        let (tx, anchor) = text_anchor(x_cursor, col_w, align);
        out.push_str(&format!(
            r#"  <text x="{tx}" y="{ty}" font-family="{f}" font-weight="700" font-size="{sz}" fill="{c}" text-anchor="{anchor}">{label}</text>
"#,
            ty = body_y + 22,
            f = SANS,
            sz = HEADER_FONT_SIZE,
            c = p.text,
            label = escape(label),
        ));
        x_cursor += col_w;
    }

    // Header bottom rule.
    out.push_str(&format!(
        r#"  <line x1="{x1}" y1="{y}" x2="{x2}" y2="{y}" stroke="{c}" stroke-width="1"/>
"#,
        x1 = body_x,
        x2 = body_x + (WIDTH - 2 * PAD),
        y = body_y + HEADER_H,
        c = p.border,
    ));

    // Body rows.
    for (row_idx, row) in table.rows.iter().enumerate() {
        let row_y = body_y + HEADER_H + ROW_H * (row_idx as u32);

        // Alternating row banding for legibility.
        if row_idx % 2 == 1 {
            out.push_str(&format!(
                r#"  <rect x="{x}" y="{y}" width="{w}" height="{hh}" fill="{c}"/>
"#,
                x = body_x,
                y = row_y,
                w = WIDTH - 2 * PAD,
                hh = ROW_H,
                c = p.card,
            ));
        }

        let mut x_cursor = body_x;
        for (col_idx, &cell) in row.iter().enumerate() {
            let col_w = table.col_widths[col_idx];
            let align = table.col_align[col_idx];
            let (tx, anchor) = text_anchor(x_cursor, col_w, align);

            let family = match table.col_font.and_then(|fs| fs.get(col_idx)) {
                Some(Family::Mono) => MONO_FAMILY,
                _ => SANS,
            };

            out.push_str(&format!(
                r#"  <text x="{tx}" y="{ty}" font-family="{family}" font-size="{sz}" fill="{c}" text-anchor="{anchor}">{txt}</text>
"#,
                ty = row_y + 20,
                sz = FONT_SIZE,
                c = p.text,
                txt = escape(cell),
            ));
            x_cursor += col_w;
        }
    }

    out.push_str("</svg>\n");
    out
}

fn text_anchor(col_x: u32, col_w: u32, align: &str) -> (u32, &str) {
    match align {
        "end" => (col_x + col_w - 8, "end"),
        _ => (col_x + 8, "start"),
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
