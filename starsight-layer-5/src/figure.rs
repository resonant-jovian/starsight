use starsight_layer_1::backend::DrawBackend;
use starsight_layer_1::backend::skia::SkiaBackend;
use starsight_layer_1::error::Result;
use starsight_layer_1::primitives::color::Color;
use starsight_layer_1::primitives::geom::Rect;
use starsight_layer_2::axis::Axis;
use starsight_layer_2::coord::CartesianCoord;
use starsight_layer_3::mark::{DataExtent, Mark};

pub struct Figure {
    marks: Vec<Box<dyn Mark>>,
    pub title: Option<String>,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub width: u32,
    pub height: u32,
}

impl Figure {
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

    pub fn title(mut self, s: impl Into<String>) -> Self {
        self.title = Some(s.into());
        self
    }

    pub fn x_label(mut self, s: impl Into<String>) -> Self {
        self.x_label = Some(s.into());
        self
    }

    pub fn y_label(mut self, s: impl Into<String>) -> Self {
        self.y_label = Some(s.into());
        self
    }

    pub fn add(mut self, mark: impl Mark + 'static) -> Self {
        self.marks.push(Box::new(mark));
        self
    }

    /// Convenience constructor for the `plot!` macro.
    pub fn from_arrays(x: impl IntoIterator<Item = f64>, y: impl IntoIterator<Item = f64>) -> Self {
        let line = starsight_layer_3::mark::LineMark::new(
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

    pub fn marks(&self) -> &[Box<dyn Mark>] {
        &self.marks
    }

    /// Render the figure to PNG bytes.
    pub fn render_png(&self) -> Result<Vec<u8>> {
        let mut backend = SkiaBackend::new(self.width, self.height)?;
        backend.fill(Color::WHITE);

        let extent = self.merged_extent().ok_or_else(|| {
            starsight_layer_1::error::StarsightError::Data("No data to render".into())
        })?;

        // Margins: left=60, right=20, top=20, bottom=40
        let plot_area = Rect::new(
            60.0,
            20.0,
            self.width as f32 - 20.0,
            self.height as f32 - 40.0,
        );

        let x_vals: Vec<f64> = vec![extent.x_min, extent.x_max];
        let y_vals: Vec<f64> = vec![extent.y_min, extent.y_max];

        let x_axis = Axis::auto_from_data(&x_vals, 5).ok_or_else(|| {
            starsight_layer_1::error::StarsightError::Scale("Cannot build X axis".into())
        })?;
        let y_axis = Axis::auto_from_data(&y_vals, 5).ok_or_else(|| {
            starsight_layer_1::error::StarsightError::Scale("Cannot build Y axis".into())
        })?;

        let coord = CartesianCoord {
            x_axis,
            y_axis,
            plot_area,
        };

        crate::render::render_background(&plot_area, &mut backend)?;
        crate::render::render_axes(&coord, &mut backend)?;

        // Clip marks to plot area
        backend.set_clip(Some(plot_area))?;
        for mark in &self.marks {
            mark.render(&coord, &mut backend)?;
        }
        backend.set_clip(None)?;

        backend.png_bytes()
    }

    /// Save the figure as a PNG file.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let bytes = self.render_png()?;
        std::fs::write(path, bytes).map_err(starsight_layer_1::error::StarsightError::Io)
    }
}
