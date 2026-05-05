//! Single monochrome palette shared by every chrome asset.
//!
//! No mauve, no Catppuccin tokens — just black, white, and four greys.

pub struct Palette {
    pub bg: &'static str,      // page / canvas
    pub card: &'static str,    // card fill (same as bg in mono)
    pub text: &'static str,    // primary values, sparkline, filled markers
    pub subtext: &'static str, // body / activity line
    pub muted: &'static str,   // eyebrows, labels, captions
    pub border: &'static str,  // card stroke, planned-marker stroke
    pub rule: &'static str,    // hairlines under sparkline + section dividers
}

pub const MONO: Palette = Palette {
    bg: "#ffffff",
    card: "#ffffff",
    text: "#1a1a1a",
    subtext: "#555555",
    muted: "#888888",
    border: "#cccccc",
    rule: "#e5e5e5",
};

/// Same colors but as `(R, G, B, A)` tuples for tiny-skia raster compositing.
pub mod rgba {
    pub const BG: (u8, u8, u8, u8) = (0xff, 0xff, 0xff, 0xff);
    pub const BORDER: (u8, u8, u8, u8) = (0xcc, 0xcc, 0xcc, 0xff);
}

pub const SANS: &str = "-apple-system, BlinkMacSystemFont, &quot;Segoe UI&quot;, Roboto, Helvetica, Arial, sans-serif";
pub const MONO_FAMILY: &str =
    "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace";
