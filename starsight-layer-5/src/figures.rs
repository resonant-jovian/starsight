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
use starsight_layer_1::primitives::Rect;
use starsight_layer_1::theme::Theme;
use starsight_layer_2::axes::Axis;
use starsight_layer_2::coords::{CartesianCoord, PolarCoord};
use starsight_layer_3::marks::{BarRenderContext, DataExtent, Mark, Orientation};

use crate::layout::{
    LayoutBuilder, LayoutCtx, Slot, TitleComponent, XAxisTitleComponent, XTickLabelsComponent,
    YAxisTitleComponent, YTickLabelsComponent,
};

/// Outer canvas padding used by every single-figure render path.
///
/// Bumped from 4.0 to 8.0 (matches `MultiPanelFigure::padding` default) so
/// every chart kind has a consistent breathable margin between the canvas
/// edge and the layout slot stack — fixes the inconsistent outer-margin
/// issue tracked as `starsight-c6h`.
pub(crate) const DEFAULT_FIGURE_PADDING_PX: f32 = 8.0;

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
    /// Polar mode: when set, the figure renders into a [`PolarCoord`] instead
    /// of a [`CartesianCoord`]. The `(theta_axis, r_axis)` tuple replaces the
    /// auto-inferred cartesian axes; only polar marks (e.g. `ArcMark`,
    /// `RadarMark`) accept this coord and cartesian marks return Config errors.
    pub polar_axes: Option<(Axis, Axis)>,
    /// Colorbar opt-out flag. When `true`, suppresses the auto-attached
    /// colormap legend that `HeatmapMark` and `ContourMark` would otherwise
    /// trigger. Default is `false` (auto-attach). Tracked as `starsight-kdi`.
    pub colorbar_disabled: bool,
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
            polar_axes: None,
            colorbar_disabled: false,
        }
    }

    /// Builder: opt out of the auto-attached colorbar that
    /// `HeatmapMark` and `ContourMark` would otherwise trigger. Pass
    /// `false` to suppress, `true` to keep auto-attach (the default).
    /// Tracked as `starsight-kdi`.
    #[must_use]
    pub fn colorbar(mut self, on: bool) -> Self {
        self.colorbar_disabled = !on;
        self
    }

    /// Builder: switch the figure into polar mode using the supplied
    /// angular and radial axes. Pair with polar marks (e.g.
    /// [`crate::Figure::add`] of an `ArcMark` or `RadarMark`); cartesian
    /// marks added to a polar figure error at render time.
    ///
    /// Polar figures bypass the cartesian tick / axis-title chrome — the
    /// inscribed disk fills the available rect, with only `title` honored
    /// from the standard chrome set. Grid lines and legend draw via
    /// [`crate::renders::render_grid_lines`]'s polar branch and the
    /// existing legend stack respectively (the legend overlaps the disk
    /// in 0.3.0 per the documented limitation).
    #[must_use]
    pub fn polar_axes(mut self, theta_axis: Axis, r_axis: Axis) -> Self {
        self.polar_axes = Some((theta_axis, r_axis));
        self
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
            let ctx = BarRenderContext {
                first_pass: true,
                ..BarRenderContext::default()
            };
            for mark in &self.marks {
                mark.render_bar(coord, backend, &ctx)?;
            }

            // Second pass: render with accumulated baselines
            let ctx = BarRenderContext {
                first_pass: false,
                stacked_baselines: self.compute_stacked_baselines(),
                ..BarRenderContext::default()
            };
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
        let viewport = Rect::new(0.0, 0.0, self.width as f32, self.height as f32);
        self.render_within(viewport, backend)
    }

    /// Render the figure into a sub-rect of the backend. Used by
    /// [`MultiPanelFigure`] to draw each panel into its assigned grid cell;
    /// callers rendering a standalone figure go through [`render_to`] which
    /// passes a viewport covering the full canvas.
    ///
    /// The layout is computed in viewport-local coords (so a 200×200 panel
    /// composes its own legend, ticks, and title independent of the parent
    /// canvas size), then every emitted rect is translated by the viewport
    /// origin before drawing.
    pub(crate) fn render_within(
        &self,
        viewport: Rect,
        backend: &mut dyn DrawBackend,
    ) -> Result<()> {
        if self.polar_axes.is_some() {
            return self.render_polar_within(viewport, backend);
        }
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

        let fonts = crate::layout::LayoutFonts::default();
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

        // Auto-attach colorbar: if any mark exposes a colormap legend
        // (HeatmapMark, ContourMark with a colormap) and the figure has not
        // opted out via `Figure::colorbar(false)`, build a Colorbar that
        // takes a Right-side slot. Tracked as `starsight-kdi`.
        let colorbar_opt: Option<crate::colorbar::Colorbar> = if self.colorbar_disabled {
            None
        } else {
            self.marks
                .iter()
                .find_map(|m| m.colormap_legend())
                .map(crate::colorbar::Colorbar::new)
        };

        let layout = {
            let ctx = LayoutCtx {
                width: viewport.width(),
                height: viewport.height(),
                backend,
                fonts,
                padding: DEFAULT_FIGURE_PADDING_PX,
            };
            let mut builder = LayoutBuilder::new(ctx);
            builder.add(&TitleComponent {
                title: self.title.as_deref(),
            });
            // Categorical x-axes pass band_width so the layout (and the
            // renderer that mirrors the same rotation decision) can
            // reserve enough vertical space for rotated labels when they
            // would crowd horizontally.
            let x_band_width = if !category_labels.is_empty() && !use_y_axis_labels {
                #[allow(clippy::cast_precision_loss)]
                let n = category_labels.len() as f32;
                if n > 0.0 {
                    Some(viewport.width() / n)
                } else {
                    None
                }
            } else {
                None
            };
            builder.add(&XTickLabelsComponent {
                labels: &x_label_strings,
                tick_len,
                gap: label_gap,
                band_width: x_band_width,
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
            if let Some(colorbar) = &colorbar_opt {
                builder.add(&crate::colorbar::ColorbarComponent { colorbar });
            }
            builder.finish()
        };

        // All layout rects are in viewport-local coords. Translate them into
        // backend-absolute coords so MultiPanelFigure's per-panel viewport
        // offsets land each panel correctly. Identity for full-canvas renders.
        let dx = viewport.left;
        let dy = viewport.top;
        let translate_rect =
            |r: Rect| Rect::new(r.left + dx, r.top + dy, r.right + dx, r.bottom + dy);
        let translate_slot = |s: Slot| Slot {
            rect: translate_rect(s.rect),
            side: s.side,
        };
        let plot_area = translate_rect(layout.plot_rect);

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
                .copied()
                .map(translate_slot);
            if let Some(slot) = slot {
                crate::renders::render_title(
                    title,
                    &slot,
                    &plot_area,
                    backend,
                    &self.theme,
                    &fonts,
                )?;
            }
        }

        let x_axis_title_slot = layout
            .slots
            .get("x_axis_title")
            .and_then(|v| v.first())
            .copied()
            .map(translate_slot);
        let y_axis_title_slot = layout
            .slots
            .get("y_axis_title")
            .and_then(|v| v.first())
            .copied()
            .map(translate_slot);
        crate::renders::render_axis_labels(
            self.x_label.as_deref(),
            self.y_label.as_deref(),
            x_axis_title_slot.as_ref(),
            y_axis_title_slot.as_ref(),
            &plot_area,
            backend,
            &self.theme,
            &fonts,
        )?;

        // Marks like PieMark are angular and want no Cartesian axis chrome.
        // Skip render_grid_lines + render_axes only when *every* mark on the
        // figure agrees — mixed charts keep the axes for the others (yrp.2).
        let suppress_axes = !self.marks.is_empty() && self.marks.iter().all(|m| !m.wants_axes());

        backend.set_clip(Some(plot_area))?;
        if !suppress_axes {
            crate::renders::render_grid_lines(&coord, backend, &self.theme)?;
        }
        self.render_marks(&coord, backend)?;
        backend.set_clip(None)?;

        if !suppress_axes {
            crate::renders::render_axes(
                &coord,
                backend,
                &category_labels,
                use_y_axis_labels,
                &self.theme,
                &fonts,
            )?;
        }

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
                        glyph: mark.legend_glyph(),
                    });
                }
                None
            })
            .collect();

        if !legend_entries.is_empty() {
            crate::renders::render_legend(
                &legend_entries,
                &plot_area,
                backend,
                &self.theme,
                &fonts,
            )?;
        }

        // Auto-attached colorbar lands after marks/axes/legend so it sits
        // on top of any background fill and never falls behind axis chrome.
        if let Some(colorbar) = &colorbar_opt {
            let slot = layout
                .slots
                .get("colorbar")
                .and_then(|v| v.first())
                .copied()
                .map(translate_slot);
            if let Some(slot) = slot {
                crate::colorbar::render_colorbar(
                    colorbar,
                    &slot,
                    &plot_area,
                    backend,
                    &self.theme,
                    &fonts,
                )?;
            }
        }

        Ok(())
    }

    /// Polar-mode render path. Mirrors [`render_within`](Self::render_within)
    /// but builds a [`PolarCoord`] inscribed in the viewport (minus a small
    /// title strip) instead of a [`CartesianCoord`], and skips the cartesian
    /// tick / axis chrome.
    fn render_polar_within(&self, viewport: Rect, backend: &mut dyn DrawBackend) -> Result<()> {
        let (theta_axis, r_axis) = self
            .polar_axes
            .as_ref()
            .expect("render_polar_within called without polar_axes set");

        let fonts = crate::layout::LayoutFonts::default();

        // Reserve only top space for the title; everything else is the
        // inscribed disk.
        let layout = {
            let ctx = LayoutCtx {
                width: viewport.width(),
                height: viewport.height(),
                backend,
                fonts,
                padding: DEFAULT_FIGURE_PADDING_PX,
            };
            let mut builder = LayoutBuilder::new(ctx);
            builder.add(&TitleComponent {
                title: self.title.as_deref(),
            });
            builder.finish()
        };

        let dx = viewport.left;
        let dy = viewport.top;
        let translate_rect =
            |r: Rect| Rect::new(r.left + dx, r.top + dy, r.right + dx, r.bottom + dy);
        let translate_slot = |s: Slot| Slot {
            rect: translate_rect(s.rect),
            side: s.side,
        };
        let plot_area = translate_rect(layout.plot_rect);

        let coord = PolarCoord::inscribed(theta_axis.clone(), r_axis.clone(), plot_area);

        crate::renders::render_background(&plot_area, backend, &self.theme)?;

        if let Some(title) = &self.title {
            let slot = layout
                .slots
                .get("title")
                .and_then(|v| v.first())
                .copied()
                .map(translate_slot);
            if let Some(slot) = slot {
                crate::renders::render_title(
                    title,
                    &slot,
                    &plot_area,
                    backend,
                    &self.theme,
                    &fonts,
                )?;
            }
        }

        backend.set_clip(Some(plot_area))?;
        crate::renders::render_grid_lines(&coord, backend, &self.theme)?;
        for mark in &self.marks {
            mark.render(&coord, backend)?;
        }
        backend.set_clip(None)?;

        // Legend: same dispatch as cartesian. Will overlap the disk per
        // documented limitation; reposition in 0.4.0.
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
                        glyph: mark.legend_glyph(),
                    });
                }
                None
            })
            .collect();

        if !legend_entries.is_empty() {
            crate::renders::render_legend(
                &legend_entries,
                &plot_area,
                backend,
                &self.theme,
                &fonts,
            )?;
        }

        Ok(())
    }
    ///
    /// # Errors
    /// - [`starsight_layer_1::errors::StarsightError::Render`]
    ///   if the backend fails to allocate or draw.
    /// - [`starsight_layer_1::errors::StarsightError::Data`]
    ///   if no marks have any data.
    /// - [`starsight_layer_1::errors::StarsightError::Scale`]
    ///   if axes cannot be built from the data extent.
    /// - [`starsight_layer_1::errors::StarsightError::Export`]
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
    /// - [`starsight_layer_1::errors::StarsightError::Data`]
    ///   if no marks have any data.
    /// - [`starsight_layer_1::errors::StarsightError::Scale`]
    ///   if axes cannot be built from the data extent.
    /// - [`starsight_layer_1::errors::StarsightError::Render`]
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
    /// - [`starsight_layer_1::errors::StarsightError::Io`]
    ///   if writing the file fails.
    /// - [`starsight_layer_1::errors::StarsightError::Export`]
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

