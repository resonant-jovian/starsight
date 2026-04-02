use crate::backend::{DrawBackend, PathCommand};
use crate::error::{Result, StarsightError};
use crate::primitives::color::Color;
use crate::primitives::geom::Rect;
use std::path::Path;
use tiny_skia::{Paint, PathBuilder, Pixmap, Stroke};
// -------------------------------------------------------------------------------------------------
pub struct SkiaBackend {
    pixmap: Pixmap,
    font_system: cosmic_text::FontSystem,
    swash_cache: cosmic_text::SwashCache,
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
        // Convert PathCommand sequence to tiny_skia::Path
        let mut pb = PathBuilder::new();
        for cmd in path.commands() {
            match cmd {
                PathCommand::MoveTo(p) => pb.move_to(p.x, p.y),
                PathCommand::LineTo(p) => pb.line_to(p.x, p.y),
                PathCommand::QuadTo(c, p) => pb.quad_to(c.x, c.y, p.x, p.y),
                PathCommand::CubicTo(c1, c2, p) => pb.cubic_to(c1.x, c1.y, c2.x, c2.y, p.x, p.y),
                PathCommand::Close => pb.close(),
            }
        }
        let sk_path = pb
            .finish()
            .ok_or_else(|| StarsightError::Render("Empty path".into()))?;

        let mut paint = Paint::default();
        paint.set_color_rgba8(
            style.stroke_color.r,
            style.stroke_color.g,
            style.stroke_color.b,
            255,
        );
        let stroke = Stroke {
            width: style.stroke_width,
            line_cap: style.line_cap,
            line_join: style.line_join,
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

    fn save_svg(&self, path: &Path) -> Result<()> {
        todo!()
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

    // draw_text and save_svg omitted for brevity — see Look up section
}
