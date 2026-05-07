//! Pixel-space rendered footprint of a mark, used by legend placement.
//!
//! [`MarkExtent`] is a finer-grained alternative to [`Rect`] for "where does
//! this mark actually paint pixels?". The legend dodge in layer-5 tests each
//! candidate corner against a slice of these per-mark contributions to find a
//! corner that does not overlap any mark; on tie, picks the corner with the
//! smallest total clipped overlap area.
//!
//! The default [`crate::marks::Mark::pixel_extent`] impl returns
//! [`MarkExtent::Bbox`] of the mark's projected data extent — fine for marks
//! whose footprint matches their bounding rect (point, bar, heatmap, candle,
//! pie, donut, rug, errorbar). Marks where the bbox over-claims coverage
//! ([`crate::marks::LineMark`] on diagonal data, [`crate::marks::AreaMark`] / contour bands
//! that fill non-rectangular regions, polar marks whose footprint is annular)
//! override to a tighter [`MarkExtent`] variant.

use starsight_layer_1::primitives::{Point, Rect};

/// Pixel-space rendered footprint of a single mark.
///
/// Returned by [`crate::marks::Mark::pixel_extent`]. Variants cover the four
/// shapes that matter for legend dodge accuracy:
///
/// - [`Bbox`](MarkExtent::Bbox) — axis-aligned rectangle. Default for compact
///   marks where the bounding rect already matches the painted footprint.
/// - [`Segments`](MarkExtent::Segments) — discrete line segments. Used by
///   [`crate::marks::LineMark`], [`crate::marks::StepMark`], and isoline-mode
///   [`crate::marks::ContourMark`] where the bbox is much larger than the actual
///   stroke.
/// - [`Rects`](MarkExtent::Rects) — multiple axis-aligned rectangles. Used by
///   marks that paint disjoint rectangles ([`crate::marks::BarMark`] when callers
///   want bar-level precision rather than the data bbox; whisker rectangles
///   on [`crate::marks::ErrorBarMark`]).
/// - [`Polygons`](MarkExtent::Polygons) — filled polygons. Used by
///   [`crate::marks::AreaMark`], filled-band [`crate::marks::ContourMark`],
///   [`crate::marks::ViolinMark`], and polar marks whose footprint is annular
///   or wedge-shaped.
#[derive(Debug, Clone)]
pub enum MarkExtent {
    /// Axis-aligned bounding box of the mark's footprint.
    Bbox(Rect),
    /// Discrete pixel-space line segments `(start, end)`.
    Segments(Vec<(Point, Point)>),
    /// Discrete axis-aligned rectangles.
    Rects(Vec<Rect>),
    /// Filled polygons (each given as a closed ring of vertices).
    Polygons(Vec<Vec<Point>>),
}

impl MarkExtent {
    /// True iff this extent overlaps `candidate` (inclusive on edges).
    #[must_use]
    pub fn intersects(&self, candidate: &Rect) -> bool {
        match self {
            Self::Bbox(b) => b.intersection(candidate).is_some(),
            Self::Segments(segs) => segs
                .iter()
                .any(|(a, b)| segment_intersects_rect(*a, *b, candidate)),
            Self::Rects(rs) => rs.iter().any(|r| r.intersection(candidate).is_some()),
            Self::Polygons(ps) => ps.iter().any(|p| polygon_intersects_rect(p, candidate)),
        }
    }

    /// Total clipped overlap "area" of this extent against `candidate`.
    ///
    /// For `Segments` the result is `clipped_length × 2.0` (a pseudo-area that
    /// approximates a 2-pixel default stroke); for `Rects` and `Polygons` it
    /// is the sum of clipped rect / polygon areas. The legend dodge uses this
    /// as a tiebreaker when no candidate corner is fully clear: smaller wins.
    #[must_use]
    pub fn overlap_area(&self, candidate: &Rect) -> f32 {
        match self {
            Self::Bbox(b) => rect_clip_area(b, candidate),
            Self::Segments(segs) => {
                segs.iter()
                    .map(|(a, b)| segment_clip_length(*a, *b, candidate))
                    .sum::<f32>()
                    * 2.0
            }
            Self::Rects(rs) => rs.iter().map(|r| rect_clip_area(r, candidate)).sum(),
            Self::Polygons(ps) => ps.iter().map(|p| polygon_clip_area(p, candidate)).sum(),
        }
    }
}

// ── Geometry helpers ────────────────────────────────────────────────────────────────────────────

fn rect_clip_area(r: &Rect, c: &Rect) -> f32 {
    r.intersection(c)
        .map_or(0.0, |clip| clip.width() * clip.height())
}

