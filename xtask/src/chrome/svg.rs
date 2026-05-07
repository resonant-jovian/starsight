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
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="100%" height="auto" role="img" aria-label="{aria}" preserveAspectRatio="xMidYMid meet">
  <title>{title}</title>
"#
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

/// Read an SVG file and return `(inner_xml, view_box)` for inline embedding.
///
/// Strips any XML/DOCTYPE declarations and the outermost `<svg ...>` open + `</svg>`
/// close, keeping just the inner shapes. The view box is read off the outer tag
/// (or fabricated from `width`/`height`) so the caller can wrap the inner XML in
/// a nested `<svg viewBox="…">` and scale it cleanly.
pub fn inline(path: &Path) -> Result<(String, String)> {
    let raw = std::fs::read_to_string(path)?;
    let body = strip_decl(&raw);

    // Outer <svg ... > tag.
    let open_start = body
        .find("<svg")
        .ok_or_else(|| anyhow::anyhow!("no <svg root in {}", path.display()))?;
    let open_end = body[open_start..]
        .find('>')
        .map(|i| open_start + i + 1)
        .ok_or_else(|| anyhow::anyhow!("unterminated <svg tag in {}", path.display()))?;
    let header = &body[open_start..open_end];

    let view_box = extract_attr(header, "viewBox").unwrap_or_else(|| {
        let w = extract_attr(header, "width").unwrap_or_else(|| "1000".into());
        let h = extract_attr(header, "height").unwrap_or_else(|| "600".into());
        let trim = |s: &str| s.trim_end_matches("px").to_string();
        format!("0 0 {} {}", trim(&w), trim(&h))
    });

    let close_idx = body
        .rfind("</svg>")
        .ok_or_else(|| anyhow::anyhow!("no </svg> in {}", path.display()))?;
    let inner = body[open_end..close_idx].to_string();
    Ok((inner, view_box))
}

fn strip_decl(s: &str) -> &str {
    let mut rest = s.trim_start();
    if let Some(after) = rest.strip_prefix("<?xml")
        && let Some(end) = after.find("?>")
    {
        rest = after[end + 2..].trim_start();
    }
    if rest.starts_with("<!DOCTYPE")
        && let Some(end) = rest.find('>')
    {
        rest = rest[end + 1..].trim_start();
    }
    rest
}

fn extract_attr(tag: &str, name: &str) -> Option<String> {
    let needle = format!("{name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(tag[start..end].to_string())
}
