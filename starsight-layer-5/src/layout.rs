//! Layout: compose the plot area from per-component space reservations.
//!
//! `LayoutBuilder` accepts components that each declare how much pixel space
//! they need on which side of the canvas (top/right/bottom/left) at what
//! priority (smaller = closer to the plot edge). `finish()` sums per-side
//! reservations to derive `plot_rect` and assigns each component a `Slot`
//! describing the band it draws into.
//!
//! Replacing the hardcoded margin block in `Figure::render_to`, this module
//! makes legend/colormap reservations a one-liner: add a new component.

#![allow(clippy::cast_precision_loss)]

use starsight_layer_1::backends::DrawBackend;
use starsight_layer_1::primitives::Rect;
use std::collections::HashMap;

/// Edge of the canvas a reservation lives on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

/// A request to reserve `size` pixels on `side`. Lower `priority` reserves
/// closer to the plot edge.
#[derive(Clone, Copy, Debug)]
pub struct Reservation {
    /// Edge of the canvas this reservation occupies.
    pub side: Side,
    /// Pixel thickness perpendicular to `side`.
    pub size: f32,
    /// Smaller is closer to the plot edge; ties keep insertion order.
    pub priority: u8,
}

/// A pixel band assigned to a component by `LayoutBuilder::finish`.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    /// The rect the component draws into. Spans the full perpendicular axis
    /// of the canvas; the component decides where within that band to render.
    pub rect: Rect,
    /// Which canvas edge the slot is on.
    pub side: Side,
}

/// Context handed to each component when it computes its reservations.
///
/// Carries `&mut DrawBackend` because `text_extent` takes `&mut self` to allow
/// glyph-cache mutation in raster backends.
pub struct LayoutCtx<'a> {
    /// Canvas width in pixels.
    pub width: f32,
    /// Canvas height in pixels.
    pub height: f32,
    /// Backend used for measuring text.
    pub backend: &'a mut dyn DrawBackend,
    /// Default text font size for tick/axis labels.
    pub font_size: f32,
    /// Title font size.
    pub title_font_size: f32,
    /// Outer canvas padding around all reservations.
    pub padding: f32,
}

/// A piece of chrome that reserves layout space.
pub trait LayoutComponent {
    /// Stable identifier used to look up the assigned slot(s) after layout.
    fn id(&self) -> &'static str;
    /// Compute reservations against the given context.
    fn reserve(&self, ctx: &mut LayoutCtx) -> Vec<Reservation>;
}

/// Accumulates reservations from components and resolves them into a `Layout`.
pub struct LayoutBuilder<'a> {
    ctx: LayoutCtx<'a>,
    entries: Vec<(&'static str, Reservation)>,
}

/// Result of `LayoutBuilder::finish`: the plot rect and per-component slots.
pub struct Layout {
    /// The rect available for marks to draw into.
    pub plot_rect: Rect,
    /// Slots assigned to each component, keyed by component id. A component
    /// can request multiple reservations and so may have multiple slots.
    pub slots: HashMap<&'static str, Vec<Slot>>,
}

impl<'a> LayoutBuilder<'a> {
    #[must_use]
    pub fn new(ctx: LayoutCtx<'a>) -> Self {
        Self {
            ctx,
            entries: Vec::new(),
        }
    }

    /// Ask a component for its reservations and record them.
    pub fn add(&mut self, c: &dyn LayoutComponent) {
        let id = c.id();
        for r in c.reserve(&mut self.ctx) {
            self.entries.push((id, r));
        }
    }

    /// Resolve reservations into a `Layout`.
    #[must_use]
    pub fn finish(self) -> Layout {
        // Group entries by side, preserving insertion order within equal priorities.
        let mut by_side: HashMap<Side, Vec<(&'static str, Reservation)>> = HashMap::new();
        for (id, r) in self.entries {
            by_side.entry(r.side).or_default().push((id, r));
        }
        for vec in by_side.values_mut() {
            vec.sort_by_key(|(_, r)| r.priority);
        }

        let total = |side: Side| -> f32 {
            by_side
                .get(&side)
                .map(|v| v.iter().map(|(_, r)| r.size).sum())
                .unwrap_or(0.0)
        };
        let top_total = total(Side::Top);
        let bottom_total = total(Side::Bottom);
        let left_total = total(Side::Left);
        let right_total = total(Side::Right);

        let pad = self.ctx.padding;
        let plot_rect = Rect::new(
            pad + left_total,
            pad + top_total,
            self.ctx.width - pad - right_total,
            self.ctx.height - pad - bottom_total,
        );

        let mut slots: HashMap<&'static str, Vec<Slot>> = HashMap::new();
        for (side, vec) in &by_side {
            let mut outward: f32 = 0.0;
            for (id, res) in vec {
                let rect = match side {
                    Side::Top => Rect::new(
                        0.0,
                        plot_rect.top - outward - res.size,
                        self.ctx.width,
                        plot_rect.top - outward,
                    ),
                    Side::Bottom => Rect::new(
                        0.0,
                        plot_rect.bottom + outward,
                        self.ctx.width,
                        plot_rect.bottom + outward + res.size,
                    ),
                    Side::Left => Rect::new(
                        plot_rect.left - outward - res.size,
                        0.0,
                        plot_rect.left - outward,
                        self.ctx.height,
                    ),
                    Side::Right => Rect::new(
                        plot_rect.right + outward,
                        0.0,
                        plot_rect.right + outward + res.size,
                        self.ctx.height,
                    ),
                };
                slots.entry(*id).or_default().push(Slot { rect, side: *side });
                outward += res.size;
            }
        }

        Layout { plot_rect, slots }
    }
}

// ── Built-in components ──────────────────────────────────────────────────────────────────────────

/// Top-side reservation for the chart title.
pub struct TitleComponent<'a> {
    pub title: Option<&'a str>,
}

