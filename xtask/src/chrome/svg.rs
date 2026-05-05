//! Tiny SVG helpers — atomic file writes + a header builder.
//!
//! Every chrome SVG follows the same shape:
//!
//! ```svg
//! <svg xmlns viewBox=… role="img" aria-label="…"><title>…</title>… </svg>
//! ```

use anyhow::Result;
use std::path::Path;

pub fn header(width: u32, height: u32, aria: &str, title: &str) -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" width="100%" height="auto" role="img" aria-label="{aria}" preserveAspectRatio="xMidYMid meet">
  <title>{title}</title>
"#,
        w = width,
        h = height,
        aria = aria,
        title = title
    )
}

pub fn write_atomic(path: &Path, body: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("svg.tmp");
    std::fs::write(&tmp, body)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
