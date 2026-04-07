use crate::backend::{DrawBackend, PathCommand};
use crate::error::{Result, StarsightError};
use crate::primitives::{
    color::Color,
    geom::{Point, Rect},
};
use svg::Document;
use svg::node::element::{Path as SvgPath, Rectangle, Text as SvgText};
// -------------------------------------------------------------------------------------------------
pub struct SvgBackend {
    width: u32,
    height: u32,
    elements: Vec<Box<dyn svg::Node>>,
    //clip_id: usize,
}
// -------------------------------------------------------------------------------------------------
impl SvgBackend {
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            elements: Vec::new(),
            //clip_id: 0,
        }
    }

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

    #[must_use]
    pub fn svg_string(&self) -> String {
        self.build_document().to_string()
    }
}
// -------------------------------------------------------------------------------------------------
impl DrawBackend for SvgBackend {
    fn draw_path(
        &mut self,
        path: &crate::backend::Path,
        style: &crate::backend::PathStyle,
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
                _ => {}
            }
        }
        let p = SvgPath::new()
            .set("d", data)
            .set("stroke", style.stroke_color.to_css_hex())
            .set("stroke-width", style.stroke_width)
            .set(
                "fill",
                style.fill_color.map_or(
                    "none".to_string(),
                    super::super::primitives::color::Color::to_css_hex,
                ),
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
        todo!()
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
