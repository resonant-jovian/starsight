//! Pure-SVG renderings of the README's GFM tables — flag/feature, capabilities,
//! backends, language translation, and library comparison.
//!
//! Each table is an SVG composed from a shared primitive (`render_table`) so
//! the styling, theming, and bordered card chrome stays consistent across the
//! README. Cells word-wrap so long descriptions don't overflow into adjacent
//! columns; row height is computed per row from the maximum line count of
//! its cells. Font metrics are approximated from `font-size` (no live font
//! measurement) — sans glyphs estimated at 0.55× and mono at 0.62× of the
//! font size, which is conservative enough to leave a small right-margin
//! cushion across the system fallback chain.
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
    /// Per-column width in SVG units (must sum to ≤ `WIDTH - 2 * PAD`).
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

const WIDTH: u32 = 920;
const PAD: u32 = 24;
const HEADER_H: u32 = 36;
const RADIUS: f32 = 12.0;
const FONT_SIZE: f32 = 12.5;
const HEADER_FONT_SIZE: f32 = 12.5;
const LINE_H: f32 = 16.5;
const VERT_PAD: f32 = 6.0;
const CELL_PAD_X: u32 = 8;

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
    assert!(
        total_cols_w + 2 * PAD <= WIDTH,
        "table {} columns overflow ({total_cols_w} + 2*{PAD} > {WIDTH})",
        table.stem,
    );

    // Pre-wrap every cell so we know each row's height before laying it out.
    let wrapped: Vec<Vec<Vec<String>>> = table
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(col_idx, &cell)| {
                    let col_w = table.col_widths[col_idx];
                    let max_text_w = col_w.saturating_sub(2 * CELL_PAD_X);
                    let family = column_family(table, col_idx);
                    wrap_text(cell, max_text_w, family, FONT_SIZE)
                })
                .collect()
        })
        .collect();

    // Each row's height = (max line count) * LINE_H + 2 * VERT_PAD.
    let row_heights: Vec<f32> = wrapped
        .iter()
        .map(|row_cells| {
            let max_lines = row_cells.iter().map(Vec::len).max().unwrap_or(1).max(1);
            (max_lines as f32) * LINE_H + 2.0 * VERT_PAD
        })
        .collect();

    let body_h: f32 = (HEADER_H as f32) + row_heights.iter().sum::<f32>();
    let h = (body_h + 2.0 * (PAD as f32)).ceil() as u32;

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

    // Header cell text.
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
    let mut row_y_cursor: f32 = (body_y + HEADER_H) as f32;
    for (row_idx, row_cells) in wrapped.iter().enumerate() {
        let row_h = row_heights[row_idx];

        if row_idx % 2 == 1 {
            out.push_str(&format!(
                r#"  <rect x="{x}" y="{y:.1}" width="{w}" height="{rh:.1}" fill="{c}"/>
"#,
                x = body_x,
                y = row_y_cursor,
                w = WIDTH - 2 * PAD,
                rh = row_h,
                c = p.card,
            ));
        }

        let mut x_cursor = body_x;
        for (col_idx, lines) in row_cells.iter().enumerate() {
            let col_w = table.col_widths[col_idx];
            let align = table.col_align[col_idx];
            let (tx, anchor) = text_anchor(x_cursor, col_w, align);
            let family = match column_family(table, col_idx) {
                Family::Mono => MONO_FAMILY,
                Family::Sans => SANS,
            };

            // First line baseline; subsequent lines via dx="0" dy=LINE_H tspans.
            let first_y = row_y_cursor + VERT_PAD + LINE_H - 4.0;
            let mut tspans = String::new();
            for (line_idx, line) in lines.iter().enumerate() {
                if line_idx == 0 {
                    tspans.push_str(&escape(line));
                } else {
                    tspans.push_str(&format!(
                        r#"<tspan x="{tx}" dy="{LINE_H}">{txt}</tspan>"#,
                        txt = escape(line),
                    ));
                }
            }

            out.push_str(&format!(
                r#"  <text x="{tx}" y="{first_y:.1}" font-family="{family}" font-size="{sz}" fill="{c}" text-anchor="{anchor}">{tspans}</text>
"#,
                sz = FONT_SIZE,
                c = p.text,
            ));
            x_cursor += col_w;
        }
        row_y_cursor += row_h;
    }

    out.push_str("</svg>\n");
    out
}

fn column_family(table: &Table, col_idx: usize) -> Family {
    table
        .col_font
        .and_then(|fs| fs.get(col_idx).copied())
        .unwrap_or(Family::Sans)
}

fn text_anchor(col_x: u32, col_w: u32, align: &str) -> (u32, &str) {
    match align {
        "end" => (col_x + col_w - CELL_PAD_X, "end"),
        _ => (col_x + CELL_PAD_X, "start"),
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Approximate per-character width in SVG units. Sans glyphs run ≈55% of the
/// font size on average across the system fallback chain (Segoe UI, Roboto,
/// Helvetica). Monospace runs ≈62%. Both numbers leave a small cushion so
/// borderline cells don't clip on reader-side font substitution.
fn char_width(family: Family, font_size: f32) -> f32 {
    match family {
        Family::Sans => font_size * 0.55,
        Family::Mono => font_size * 0.62,
    }
}

/// Greedy word-wrap to fit `max_width` SVG units. Keeps existing whitespace.
/// Words that exceed the column on their own (long type names, URLs) are not
/// hyphenated — they sit on their own line and may overflow slightly; in
/// practice all our columns are wide enough that this never triggers.
fn wrap_text(s: &str, max_width: u32, family: Family, font_size: f32) -> Vec<String> {
    let cw = char_width(family, font_size);
    if cw <= 0.0 {
        return vec![s.to_string()];
    }
    let max_chars = ((max_width as f32) / cw).floor().max(1.0) as usize;

    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in s.split(' ') {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }
        // +1 for the space we'd insert.
        if current.chars().count() + 1 + word.chars().count() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
