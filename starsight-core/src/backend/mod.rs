use crate::error::Result;

pub mod pdf;
pub mod svg;
pub mod tiny_skia;

pub trait DrawBackend {
    fn draw_path(&mut self, path: &Path, style: &PathStyle) -> Result<()>;
    //fn draw_text(&mut self, text: &TextBlock, position: Point) -> Result<()>;
    //fn draw_image(&mut self, image: &ImageData, rect: Rect) -> Result<()>;
    fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()>;
    fn dimensions(&self) -> (u32, u32);
    fn save_png(&self, path: &std::path::Path) -> Result<()>;
    fn save_svg(&self, path: &std::path::Path) -> Result<()>;
}

use ::tiny_skia::ColorU8;

pub struct PathStyle {

}

pub type Path = PathCommand;

pub struct PathCommand {

}
pub struct Point {}
pub struct Size {}
pub struct Rect {}


pub type Color = ColorU8;
