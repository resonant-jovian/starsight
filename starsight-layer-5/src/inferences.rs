//! Chart-type auto-inference: pick the right mark from raw data shape.
//!
//! When the user calls `plot!(x, y)` without specifying a mark, this module
//! decides whether to draw a line, points, bars, or a histogram based on the
//! data's shape and types.
//!
//! Status: stub. Implementation lands in 0.2.0.

// ── ChartKind ────────────────────────────────────────────────────────────────────────────────────

/// Inferred chart type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ChartKind {
    /// Line chart for continuous numeric data.
    #[default]
    Line,
    /// Point/scatter chart.
    Point,
    /// Bar chart for categorical x data.
    Bar,
    /// Histogram for single array.
    Histogram,
    /// Heatmap for 2D data.
    Heatmap,
    /// Contour plot of a 2-D scalar grid. Inferred only when a `Grid`-shaped
    /// input is given; the array-based [`infer_chart_kind`] never returns
    /// this variant — it's reserved for explicit dispatch by [`ContourMark`].
    ///
    /// [`ContourMark`]: starsight_layer_3::marks::ContourMark
    Contour,
}

// ── infer_chart_kind ─────────────────────────────────────────────────────────────────────────────

fn count_unique_f64(data: &[f64]) -> usize {
    if data.is_empty() {
        return 0;
    }
    let mut sorted: Vec<f64> = data.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut unique_count = 1;
    for i in 1..sorted.len() {
        if (sorted[i] - sorted[i - 1]).abs() > f64::EPSILON * 10.0 {
            unique_count += 1;
        }
    }
    unique_count
}

/// Infer the best chart type from the data shape.
///
/// - `y` has unique values > 50% of total → `Point`
/// - `x` has string labels → `Bar`  
/// - only `y` provided → `Histogram`
/// - otherwise → `Line`
#[must_use]
pub fn infer_chart_kind(x: &[f64], y: &[f64]) -> ChartKind {
    if y.is_empty() {
        return ChartKind::default();
    }

    // If only y is provided, it's a histogram
    if x.is_empty() {
        return ChartKind::Histogram;
    }

    let y_unique = count_unique_f64(y);
    let y_ratio = y_unique as f64 / y.len() as f64;

    // Many unique y values relative to length → scatter
    if y_ratio > 0.5 {
        return ChartKind::Point;
    }

    // Check if x looks categorical (integers with few unique values)
    let x_unique = count_unique_f64(x);
    let x_is_categorical = x_unique < x.len() / 2 && x.iter().all(|v| v.fract() == 0.0);

    if x_is_categorical {
        return ChartKind::Bar;
    }

    ChartKind::Line
}

/// Infer if x data is categorical (string-like).
#[must_use]
pub fn is_categorical(x: &[f64]) -> bool {
    if x.is_empty() {
        return false;
    }
    let x_unique = count_unique_f64(x);
    x_unique < x.len() / 2 && x.iter().all(|v| v.fract() == 0.0)
}

#[cfg(test)]
mod tests {
    use super::{ChartKind, infer_chart_kind, is_categorical};

    #[test]
    fn infer_histogram_single_array() {
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let kind = infer_chart_kind(&[], &y);
        assert_eq!(kind, ChartKind::Histogram);
    }

    #[test]
    fn infer_empty_y() {
        let kind = infer_chart_kind(&[1.0], &[]);
        assert_eq!(kind, ChartKind::default());
    }

    #[test]
    fn infer_line_continuous() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y = vec![1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0];
        let kind = infer_chart_kind(&x, &y);
        assert_eq!(kind, ChartKind::Line);
    }

    #[test]
    fn infer_point_many_unique() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let y: Vec<f64> = (0..10).map(|i| f64::from(i) * 1.1).collect();
        let kind = infer_chart_kind(&x, &y);
        assert_eq!(kind, ChartKind::Point);
    }

    #[test]
    fn infer_bar_categorical() {
        let x = vec![1.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 2.0];
        let y = vec![10.0, 10.0, 10.0, 10.0, 20.0, 20.0, 20.0, 20.0];
        let kind = infer_chart_kind(&x, &y);
        assert_eq!(kind, ChartKind::Bar);
    }

    #[test]
    fn is_categorical_true() {
        let x = vec![1.0, 1.0, 2.0, 2.0, 2.0, 2.0, 3.0, 3.0];
        assert!(is_categorical(&x));
    }

    #[test]
    fn is_categorical_false() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert!(!is_categorical(&x));
    }

    #[test]
    fn is_categorical_empty() {
        assert!(!is_categorical(&[]));
    }

    #[test]
    fn count_unique_f64() {
        let data = vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0];
        let unique = super::count_unique_f64(&data);
        assert_eq!(unique, 3);
    }

    #[test]
    fn count_unique_f64_empty_returns_zero() {
        assert_eq!(super::count_unique_f64(&[]), 0);
    }
}
