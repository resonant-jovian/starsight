//! starsight — a unified scientific visualization crate for Rust.
//!
//! starsight is the only crate users add to `Cargo.toml`. It re-exports types
//! from the seven layer crates underneath through three access patterns:
//!
//! 1. **The prelude** — `use starsight::prelude::*;` for the common types.
//! 2. **By category** — `use starsight::marks::LineMark;`, `use starsight::backends::SkiaBackend;`.
//! 3. **By layer** — `use starsight::components::marks::LineMark;` (Latin layer aliases).
//!
//! ```no_run
//! use starsight::prelude::*;
//!
//! fn main() -> starsight::Result<()> {
//!     plot!(&[1.0, 2.0, 3.0, 4.0], &[10.0, 20.0, 15.0, 25.0]).save("chart.png")
//! }
//! ```

// ── Layer aliases (Latin/Greek-rooted) ───────────────────────────────────────────────────────────

pub use starsight_layer_1 as background;
pub use starsight_layer_2 as modifiers;
pub use starsight_layer_3 as components;
pub use starsight_layer_4 as composition;
pub use starsight_layer_5 as common;
pub use starsight_layer_6 as interactivity;
pub use starsight_layer_7 as export;

// ── Semantic facade modules (by category) ────────────────────────────────────────────────────────

pub mod aesthetics;
pub mod axes;
pub mod backends;
/// Re-exports of the colormap types and built-in colormap constants.
pub mod colormap;
pub mod coords;
pub mod exports;
pub mod figures;
pub mod inferences;
pub mod layouts;
pub mod legends;
pub mod marks;
pub mod paths;
pub mod prelude;
pub mod primitives;
pub mod renders;
pub mod scales;
pub mod sources;
pub mod statistics;
/// Re-exports of theming types and the default light/dark themes.
pub mod theme;
pub mod ticks;

// ── Top-level convenience aliases ────────────────────────────────────────────────────────────────

/// Workspace `Result<T, StarsightError>` alias.
pub type Result<T> = crate::background::errors::Result<T>;
/// Top-level error enum re-export.
pub use crate::background::errors::StarsightError;

// ── plot! macro ──────────────────────────────────────────────────────────────────────────────────

/// One-liner: `plot!(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]).save("out.png").unwrap();`
///
/// With the `polars` feature, the macro grows a `DataFrame` arm:
/// `plot!(df, x = "col_a", y = "col_b")` extracts the named columns from a
/// Polars frame and dispatches the appropriate mark (`LineMark` for numeric
/// x, `BarMark` for categorical x). Add `color = "col_c"` to partition rows
/// by a third column and emit one mark per group with cycled palette
/// colours and per-group legend labels.
#[macro_export]
macro_rules! plot {
    ($x:expr, $y:expr $(,)?) => {{
        $crate::common::Figure::from_arrays(
            $x.into_iter().map(|&v| v as f64),
            $y.into_iter().map(|&v| v as f64),
        )
    }};
    ($df:expr, x = $x:expr, y = $y:expr $(,)?) => {{ $crate::common::sources::plot_dataframe($df, $x, $y, None) }};
    ($df:expr, x = $x:expr, y = $y:expr, color = $color:expr $(,)?) => {{ $crate::common::sources::plot_dataframe($df, $x, $y, Some($color)) }};
}
