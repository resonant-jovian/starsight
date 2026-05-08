//! `assets/social/card-{light,dark}.{svg,png}` — 1280×640 social / open-graph card.
//!
//! Both formats:
//! - **SVG** is the canonical output and what the README references — zooms cleanly.
//! - **PNG** is kept as the OG / Twitter / Slack unfurl fallback (those surfaces
//!   only render raster meta-images).
//!
//! GitHub's recommended OG template uses a 40-pt safe margin around important
//! content; we use ~96px (≈75pt at 1280×640) for cushion. The card is the
//! wordmark lockup centered, with a one-line tagline below and a lower meta
//! strip pulled from `Cargo.toml`.

use anyhow::{Context, Result, anyhow};
use std::path::Path;
use tiny_skia::{Pixmap, Transform};

use super::eclipse;
use super::palette::{MONO_FAMILY, SANS, Theme, palette};
use super::svg::write_atomic;

const W: u32 = 1280;
const H: u32 = 640;
const RADIUS: f32 = 24.0;
const SAFE: u32 = 96;

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let meta = read_meta(root)?;
    let svg = render_svg(theme, &meta);

    let out_dir = root.join("assets/social");
    std::fs::create_dir_all(&out_dir)?;

    let svg_path = out_dir.join(format!("card-{}.svg", theme.suffix()));
    write_atomic(&svg_path, &svg)?;
    println!("wrote {} ({} bytes)", svg_path.display(), svg.len());

    let png_path = out_dir.join(format!("card-{}.png", theme.suffix()));
    let png = rasterize(&svg, W, H)?;
    png.save_png(&png_path)
        .map_err(|e| anyhow!("write social card png: {e}"))?;
    println!(
        "wrote {} ({} bytes, {}×{})",
        png_path.display(),
        std::fs::metadata(&png_path)?.len(),
        W,
        H,
    );
    Ok(())
}

struct Meta {
    version: String,
    msrv: String,
    license: String,
    edition: String,
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
        msrv: s("rust-version", "1.89"),
        license: s("license", "GPL-3.0-only"),
        edition: s("edition", "2024"),
    })
}

fn render_svg(theme: Theme, meta: &Meta) -> String {
    let p = palette(theme);
    let eclipse_size: u32 = 240;
    let center_y: u32 = H / 2 - 20;
    let ex = SAFE;
    let ey = center_y - eclipse_size / 2;

    let wm_x = ex + eclipse_size + 56;
    let wm_y = center_y + 36;
    let tag_y = center_y + 96;

    let meta_text = format!(
        "v{}  ·  rust {}  ·  edition {}  ·  {}",
        meta.version, meta.msrv, meta.edition, meta.license
    );

    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}" width="100%" height="auto" role="img" aria-label="starsight social card" preserveAspectRatio="xMidYMid meet">
  <title>starsight</title>
  <rect x="0.5" y="0.5" width="{w}" height="{h}" rx="{r}" fill="{bg}" stroke="{s}" stroke-width="1"/>
  <g transform="translate({ex},{ey}) scale({sc})">
{eclipse_inner}  </g>
  <text x="{wm_x}" y="{wm_y}" font-family="{sans}" font-weight="700" font-size="156" fill="{text}" letter-spacing="-4">starsight</text>
  <text x="{wm_x}" y="{tag_y}" font-family="{sans}" font-size="26" fill="{sub}">scientific visualization for Rust — typed, layered, eight backends</text>
  <text x="{meta_x}" y="{meta_y}" font-family="{mono}" font-size="20" fill="{muted}">{meta}</text>
  <text x="{repo_x}" y="{meta_y}" font-family="{mono}" font-size="20" fill="{muted}" text-anchor="end">github.com/resonant-jovian/starsight</text>
</svg>
"#,
        W = W,
        H = H,
        w = W - 1,
        h = H - 1,
        r = RADIUS,
        bg = p.bg,
        s = p.border,
        ex = ex,
        ey = ey,
        sc = eclipse_size as f32 / 100.0,
        eclipse_inner = eclipse::svg_inner(p),
        wm_x = wm_x,
        wm_y = wm_y,
        tag_y = tag_y,
        meta_x = SAFE,
        meta_y = H - SAFE / 2,
        repo_x = W - SAFE,
        sans = SANS,
        mono = MONO_FAMILY,
        text = p.text,
        sub = p.subtext,
        muted = p.muted,
        meta = meta_text,
    )
}

fn rasterize(svg: &str, w: u32, h: u32) -> Result<Pixmap> {
    let mut opts = usvg::Options::default();
    // Bundled DejaVu first so the wordmark / tagline / meta strip survive on
    // CI runners that lack the Apple/Segoe family names that lead `SANS`.
    super::fonts::load_into(&mut opts);
    opts.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_str(svg, &opts).context("parse social card svg")?;
    let mut pix = Pixmap::new(w, h).ok_or_else(|| anyhow!("alloc social pixmap"))?;
    let sx = (w as f32) / tree.size().width();
    let sy = (h as f32) / tree.size().height();
    resvg::render(&tree, Transform::from_scale(sx, sy), &mut pix.as_mut());
    Ok(pix)
}
