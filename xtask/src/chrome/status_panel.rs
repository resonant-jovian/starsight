//! `assets/status/panel-light.svg` — live crates.io status panel.
//!
//! Layout (880×148):
//! - eyebrow `// status · live from crates.io api` (mono 11)
//! - 4-column fact strip: CURRENT · MSRV · LICENSE · EDITION
//! - hairline rule
//! - 30-point sparkline (real per-day downloads) + caption + activity line
//!
//! On API failure: returns Ok(()) without writing — the existing panel stays.

use anyhow::Result;
use std::path::Path;

use super::crates_io;
use super::palette::{MONO, MONO_FAMILY, SANS};
use super::svg::{header, write_atomic};

const W: u32 = 880;
const H: u32 = 148;
const PAD_X: f32 = 24.0;

pub fn regen(root: &Path) -> Result<()> {
    let stats = match crates_io::fetch() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("status_panel: skipping — crates.io fetch failed: {e}");
            return Ok(());
        }
    };
    let svg = render(&stats);
    let out = root.join("assets/status/panel-light.svg");
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());
    Ok(())
}

fn render(s: &crates_io::Stats) -> String {
    let p = &MONO;
    let mut out = header(
        W,
        H,
        &format!(
            "starsight live status · v{} · rust {} · {}",
            s.version, s.msrv, s.license
        ),
        "starsight status panel",
    );

    // outer card
    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{}" height="{}" rx="8" fill="{}" stroke="{}" stroke-width="1"/>
"#,
        W - 1,
        H - 1,
        p.card,
        p.border
    ));

    // eyebrow
    out.push_str(&format!(
        r#"  <text x="{x}" y="22" font-family="{f}" font-size="11" fill="{c}" letter-spacing="0.6">// status · live from crates.io api</text>
"#,
        x = PAD_X,
        f = MONO_FAMILY,
        c = p.muted
    ));

    // 4 fact columns
    let facts = [
        ("CURRENT", format!("v{}", s.version)),
        ("MSRV", format!("rust {}", s.msrv)),
        ("LICENSE", s.license.clone()),
        ("EDITION", s.edition.clone()),
    ];
    let inner = (W as f32) - 2.0 * PAD_X;
    let col_w = inner / facts.len() as f32;
    for (i, (label, value)) in facts.iter().enumerate() {
        let cx = PAD_X + col_w * i as f32 + 12.0;
        out.push_str(&format!(
            r#"  <text x="{cx:.1}" y="50" font-family="{f}" font-size="10" fill="{c}" letter-spacing="0.4">{label}</text>
"#,
            f = MONO_FAMILY,
            c = p.muted
        ));
        out.push_str(&format!(
            r#"  <text x="{cx:.1}" y="70" font-family="{f}" font-weight="700" font-size="16" fill="{c}">{value}</text>
"#,
            f = SANS,
            c = p.text
        ));
    }

    // rule between facts and sparkline
    out.push_str(&format!(
        r#"  <line x1="{l}" y1="88" x2="{r}" y2="88" stroke="{c}" stroke-width="0.8"/>
"#,
        l = PAD_X,
        r = (W as f32) - PAD_X,
        c = p.rule
    ));

    // sparkline (left)
    let spark_x0: f32 = PAD_X;
    let spark_y_mid: f32 = 110.0;
    let spark_w: f32 = 200.0;
    let spark_h: f32 = 26.0;
    let series = &s.downloads_30d;
    if series.len() > 1 {
        let max_v = (*series.iter().max().unwrap_or(&1)).max(1) as f32;

        // background hairlines (3 inner gridlines)
        for v in [0.25_f32, 0.5, 0.75] {
            let yy = spark_y_mid + spark_h / 2.0 - v * spark_h;
            out.push_str(&format!(
                r#"  <line x1="{x0}" y1="{yy:.1}" x2="{x1}" y2="{yy:.1}" stroke="{c}" stroke-width="0.5"/>
"#,
                x0 = spark_x0,
                x1 = spark_x0 + spark_w,
                c = p.rule
            ));
        }

        let mut pts = String::new();
        for (i, v) in series.iter().enumerate() {
            let x = spark_x0 + i as f32 * spark_w / (series.len() - 1) as f32;
            let y = spark_y_mid + spark_h / 2.0 - (*v as f32 / max_v) * spark_h;
            if i > 0 {
                pts.push(' ');
            }
            pts.push_str(&format!("{x:.1},{y:.1}"));
        }
        out.push_str(&format!(
            r#"  <polyline points="{pts}" fill="none" stroke="{c}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
"#,
            c = p.text
        ));

        out.push_str(&format!(
            r#"  <text x="{x}" y="{y:.1}" font-family="{f}" font-size="10" fill="{c}">last 30d downloads · {sum} total</text>
"#,
            x = spark_x0,
            y = spark_y_mid + spark_h / 2.0 + 14.0,
            f = MONO_FAMILY,
            c = p.muted,
            sum = s.downloads_30d_total
        ));
    }

    // activity text (right)
    let activity = format!(
        "{tot} downloads since {since}  ·  {dep} dependents  ·  updated {d}d ago",
        tot = s.downloads_lifetime,
        since = s.first_publish,
        dep = s.dependents,
        d = s.updated_days_ago
    );
    out.push_str(&format!(
        r#"  <text x="{x}" y="{y:.1}" font-family="{f}" font-size="13" fill="{c}" text-anchor="end">{activity}</text>
"#,
        x = (W as f32) - PAD_X,
        y = spark_y_mid + 4.0,
        f = SANS,
        c = p.subtext
    ));

    out.push_str("</svg>\n");
    out
}
