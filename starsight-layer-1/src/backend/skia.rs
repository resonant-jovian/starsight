//!
use crate::backend::{DrawBackend, PathCommand};
use crate::error::{Result, StarsightError};
use crate::primitives::color::Color;
use crate::primitives::geom::{Point, Rect};
use std::path::Path;
use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke};
// -------------------------------------------------------------------------------------------------
pub struct SkiaBackend {
    pixmap: Pixmap,
    font_system: cosmic_text::FontSystem,
    swash_cache: cosmic_text::SwashCache,
    clip_mask: Option<tiny_skia::Mask>,
}

impl SkiaBackend {
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let pixmap = Pixmap::new(width, height).ok_or_else(|| {
            StarsightError::Render(format!("Failed to create {width}x{height} pixmap"))
        })?;
        Ok(Self {
            pixmap,
            font_system: cosmic_text::FontSystem::new(),
            swash_cache: cosmic_text::SwashCache::new(),
            clip_mask: None,
        })
    }

    pub fn fill(&mut self, color: Color) {
        self.pixmap.fill(color.to_tiny_skia());
    }

    pub fn png_bytes(&self) -> Result<Vec<u8>> {
        self.pixmap
            .encode_png()
            .map_err(|e| StarsightError::Export(e.to_string()))
    }
}
// -------------------------------------------------------------------------------------------------
impl DrawBackend for SkiaBackend {
    fn draw_path(
        &mut self,
        path: &crate::backend::Path,
        style: &crate::backend::PathStyle,
    ) -> Result<()> {
        let mut pb = PathBuilder::new();
        for cmd in &path.commands {
            match cmd {
                PathCommand::MoveTo(p) => pb.move_to(p.x, p.y),
                PathCommand::LineTo(p) => pb.line_to(p.x, p.y),
                PathCommand::QuadTo(c, p) => pb.quad_to(c.x, c.y, p.x, p.y),
                PathCommand::CubicTo(c1, c2, p) => {
                    pb.cubic_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y);
                }
                PathCommand::Close => pb.close(),
            }
        }
        let sk_path = pb
            .finish()
            .ok_or_else(|| StarsightError::Render("Empty path".into()))?;

        let mut paint = Paint::default();

        // Fill first if requested
        if let Some(fill) = style.fill_color {
            paint.set_color_rgba8(fill.r, fill.g, fill.b, (style.opacity * 255.0) as u8);
            self.pixmap.fill_path(
                &sk_path,
                &paint,
                tiny_skia::FillRule::Winding,
                tiny_skia::Transform::identity(),
                None,
            );
        }

        // Stroke
        if style.stroke_width > 0.0 {
            paint.set_color_rgba8(
                style.stroke_color.r,
                style.stroke_color.g,
                style.stroke_color.b,
                (style.opacity * 255.0) as u8,
            );
            let stroke = Stroke {
                width: style.stroke_width,
                line_cap: style.line_cap.to_tiny_skia(),
                line_join: style.line_join.to_tiny_skia(),
                dash: style
                    .dash_pattern
                    .and_then(|(len, gap)| tiny_skia::StrokeDash::new(vec![len, gap], 0.0)),
                ..Stroke::default()
            };
            self.pixmap.stroke_path(
                &sk_path,
                &paint,
                &stroke,
                tiny_skia::Transform::identity(),
                None,
            );
        }

        Ok(())
    }

    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        font_size: f32,
        color: Color,
    ) -> Result<()> {
        let metrics = cosmic_text::Metrics::new(font_size, font_size * 1.2);
        let mut buffer = cosmic_text::Buffer::new(&mut self.font_system, metrics);
        buffer.set_text(
            &mut self.font_system,
            text,
            &cosmic_text::Attrs::new(),
            cosmic_text::Shaping::Advanced,
            None,
        );
        buffer.set_size(
            &mut self.font_system,
            Some(self.pixmap.width() as f32),
            None,
        );
        buffer.shape_until_scroll(&mut self.font_system, true);

        let text_color = cosmic_text::Color::rgba(color.r, color.g, color.b, 255);
        let mut paint = Paint::default();
        buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            text_color,
            |x, y, w, h, c| {
                paint.set_color_rgba8(c.r(), c.g(), c.b(), c.a());
                let px = x as f32 + position.x;
                let py = y as f32 + position.y;
                if let Some(rect) = tiny_skia::Rect::from_xywh(px, py, w as f32, h as f32) {
                    self.pixmap
                        .fill_rect(rect, &paint, tiny_skia::Transform::identity(), None);
                }
            },
        );
        Ok(())
    }
    fn set_clip(&mut self, rect: Option<Rect>) -> Result<()> {
        match rect {
            Some(r) => {
                let mut mask = tiny_skia::Mask::new(self.pixmap.width(), self.pixmap.height())
                    .ok_or_else(|| StarsightError::Render("Failed to create mask".into()))?;
                let clip_path = PathBuilder::from_rect(
                    r.to_tiny_skia()
                        .ok_or_else(|| StarsightError::Render("Invalid clip rect".into()))?,
                );
                mask.fill_path(
                    &clip_path,
                    tiny_skia::FillRule::Winding,
                    false,
                    tiny_skia::Transform::identity(),
                );
                self.clip_mask = Some(mask);
            }
            None => {
                self.clip_mask = None;
            }
        }
        Ok(())
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.pixmap.width(), self.pixmap.height())
    }

    fn save_png(&self, path: &std::path::Path) -> Result<()> {
        self.pixmap
            .save_png(path)
            .map_err(|e| StarsightError::Export(e.to_string()))
    }
    fn save_svg(&self, _path: &Path) -> Result<()> {
        Err(StarsightError::Export(
            "Raster backend cannot save SVG directly; use SvgBackend".into(),
        ))
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
        let sk_rect = rect
            .to_tiny_skia()
            .ok_or_else(|| StarsightError::Render("Invalid rect".into()))?;
        let mut paint = Paint::default();
        paint.set_color_rgba8(color.r, color.g, color.b, 255);
        self.pixmap
            .fill_rect(sk_rect, &paint, tiny_skia::Transform::identity(), None);
        Ok(())
    }
}
