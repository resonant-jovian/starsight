//! Figure: the top-level builder users assemble marks into.
//!
//! `Figure` owns the marks (`Vec<Box<dyn Mark>>`) and rendering parameters
//! (size, title, axis labels). It exposes a chainable builder API and a
//! `save` method that dispatches to the appropriate backend by file extension.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::should_implement_trait
)]

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::backends::rasters::SkiaBackend;
use starsight_layer_1::backends::vectors::SvgBackend;
use starsight_layer_1::errors::{Result, StarsightError};
use starsight_layer_1::primitives::{Color, Rect};
use starsight_layer_2::axes::Axis;
use starsight_layer_2::coords::CartesianCoord;
use starsight_layer_3::marks::{DataExtent, Mark};

// ── Figure ───────────────────────────────────────────────────────────────────────────────────────

/// Top-level chart builder.
pub struct Figure {
    marks: Vec<Box<dyn Mark>>,
    /// Optional chart title rendered above the plot area.
    pub title: Option<String>,
    /// Optional x-axis label.
    pub x_label: Option<String>,
    /// Optional y-axis label.
    pub y_label: Option<String>,
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
}

impl Figure {
    /// Empty figure of `width × height` pixels.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            marks: Vec::new(),
            title: None,
            x_label: None,
            y_label: None,
            width,
            height,
        }
    }

    /// Builder: set chart title.
    #[must_use]
    pub fn title(mut self, s: impl Into<String>) -> Self {
        self.title = Some(s.into());
        self
    }

    /// Builder: set the x-axis label.
    #[must_use]
    pub fn x_label(mut self, s: impl Into<String>) -> Self {
        self.x_label = Some(s.into());
        self
    }

    /// Builder: set the y-axis label.
    #[must_use]
    pub fn y_label(mut self, s: impl Into<String>) -> Self {
        self.y_label = Some(s.into());
        self
    }

    /// Append a mark.
    #[must_use]
    pub fn add(mut self, mark: impl Mark + 'static) -> Self {
        self.marks.push(Box::new(mark));
        self
    }

    /// Convenience constructor used by the `starsight::plot!` macro.
    pub fn from_arrays(x: impl IntoIterator<Item = f64>, y: impl IntoIterator<Item = f64>) -> Self {
        let line = starsight_layer_3::marks::LineMark::new(
            x.into_iter().collect(),
            y.into_iter().collect(),
        );
        Self::new(800, 600).add(line)
    }

    /// Compute the merged data extent across all marks.
    fn merged_extent(&self) -> Option<DataExtent> {
        let mut merged: Option<DataExtent> = None;
        for mark in &self.marks {
            if let Some(ext) = mark.data_extent() {
                merged = Some(match merged {
                    None => ext,
                    Some(m) => DataExtent {
                        x_min: m.x_min.min(ext.x_min),
                        x_max: m.x_max.max(ext.x_max),
                        y_min: m.y_min.min(ext.y_min),
                        y_max: m.y_max.max(ext.y_max),
                    },
                });
            }
        }
        merged
    }

    /// Borrow the underlying mark list.
    #[must_use]
    pub fn marks(&self) -> &[Box<dyn Mark>] {
        &self.marks
    }

    /// Shared render path used by both [`render_png`](Self::render_png) and
    /// [`render_svg`](Self::render_svg). Computes the data extent, builds the
    /// axes and coordinate system, then dispatches drawing to the supplied
    /// backend trait object.
    fn render_to(&self, backend: &mut dyn DrawBackend) -> Result<()> {
        let extent = self
            .merged_extent()
            .ok_or_else(|| StarsightError::Data("No data to render".into()))?;

        // Margins: left=60, right=20, top=20, bottom=40.
        let plot_area = Rect::new(
            60.0,
            20.0,
            self.width as f32 - 20.0,
            self.height as f32 - 40.0,
        );

        let x_vals: Vec<f64> = vec![extent.x_min, extent.x_max];
        let y_vals: Vec<f64> = vec![extent.y_min, extent.y_max];

        let x_axis = Axis::auto_from_data(&x_vals, 5)
            .ok_or_else(|| StarsightError::Scale("Cannot build X axis".into()))?;
        let y_axis = Axis::auto_from_data(&y_vals, 5)
            .ok_or_else(|| StarsightError::Scale("Cannot build Y axis".into()))?;

        let coord = CartesianCoord {
            x_axis,
            y_axis,
            plot_area,
        };

        crate::renders::render_background(&plot_area, backend)?;
        crate::renders::render_axes(&coord, backend)?;

        backend.set_clip(Some(plot_area))?;
        for mark in &self.marks {
            mark.render(&coord, backend)?;
        }
        backend.set_clip(None)?;

        Ok(())
    }

    /// Render the figure to in-memory PNG bytes via the raster backend.
    ///
    /// # Errors
    /// - [`StarsightError::Render`](starsight_layer_1::errors::StarsightError::Render)
    ///   if the backend fails to allocate or draw.
    /// - [`StarsightError::Data`](starsight_layer_1::errors::StarsightError::Data)
    ///   if no marks have any data.
    /// - [`StarsightError::Scale`](starsight_layer_1::errors::StarsightError::Scale)
    ///   if axes cannot be built from the data extent.
    /// - [`StarsightError::Export`](starsight_layer_1::errors::StarsightError::Export)
    ///   if PNG encoding fails.
    pub fn render_png(&self) -> Result<Vec<u8>> {
        let mut backend = SkiaBackend::new(self.width, self.height)?;
        backend.fill(Color::WHITE);
        self.render_to(&mut backend)?;
        backend.png_bytes()
    }

    /// Render the figure as an in-memory SVG document.
    ///
    /// SVG output keeps text as `<text>` elements (no glyph rasterization), so
    /// the result is deterministic across operating systems and font setups —
    /// which makes it the right format for snapshot tests in CI.
    ///
    /// # Errors
    /// - [`StarsightError::Data`](starsight_layer_1::errors::StarsightError::Data)
    ///   if no marks have any data.
    /// - [`StarsightError::Scale`](starsight_layer_1::errors::StarsightError::Scale)
    ///   if axes cannot be built from the data extent.
    /// - [`StarsightError::Render`](starsight_layer_1::errors::StarsightError::Render)
    ///   if the backend rejects a draw call.
    pub fn render_svg(&self) -> Result<String> {
        let mut backend = SvgBackend::new(self.width, self.height);
        // SvgBackend has no `fill()` (it's not a pixmap); the background
        // `fill_rect` issued by `render_background` covers the canvas.
        self.render_to(&mut backend)?;
        Ok(backend.svg_string())
    }

    /// Save the figure to a file. The format is chosen by extension: `.png`
    /// uses the raster backend; `.svg` uses the vector backend.
    ///
    /// # Errors
    /// - Any error from [`render_png`](Self::render_png) or
    ///   [`render_svg`](Self::render_svg).
    /// - [`StarsightError::Io`](starsight_layer_1::errors::StarsightError::Io)
    ///   if writing the file fails.
    /// - [`StarsightError::Export`](starsight_layer_1::errors::StarsightError::Export)
    ///   if the extension is unsupported.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let path = path.as_ref();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(str::to_ascii_lowercase);

        match ext.as_deref() {
            Some("svg") => {
                let svg = self.render_svg()?;
                std::fs::write(path, svg).map_err(StarsightError::Io)
            }
            Some("png") | None => {
                let bytes = self.render_png()?;
                std::fs::write(path, bytes).map_err(StarsightError::Io)
            }
            Some(other) => Err(StarsightError::Export(format!(
                "unsupported file extension: .{other}"
            ))),
        }
    }
}
