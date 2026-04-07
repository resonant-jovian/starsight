//! Export trait + format dispatch.
//!
//! `Export` is the umbrella trait every concrete exporter implements. The
//! dispatch helper picks the right exporter based on a file extension.
//!
//! Status: stub. Concrete implementations land per their feature flags.

// ── Export ───────────────────────────────────────────────────────────────────────────────────────
// TODO(0.10.0): pub trait Export {
//     fn extension(&self) -> &'static str;
//     fn write_to(&self, figure: &Figure, path: &Path) -> Result<()>;
// }

// ── dispatch ─────────────────────────────────────────────────────────────────────────────────────
// TODO(0.10.0): pub fn export_by_extension(figure: &Figure, path: &Path) -> Result<()> {
//     match path.extension() { Some("png") => ..., Some("svg") => ..., Some("pdf") => ..., ... }
// }
