//! Vector backend: produces SVG documents using the `svg` crate.
//!
//! Each draw call appends an XML element to an in-memory list. `save_svg`
//! serializes the document to a file. The backend is resolution-independent and
//! preserves text as `<text>` elements rather than rasterizing glyphs.

use crate::backends::DrawBackend;
use crate::errors::{Result, StarsightError};
use crate::paths::PathCommand;
use crate::primitives::{Color, Point, Rect};
use svg::Document;
use svg::node::element::{Path as SvgPath, Rectangle, Text as SvgText};

// ── SvgBackend ───────────────────────────────────────────────────────────────────────────────────

/// In-memory SVG document builder.
pub struct SvgBackend {
    width: u32,
    height: u32,
    elements: Vec<Box<dyn svg::Node>>,
}

impl SvgBackend {
    /// Create an empty SVG document of `width × height` pixels.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            elements: Vec::new(),
        }
    }

    /// Build an `svg::Document` from the accumulated elements.
    fn build_document(&self) -> Document {
        let mut doc = Document::new()
            .set("viewBox", (0, 0, self.width, self.height))
            .set("xmlns", "http://www.w3.org/2000/svg")
            .set("width", self.width)
            .set("height", self.height);
        for el in &self.elements {
            doc = doc.add((*el).clone());
        }
        doc
    }

    /// Serialize the current document to a string.
    #[must_use]
    pub fn svg_string(&self) -> String {
        self.build_document().to_string()
    }
}

// ── DrawBackend impl ─────────────────────────────────────────────────────────────────────────────

