//! Shared palettes for marks that auto-cycle colors when the user doesn't
//! provide an explicit `Vec<Color>`.
//!
//! Currently exposes one palette: [`POLAR_DEFAULT`], used by [`crate::marks::ArcMark`]
//! and [`crate::marks::PolarBarMark`] for default wedge / bar colors. The
//! palette is **Tableau 10 (vibrant)** — the modern matplotlib categorical
//! default — chosen for higher saturation and pairwise contrast than the
//! older seaborn-deep set, addressing the "Nightingale + Sunburst examples
//! look muddy" feedback in `starsight-dbh`.

use starsight_layer_1::primitives::Color;

/// Default 10-color cycle used by polar-data-mapped wedge marks (`ArcMark`,
/// `PolarBarMark`) when the caller does not set `.colors(...)`. Sequence
/// matches matplotlib's `tab10` (Tableau 10 vibrant).
pub(crate) const POLAR_DEFAULT: [Color; 10] = [
    Color::from_hex(0x004E_79A7), // blue
    Color::from_hex(0x00F2_8E2B), // orange
    Color::from_hex(0x00E1_5759), // red
    Color::from_hex(0x0076_B7B2), // teal
    Color::from_hex(0x0059_A14F), // green
    Color::from_hex(0x00ED_C948), // yellow
    Color::from_hex(0x00B0_7AA1), // purple
    Color::from_hex(0x00FF_9DA7), // pink
    Color::from_hex(0x009C_755F), // brown
    Color::from_hex(0x00BA_B0AC), // grey
];
