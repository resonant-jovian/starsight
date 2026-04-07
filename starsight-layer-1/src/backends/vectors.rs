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
                PathCommand::Close => {
                    data = data.close();
                }
                PathCommand::QuadTo(_, _) => {}
            }
        }
        let p = SvgPath::new()
            .set("d", data)
            .set("stroke", style.stroke_color.to_css_hex())
            .set("stroke-width", style.stroke_width)
            .set(
                "fill",
                style
                    .fill_color
                    .map_or("none".to_string(), Color::to_css_hex),
            );
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
            .set("fill", color.to_css_hex());
        self.elements.push(Box::new(r));
        Ok(())
    }
}
