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
    /// Top edge.
    Top,
    /// Right edge.
    Right,
    /// Bottom edge.
    Bottom,
    /// Left edge.
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
    /// Create an empty builder bound to the given layout context.
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
                .map_or(0.0, |v| v.iter().map(|(_, r)| r.size).sum())
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
                slots
                    .entry(*id)
                    .or_default()
                    .push(Slot { rect, side: *side });
                outward += res.size;
            }
        }

        Layout { plot_rect, slots }
    }
}

// ── Built-in components ──────────────────────────────────────────────────────────────────────────

/// Top-side reservation for the chart title.
pub struct TitleComponent<'a> {
    /// Title text, or `None` to skip the reservation.
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
            .map_or(ctx.title_font_size, |(_, h)| h);
        // Priority 1 so the Y-tick-label top gutter (priority 0) sits flush
        // against plot.top and the title goes above it.
        vec![Reservation {
            side: Side::Top,
            size: h + 12.0,
            priority: 1,
        }]
    }
}

/// Bottom-side reservation for x-axis tick labels (closest to plot, priority 0).
/// Also reserves on the right so the rightmost label, which is centered on
/// `plot.right`, doesn't overhang the canvas edge.
pub struct XTickLabelsComponent<'a> {
    /// Tick labels in display order.
    pub labels: &'a [String],
    /// Tick mark length in pixels.
    pub tick_len: f32,
    /// Gap between tick marks and labels, in pixels.
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
        let mut max_w: f32 = 0.0;
        let mut max_h: f32 = 0.0;
        for l in self.labels {
            if let Ok((w, h)) = ctx.backend.text_extent(l, ctx.font_size) {
                if w > max_w {
                    max_w = w;
                }
                if h > max_h {
                    max_h = h;
                }
            }
        }
        if max_h <= 0.0 {
            max_h = ctx.font_size;
        }
        vec![
            Reservation {
                side: Side::Bottom,
                size: self.tick_len + self.gap + max_h,
                priority: 0,
            },
            Reservation {
                side: Side::Right,
                size: max_w / 2.0 + 2.0,
                priority: 0,
            },
        ]
    }
}

/// Left-side reservation for y-axis tick labels (closest to plot, priority 0).
/// Also reserves on the top so the topmost label, which is centered on
/// `plot.top`, doesn't overhang the canvas edge.
pub struct YTickLabelsComponent<'a> {
    /// Tick labels in display order.
    pub labels: &'a [String],
    /// Tick mark length in pixels.
    pub tick_len: f32,
    /// Gap between tick marks and labels, in pixels.
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
        let mut max_h: f32 = 0.0;
        for l in self.labels {
            if let Ok((w, h)) = ctx.backend.text_extent(l, ctx.font_size) {
                if w > max_w {
                    max_w = w;
                }
                if h > max_h {
                    max_h = h;
                }
            }
        }
        if max_h <= 0.0 {
            max_h = ctx.font_size;
        }
        vec![
            Reservation {
                side: Side::Left,
                size: self.tick_len + self.gap + max_w,
                priority: 0,
            },
            Reservation {
                side: Side::Top,
                size: max_h / 2.0 + 2.0,
                priority: 0,
            },
        ]
    }
}

/// Bottom-side reservation for the x-axis title (sits below tick labels, priority 1).
pub struct XAxisTitleComponent<'a> {
    /// Axis title text, or `None` to skip the reservation.
    pub label: Option<&'a str>,
    /// Gap between tick labels and the title, in pixels.
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
            .map_or(ctx.font_size, |(_, h)| h);
        vec![Reservation {
            side: Side::Bottom,
            size: self.gap + h,
            priority: 1,
        }]
    }
}