// ── MultiPanelFigure ─────────────────────────────────────────────────────────────────────────────

/// Grid of independent [`Figure`] panels rendered into a single canvas.
///
/// Each panel keeps its own marks, axes, title, and legend; layout-wise the
/// canvas is split into a uniform `rows × cols` grid with `padding` pixels of
/// gap between panels (and the same padding around the outer edge). Per-panel
/// `width`/`height` are ignored — the parent canvas dimensions decide the cell
/// size — so panels constructed at any nominal size compose cleanly.
///
/// 0.3.0 limitation: each panel computes its axes independently. Shared axes
/// across rows/columns (and a per-row/per-column title) land in 0.4.0.
pub struct MultiPanelFigure {
    panels: Vec<Figure>,
    /// Number of grid rows.
    pub rows: u32,
    /// Number of grid columns.
    pub cols: u32,
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// Padding (px) between panels and around the outer canvas edge.
    pub padding: f32,
    /// Background theme for the canvas (each panel keeps its own theme).
    pub theme: Theme,
}

impl MultiPanelFigure {
    /// New empty `rows × cols` grid sized to `width × height` pixels.
    #[must_use]
    pub fn new(width: u32, height: u32, rows: u32, cols: u32) -> Self {
        Self {
            panels: Vec::new(),
            rows,
            cols,
            width,
            height,
            padding: 8.0,
            theme: Theme::default(),
        }
    }

