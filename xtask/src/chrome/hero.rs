//! `assets/hero/starsight-hero-{light,dark}.png` — composite hero, paired variants.
//!
//! Layout (880 × ~870):
//! - rounded outer card (rx=8, 1px border)
//! - top strip (156 px): eclipse + wordmark + tagline + meta
//! - 3×3 grid: theme-matched example renders
//!
//! Light variant uses `<name>.png`; dark variant uses `<name>_dark.png` from
//! the example output directories. Dark siblings come from re-running the
//! examples with `STARSIGHT_THEME=dark`.

use anyhow::{Context, Result, anyhow};
use std::path::Path;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, PixmapPaint, Shader, Stroke, Transform};

use super::eclipse;
use super::palette::{MONO_FAMILY, SANS, Theme, palette, rgba};

const W: u32 = 880;
const PAD: u32 = 24;
const TOP_H: u32 = 156;
const GUTTER: u32 = 10;
const COLS: u32 = 3;
const ROWS: u32 = 3;
const RADIUS: f32 = 12.0;

const HERO_BASES: &[&str] = &[
    "examples/basics/line_chart",
    "examples/basics/scatter",
    "examples/basics/bar_chart",
    "examples/basics/histogram",
    "examples/scientific/contour_fields",
    "examples/scientific/nightingale",
    "examples/scientific/candlestick",
    "examples/scientific/radar_spider",
    "examples/scientific/lorenz_line",
];

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let meta = read_meta(root)?;
    let canvas = compose(root, &meta, theme)?;
    let out = root.join(format!("assets/hero/starsight-hero-{}.png", theme.suffix()));
    std::fs::create_dir_all(out.parent().unwrap())?;
    canvas
        .save_png(&out)
        .map_err(|e| anyhow!("write hero png: {e}"))?;
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
    edition: String,
    msrv: String,
    license: String,
}

fn read_meta(root: &Path) -> Result<Meta> {
    let cargo = std::fs::read_to_string(root.join("Cargo.toml"))?;
    let toml: toml::Value = toml::from_str(&cargo)?;
    let pkg = toml
        .get("workspace")
        .and_then(|w| w.get("package"))
        .ok_or_else(|| anyhow!("no [workspace.package]"))?;
    let version = pkg
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.0.0")
        .to_string();
    let edition = pkg
        .get("edition")
        .and_then(|v| v.as_str())
        .unwrap_or("2024")
        .to_string();
    let msrv = pkg
        .get("rust-version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.89")
        .to_string();
    let license = pkg
        .get("license")
        .and_then(|v| v.as_str())
        .unwrap_or("GPL-3.0-only")
        .to_string();
    Ok(Meta {
        version,
        edition,
        msrv,
        license,
    })
}

fn compose(root: &Path, meta: &Meta, theme: Theme) -> Result<Pixmap> {
    let cell_w = (W - 2 * PAD - GUTTER * (COLS - 1)) / COLS;
    let cell_h = ((cell_w as f32) * 0.62) as u32;
    let grid_h = cell_h * ROWS + GUTTER * (ROWS - 1);
    let h = TOP_H + grid_h + 2 * PAD;

    let mut canvas = Pixmap::new(W, h).ok_or_else(|| anyhow!("alloc canvas"))?;
    let (br, bg, bb, ba) = rgba::bg(theme);
    canvas.fill(tiny_skia::Color::from_rgba8(br, bg, bb, ba));

    // top strip via SVG → raster
    let top = render_top_strip(meta, theme)?;
    canvas.draw_pixmap(
        0,
        PAD as i32,
        top.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    // hairline rule under the top strip
    fill_rect(
        &mut canvas,
        PAD as i32,
        (PAD + TOP_H - 1) as i32,
        W - 2 * PAD,
        1,
        rgba::border(theme),
    );

    // 3×3 grid of theme-matched thumbnails
    let suffix = theme.example_suffix();
    for (i, base) in HERO_BASES.iter().enumerate() {
        let col = (i as u32) % COLS;
        let row = (i as u32) / COLS;
        let x0 = PAD + col * (cell_w + GUTTER);
        let y0 = PAD + TOP_H + row * (cell_h + GUTTER);
        let path = root.join(format!("{base}{suffix}.png"));
        composite_thumb(
            &mut canvas,
            &path,
            x0 as i32,
            y0 as i32,
            cell_w,
            cell_h,
            theme,
        )?;
    }

    // outer rounded card stroke — drawn last so no later blit can overwrite it
    draw_card(&mut canvas, theme);
    Ok(canvas)
}

fn render_top_strip(meta: &Meta, theme: Theme) -> Result<Pixmap> {
    let p = palette(theme);
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}">
  <rect x="0" y="0" width="{w}" height="{h}" fill="{bg}"/>
  <g transform="translate({ex},{ey}) scale(0.92)">
{eclipse_inner}  </g>
  <text x="{wm_x}" y="{wm_y}" font-family="{sans}" font-weight="700" font-size="56" fill="{text}" letter-spacing="-1.5">starsight</text>
  <text x="{tag_x}" y="{tag_y}" font-family="{sans}" font-size="16" fill="{sub}">scientific visualization for Rust — typed, layered, eight backends</text>
  <text x="{meta_x}" y="{meta_y}" font-family="{mono}" font-size="11" fill="{muted}" text-anchor="end">v{ver}  ·  rust {msrv}  ·  edition {ed}  ·  {lic}</text>
</svg>
"#,
        w = W,
        h = TOP_H,
        bg = p.card,
        ex = PAD,
        ey = (TOP_H - 96) / 2,
        eclipse_inner = eclipse::svg_inner(p),
        wm_x = PAD + 110,
        wm_y = 70,
        tag_x = PAD + 114,
        tag_y = 110,
        meta_x = W - PAD,
        meta_y = TOP_H - 14,
        sans = SANS,
        mono = MONO_FAMILY,
        text = p.text,
        sub = p.subtext,
        muted = p.muted,
        ver = meta.version,
        msrv = meta.msrv,
        ed = meta.edition,
        lic = meta.license,
    );
    rasterize_svg(&svg, W, TOP_H)
}

