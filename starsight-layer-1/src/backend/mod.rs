mod svg;
mod skia;
mod wgpu;
mod pdf;
mod terminal;

use crate::error::Result;

pub trait DrawBackend {
    ///
    fn draw_path(&mut self, path: &Path, style: &PathStyle) -> Result<()>;
    ///
    //fn draw_text(&mut self, text: &TextBlock, position: Point) -> Result<()>;
    ///
    //fn draw_image(&mut self, image: &ImageData, rect: Rect) -> Result<()>;
    //fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()>;
    ///
    fn dimensions(&self) -> (u32, u32);
    ///
    fn save_png(&self, path: &std::path::Path) -> Result<()>;
    ///
    fn save_svg(&self, path: &std::path::Path) -> Result<()>;
}
///
pub struct PathStyle {
    ///
    //stroke_color: Color,
    ///
    stroke_width: f32,
    ///
    //fill_color: Color,
    ///
    dash_pattern: Option<(f32, f32)>,
    ///
    //line_cap: LineCap,
    ///
    //line_join: LineJoin,
    ///
    opacity: f32,
}
///
pub type Path = PathCommand;
///
pub enum PathCommand {
    ///
    //MoveTo(Point),
    ///
    //LineTo(Point),
    ///
    //QuadTo(Point, Point),
    ///
    //CubicTo(Point, Point, Point),
    ///
    Close,
}