    /// Builder: append one panel. Panels fill the grid in row-major order
    /// (top-left first, then across, then down).
    #[must_use]
    pub fn add(mut self, panel: Figure) -> Self {
        self.panels.push(panel);
        self
    }

    /// Builder: padding (px) between panels and around the canvas edge.
    #[must_use]
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Builder: canvas background theme.
    #[must_use]
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Borrow the underlying panel list.
    #[must_use]
    pub fn panels(&self) -> &[Figure] {
        &self.panels
    }

    /// Allocate the rect for the panel at `(row, col)` within the canvas.
    fn panel_rect(&self, row: u32, col: u32) -> Rect {
        let rows = self.rows.max(1) as f32;
        let cols = self.cols.max(1) as f32;
        let w = self.width as f32;
        let h = self.height as f32;
        let pad = self.padding;
        let cell_w = ((w - pad * (cols + 1.0)) / cols).max(1.0);
        let cell_h = ((h - pad * (rows + 1.0)) / rows).max(1.0);
        let left = pad + col as f32 * (cell_w + pad);
        let top = pad + row as f32 * (cell_h + pad);
        Rect::new(left, top, left + cell_w, top + cell_h)
    }

    fn render_to(&self, backend: &mut dyn DrawBackend) -> Result<()> {
        let canvas = Rect::new(0.0, 0.0, self.width as f32, self.height as f32);
        crate::renders::render_background(&canvas, backend, &self.theme)?;
        for (idx, panel) in self.panels.iter().enumerate() {
            let row = idx as u32 / self.cols.max(1);
            let col = idx as u32 % self.cols.max(1);
            if row >= self.rows {
                break; // Extra panels past rows × cols are ignored.
            }
            let viewport = self.panel_rect(row, col);
            panel.render_within(viewport, backend)?;
        }
        Ok(())
    }

