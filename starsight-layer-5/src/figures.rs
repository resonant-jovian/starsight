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
use starsight_layer_1::theme::Theme;
use starsight_layer_2::axes::Axis;
use starsight_layer_2::coords::CartesianCoord;
use starsight_layer_3::marks::{BarRenderContext, DataExtent, Mark, Orientation};

use crate::layout::{
    LayoutBuilder, LayoutCtx, TitleComponent, XAxisTitleComponent, XTickLabelsComponent,
    YAxisTitleComponent, YTickLabelsComponent,
};

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
    /// Theme for colors.
    pub theme: Theme,
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
            theme: Theme::default(),
        }
    }

    /// Builder: set the theme for colors.
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
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

    /// Check if any marks need bar context (have group, stack, or base set).
    fn has_bar_marks(&self) -> bool {
        self.marks.iter().any(|m| m.as_bar_info().is_some())
    }
    fn compute_bar_context(&self) -> BarRenderContext {
        let mut ctx = BarRenderContext::default();

        // First: check if ANY marks have group or stack (need special rendering)
        let has_any_grouped = self
            .marks
            .iter()
            .any(|m| m.as_bar_info().is_some_and(|(group, _, _)| group.is_some()));
        let has_any_stacked = self
            .marks
            .iter()
            .any(|m| m.as_bar_info().is_some_and(|(_, stack, _)| stack.is_some()));

        if has_any_grouped {
            // For grouped bars: collect all unique groups and calculate total
            let mut groups: Vec<String> = Vec::new();
            for mark in &self.marks {
                if let Some((group, _, _)) = mark.as_bar_info()
                    && let Some(g) = group
                    && !groups.contains(&g.to_string())
                {
                    groups.push(g.to_string());
                }
            }

            // Total groups = count of ALL marks that have group set, not per-group count
            let total_groups = self
                .marks
                .iter()
                .filter(|m| m.as_bar_info().is_some_and(|(g, _, _)| g.is_some()))
                .count() as i32;

            // Assign index to each group
            for (idx, g) in groups.iter().enumerate() {
                ctx.group_offsets
                    .insert(g.clone(), (idx as i32, total_groups));
            }
        }

        if has_any_stacked {
            // For stacked bars: compute cumulative baselines per category
            // We need to iterate in order (first mark adds to baseline 0, second adds to that, etc.)
            // The issue is we can't access BarMark fields from dyn Mark

            // For now: use simple approach - same width as non-stacked, will fix stacking in render_bar
            // ctx.first_pass = false
        }

        ctx
    }

    /// Render marks, passing bar context for grouped/stacked bar rendering.
    fn render_marks(&self, coord: &CartesianCoord, backend: &mut dyn DrawBackend) -> Result<()> {
        let has_any_stacked = self
            .marks
            .iter()
            .any(|m| m.as_bar_info().is_some_and(|(_, stack, _)| stack.is_some()));

        if has_any_stacked {
            // First pass: compute stacked baselines
            let mut ctx = BarRenderContext::default();
            ctx.first_pass = true;
            for mark in &self.marks {
                mark.render_bar(coord, backend, &ctx)?;
            }

            // Second pass: render with accumulated baselines
            let mut ctx = BarRenderContext::default();
            ctx.first_pass = false;
            ctx.stacked_baselines = self.compute_stacked_baselines();
            for mark in &self.marks {
                mark.render_bar(coord, backend, &ctx)?;
            }
        } else if self.has_bar_marks() {
            let bar_ctx = self.compute_bar_context();
            for mark in &self.marks {
                mark.render_bar(coord, backend, &bar_ctx)?;
            }
        } else {
            for mark in &self.marks {
                mark.render(coord, backend)?;
            }
        }
        Ok(())
    }

    /// Compute stacked baselines - returns map of label -> cumulative END position after EACH bar.
    /// This is used by `render_bar` to know where each bar should start.
    fn compute_stacked_baselines(&self) -> std::collections::HashMap<String, f64> {
        let mut baselines = std::collections::HashMap::new();

        // Iterate marks in order - each stacked bar adds to the baseline
        for mark in &self.marks {
            if let Some((_, stack, _)) = mark.as_bar_info()
                && stack.is_some()
                && let Some((labels, values)) = mark.as_bar_data()
            {
                for (label, value) in labels.iter().zip(values.iter()) {
                    if !label.is_empty() && !value.is_nan() {
                        // Get current baseline (where previous stack ended)
                        let current_baseline = *baselines.get(label).unwrap_or(&0.0);
                        // Store the END position (baseline + this bar's value)
                        // This becomes the baseline for the NEXT stacked bar
                        baselines.insert(label.clone(), current_baseline + value);
                    }
                }
            }
        }

        baselines
    }

    /// Get category labels from bar marks (for category axis labels).
    fn category_labels(&self) -> Vec<String> {
        for mark in &self.marks {
            if let Some((labels, _)) = mark.as_bar_data()
                && !labels.is_empty()
            {
                return labels.to_vec();
            }
        }
        vec![]
    }

    /// Check if we have horizontal bar marks (labels go on Y-axis).
    fn has_horizontal_bars(&self) -> bool {
        for mark in &self.marks {
            if let Some((_, _, o)) = mark.as_bar_info()
                && matches!(o, Orientation::Horizontal)
            {
                return true;
            }
        }
        false
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

        let x_vals: Vec<f64> = vec![extent.x_min, extent.x_max];
        let y_vals: Vec<f64> = vec![extent.y_min, extent.y_max];

        let category_labels = self.category_labels();
        let use_y_axis_labels = self.has_horizontal_bars();

        // Bar charts get a category axis on the orientation-appropriate side so
        // bars span the full plot area instead of being squeezed into a Wilkinson-
        // extended numeric range. Numeric axes still use Wilkinson for "nice" ticks.
        let x_axis = if !category_labels.is_empty() && !use_y_axis_labels {
            Axis::category(&category_labels)
        } else {
            Axis::auto_from_data(&x_vals, 5)
                .ok_or_else(|| StarsightError::Scale("Cannot build X axis".into()))?
        };
        let y_axis = if !category_labels.is_empty() && use_y_axis_labels {
            Axis::category(&category_labels)
        } else {
            Axis::auto_from_data(&y_vals, 5)
                .ok_or_else(|| StarsightError::Scale("Cannot build Y axis".into()))?
        };

        let font_size: f32 = 12.0;
        let title_font_size: f32 = 16.0;
        let tick_len: f32 = 5.0;
        let label_gap: f32 = 4.0;

        // Tick-label space comes from category labels when this axis is the
        // category axis, otherwise from the numeric tick labels. The wider of
        // the two is what we need to reserve for either case.
        let x_label_strings: Vec<String> = if !category_labels.is_empty() && !use_y_axis_labels {
            category_labels.clone()
        } else {
            x_axis.tick_labels.clone()
        };
        let y_label_strings: Vec<String> = if !category_labels.is_empty() && use_y_axis_labels {
            category_labels.clone()
        } else {
            y_axis.tick_labels.clone()
        };

        let layout = {
            let ctx = LayoutCtx {
                width: self.width as f32,
                height: self.height as f32,
                backend,
                font_size,
                title_font_size,
                padding: 4.0,
            };
            let mut builder = LayoutBuilder::new(ctx);
            builder.add(&TitleComponent {
                title: self.title.as_deref(),
            });
            builder.add(&XTickLabelsComponent {
                labels: &x_label_strings,
                tick_len,
                gap: label_gap,
            });
            builder.add(&YTickLabelsComponent {
                labels: &y_label_strings,
                tick_len,
                gap: label_gap,
            });
            builder.add(&XAxisTitleComponent {
                label: self.x_label.as_deref(),
                gap: label_gap,
            });
            builder.add(&YAxisTitleComponent {
                label: self.y_label.as_deref(),
                gap: label_gap,
            });
            builder.finish()
        };
        let plot_area = layout.plot_rect;

        let coord = CartesianCoord {
            x_axis,
            y_axis,
            plot_area,
        };

        crate::renders::render_background(&plot_area, backend, &self.theme)?;

        if let Some(title) = &self.title {
            let slot = layout
                .slots
                .get("title")
                .and_then(|v| v.first())
                .copied();
            if let Some(slot) = slot {
                crate::renders::render_title(title, &slot, backend, &self.theme)?;
            }
        }

        let x_axis_title_slot = layout
            .slots
            .get("x_axis_title")
            .and_then(|v| v.first())
            .copied();
        let y_axis_title_slot = layout
            .slots
            .get("y_axis_title")
            .and_then(|v| v.first())
            .copied();
        crate::renders::render_axis_labels(
            self.x_label.as_deref(),
            self.y_label.as_deref(),
            x_axis_title_slot.as_ref(),
            y_axis_title_slot.as_ref(),
            &plot_area,
            backend,
            &self.theme,
        )?;

        backend.set_clip(Some(plot_area))?;
        crate::renders::render_grid_lines(&coord, backend, &self.theme)?;
        self.render_marks(&coord, backend)?;
        backend.set_clip(None)?;

        crate::renders::render_axes(
            &coord,
            backend,
            &category_labels,
            use_y_axis_labels,
            &self.theme,
        )?;

        let legend_entries: Vec<crate::renders::LegendEntry> = self
            .marks
            .iter()
            .filter_map(|mark| {
                if let (Some(color), Some(label)) = (mark.legend_color(), mark.legend_label())
                    && !label.is_empty()
                {
                    return Some(crate::renders::LegendEntry {
                        color,
                        label: label.to_string(),
                    });
                }
                None
            })
            .collect();

        if !legend_entries.is_empty() {
            crate::renders::render_legend(&legend_entries, &plot_area, backend, &self.theme)?;
        }

        Ok(())
    }
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
        backend.fill(self.theme.background);
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