/// Liang-Barsky parametric segment-vs-rect clip. Returns true iff any portion
/// of segment `start → end` falls within `rect`.
fn segment_intersects_rect(start: Point, end: Point, rect: &Rect) -> bool {
    liang_barsky(start, end, rect).is_some()
}

/// Length of segment `start → end` after clipping to `rect`. Returns 0.0 when
/// the segment is fully outside.
fn segment_clip_length(start: Point, end: Point, rect: &Rect) -> f32 {
    if let Some((t_min, t_max)) = liang_barsky(start, end, rect) {
        let dx = (end.x - start.x) * (t_max - t_min);
        let dy = (end.y - start.y) * (t_max - t_min);
        (dx * dx + dy * dy).sqrt()
    } else {
        0.0
    }
}

/// Liang-Barsky returns `(t_min, t_max)` of the clipped sub-segment in
/// parameter space, or `None` if fully outside.
fn liang_barsky(start: Point, end: Point, rect: &Rect) -> Option<(f32, f32)> {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let denom = [-dx, dx, -dy, dy];
    let dist = [
        start.x - rect.left,
        rect.right - start.x,
        start.y - rect.top,
        rect.bottom - start.y,
    ];

    let mut t_min = 0.0_f32;
    let mut t_max = 1.0_f32;
    for i in 0..4 {
        if denom[i].abs() < f32::EPSILON {
            // Parallel to this edge; reject if outside.
            if dist[i] < 0.0 {
                return None;
            }
        } else {
            let t = dist[i] / denom[i];
            if denom[i] < 0.0 {
                if t > t_max {
                    return None;
                }
                if t > t_min {
                    t_min = t;
                }
            } else {
                if t < t_min {
                    return None;
                }
                if t < t_max {
                    t_max = t;
                }
            }
        }
    }
    Some((t_min, t_max))
}

/// Sutherland-Hodgman clip of `poly` against the four edges of `rect`. Returns
/// the (possibly empty) clipped polygon. Reused by both
/// [`polygon_intersects_rect`] and [`polygon_clip_area`].
fn sutherland_hodgman(poly: &[Point], rect: &Rect) -> Vec<Point> {
    if poly.is_empty() {
        return Vec::new();
    }
    let mut out: Vec<Point> = poly.to_vec();
    // Clip against each rect edge in turn: left, right, top, bottom.
    let edges: [(Edge, f32); 4] = [
        (Edge::Left, rect.left),
        (Edge::Right, rect.right),
        (Edge::Top, rect.top),
        (Edge::Bottom, rect.bottom),
    ];
    for (edge, threshold) in edges {
        if out.is_empty() {
            return out;
        }
        let input = std::mem::take(&mut out);
        let n = input.len();
        for i in 0..n {
            let curr = input[i];
            let next = input[(i + 1) % n];
            let curr_in = inside(curr, edge, threshold);
            let next_in = inside(next, edge, threshold);
            if curr_in {
                out.push(curr);
                if !next_in {
                    out.push(intersect_edge(curr, next, edge, threshold));
                }
            } else if next_in {
                out.push(intersect_edge(curr, next, edge, threshold));
            }
        }
    }
    out
}

#[derive(Clone, Copy)]
enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

fn inside(p: Point, edge: Edge, t: f32) -> bool {
    match edge {
        Edge::Left => p.x >= t,
        Edge::Right => p.x <= t,
        Edge::Top => p.y >= t,
        Edge::Bottom => p.y <= t,
    }
}

fn intersect_edge(a: Point, b: Point, edge: Edge, t: f32) -> Point {
    match edge {
        Edge::Left | Edge::Right => {
            // Vertical edge: solve for y at x = t.
            let dx = b.x - a.x;
            if dx.abs() < f32::EPSILON {
                return Point::new(t, a.y);
            }
            let frac = (t - a.x) / dx;
            Point::new(t, a.y + (b.y - a.y) * frac)
        }
        Edge::Top | Edge::Bottom => {
            // Horizontal edge: solve for x at y = t.
            let dy = b.y - a.y;
            if dy.abs() < f32::EPSILON {
                return Point::new(a.x, t);
            }
            let frac = (t - a.y) / dy;
            Point::new(a.x + (b.x - a.x) * frac, t)
        }
    }
}

fn polygon_intersects_rect(poly: &[Point], rect: &Rect) -> bool {
    if poly.is_empty() {
        return false;
    }
    // Quick AABB reject before SH clipping.
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for p in poly {
        if p.x < min_x {
            min_x = p.x;
        }
        if p.x > max_x {
            max_x = p.x;
        }
        if p.y < min_y {
            min_y = p.y;
        }
        if p.y > max_y {
            max_y = p.y;
        }
    }
    if max_x < rect.left || min_x > rect.right || max_y < rect.top || min_y > rect.bottom {
        return false;
    }
    !sutherland_hodgman(poly, rect).is_empty()
}

