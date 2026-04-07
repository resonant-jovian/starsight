//! PDF backend: vector PDF output via the `krilla` crate.
//!
//! Status: stub. Lands in 0.10.0 (feature-gated `pdf`). Until then,
//! PDF export is unsupported and `Figure::save("foo.pdf")` returns
//! `StarsightError::Export`.

// ── PdfBackend ───────────────────────────────────────────────────────────────────────────────────
// TODO(0.10.0): pub struct PdfBackend {
//     document: krilla::Document,
//     width: u32,
//     height: u32,
// }

// ── DrawBackend impl ─────────────────────────────────────────────────────────────────────────────
// TODO(0.10.0): impl DrawBackend for PdfBackend { ... }
