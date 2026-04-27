//! Backends: every concrete renderer implements [`DrawBackend`].
//!
//! Sub-modules group backends by output category:
//! - [`rasters`]: CPU pixel rasterization (tiny-skia).
//! - [`vectors`]: vector XML (SVG).
//! - [`prints`]: paged vector for printing (PDF).
//! - [`gpus`]: GPU rendering (wgpu native and web).
//! - [`terminals`]: inline rendering in modern terminals.
//!
//! `DrawBackend` is intentionally object-safe so the chart-building code can
//! store and dispatch over `&mut dyn DrawBackend` regardless of the concrete
//! type. The user picks the backend at runtime by file extension or feature.

pub mod gpus;
pub mod prints;
pub mod rasters;
pub mod terminals;
pub mod vectors;

use crate::errors::Result;
use crate::paths::{Path, PathStyle};
use crate::primitives::{Color, Point, Rect};

// ── DrawBackend ──────────────────────────────────────────────────────────────────────────────────

/// Object-safe trait every renderer implements.
///
/// Method shape rules: no generics, no `Self` in return position, no `Sized`
/// bound. This keeps the trait usable through `&mut dyn DrawBackend`, which is
/// how the chart pipeline dispatches without knowing the concrete backend.
pub trait DrawBackend {
    /// Stroke and/or fill `path` according to `style`.
    ///
    /// # Errors
    /// Returns [`StarsightError::Render`](crate::errors::StarsightError::Render) if
    /// the backend cannot tessellate, allocate, or otherwise process the path.
    fn draw_path(&mut self, path: &Path, style: &PathStyle) -> Result<()>;

    /// Render `text` at `position` with the given size and color.
    ///
    /// # Errors
    /// Returns [`StarsightError::Render`](crate::errors::StarsightError::Render) if
    /// font shaping or glyph rasterization fails.
    fn draw_text(
        &mut self,
        text: &str,
        position: Point,
        font_size: f32,
        color: Color,
    ) -> Result<()>;

    /// Measure text extent (width, height) at the given font size.
    ///
    /// Returns `(width, height)` in pixels.
    ///
    /// # Errors
    /// Returns [`StarsightError::Render`](crate::errors::StarsightError::Render) if
    /// font shaping fails.
    fn text_extent(&mut self, text: &str, font_size: f32) -> Result<(f32, f32)>;

    /// Render rotated text at `position`.
    ///
    /// Rotation is in degrees clockwise. 0 = normal, 90 = rotated 90° clockwise.
    ///
    /// # Errors
    /// Returns [`StarsightError::Render`](crate::errors::StarsightError::Render) if
    /// font shaping or backend rendering fails.
    fn draw_rotated_text(
        &mut self,
        text: &str,
        position: Point,
        font_size: f32,
        color: Color,
        rotation: f32,
    ) -> Result<()>;

    /// Set the clip rectangle. `None` clears the clip.
    ///
    /// # Errors
    /// Returns [`StarsightError::Render`](crate::errors::StarsightError::Render) if
    /// the backend cannot construct the clip mask (e.g. invalid dimensions).
    fn set_clip(&mut self, rect: Option<Rect>) -> Result<()>;

    /// Output dimensions in pixels.
    fn dimensions(&self) -> (u32, u32);

    /// Save the current state to a PNG file.
    ///
    /// # Errors
    /// Returns [`StarsightError::Export`](crate::errors::StarsightError::Export)
    /// for non-raster backends or for I/O failures while writing the file.
    fn save_png(&self, path: &std::path::Path) -> Result<()>;

    /// Save the current state to an SVG file.
    ///
    /// # Errors
    /// Returns [`StarsightError::Export`](crate::errors::StarsightError::Export)
    /// for non-vector backends or for I/O failures while writing the file.
    fn save_svg(&self, path: &std::path::Path) -> Result<()>;

    /// Fill an axis-aligned rectangle with a solid color.
    ///
    /// # Errors
    /// Returns [`StarsightError::Render`](crate::errors::StarsightError::Render) if
    /// `rect` has non-positive width/height or the backend rejects the call.
    fn fill_rect(&mut self, rect: Rect, color: Color) -> Result<()>;
}
