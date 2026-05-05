//! `assets/roadmap-light.svg` — horizontal 0.1 → 1.0 timeline.
//!
//! Status is encoded by *shape* (no color):
//! - shipped : filled black disc
//! - current : filled disc with outer ring (large dot)
//! - planned : hollow ring
//!
//! Which milestone is "current" comes from `[workspace.package].version` in
//! the root `Cargo.toml` — anything ≤ that minor is shipped, equal is current,
//! greater is planned.

use anyhow::Result;
use std::path::Path;

use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::svg::{header, write_atomic};

const STOPS: &[(&str, &str)] = &[
    ("0.1", "found"),
    ("0.2", "charts"),
    ("0.3", "stats"),
    ("0.4", "layout"),
    ("0.5", "scales"),
    ("0.6", "gpu"),
    ("0.7", "anim"),
    ("0.8", "term"),
    ("0.9", "3d"),
    ("0.10", "export"),
    ("0.11", "data"),
    ("0.12", "polish"),
    ("1.0", "stable"),
];

const W: u32 = 880;
const H: u32 = 200;

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let current = read_current_minor(root).unwrap_or_else(|_| "0.3".to_string());
    let svg = render(&current, theme);
    let out = root.join(format!("assets/roadmap-{}.svg", theme.suffix()));
    write_atomic(&out, &svg)?;
    println!("wrote {} ({} bytes)", out.display(), svg.len());
    Ok(())
}

fn render(current: &str, theme: Theme) -> String {
    let p = palette(theme);
    let mut out = header(
        W,
        H,
        &format!("starsight roadmap · 0.1 → 1.0 · current {current}"),
        "starsight roadmap",
    );
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
        r#"  <text x="24" y="26" font-family="{f}" font-size="11" fill="{c}" letter-spacing="0.6">// roadmap · 0.x → 1.0</text>
"#,
        f = MONO_FAMILY,
        c = p.muted
    ));

    // axis line
    let pad_x: f32 = 36.0;
    let axis_y: f32 = 100.0;
    let n = STOPS.len();
    out.push_str(&format!(
        r#"  <line x1="{l:.1}" y1="{y}" x2="{r:.1}" y2="{y}" stroke="{c}" stroke-width="1"/>
"#,
        l = pad_x,
        r = (W as f32) - pad_x,
        y = axis_y,
        c = p.border
    ));

    // markers
    let span = (W as f32) - 2.0 * pad_x;
    for (i, (label, caption)) in STOPS.iter().enumerate() {
        let cx = pad_x + span * (i as f32) / ((n - 1) as f32);
        let state = compare_minor(label, current);

        match state {
            State::Shipped => {
                out.push_str(&format!(
                    r#"  <circle cx="{cx:.1}" cy="{y}" r="5" fill="{c}"/>
"#,
                    y = axis_y,
                    c = p.text
                ));
            }
            State::Current => {
                out.push_str(&format!(
                    r#"  <circle cx="{cx:.1}" cy="{y}" r="9" fill="none" stroke="{c}" stroke-width="1.5"/>
  <circle cx="{cx:.1}" cy="{y}" r="5" fill="{c}"/>
"#,
                    y = axis_y,
                    c = p.text
                ));
            }
            State::Planned => {
                out.push_str(&format!(
                    r#"  <circle cx="{cx:.1}" cy="{y}" r="5" fill="{bg}" stroke="{c}" stroke-width="1.2"/>
"#,
                    y = axis_y,
                    bg = p.card,
                    c = p.muted
                ));
            }
        }

        let label_color = match state {
            State::Shipped | State::Current => p.text,
            State::Planned => p.muted,
        };
        let caption_color = match state {
            State::Current => p.subtext,
            _ => p.muted,
        };

        // label above, caption below
        out.push_str(&format!(
            r#"  <text x="{cx:.1}" y="{y}" font-family="{f}" font-weight="700" font-size="13" fill="{c}" text-anchor="middle">{label}</text>
"#,
            y = axis_y - 22.0,
            f = SANS,
            c = label_color
        ));
        out.push_str(&format!(
            r#"  <text x="{cx:.1}" y="{y}" font-family="{f}" font-size="10" fill="{c}" text-anchor="middle">{caption}</text>
"#,
            y = axis_y + 26.0,
            f = MONO_FAMILY,
            c = caption_color
        ));
    }

    // legend
    let lx: f32 = 24.0;
    let ly: f32 = (H as f32) - 26.0;
    let items = [
        (lx + 6.0, "shipped", State::Shipped),
        (lx + 110.0, "current", State::Current),
        (lx + 215.0, "planned", State::Planned),
    ];
    for (x, label, state) in items {
        match state {
            State::Shipped => out.push_str(&format!(
                r#"  <circle cx="{x:.1}" cy="{ly:.1}" r="4" fill="{c}"/>
"#,
                c = p.text
            )),
            State::Current => out.push_str(&format!(
                r#"  <circle cx="{x:.1}" cy="{ly:.1}" r="7" fill="none" stroke="{c}" stroke-width="1.2"/>
  <circle cx="{x:.1}" cy="{ly:.1}" r="3.5" fill="{c}"/>
"#,
                c = p.text
            )),
            State::Planned => out.push_str(&format!(
                r#"  <circle cx="{x:.1}" cy="{ly:.1}" r="4" fill="{bg}" stroke="{c}" stroke-width="1"/>
"#,
                bg = p.card,
                c = p.muted
            )),
        }
        out.push_str(&format!(
            r#"  <text x="{lx:.1}" y="{ty:.1}" font-family="{f}" font-size="11" fill="{c}">{label}</text>
"#,
            lx = x + 10.0,
            ty = ly + 4.0,
            f = MONO_FAMILY,
            c = p.subtext
        ));
    }

    out.push_str("</svg>\n");
    out
}

#[derive(Copy, Clone)]
enum State {
    Shipped,
    Current,
    Planned,
}

fn compare_minor(stop: &str, current: &str) -> State {
    let a = parse_minor(stop);
    let b = parse_minor(current);
    match a.cmp(&b) {
        std::cmp::Ordering::Less => State::Shipped,
        std::cmp::Ordering::Equal => State::Current,
        std::cmp::Ordering::Greater => State::Planned,
    }
}

fn parse_minor(s: &str) -> (u32, u32) {
    let mut parts = s.split('.');
    let major: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
    let minor: u32 = parts.next().unwrap_or("0").parse().unwrap_or(0);
    (major, minor)
}

fn read_current_minor(root: &Path) -> Result<String> {
    let cargo = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let toml: toml::Value = toml::from_str(&cargo)?;
    let v = toml
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("no [workspace.package].version"))?;
    let mut parts = v.split('.');
    let maj = parts.next().unwrap_or("0");
    let min = parts.next().unwrap_or("0");
    Ok(format!("{maj}.{min}"))
}
