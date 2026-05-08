//! Tiny SVG → PNG helpers — scale-aware rasterization + atomic file writes.
//!
//! Generalizes the rasterization recipe from `social_card::rasterize` so
//! `hero`, `gallery`, and `lorenz_card` can emit a 2× retina PNG alongside
//! their canonical SVG without each duplicating the `usvg`/`resvg`/`tiny_skia`
//! plumbing.

use anyhow::{Context, Result, anyhow};
use std::path::Path;
use tiny_skia::{Pixmap, Transform};

/// Parse `svg` and rasterize it into a `Pixmap` sized at `tree.size() * scale`,
/// rendered with that same scale on the resvg transform. The bundled `DejaVu`
/// faces are loaded first so `<text>` shaping is deterministic across macOS /
/// Linux / Windows runners; system fonts are loaded second as fallback for
/// any glyph `DejaVu` lacks. `resources_dir` is the root that relative
/// `<image href="...">` paths resolve against — needed when the composite
/// embeds example PNGs at native resolution to preserve stroke widths that
/// subpixel SVG strokes would otherwise lose at 2× cell scale.
pub fn rasterize_at_scale(svg: &str, scale: f32, resources_dir: &Path) -> Result<Pixmap> {
    let mut opts = usvg::Options::default();
    super::fonts::load_into(&mut opts);
    opts.fontdb_mut().load_system_fonts();
    opts.resources_dir = Some(resources_dir.to_path_buf());
    let tree = usvg::Tree::from_str(svg, &opts).context("parse svg for rasterization")?;
    let size = tree.size();
    let w = (size.width() * scale).ceil() as u32;
    let h = (size.height() * scale).ceil() as u32;
    let mut pix = Pixmap::new(w, h).ok_or_else(|| anyhow!("alloc pixmap {w}x{h} for rasterize"))?;
    resvg::render(
        &tree,
        Transform::from_scale(scale, scale),
        &mut pix.as_mut(),
    );
    Ok(pix)
}

/// Atomically write `pixmap` to `path` as PNG via a `*.png.tmp` rename. Mirrors
/// the SVG side at `crate::chrome::svg::write_atomic`.
pub fn write_png_atomic(pixmap: &Pixmap, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("png.tmp");
    pixmap
        .save_png(&tmp)
        .map_err(|e| anyhow!("write png {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
