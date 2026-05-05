//! `assets/gallery-{light,dark}.png` — showcase composite, paired variants.

use anyhow::{Context, Result, anyhow};
use std::path::Path;
use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapPaint, Shader, Stroke, Transform};

use super::palette::{MONO_FAMILY, SANS, Theme, palette, rgba};

const W: u32 = 880;
const PAD: u32 = 24;
const GUTTER: u32 = 10;
const COLS: u32 = 3;
const ROWS: u32 = 3;
const EYEBROW_H: u32 = 36;
const CAP_H: u32 = 30;
const RADIUS: f32 = 12.0;

const GALLERY: &[(&str, &str)] = &[
    ("examples/basics/heatmap", "heatmap"),
    ("examples/basics/bubble_scatter", "bubble · scatter"),
    ("examples/basics/movie_heatmap", "categorical heatmap"),
    ("examples/scientific/gauge", "gauge · polar arc"),
    ("examples/scientific/wind_rose", "wind rose · polar bar"),
    ("examples/scientific/polar_calendar", "polar calendar"),
    (
        "examples/scientific/kruskal_szekeres_line",
        "kruskal–szekeres",
    ),
    ("examples/scientific/laser_plasma", "laser plasma · contour"),
    ("examples/scientific/error_bars", "error bars · rug"),
];

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let canvas = compose(root, theme)?;
    let out = root.join(format!("assets/gallery-{}.png", theme.suffix()));
    canvas
        .save_png(&out)
        .map_err(|e| anyhow!("write gallery png: {e}"))?;
    println!(
        "wrote {} ({} bytes, {}×{})",
        out.display(),
        std::fs::metadata(&out)?.len(),
        canvas.width(),
        canvas.height()
    );
    Ok(())
}

fn compose(root: &Path, theme: Theme) -> Result<Pixmap> {
    let cell_w = (W - 2 * PAD - GUTTER * (COLS - 1)) / COLS;
    let cell_img_h = ((cell_w as f32) * 0.62) as u32;
    let cell_h = cell_img_h + CAP_H;
    let grid_h = cell_h * ROWS + GUTTER * (ROWS - 1);
    let h = EYEBROW_H + grid_h + 2 * PAD;

    let mut canvas = Pixmap::new(W, h).ok_or_else(|| anyhow!("alloc gallery"))?;
    let (br, bg, bb, ba) = rgba::bg(theme);
    canvas.fill(tiny_skia::Color::from_rgba8(br, bg, bb, ba));

    let p = palette(theme);
    let eyebrow = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}">
  <text x="24" y="22" font-family="{f}" font-size="12" fill="{c}" letter-spacing="0.6">// showcase  ·  9 of 38 examples  ·  source under examples/</text>
</svg>"#,
        w = W,
        h = EYEBROW_H,
        f = MONO_FAMILY,
        c = p.muted
    );
    let strip = rasterize_svg(&eyebrow, W, EYEBROW_H)?;
    canvas.draw_pixmap(
        0,
        PAD as i32,
        strip.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    let suffix = theme.example_suffix();
    for (i, (base, caption)) in GALLERY.iter().enumerate() {
        let col = (i as u32) % COLS;
        let row = (i as u32) / COLS;
        let x0 = PAD + col * (cell_w + GUTTER);
        let y0 = EYEBROW_H + PAD + row * (cell_h + GUTTER);

        let png = root.join(format!("{base}{suffix}.png"));
        composite_thumb(
            &mut canvas,
            &png,
            x0 as i32,
            y0 as i32,
            cell_w,
            cell_img_h,
            theme,
        )?;

        let cap_svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}">
  <text x="{cx}" y="20" font-family="{f}" font-weight="700" font-size="12" fill="{c}" text-anchor="middle">{caption}</text>
</svg>"#,
            w = cell_w,
            h = CAP_H,
            cx = cell_w / 2,
            f = SANS,
            c = p.text,
            caption = caption
        );
        let cap_pix = rasterize_svg(&cap_svg, cell_w, CAP_H)?;
        canvas.draw_pixmap(
            x0 as i32,
            (y0 + cell_img_h) as i32,
            cap_pix.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }
    draw_card(&mut canvas, theme);
    Ok(canvas)
}

fn rasterize_svg(svg: &str, w: u32, h: u32) -> Result<Pixmap> {
    let mut opts = usvg::Options::default();
    opts.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_str(svg, &opts).context("parse gallery svg slice")?;
    let mut pix = Pixmap::new(w, h).ok_or_else(|| anyhow!("alloc gallery slice pixmap"))?;
    let sx = (w as f32) / tree.size().width();
    let sy = (h as f32) / tree.size().height();
    resvg::render(&tree, Transform::from_scale(sx, sy), &mut pix.as_mut());
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
    fill_rect(canvas, x, y, cell_w, cell_h, rgba::card(theme));
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
