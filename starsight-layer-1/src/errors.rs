//! Error types for starsight: a single non-exhaustive enum and a `Result` alias.
//!
//! Every fallible function in the workspace returns [`Result`]. Each variant
//! carries a descriptive string so context survives propagation through `?`.

use thiserror::Error;

// ── StarsightError ───────────────────────────────────────────────────────────────────────────────

/// All failure modes of the rendering and data pipeline.
///
/// Marked `#[non_exhaustive]` so new variants can be added in patch releases
/// without breaking downstream `match` statements.
#[non_exhaustive]
#[derive(Error, Debug)]
pub enum StarsightError {
    /// Failure inside a backend: path build, paint creation, mask, ...
    #[error("Rendering backend failure: `{0}`")]
    Render(String),

    /// Data shape, length, or type mismatch detected before rendering.
    #[error("Data shape/type mismatch: `{0}`")]
    Data(String),

    /// File-system or I/O failure (auto-converts via `?` from [`std::io::Error`]).
    #[error("File I/O error: `{0}`")]
    Io(#[from] std::io::Error),

    /// Scale domain/range invariants violated (empty domain, zero span, ...).
    #[error("Scale domain/range error: `{0}`")]
    Scale(String),

    /// Output format error during file export.
    #[error("Export format error: `{0}`")]
    Export(String),

    /// Invalid configuration: features, theme, layout, ...
    #[error("Invalid configuration error: `{0}`")]
    Config(String),

    /// Anything that does not fit the categories above.
    #[error("Unknown error: `{0}`")]
    Unknown(String),
}

// ── Result ───────────────────────────────────────────────────────────────────────────────────────

/// Convenience alias used throughout the workspace: `Result<T, StarsightError>`.
pub type Result<T> = std::result::Result<T, StarsightError>;
