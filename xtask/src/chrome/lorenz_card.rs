//! `assets/lorenz-{light,dark}.png` — bordered "worked example" card.
//!
//! Wraps the `examples/scientific/lorenz_line{,_dark}.png` chart in a chrome
//! card (rounded rect, 1px border) so it sits visually next to the hero,
//! gallery, status, etc. assets in the README. No caption — the surrounding
//! prose carries that.

use anyhow::{Context, Result, anyhow};
use std::path::Path;
use tiny_skia::{Paint, PathBuilder, Pixmap, PixmapPaint, Shader, Stroke, Transform};

use super::palette::{Theme, rgba};

const W: u32 = 880;
const PAD: u32 = 16;
const RADIUS: f32 = 12.0;

pub fn regen(root: &Path, theme: Theme) -> Result<()> {
    let canvas = compose(root, theme)?;
    let out = root.join(format!("assets/lorenz-{}.png", theme.suffix()));
    canvas
        .save_png(&out)
        .map_err(|e| anyhow!("write lorenz card png: {e}"))?;
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
    let suffix = theme.example_suffix();
    let src = root.join(format!("examples/scientific/lorenz_line{suffix}.png"));
    let img = image::open(&src)
        .with_context(|| format!("opening {}", src.display()))?
        .to_rgba8();
    let (sw, sh) = img.dimensions();

    // Inset the chart with PAD margin; preserve aspect.
    let inner_w = W - 2 * PAD;
    let inner_h = ((inner_w as f32) * (sh as f32) / (sw as f32)).round() as u32;
    let h = inner_h + 2 * PAD;

    let mut canvas = Pixmap::new(W, h).ok_or_else(|| anyhow!("alloc lorenz canvas"))?;
    let (br, bg, bb, ba) = rgba::bg(theme);
    canvas.fill(tiny_skia::Color::from_rgba8(br, bg, bb, ba));

    let resized = image::imageops::resize(
        &img,
        inner_w,
        inner_h,
        image::imageops::FilterType::Lanczos3,
    );

    let mut tp = Pixmap::new(inner_w, inner_h).ok_or_else(|| anyhow!("alloc thumb"))?;
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
        PAD as i32,
        PAD as i32,
        tp.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );

    draw_card(&mut canvas, theme);
    Ok(canvas)
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
