//! `assets/social/card-{light,dark}.png` — 1280×640 GitHub social / open-graph card.
//!
//! GitHub's recommended template uses a 40-pt safe margin around the important
//! content; we use ~96px (~75pt at 1280×640 → 80pt cushion). The card is the
//! wordmark lockup centered, with a one-line tagline below and a lower-corner
//! meta strip pulled from `Cargo.toml`.
//!
//! Useful for:
//! - GitHub repository "Social preview" (`Settings → General → Social preview`).
//! - README footer image.
//! - Slack / Twitter unfurls when the repo URL is shared.

use anyhow::{Context, Result, anyhow};
use std::path::Path;
use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapPaint, Shader, Stroke, Transform};

use super::eclipse;
use super::palette::{MONO_FAMILY, SANS, Theme, palette, rgba};

const W: u32 = 1280;
const H: u32 = 640;
const RADIUS: f32 = 24.0;
const SAFE: u32 = 96;

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let meta = read_meta(root)?;
    let canvas = compose(theme, &meta)?;
    let out_dir = root.join("assets/social");
    std::fs::create_dir_all(&out_dir)?;
    let out = out_dir.join(format!("card-{}.png", theme.suffix()));
    canvas
        .save_png(&out)
        .map_err(|e| anyhow!("write social card png: {e}"))?;
    println!(
        "wrote {} ({} bytes, {}×{})",
        out.display(),
        std::fs::metadata(&out)?.len(),
        canvas.width(),
        canvas.height()
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
    Ok(Meta {
        version: pkg
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string(),
        msrv: pkg
            .get("rust-version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.89")
            .to_string(),
        license: pkg
            .get("license")
            .and_then(|v| v.as_str())
            .unwrap_or("GPL-3.0-only")
            .to_string(),
        edition: pkg
            .get("edition")
            .and_then(|v| v.as_str())
            .unwrap_or("2024")
            .to_string(),
    })
}

fn compose(theme: Theme, meta: &Meta) -> Result<Pixmap> {
    let mut canvas = Pixmap::new(W, H).ok_or_else(|| anyhow!("alloc social card"))?;
    let (br, bg, bb, ba) = rgba::bg(theme);
    canvas.fill(tiny_skia::Color::from_rgba8(br, bg, bb, ba));

    let card = render_card_svg(theme, meta);
    let pix = rasterize_svg(&card, W, H)?;
    canvas.draw_pixmap(
        0,
        0,
        pix.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    draw_card(&mut canvas, theme);
    Ok(canvas)
}

fn render_card_svg(theme: Theme, meta: &Meta) -> String {
    let p = palette(theme);
    // Eclipse mark size 240 px, vertically centered around y=H/2 - 20.
    let eclipse_size: u32 = 240;
    let center_y: u32 = H / 2 - 20;
    let ex = SAFE;
    let ey = center_y - eclipse_size / 2;

    let wm_x = ex + eclipse_size + 56;
    let wm_y = center_y + 36; // baseline of "starsight" text
    let tag_y = center_y + 96;

    let meta_text = format!(
        "v{}  ·  rust {}  ·  edition {}  ·  {}",
        meta.version, meta.msrv, meta.edition, meta.license
    );

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {W} {H}">
  <rect x="0" y="0" width="{W}" height="{H}" fill="{bg}"/>
  <g transform="translate({ex},{ey}) scale({s})">
{eclipse_inner}  </g>
  <text x="{wm_x}" y="{wm_y}" font-family="{sans}" font-weight="700" font-size="156" fill="{text}" letter-spacing="-4">starsight</text>
  <text x="{wm_x}" y="{tag_y}" font-family="{sans}" font-size="26" fill="{sub}">scientific visualization for Rust — typed, layered, eight backends</text>
  <text x="{meta_x}" y="{meta_y}" font-family="{mono}" font-size="20" fill="{muted}">{meta}</text>
  <text x="{repo_x}" y="{meta_y}" font-family="{mono}" font-size="20" fill="{muted}" text-anchor="end">github.com/resonant-jovian/starsight</text>
</svg>
"##,
        W = W,
        H = H,
        bg = p.bg,
        ex = ex,
        ey = ey,
        s = eclipse_size as f32 / 100.0, // eclipse svg viewBox is 0..100
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

fn rasterize_svg(svg: &str, w: u32, h: u32) -> Result<Pixmap> {
    let mut opts = usvg::Options::default();
    opts.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_str(svg, &opts).context("parse social card svg")?;
    let mut pix = Pixmap::new(w, h).ok_or_else(|| anyhow!("alloc social pixmap"))?;
    let sx = (w as f32) / tree.size().width();
    let sy = (h as f32) / tree.size().height();
    resvg::render(&tree, Transform::from_scale(sx, sy), &mut pix.as_mut());
    Ok(pix)
}

fn draw_card(canvas: &mut Pixmap, theme: Theme) {
    let w = canvas.width() as f32;
    let h = canvas.height() as f32;
    let mut pb = PathBuilder::new();
    add_round_rect(&mut pb, 0.5, 0.5, w - 1.0, h - 1.0, RADIUS);
    let path = pb.finish().expect("rounded card path");
    let mut paint = Paint::default();
    let (r, g, b, a) = rgba::border(theme);
    paint.shader = Shader::SolidColor(tiny_skia::Color::from_rgba8(r, g, b, a));
    paint.anti_alias = true;
    let mut stroke = Stroke::default();
    stroke.width = 1.0;
    canvas.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
}

fn add_round_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
}