fn rasterize_svg(svg: &str, w: u32, h: u32) -> Result<Pixmap> {
    let mut opts = usvg::Options::default();
    opts.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_str(svg, &opts).context("parse top-strip svg")?;
    let mut pix = Pixmap::new(w, h).ok_or_else(|| anyhow!("alloc top-strip pixmap"))?;
    let scale_x = (w as f32) / tree.size().width();
    let scale_y = (h as f32) / tree.size().height();
    resvg::render(
        &tree,
        Transform::from_scale(scale_x, scale_y),
        &mut pix.as_mut(),
    );
    Ok(pix)
}

fn composite_thumb(
    canvas: &mut Pixmap,
    src: &Path,
    x: i32,
    y: i32,
    cell_w: u32,
    cell_h: u32,
    theme: Theme,
) -> Result<()> {
    let card = rgba::card(theme);
    fill_rect(canvas, x, y, cell_w, cell_h, card);

    if !src.exists() {
        draw_rect_outline(canvas, x, y, cell_w, cell_h, rgba::border(theme));
        return Ok(());
    }
    let img = image::open(src)?.to_rgba8();
    let (sw, sh) = img.dimensions();
    let scale = (cell_w as f32 / sw as f32).min(cell_h as f32 / sh as f32);
    let tw = (sw as f32 * scale).round() as u32;
    let th = (sh as f32 * scale).round() as u32;
    let resized = image::imageops::resize(&img, tw, th, image::imageops::FilterType::Lanczos3);

    let tx = x + ((cell_w - tw) / 2) as i32;
    let ty = y + ((cell_h - th) / 2) as i32;

    let mut tp = Pixmap::new(tw, th).ok_or_else(|| anyhow!("alloc thumb"))?;
    {
        let dst = tp.data_mut();
        for (i, px) in resized.pixels().enumerate() {
            let [r, g, b, a] = px.0;
            let af = u32::from(a);
            dst[i * 4] = (u32::from(r) * af / 255) as u8;
            dst[i * 4 + 1] = (u32::from(g) * af / 255) as u8;
            dst[i * 4 + 2] = (u32::from(b) * af / 255) as u8;
            dst[i * 4 + 3] = a;
        }
    }
    canvas.draw_pixmap(
        tx,
        ty,
        tp.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    draw_rect_outline(canvas, x, y, cell_w, cell_h, rgba::border(theme));
    Ok(())
}

/// Outer rounded card border on the canvas.
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

fn fill_rect(p: &mut Pixmap, x: i32, y: i32, w: u32, h: u32, (r, g, b, a): (u8, u8, u8, u8)) {
    let cw = p.width() as i32;
    let ch = p.height() as i32;
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + w as i32).min(cw);
    let y1 = (y + h as i32).min(ch);
    let stride = p.width() as usize * 4;
    let data = p.data_mut();
    let af = u32::from(a);
    for yy in y0..y1 {
        for xx in x0..x1 {
            let i = yy as usize * stride + xx as usize * 4;
            data[i] = (u32::from(r) * af / 255) as u8;
            data[i + 1] = (u32::from(g) * af / 255) as u8;
            data[i + 2] = (u32::from(b) * af / 255) as u8;
            data[i + 3] = a;
        }
    }
}

fn draw_rect_outline(p: &mut Pixmap, x: i32, y: i32, w: u32, h: u32, c: (u8, u8, u8, u8)) {
    let r = w as i32;
    let b = h as i32;
    fill_rect(p, x, y, w, 1, c);
    fill_rect(p, x, y + b - 1, w, 1, c);
    fill_rect(p, x, y, 1, h, c);
    fill_rect(p, x + r - 1, y, 1, h, c);
}

// silence unused FillRule import (kept for potential future use)
#[allow(dead_code)]
const _SILENCE: FillRule = FillRule::Winding;
