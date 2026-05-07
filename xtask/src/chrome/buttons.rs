//! `assets/buttons/<name>-{light,dark}.svg` — pure-SVG link buttons.
//!
//! Five buttons rendered as separate SVGs with identical 184×40 dimensions
//! so the README can scale them uniformly with `width="19%"`. Pixel-perfect
//! uniform heights are guaranteed by construction (identical viewBox aspect
//! across all five). Fully under our control — no shields.io aspect-ratio
//! drift, no GitHub cell-border bleed-through.
//!
//! Live data: version + license read from the workspace `Cargo.toml`,
//! coverage % read from `assets/status/coverage.json` if present. CI and
//! docs.rs default to "passing" (the typical state — refreshed by the
//! daily chrome cron when it isn't).

use anyhow::{Result, anyhow};
use std::path::Path;

use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::svg::{header, write_atomic};

const W: u32 = 184;
const H: u32 = 40;
const RADIUS: f32 = 4.0;

/// Each button has a label half (dark, mono uppercase tag) and a value half
/// (slightly different shade, sans bold). Label width is fixed so the seam
/// between halves is consistent across all five buttons.
const LABEL_W: u32 = 88;

struct Button<'a> {
    stem: &'a str,
    label: &'a str,
    value: &'a str,
}

pub fn regen_all(root: &Path, theme: Theme) -> Result<()> {
    let meta = read_meta(root)?;
    let coverage = read_coverage(root);

    let cov_label = coverage.map_or_else(|| "n/a".to_string(), |c| format!("{c:.0}%"));
    let version_label = format!("v{}", meta.version);

    let buttons = [
        Button {
            stem: "crates",
            label: "CRATES.IO",
            value: &version_label,
        },
        Button {
            stem: "docs",
            label: "DOCS.RS",
            value: "passing",
        },
        Button {
            stem: "codecov",
            label: "CODECOV",
            value: &cov_label,
        },
        Button {
            stem: "ci",
            label: "CI",
            value: "passing",
        },
        Button {
            stem: "license",
            label: "LICENSE",
            value: &meta.license,
        },
    ];

    let dir = root.join("assets/buttons");
    std::fs::create_dir_all(&dir)?;

    for b in &buttons {
        let svg = render(theme, b);
        let out = dir.join(format!("{}-{}.svg", b.stem, theme.suffix()));
        write_atomic(&out, &svg)?;
        println!("wrote {} ({} bytes)", out.display(), svg.len());
    }
    Ok(())
}

struct Meta {
    version: String,
    license: String,
}

fn read_meta(root: &Path) -> Result<Meta> {
    let cargo = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let toml: toml::Value = toml::from_str(&cargo)?;
    let pkg = toml
        .get("workspace")
        .and_then(|w| w.get("package"))
        .ok_or_else(|| anyhow!("no [workspace.package]"))?;
    let s = |k: &str, d: &str| pkg.get(k).and_then(|v| v.as_str()).unwrap_or(d).to_string();
    Ok(Meta {
        version: s("version", "0.0.0"),
        license: s("license", "GPL-3.0"),
    })
}

fn read_coverage(root: &Path) -> Option<f32> {
    let path = root.join("assets/status/coverage.json");
    let raw = std::fs::read_to_string(&path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    v.get("coverage")
        .and_then(serde_json::Value::as_f64)
        .map(|n| n as f32)
}

fn render(theme: Theme, b: &Button) -> String {
    let _ = palette(theme);

    // Two-tone monochrome per theme. Light variant is light-on-light so it
    // sits naturally on white READMEs; dark variant uses near-black halves
    // for dark-mode readers. Picking high-contrast text colours so the
    // value reads as the primary content.
    let (label_bg, value_bg, label_fg, value_fg, border) = match theme {
        Theme::Light => ("#ebedef", "#ffffff", "#555555", "#1a1a1a", "#cccccc"),
        Theme::Dark => ("#0e0e10", "#1f1f23", "#a0a0a8", "#ffffff", "#2c2c33"),
    };

    let mut out = header(
        W,
        H,
        &format!("{} — {}", b.label, b.value),
        &format!("{} — {}", b.label, b.value),
    );

    // Outer rounded background filled with the label-half colour.
    out.push_str(&format!(
        r#"  <rect x="0.5" y="0.5" width="{w}" height="{hh}" rx="{r}" fill="{label_bg}" stroke="{s}" stroke-width="1"/>
"#,
        w = W - 1,
        hh = H - 1,
        r = RADIUS,
        s = border,
    ));

    // Value-half polygon — straight left edge at LABEL_W, rounded right edge
    // matching the outer corner radius.
    out.push_str(&format!(
        r#"  <path d="M{lw} 1 H{rx} A{r} {r} 0 0 1 {w} {r} V{vh} A{r} {r} 0 0 1 {rx} {h} H{lw} Z" fill="{value_bg}"/>
"#,
        lw = LABEL_W,
        rx = W - 1 - RADIUS as u32,
        r = RADIUS as u32,
        w = W - 1,
        vh = H - 1 - RADIUS as u32,
        h = H - 1,
    ));

    // Label text — mono uppercase, vertically centred via dominant-baseline.
    out.push_str(&format!(
        r#"  <text x="{lx}" y="{cy}" font-family="{f}" font-weight="700" font-size="11" fill="{c}" letter-spacing="0.6" text-anchor="middle" dominant-baseline="central">{label}</text>
"#,
        lx = LABEL_W / 2,
        cy = H / 2,
        f = MONO_FAMILY,
        c = label_fg,
        label = escape(b.label),
    ));

    // Value text — sans bold, vertically centred.
    out.push_str(&format!(
        r#"  <text x="{vx}" y="{cy}" font-family="{f}" font-weight="700" font-size="13" fill="{c}" text-anchor="middle" dominant-baseline="central">{value}</text>
"#,
        vx = LABEL_W + (W - LABEL_W) / 2,
        cy = H / 2,
        f = SANS,
        c = value_fg,
        value = escape(b.value),
    ));

    out.push_str("</svg>\n");
    out
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
