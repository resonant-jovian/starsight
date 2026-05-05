//! `assets/architecture-light.svg` — 7-layer stack diagram (static).
//!
//! No live data, but regenerated through xtask so the visual style stays in
//! lockstep with the rest of the chrome assets.

use anyhow::Result;
use std::path::Path;

use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::svg::{header, write_atomic};

const W: u32 = 880;

const LAYERS: &[(&str, &str, &str, Status)] = &[
    (
        "L7",
        "export",
        "interchange · data sources · pdf",
        Status::Planned,
    ),
    (
        "L6",
        "interactive",
        "input · animation · windowing",
        Status::Planned,
    ),
    (
        "L5",
        "common",
        "figures · plot! · render helpers",
        Status::Shipped,
    ),
    (
        "L4",
        "composition",
        "layouts · faceting · legends · colorbars",
        Status::Shipped,
    ),
    (
        "L3",
        "components",
        "marks · stats · aesthetics · adjustments",
        Status::Shipped,
    ),
    (
        "L2",
        "modifiers",
        "scales · ticks · axes · coords",
        Status::Shipped,
    ),
    (
        "L1",
        "background",
        "primitives · errors · drawing · backends",
        Status::Shipped,
    ),
];

#[derive(Copy, Clone)]
enum Status {
    Shipped,
    Planned,
}

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let svg = render(theme);
    let out = root.join(format!("assets/architecture-{}.svg", theme.suffix()));
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());
    Ok(())
}

fn render(theme: Theme) -> String {
    let p = palette(theme);
    let pad: f32 = 24.0;
    let row_h: f32 = 56.0;
    let header_h: f32 = 60.0;
    let footer_h: f32 = 50.0;
    let h: u32 = (header_h + LAYERS.len() as f32 * row_h + footer_h + pad * 2.0) as u32;

    let mut out = header(
        W,
        h,
        "starsight architecture · 7 layers · facade re-exports · L_n may depend only on L_1..L_{n-1}",
        "starsight architecture",
    );
    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{}" height="{}" rx="8" fill="{}" stroke="{}" stroke-width="1"/>
"#,
        W - 1,
        h - 1,
        p.card,
        p.border
    ));

    // eyebrow
    out.push_str(&format!(
        r#"  <text x="24" y="26" font-family="{f}" font-size="11" fill="{c}" letter-spacing="0.6">// architecture · facade re-exports the seven layers</text>
"#,
        f = MONO_FAMILY,
        c = p.muted
    ));

    // facade strip
    let facade_y: f32 = pad + 12.0;
    out.push_str(&format!(
        r#"  <rect x="{x}" y="{y:.1}" width="{w}" height="36" rx="4" fill="{c}" stroke="{s}" stroke-width="1"/>
"#,
        x = pad,
        y = facade_y + 14.0,
        w = (W as f32) - 2.0 * pad,
        c = p.bg,
        s = p.text
    ));
    out.push_str(&format!(
        r#"  <text x="{cx:.1}" y="{y:.1}" font-family="{f}" font-weight="700" font-size="14" fill="{c}" text-anchor="middle">starsight (facade) — prelude · semantic modules · latin layer aliases</text>
"#,
        cx = (W as f32) / 2.0,
        y = facade_y + 38.0,
        f = SANS,
        c = p.text
    ));

    // 7 layer rows
    let rows_y0: f32 = facade_y + 60.0;
    for (i, (id, name, desc, status)) in LAYERS.iter().enumerate() {
        let y = rows_y0 + i as f32 * row_h;
        // row card
        out.push_str(&format!(
            r#"  <rect x="{x}" y="{y:.1}" width="{w}" height="{rh}" rx="4" fill="{c}" stroke="{s}" stroke-width="1"/>
"#,
            x = pad,
            w = (W as f32) - 2.0 * pad,
            rh = row_h - 8.0,
            c = p.card,
            s = p.border
        ));
        // L7/L6/etc id
        out.push_str(&format!(
            r#"  <text x="{x:.1}" y="{ty:.1}" font-family="{f}" font-weight="700" font-size="13" fill="{c}">{id}</text>
"#,
            x = pad + 14.0,
            ty = y + 22.0,
            f = SANS,
            c = p.text
        ));
        // name
        out.push_str(&format!(
            r#"  <text x="{x:.1}" y="{ty:.1}" font-family="{f}" font-weight="700" font-size="13" fill="{c}">{name}</text>
"#,
            x = pad + 60.0,
            ty = y + 22.0,
            f = SANS,
            c = p.text
        ));
        // desc
        out.push_str(&format!(
            r#"  <text x="{x:.1}" y="{ty:.1}" font-family="{f}" font-size="11" fill="{c}">{desc}</text>
"#,
            x = pad + 60.0,
            ty = y + 38.0,
            f = MONO_FAMILY,
            c = p.subtext
        ));
        // status pill on the right
        let pill_x: f32 = (W as f32) - pad - 90.0;
        let pill_y: f32 = y + 12.0;
        let (text, fill, stroke) = match status {
            Status::Shipped => ("shipped", p.text, p.text),
            Status::Planned => ("planned", p.bg, p.muted),
        };
        out.push_str(&format!(
            r#"  <rect x="{px:.1}" y="{py:.1}" width="80" height="22" rx="11" fill="{f}" stroke="{s}" stroke-width="1"/>
"#,
            px = pill_x,
            py = pill_y,
            f = fill,
            s = stroke
        ));
        let pill_text_color = match status {
            Status::Shipped => p.bg,
            _ => p.subtext,
        };
        out.push_str(&format!(
            r#"  <text x="{tx:.1}" y="{ty:.1}" font-family="{f}" font-weight="700" font-size="10" fill="{c}" text-anchor="middle" letter-spacing="0.4">{text}</text>
"#,
            tx = pill_x + 40.0,
            ty = pill_y + 15.0,
            f = MONO_FAMILY,
            c = pill_text_color
        ));
    }

    // footer rule
    let footer_y: f32 = rows_y0 + LAYERS.len() as f32 * row_h + 8.0;
    out.push_str(&format!(
        r#"  <text x="{cx:.1}" y="{y:.1}" font-family="{f}" font-size="11" fill="{c}" text-anchor="middle">layer N may depend only on layers N-1 through 1 · enforced at workspace Cargo.toml level</text>
"#,
        cx = (W as f32) / 2.0,
        y = footer_y + 24.0,
        f = MONO_FAMILY,
        c = p.muted
    ));

    out.push_str("</svg>\n");
    out
}
