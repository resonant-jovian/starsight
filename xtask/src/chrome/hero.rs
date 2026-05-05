//! `assets/hero/starsight-hero.png` — composite hero (light, monochrome).
//!
//! Layout (880 × ~870):
//! - top strip (156 px): eclipse mark + wordmark + tagline + meta
//! - 3×3 grid: real example renders, rounded corners, hairline border
//!
//! Top strip is built as SVG and rasterized through resvg so text rendering
//! flows through usvg's system-font path — no cosmic-text/swash plumbing.

use anyhow::{Context, Result, anyhow};
use std::path::Path;
use tiny_skia::{Color, Pixmap, PixmapPaint, Transform};

use super::eclipse;
use super::palette::{MONO, MONO_FAMILY, SANS, rgba};

const W: u32 = 880;
const PAD: u32 = 24;
const TOP_H: u32 = 156;
const GUTTER: u32 = 10;
const COLS: u32 = 3;
const ROWS: u32 = 3;

const HERO_PNGS: &[&str] = &[
    "examples/basics/line_chart.png",
    "examples/basics/scatter.png",
    "examples/basics/bar_chart.png",
    "examples/basics/histogram.png",
    "examples/scientific/contour_fields.png",
    "examples/scientific/nightingale.png",
    "examples/scientific/candlestick.png",
    "examples/scientific/radar_spider.png",
    "examples/scientific/lorenz_line.png",
];

pub fn regen(root: &Path) -> Result<()> {
    let meta = read_meta(root)?;
    let canvas = compose(root, &meta)?;
    let out = root.join("assets/hero/starsight-hero-light.png");
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
    Ok(Meta { version, edition, msrv, license })
}