/// Left-side reservation for the y-axis title (sits left of tick labels, priority 1).
pub struct YAxisTitleComponent<'a> {
    /// Axis title text, or `None` to skip the reservation.
    pub label: Option<&'a str>,
    /// Gap between tick labels and the title, in pixels.
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
            .map_or(ctx.font_size, |(_, h)| h);
        vec![Reservation {
            side: Side::Left,
            size: self.gap + h,
            priority: 1,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LayoutBuilder, LayoutComponent, LayoutCtx, Reservation, Side, TitleComponent,
        XAxisTitleComponent, XTickLabelsComponent, YAxisTitleComponent, YTickLabelsComponent,
    };
    use starsight_layer_1::backends::vectors::SvgBackend;

    fn make_ctx(backend: &mut SvgBackend) -> LayoutCtx<'_> {
        LayoutCtx {
            width: 200.0,
            height: 200.0,
            backend,
            font_size: 12.0,
            title_font_size: 16.0,
            padding: 4.0,
        }
    }

    #[test]
    fn empty_layout_has_full_plot_rect() {
        let mut backend = SvgBackend::new(200, 200);
        let layout = LayoutBuilder::new(make_ctx(&mut backend)).finish();
        assert!(layout.plot_rect.width() > 0.0);
        assert!(layout.plot_rect.height() > 0.0);
        assert!(layout.slots.is_empty());
    }

    #[test]
    fn title_component_no_title_reserves_nothing() {
        let mut backend = SvgBackend::new(200, 200);
        let mut ctx = make_ctx(&mut backend);
        let comp = TitleComponent { title: None };
        assert!(comp.reserve(&mut ctx).is_empty());
        assert_eq!(comp.id(), "title");
    }

    #[test]
    fn title_component_with_title_reserves_top() {
        let mut backend = SvgBackend::new(200, 200);
        let mut ctx = make_ctx(&mut backend);
        let comp = TitleComponent { title: Some("hi") };
        let res = comp.reserve(&mut ctx);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].side, Side::Top);
    }

    #[test]
    fn x_tick_labels_empty_reserves_nothing() {
        let mut backend = SvgBackend::new(200, 200);
        let mut ctx = make_ctx(&mut backend);
        let comp = XTickLabelsComponent {
            labels: &[],
            tick_len: 5.0,
            gap: 4.0,
        };
        assert!(comp.reserve(&mut ctx).is_empty());
        assert_eq!(comp.id(), "x_tick_labels");
    }

    #[test]
    fn x_tick_labels_with_labels_reserves_bottom_and_right() {
        let mut backend = SvgBackend::new(200, 200);
        let mut ctx = make_ctx(&mut backend);
        let labels = vec!["a".to_string(), "b".to_string()];
        let comp = XTickLabelsComponent {
            labels: &labels,
            tick_len: 5.0,
            gap: 4.0,
        };
        let res = comp.reserve(&mut ctx);
        assert_eq!(res.len(), 2);
        assert!(res.iter().any(|r| r.side == Side::Bottom));
        assert!(res.iter().any(|r| r.side == Side::Right));
    }

    #[test]
    fn y_tick_labels_empty_reserves_nothing() {
        let mut backend = SvgBackend::new(200, 200);
        let mut ctx = make_ctx(&mut backend);
        let comp = YTickLabelsComponent {
            labels: &[],
            tick_len: 5.0,
            gap: 4.0,
        };
        assert!(comp.reserve(&mut ctx).is_empty());
        assert_eq!(comp.id(), "y_tick_labels");
    }

    #[test]
    fn y_tick_labels_with_labels_reserves_left_and_top() {
        let mut backend = SvgBackend::new(200, 200);
        let mut ctx = make_ctx(&mut backend);
        let labels = vec!["123".to_string(), "456".to_string()];
        let comp = YTickLabelsComponent {
            labels: &labels,
            tick_len: 5.0,
            gap: 4.0,
        };
        let res = comp.reserve(&mut ctx);
        assert_eq!(res.len(), 2);
        assert!(res.iter().any(|r| r.side == Side::Left));
        assert!(res.iter().any(|r| r.side == Side::Top));
    }

    #[test]
    fn x_axis_title_no_label_reserves_nothing() {
        let mut backend = SvgBackend::new(200, 200);
        let mut ctx = make_ctx(&mut backend);
        let comp = XAxisTitleComponent {
            label: None,
            gap: 4.0,
        };
        assert!(comp.reserve(&mut ctx).is_empty());
        assert_eq!(comp.id(), "x_axis_title");
    }

    #[test]
    fn x_axis_title_with_label_reserves_bottom() {
        let mut backend = SvgBackend::new(200, 200);
        let mut ctx = make_ctx(&mut backend);
        let comp = XAxisTitleComponent {
            label: Some("X"),
            gap: 4.0,
        };
        let res = comp.reserve(&mut ctx);
        assert_eq!(res[0].side, Side::Bottom);
    }

    #[test]
    fn y_axis_title_no_label_reserves_nothing() {
        let mut backend = SvgBackend::new(200, 200);
        let mut ctx = make_ctx(&mut backend);
        let comp = YAxisTitleComponent {
            label: None,
            gap: 4.0,
        };
        assert!(comp.reserve(&mut ctx).is_empty());
        assert_eq!(comp.id(), "y_axis_title");
    }

    #[test]
    fn y_axis_title_with_label_reserves_left() {
        let mut backend = SvgBackend::new(200, 200);
        let mut ctx = make_ctx(&mut backend);
        let comp = YAxisTitleComponent {
            label: Some("Y"),
            gap: 4.0,
        };
        let res = comp.reserve(&mut ctx);
        assert_eq!(res[0].side, Side::Left);
    }

    #[test]
    fn finish_assigns_slots_for_all_sides() {
        let mut backend = SvgBackend::new(400, 300);
        let ctx = make_ctx(&mut backend);
        let mut builder = LayoutBuilder::new(ctx);
        let labels = vec!["one".to_string(), "two".to_string()];
        builder.add(&TitleComponent {
            title: Some("title"),
        });
        builder.add(&XTickLabelsComponent {
            labels: &labels,
            tick_len: 5.0,
            gap: 4.0,
        });
        builder.add(&YTickLabelsComponent {
            labels: &labels,
            tick_len: 5.0,
            gap: 4.0,
        });
        builder.add(&XAxisTitleComponent {
            label: Some("x"),
            gap: 4.0,
        });
        builder.add(&YAxisTitleComponent {
            label: Some("y"),
            gap: 4.0,
        });
        let layout = builder.finish();
        assert!(layout.slots.contains_key("title"));
        assert!(layout.slots.contains_key("x_tick_labels"));
        assert!(layout.slots.contains_key("y_tick_labels"));
        assert!(layout.slots.contains_key("x_axis_title"));
        assert!(layout.slots.contains_key("y_axis_title"));
    }

    #[test]
    fn reservation_struct_can_be_constructed() {
        let r = Reservation {
            side: Side::Top,
            size: 10.0,
            priority: 0,
        };
        assert_eq!(r.size, 10.0);
    }

    /// Minimal `DrawBackend` whose `text_extent` returns `(0.0, 0.0)`. Exercises the
    /// `max_h <= 0.0` fallback in [`XTickLabelsComponent`] and [`YTickLabelsComponent`].
    struct ZeroExtentBackend;
    impl starsight_layer_1::backends::DrawBackend for ZeroExtentBackend {
        fn draw_path(
            &mut self,
            _: &starsight_layer_1::paths::Path,
            _: &starsight_layer_1::paths::PathStyle,
        ) -> starsight_layer_1::errors::Result<()> {
            Ok(())
        }
        fn draw_text(
            &mut self,
            _: &str,
            _: starsight_layer_1::primitives::Point,
            _: f32,
            _: starsight_layer_1::primitives::Color,
        ) -> starsight_layer_1::errors::Result<()> {
            Ok(())
        }
        fn text_extent(
            &mut self,
            _: &str,
            _: f32,
        ) -> starsight_layer_1::errors::Result<(f32, f32)> {
            Ok((0.0, 0.0))
        }
        fn draw_rotated_text(
            &mut self,
            _: &str,
            _: starsight_layer_1::primitives::Point,
            _: f32,
            _: starsight_layer_1::primitives::Color,
            _: f32,
        ) -> starsight_layer_1::errors::Result<()> {
            Ok(())
        }
        fn set_clip(
            &mut self,
            _: Option<starsight_layer_1::primitives::Rect>,
        ) -> starsight_layer_1::errors::Result<()> {
            Ok(())
        }
        fn dimensions(&self) -> (u32, u32) {
            (100, 100)
        }
        fn save_png(&self, _: &std::path::Path) -> starsight_layer_1::errors::Result<()> {
            Ok(())
        }
        fn save_svg(&self, _: &std::path::Path) -> starsight_layer_1::errors::Result<()> {
            Ok(())
        }
        fn fill_rect(
            &mut self,
            _: starsight_layer_1::primitives::Rect,
            _: starsight_layer_1::primitives::Color,
        ) -> starsight_layer_1::errors::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn x_tick_labels_zero_extent_uses_font_size_fallback() {
        let mut backend = ZeroExtentBackend;
        let mut ctx = LayoutCtx {
            width: 200.0,
            height: 200.0,
            backend: &mut backend,
            font_size: 12.0,
            title_font_size: 16.0,
            padding: 4.0,
        };
        let labels = vec!["a".to_string()];
        let comp = XTickLabelsComponent {
            labels: &labels,
            tick_len: 5.0,
            gap: 4.0,
        };
        let res = comp.reserve(&mut ctx);
        // The Bottom reservation includes max_h = font_size = 12.0 fallback.
        let bottom = res.iter().find(|r| r.side == Side::Bottom).unwrap();
        assert_eq!(bottom.size, 5.0 + 4.0 + 12.0);
    }

    #[test]
    fn y_tick_labels_zero_extent_uses_font_size_fallback() {
        let mut backend = ZeroExtentBackend;
        let mut ctx = LayoutCtx {
            width: 200.0,
            height: 200.0,
            backend: &mut backend,
            font_size: 12.0,
            title_font_size: 16.0,
            padding: 4.0,
        };
        let labels = vec!["a".to_string()];
        let comp = YTickLabelsComponent {
            labels: &labels,
            tick_len: 5.0,
            gap: 4.0,
        };
        let res = comp.reserve(&mut ctx);
        let top = res.iter().find(|r| r.side == Side::Top).unwrap();
        // max_h fallback to font_size 12.0; reservation is max_h/2 + 2.0
        assert_eq!(top.size, 12.0 / 2.0 + 2.0);
    }
}