impl<'a> LayoutComponent for TitleComponent<'a> {
    fn id(&self) -> &'static str {
        "title"
    }
    fn reserve(&self, ctx: &mut LayoutCtx) -> Vec<Reservation> {
        let Some(t) = self.title else {
            return vec![];
        };
        let h = ctx
            .backend
            .text_extent(t, ctx.title_font_size)
            .map(|(_, h)| h)
            .unwrap_or(ctx.title_font_size);
        // Bit of breathing room above and below the glyph box.
        vec![Reservation {
            side: Side::Top,
            size: h + 12.0,
            priority: 0,
        }]
    }
}

/// Bottom-side reservation for x-axis tick labels (closest to plot, priority 0).
pub struct XTickLabelsComponent<'a> {
    pub labels: &'a [String],
    pub tick_len: f32,
    pub gap: f32,
}

impl<'a> LayoutComponent for XTickLabelsComponent<'a> {
    fn id(&self) -> &'static str {
        "x_tick_labels"
    }
    fn reserve(&self, ctx: &mut LayoutCtx) -> Vec<Reservation> {
        if self.labels.is_empty() {
            return vec![];
        }
        let mut max_h: f32 = 0.0;
        for l in self.labels {
            if let Ok((_, h)) = ctx.backend.text_extent(l, ctx.font_size) {
                if h > max_h {
                    max_h = h;
                }
            }
        }
        if max_h <= 0.0 {
            max_h = ctx.font_size;
        }
        vec![Reservation {
            side: Side::Bottom,
            size: self.tick_len + self.gap + max_h,
            priority: 0,
        }]
    }
}

/// Left-side reservation for y-axis tick labels (closest to plot, priority 0).
pub struct YTickLabelsComponent<'a> {
    pub labels: &'a [String],
    pub tick_len: f32,
    pub gap: f32,
}

impl<'a> LayoutComponent for YTickLabelsComponent<'a> {
    fn id(&self) -> &'static str {
        "y_tick_labels"
    }
    fn reserve(&self, ctx: &mut LayoutCtx) -> Vec<Reservation> {
        if self.labels.is_empty() {
            return vec![];
        }
        let mut max_w: f32 = 0.0;
        for l in self.labels {
            if let Ok((w, _)) = ctx.backend.text_extent(l, ctx.font_size) {
                if w > max_w {
                    max_w = w;
                }
            }
        }
        vec![Reservation {
            side: Side::Left,
            size: self.tick_len + self.gap + max_w,
            priority: 0,
        }]
    }
}

/// Bottom-side reservation for the x-axis title (sits below tick labels, priority 1).
pub struct XAxisTitleComponent<'a> {
    pub label: Option<&'a str>,
    pub gap: f32,
}

impl<'a> LayoutComponent for XAxisTitleComponent<'a> {
    fn id(&self) -> &'static str {
        "x_axis_title"
    }
    fn reserve(&self, ctx: &mut LayoutCtx) -> Vec<Reservation> {
        let Some(l) = self.label else {
            return vec![];
        };
        let h = ctx
            .backend
            .text_extent(l, ctx.font_size)
            .map(|(_, h)| h)
            .unwrap_or(ctx.font_size);
        vec![Reservation {
            side: Side::Bottom,
            size: self.gap + h,
            priority: 1,
        }]
    }
}

/// Left-side reservation for the y-axis title (sits left of tick labels, priority 1).
pub struct YAxisTitleComponent<'a> {
    pub label: Option<&'a str>,
    pub gap: f32,
}

impl<'a> LayoutComponent for YAxisTitleComponent<'a> {
    fn id(&self) -> &'static str {
        "y_axis_title"
    }
    fn reserve(&self, ctx: &mut LayoutCtx) -> Vec<Reservation> {
        let Some(l) = self.label else {
            return vec![];
        };
        // Y-axis title is rotated -90°, so we reserve perpendicular space equal
        // to the *height* of the unrotated glyph box (becomes width after rotation).
        let h = ctx
            .backend
            .text_extent(l, ctx.font_size)
            .map(|(_, h)| h)
            .unwrap_or(ctx.font_size);
        vec![Reservation {
            side: Side::Left,
            size: self.gap + h,
            priority: 1,
        }]
    }
}