    /// Render the panel grid to a PNG byte buffer.
    ///
    /// # Errors
    /// - [`StarsightError::Render`] if the backend fails.
    /// - [`StarsightError::Data`] if any panel has no data (panels error
    ///   independently; the first failure short-circuits).
    /// - [`StarsightError::Scale`] if any panel cannot build its axes.
    /// - [`StarsightError::Export`] if PNG encoding fails.
    pub fn render_png(&self) -> Result<Vec<u8>> {
        let mut backend = SkiaBackend::new(self.width, self.height)?;
        backend.fill(self.theme.background);
        self.render_to(&mut backend)?;
        backend.png_bytes()
    }

    /// Render the panel grid to an in-memory SVG string.
    ///
    /// # Errors
    /// Same conditions as [`render_png`](Self::render_png), minus the PNG
    /// encode step.
    pub fn render_svg(&self) -> Result<String> {
        let mut backend = SvgBackend::new(self.width, self.height);
        self.render_to(&mut backend)?;
        Ok(backend.svg_string())
    }

    /// Save the panel grid to disk; format chosen by extension (`.png` /
    /// `.svg`).
    ///
    /// # Errors
    /// - Any error from [`render_png`](Self::render_png) or
    ///   [`render_svg`](Self::render_svg).
    /// - [`StarsightError::Io`] if writing fails.
    /// - [`StarsightError::Export`] if the extension is unsupported.
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

#[cfg(test)]
mod tests {
    use super::{Figure, MultiPanelFigure};
    use starsight_layer_1::errors::StarsightError;
    use starsight_layer_1::primitives::Color;
    use starsight_layer_1::theme::DEFAULT_DARK;
    use starsight_layer_3::marks::{BarMark, LineMark};