fn compose(root: &Path, meta: &Meta) -> Result<Pixmap> {
    let cell_w = (W - 2 * PAD - GUTTER * (COLS - 1)) / COLS;
    let cell_h = ((cell_w as f32) * 0.62) as u32;
    let grid_h = cell_h * ROWS + GUTTER * (ROWS - 1);
    let h = TOP_H + grid_h + 2 * PAD;

    let mut canvas =
        Pixmap::new(W, h).ok_or_else(|| anyhow!("alloc canvas {W}×{h}"))?;
    let (br, bg, bb, ba) = rgba::BG;
    canvas.fill(Color::from_rgba8(br, bg, bb, ba));

    // top strip (SVG → raster)
    let top = render_top_strip(meta)?;
    canvas.draw_pixmap(
        0,
        PAD as i32,
        top.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    // bottom rule between top and grid
    draw_hline(
        &mut canvas,
        PAD as i32,
        (W - PAD) as i32,
        (PAD + TOP_H - 1) as i32,
        rgba::BORDER,
    );

    // 3×3 grid
    for (i, path) in HERO_PNGS.iter().enumerate() {
        let col = (i as u32) % COLS;
        let row = (i as u32) / COLS;
        let x0 = PAD + col * (cell_w + GUTTER);
        let y0 = PAD + TOP_H + row * (cell_h + GUTTER);
        composite_thumb(
            &mut canvas,
            &root.join(path),
            x0 as i32,
            y0 as i32,
            cell_w,
            cell_h,
        )?;
    }
    Ok(canvas)
}

/// Render the top-strip SVG (880 × TOP_H) at 1× and return the rasterized pixmap.
fn render_top_strip(meta: &Meta) -> Result<Pixmap> {
    let p = &MONO;
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}">
  <rect x="0" y="0" width="{w}" height="{h}" fill="{bg}"/>
  <g transform="translate({ex},{ey}) scale(0.92)">
{eclipse_inner}  </g>
  <text x="{wm_x}" y="{wm_y}" font-family="{sans}" font-weight="700" font-size="56" fill="{text}" letter-spacing="-1.5">starsight</text>
  <text x="{tag_x}" y="{tag_y}" font-family="{sans}" font-size="16" fill="{sub}">scientific visualization for Rust — typed, layered, eight backends</text>
  <text x="{meta_x}" y="{meta_y}" font-family="{mono}" font-size="11" fill="{muted}" text-anchor="end">v{ver}  ·  rust {msrv}  ·  edition {ed}  ·  {lic}</text>
</svg>
"##,
        w = W,
        h = TOP_H,
        bg = p.bg,
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
) -> Result<()> {
    if !src.exists() {
        // placeholder card
        draw_rect_outline(canvas, x, y, cell_w, cell_h, rgba::BORDER);
        return Ok(());
    }
    let img = image::open(src)?.to_rgba8();
    let (sw, sh) = img.dimensions();

    // Fit-contain into the cell.
    let scale = (cell_w as f32 / sw as f32).min(cell_h as f32 / sh as f32);
    let tw = (sw as f32 * scale).round() as u32;
    let th = (sh as f32 * scale).round() as u32;
    let resized = image::imageops::resize(
        &img,
        tw,
        th,
        image::imageops::FilterType::Lanczos3,
    );

    // Cell card (white) so transparent thumbnails sit on white.
    fill_rect(canvas, x, y, cell_w, cell_h, rgba::BG);

    // center the thumb in the cell
    let tx = x + ((cell_w - tw) / 2) as i32;
    let ty = y + ((cell_h - th) / 2) as i32;

    // build a Pixmap from the resized rgba bytes
    let mut tp = Pixmap::new(tw, th)
        .ok_or_else(|| anyhow!("alloc thumb {tw}×{th}"))?;
    {
        let dst = tp.data_mut();
        // Convert RGBA → premultiplied BGRA (tiny-skia native).
        for (i, px) in resized.pixels().enumerate() {
            let [r, g, b, a] = px.0;
            // tiny-skia uses premultiplied RGBA in memory.
            let af = a as u32;
            let pr = (r as u32 * af / 255) as u8;
            let pg = (g as u32 * af / 255) as u8;
            let pb = (b as u32 * af / 255) as u8;
            dst[i * 4] = pr;
            dst[i * 4 + 1] = pg;
            dst[i * 4 + 2] = pb;
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

    // 1px hairline border around the cell
    draw_rect_outline(canvas, x, y, cell_w, cell_h, rgba::BORDER);
    Ok(())
}

// --- minimal pixel ops ---

fn fill_rect(p: &mut Pixmap, x: i32, y: i32, w: u32, h: u32, (r, g, b, a): (u8, u8, u8, u8)) {
    let cw = p.width() as i32;
    let ch = p.height() as i32;
    let x0 = x.max(0);
    let y0 = y.max(0);
    let x1 = (x + w as i32).min(cw);
    let y1 = (y + h as i32).min(ch);
    let stride = p.width() as usize * 4;
    let data = p.data_mut();
    let af = a as u32;
    let pr = (r as u32 * af / 255) as u8;
    let pg = (g as u32 * af / 255) as u8;
    let pb = (b as u32 * af / 255) as u8;
    for yy in y0..y1 {
        for xx in x0..x1 {
            let i = yy as usize * stride + xx as usize * 4;
            data[i] = pr;
            data[i + 1] = pg;
            data[i + 2] = pb;
            data[i + 3] = a;
        }
    }
}

fn draw_rect_outline(p: &mut Pixmap, x: i32, y: i32, w: u32, h: u32, c: (u8, u8, u8, u8)) {
    let r = w as i32;
    let b = h as i32;
    draw_hline(p, x, x + r, y, c);
    draw_hline(p, x, x + r, y + b - 1, c);
    draw_vline(p, x, y, y + b, c);
    draw_vline(p, x + r - 1, y, y + b, c);
}

fn draw_hline(p: &mut Pixmap, x0: i32, x1: i32, y: i32, c: (u8, u8, u8, u8)) {
    fill_rect(p, x0, y, (x1 - x0) as u32, 1, c);
}
fn draw_vline(p: &mut Pixmap, x: i32, y0: i32, y1: i32, c: (u8, u8, u8, u8)) {
    fill_rect(p, x, y0, 1, (y1 - y0) as u32, c);
}