impl DrawBackend for SvgBackend {
    fn draw_path(
        &mut self,
        path: &crate::paths::Path,
        style: &crate::paths::PathStyle,
    ) -> Result<()> {
        let mut data = svg::node::element::path::Data::new();
        for cmd in &path.commands {
            match cmd {
                PathCommand::MoveTo(p) => {
                    data = data.move_to((p.x, p.y));
                }
                PathCommand::LineTo(p) => {
                    data = data.line_to((p.x, p.y));
                }
                PathCommand::CubicTo(c1, c2, p) => {
                    data = data.cubic_curve_to((c1.x, c1.y, c2.x, c2.y, p.x, p.y));
                }
                PathCommand::QuadTo(c, p) => {
                    data = data.quadratic_curve_to((c.x, c.y, p.x, p.y));
                }
                PathCommand::Close => {
                    data = data.close();
                }
            }
        }
        let mut p = SvgPath::new()
            .set("d", data)
            .set("stroke", style.stroke_color.to_css_hex())
            .set("stroke-width", style.stroke_width)
            .set(
                "fill",
                style
                    .fill_color
                    .map_or("none".to_string(), Color::to_css_hex),
            );
        if style.opacity < 1.0 {
            p = p
                .set("opacity", style.opacity)
                .set("fill-opacity", style.opacity)
                .set("stroke-opacity", style.opacity);
        }
        // Crisp 1-px hairlines for axis-aligned grid / tick / axis paths
        // — SVG renderers honour shape-rendering="crispEdges" by snapping
        // strokes to the pixel grid (yrp.6). Curves and diagonals keep
        // the default browser shape-rendering setting.
        if path.is_axis_aligned() {
            p = p.set("shape-rendering", "crispEdges");
        }
        self.elements.push(Box::new(p));
        Ok(())
    }

    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        font_size: f32,
        color: Color,
    ) -> Result<()> {
        let t = SvgText::new(text)
            .set("x", position.x)
            .set("y", position.y)
            .set("font-size", font_size)
            .set("fill", color.to_css_hex())
            .set("font-family", "sans-serif");
        self.elements.push(Box::new(t));
        Ok(())
    }

    fn text_extent(&mut self, text: &str, font_size: f32) -> Result<(f32, f32)> {
        // SVG fallback estimate: text length is small (UI labels), f32 precision is sufficient.
        #[allow(clippy::cast_precision_loss)]
        let width = text.len() as f32 * font_size * 0.6;
        let height = font_size;
        Ok((width, height))
    }

    fn draw_rotated_text(
        &mut self,
        text: &str,
        position: Point,
        font_size: f32,
        color: Color,
        rotation: f32,
    ) -> Result<()> {
        if rotation.abs() < 0.1 {
            return self.draw_text(text, position, font_size, color);
        }

        let transform = format!("rotate({} {}, {})", rotation, position.x, position.y);

        let t = SvgText::new(text)
            .set("x", position.x)
            .set("y", position.y)
            .set("font-size", font_size)
            .set("fill", color.to_css_hex())
            .set("font-family", "sans-serif")
            .set("transform", transform);
        self.elements.push(Box::new(t));
        Ok(())
    }

    fn set_clip(&mut self, _rect: Option<Rect>) -> Result<()> {
        // TODO(0.2.0): emit a <clipPath id=...> element and reference it from drawn elements.
        Ok(())
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn save_png(&self, _path: &std::path::Path) -> Result<()> {
        Err(StarsightError::Export(
            "SVG backend cannot save PNG directly; use SkiaBackend or resvg".into(),
        ))
    }

    fn save_svg(&self, path: &std::path::Path) -> Result<()> {
        svg::save(path, &self.build_document()).map_err(|e| StarsightError::Export(e.to_string()))
    }

    fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()> {
        let r = Rectangle::new()
            .set("x", rect.left)
            .set("y", rect.top)
            .set("width", rect.width())
            .set("height", rect.height())
            .set("fill", color.to_css_hex())
            // Rectangles are axis-aligned by construction (yrp.6).
            .set("shape-rendering", "crispEdges");
        self.elements.push(Box::new(r));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SvgBackend;
    use crate::backends::DrawBackend;
    use crate::errors::StarsightError;
    use crate::paths::{Path, PathCommand, PathStyle};
    use crate::primitives::{Color, Point, Rect};

    #[test]
    fn dimensions_returns_size() {
        let b = SvgBackend::new(300, 200);
        assert_eq!(b.dimensions(), (300, 200));
    }

    #[test]
    fn draw_path_with_all_command_kinds() {
        let mut b = SvgBackend::new(200, 200);
        let path = Path {
            commands: vec![
                PathCommand::MoveTo(Point::new(10.0, 10.0)),
                PathCommand::LineTo(Point::new(20.0, 20.0)),
                PathCommand::QuadTo(Point::new(30.0, 30.0), Point::new(40.0, 40.0)),
                PathCommand::CubicTo(
                    Point::new(50.0, 50.0),
                    Point::new(60.0, 60.0),
                    Point::new(70.0, 70.0),
                ),
                PathCommand::Close,
            ],
        };
        b.draw_path(&path, &PathStyle::stroke(Color::BLACK, 1.0))
            .unwrap();
        let svg = b.svg_string();
        assert!(svg.contains("<path"));
    }

    #[test]
    fn draw_path_with_opacity_emits_opacity_attrs() {
        let mut b = SvgBackend::new(100, 100);
        let path = Path::new()
            .move_to(Point::new(0.0, 0.0))
            .line_to(Point::new(10.0, 10.0));
        let mut style = PathStyle::stroke(Color::BLUE, 1.0);
        style.fill_color = Some(Color::RED);
        style.opacity = 0.4;
        b.draw_path(&path, &style).unwrap();
        let svg = b.svg_string();
        assert!(svg.contains("opacity"));
    }

    #[test]
    fn draw_text_emits_text_element() {
        let mut b = SvgBackend::new(100, 100);
        b.draw_text("hello", Point::new(10.0, 50.0), 14.0, Color::BLACK)
            .unwrap();
        let svg = b.svg_string();
        assert!(svg.contains("hello"));
    }

    #[test]
    fn draw_rotated_text_zero_rotation_uses_fast_path() {
        let mut b = SvgBackend::new(100, 100);
        b.draw_rotated_text("hi", Point::new(10.0, 50.0), 12.0, Color::BLACK, 0.0)
            .unwrap();
        let svg = b.svg_string();
        assert!(!svg.contains("transform"));
        assert!(svg.contains("hi"));
    }

    #[test]
    fn draw_rotated_text_emits_transform() {
        let mut b = SvgBackend::new(100, 100);
        b.draw_rotated_text("rotated", Point::new(10.0, 50.0), 12.0, Color::BLACK, 45.0)
            .unwrap();
        let svg = b.svg_string();
        assert!(svg.contains("transform"));
    }

    #[test]
    fn text_extent_proportional() {
        let mut b = SvgBackend::new(100, 100);
        let (w, h) = b.text_extent("test", 10.0).unwrap();
        assert!(w > 0.0);
        assert_eq!(h, 10.0);
    }

    #[test]
    fn set_clip_is_no_op() {
        let mut b = SvgBackend::new(100, 100);
        b.set_clip(Some(Rect::new(0.0, 0.0, 50.0, 50.0))).unwrap();
        b.set_clip(None).unwrap();
    }

    #[test]
    fn save_png_returns_export_error() {
        let b = SvgBackend::new(20, 20);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.png");
        let r = b.save_png(&path);
        assert!(matches!(r, Err(StarsightError::Export(_))));
    }

    #[test]
    fn save_svg_writes_file() {
        let mut b = SvgBackend::new(20, 20);
        b.fill_rect(Rect::new(0.0, 0.0, 10.0, 10.0), Color::RED)
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.svg");
        b.save_svg(&path).unwrap();
        assert!(path.exists());
    }
}