    #[test]
    fn theme_builder_sets_theme() {
        let fig = Figure::new(100, 100).theme(DEFAULT_DARK);
        assert!(fig.theme.is_dark);
    }

    #[test]
    fn marks_accessor_returns_added_marks() {
        let fig = Figure::new(100, 100).add(LineMark::new(vec![0.0], vec![0.0]));
        assert_eq!(fig.marks().len(), 1);
    }

    #[test]
    fn from_arrays_creates_line_chart() {
        let fig = Figure::from_arrays([0.0, 1.0, 2.0], [0.0, 1.0, 4.0]);
        assert_eq!(fig.width, 800);
        assert_eq!(fig.height, 600);
        assert_eq!(fig.marks().len(), 1);
    }

    #[test]
    fn render_png_with_no_marks_errors() {
        let fig = Figure::new(100, 100);
        let result = fig.render_png();
        assert!(matches!(result, Err(StarsightError::Data(_))));
    }

    #[test]
    fn render_svg_with_legend_entry() {
        // Mark with a non-empty label triggers the legend code path
        let fig = Figure::new(400, 300).add(
            LineMark::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 4.0])
                .label("series1")
                .color(Color::RED),
        );
        let svg = fig.render_svg().unwrap();
        assert!(svg.contains("series1"));
    }

    #[test]
    fn render_svg_with_horizontal_bars_uses_y_category_axis() {
        let fig = Figure::new(400, 300)
            .add(BarMark::new(vec!["a".into(), "b".into()], vec![1.0, 2.0]).horizontal());
        let svg = fig.render_svg().unwrap();
        assert!(svg.contains('a'));
    }

    #[test]
    fn render_svg_with_grouped_bars() {
        let fig = Figure::new(400, 300)
            .add(BarMark::new(vec!["a".into(), "b".into()], vec![1.0, 2.0]).group("g1"))
            .add(BarMark::new(vec!["a".into(), "b".into()], vec![3.0, 4.0]).group("g2"));
        let svg = fig.render_svg().unwrap();
        assert!(!svg.is_empty());
    }

    #[test]
    fn render_svg_with_stacked_bars() {
        let fig = Figure::new(400, 300)
            .add(BarMark::new(vec!["a".into(), "b".into()], vec![1.0, 2.0]).stack("s"))
            .add(BarMark::new(vec!["a".into(), "b".into()], vec![3.0, 4.0]).stack("s"));
        let svg = fig.render_svg().unwrap();
        assert!(!svg.is_empty());
    }

    #[test]
    fn save_to_svg_writes_file() {
        let fig = Figure::new(100, 100).add(LineMark::new(vec![0.0, 1.0], vec![0.0, 1.0]));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.svg");
        fig.save(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn save_to_png_writes_file() {
        let fig = Figure::new(100, 100).add(LineMark::new(vec![0.0, 1.0], vec![0.0, 1.0]));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.png");
        fig.save(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn save_to_no_extension_defaults_to_png() {
        let fig = Figure::new(100, 100).add(LineMark::new(vec![0.0, 1.0], vec![0.0, 1.0]));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out_noext");
        fig.save(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn save_to_unsupported_extension_errors() {
        let fig = Figure::new(100, 100).add(LineMark::new(vec![0.0, 1.0], vec![0.0, 1.0]));
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.bmp");
        let r = fig.save(&path);
        assert!(matches!(r, Err(StarsightError::Export(_))));
    }

    // ── MultiPanelFigure ─────────────────────────────────────────────────

    fn line_panel() -> Figure {
        Figure::new(200, 200).add(LineMark::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 4.0]))
    }

    #[test]
    fn multi_panel_new_has_no_panels() {
        let mp = MultiPanelFigure::new(400, 300, 2, 2);
        assert!(mp.panels().is_empty());
        assert_eq!(mp.rows, 2);
        assert_eq!(mp.cols, 2);
    }

    #[test]
    fn multi_panel_add_appends() {
        let mp = MultiPanelFigure::new(400, 300, 1, 2)
            .add(line_panel())
            .add(line_panel());
        assert_eq!(mp.panels().len(), 2);
    }

    #[test]
    fn multi_panel_padding_builder() {
        let mp = MultiPanelFigure::new(400, 300, 2, 2).padding(20.0);
        assert!((mp.padding - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn multi_panel_rect_partitions_canvas() {
        // 400×300 canvas, 2×2 grid, 8px padding → cell ≈ 188×134.
        let mp = MultiPanelFigure::new(400, 300, 2, 2);
        let r00 = mp.panel_rect(0, 0);
        let r01 = mp.panel_rect(0, 1);
        let r10 = mp.panel_rect(1, 0);
        // Same cell size everywhere.
        assert!((r00.width() - r01.width()).abs() < 1e-3);
        assert!((r00.height() - r10.height()).abs() < 1e-3);
        // Top-left starts at padding offset.
        assert!((r00.left - 8.0).abs() < 1e-3);
        assert!((r00.top - 8.0).abs() < 1e-3);
        // Right column shifted by cell + padding.
        assert!(r01.left > r00.right);
    }

    #[test]
    fn multi_panel_render_svg_with_two_panels_succeeds() {
        let mp = MultiPanelFigure::new(600, 400, 1, 2)
            .add(line_panel())
            .add(line_panel());
        let svg = mp.render_svg().unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn multi_panel_save_svg_writes_file() {
        let mp = MultiPanelFigure::new(400, 300, 1, 1).add(line_panel());
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("multi.svg");
        mp.save(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn multi_panel_extra_panels_past_grid_are_ignored() {
        // 1x1 grid with 3 panels — only the first should render; the rest are
        // dropped (no panic, no error).
        let mp = MultiPanelFigure::new(200, 200, 1, 1)
            .add(line_panel())
            .add(line_panel())
            .add(line_panel());
        let svg = mp.render_svg().unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn multi_panel_empty_panel_errors() {
        // A panel with no marks fails (StarsightError::Data).
        let mp = MultiPanelFigure::new(200, 200, 1, 1).add(Figure::new(200, 200));
        assert!(matches!(mp.render_svg(), Err(StarsightError::Data(_))));
    }
}