fn polygon_clip_area(poly: &[Point], rect: &Rect) -> f32 {
    let clipped = sutherland_hodgman(poly, rect);
    polygon_area(&clipped)
}

/// Shoelace formula. Returns the absolute area of a (potentially non-convex)
/// polygon. Empty / degenerate inputs return 0.
fn polygon_area(poly: &[Point]) -> f32 {
    if poly.len() < 3 {
        return 0.0;
    }
    let n = poly.len();
    let mut acc = 0.0_f32;
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        acc += a.x * b.y - b.x * a.y;
    }
    (acc * 0.5).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(left: f32, top: f32, right: f32, bottom: f32) -> Rect {
        Rect::new(left, top, right, bottom)
    }
    fn p(x: f32, y: f32) -> Point {
        Point::new(x, y)
    }

    #[test]
    fn bbox_intersects_overlapping() {
        let e = MarkExtent::Bbox(r(0.0, 0.0, 10.0, 10.0));
        assert!(e.intersects(&r(5.0, 5.0, 15.0, 15.0)));
    }

    #[test]
    fn bbox_does_not_intersect_disjoint() {
        let e = MarkExtent::Bbox(r(0.0, 0.0, 10.0, 10.0));
        assert!(!e.intersects(&r(20.0, 20.0, 30.0, 30.0)));
    }

    #[test]
    fn bbox_overlap_area_full_containment() {
        let e = MarkExtent::Bbox(r(0.0, 0.0, 10.0, 10.0));
        let candidate = r(2.0, 2.0, 8.0, 8.0);
        assert!((e.overlap_area(&candidate) - 36.0).abs() < 1e-3);
    }

    #[test]
    fn segments_diagonal_misses_off_corners() {
        // Segment from BL (0,10) → TR (10,0) of a 10×10 square (line y = 10-x).
        // The "off-diagonal" corners — TL (around 0,0) and BR (around 10,10) —
        // sit far above and below the segment, so the dodge should see them
        // as clear. The center is on the line and intersects.
        let segs = MarkExtent::Segments(vec![(p(0.0, 10.0), p(10.0, 0.0))]);
        // TL candidate (-1..1, -1..1) — segment y at x=0 is 10, far below.
        assert!(!segs.intersects(&r(-1.0, -1.0, 1.0, 1.0)));
        // BR candidate (9..11, 9..11) — segment y at x=9 is 1, far above.
        assert!(!segs.intersects(&r(9.0, 9.0, 11.0, 11.0)));
        // Center candidate (4..6, 4..6) — segment passes through (5,5).
        assert!(segs.intersects(&r(4.0, 4.0, 6.0, 6.0)));
    }

    #[test]
    fn segments_overlap_area_clipped_length_times_two() {
        // Horizontal segment of length 10 fully inside candidate of width 4.
        let segs = MarkExtent::Segments(vec![(p(0.0, 5.0), p(10.0, 5.0))]);
        let candidate = r(3.0, 0.0, 7.0, 10.0);
        // Clipped length = 4; overlap_area = length × 2 = 8.
        assert!((segs.overlap_area(&candidate) - 8.0).abs() < 1e-3);
    }

    #[test]
    fn rects_intersection_any() {
        let e = MarkExtent::Rects(vec![r(0.0, 0.0, 5.0, 5.0), r(20.0, 0.0, 25.0, 5.0)]);
        // Candidate hits second rect only.
        assert!(e.intersects(&r(22.0, 2.0, 23.0, 3.0)));
        // Candidate misses both.
        assert!(!e.intersects(&r(10.0, 10.0, 15.0, 15.0)));
    }

    #[test]
    fn polygon_triangle_clip_area() {
        // Right triangle (0,0)-(10,0)-(0,10), area 50.
        let triangle = vec![p(0.0, 0.0), p(10.0, 0.0), p(0.0, 10.0)];
        let e = MarkExtent::Polygons(vec![triangle]);
        // Full bbox clip = 50.
        assert!((e.overlap_area(&r(0.0, 0.0, 10.0, 10.0)) - 50.0).abs() < 1e-3);
        // Disjoint clip = 0.
        assert!((e.overlap_area(&r(20.0, 20.0, 30.0, 30.0))).abs() < 1e-3);
    }

    #[test]
    fn polygon_intersects_uses_aabb_reject() {
        let triangle = vec![p(0.0, 0.0), p(10.0, 0.0), p(0.0, 10.0)];
        let e = MarkExtent::Polygons(vec![triangle]);
        assert!(!e.intersects(&r(20.0, 20.0, 30.0, 30.0)));
    }
}
