//! Bundled fonts loaded into `usvg::Options` so chrome PNG rasterization is
//! deterministic across macOS, Linux, and Windows runners.
//!
//! The `SANS` and `MONO_FAMILY` lists in `palette` lead with the Apple/Segoe
//! family names that browsers prefer for SVG text. On the `GitHub` Actions
//! Ubuntu runner none of those families resolve, so `usvg`'s text shaping
//! silently drops glyphs and the rasterized PNG ships without the wordmark,
//! tagline, or meta strip. Bundling `DejaVu` (with `"DejaVu Sans"` /
//! `"DejaVu Sans Mono"` prepended to the family list in `palette`) pins the
//! PNG path to a font we ship in-tree — the SVG path is still browser-resolved,
//! so vector consumers see system-native rendering on macOS/Windows/Linux as
//! before.
//!
//! License: Bitstream Vera + `DejaVu` (permissive, public-domain-ish for the
//! `DejaVu` modifications). See `xtask/src/chrome/fonts/LICENSE`.

const DEJAVU_SANS: &[u8] = include_bytes!("fonts/DejaVuSans.ttf");
const DEJAVU_SANS_MONO: &[u8] = include_bytes!("fonts/DejaVuSansMono.ttf");

/// Load the bundled `DejaVu` Sans + `DejaVu` Sans Mono into `opts.fontdb` so
/// `usvg::Tree::from_str` can shape `<text>` elements regardless of what the
/// host system has installed. Call BEFORE `load_system_fonts()` so the
/// bundled families are matched first.
pub fn load_into(opts: &mut usvg::Options<'_>) {
    let db = opts.fontdb_mut();
    db.load_font_data(DEJAVU_SANS.to_vec());
    db.load_font_data(DEJAVU_SANS_MONO.to_vec());
}
