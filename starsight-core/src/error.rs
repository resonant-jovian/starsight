//! Crate defining error definitions

///
use thiserror::Error;

/// Error handling enum
#[derive(Error, Debug)]
pub enum StarsightError {
    /// Rendering backend failures
    #[error("Rendering backend failure: `{0}`")]
    Render(String),
    /// Data shape/type mismatches
    #[error("Data shape/type mismatch: `{0}`")]
    Data(String),
    /// File I/O errors
    #[error("File I/O error: `{0}`")]
    Io(#[from] std::io::Error),
    /// Scale domain/range errors
    #[error("Scale domain/range error: `{0}`")]
    Scale(String),
    /// Export format errors
    #[error("Export format error: `{0}`")]
    Export(String),
    /// Invalid configuration
    #[error("Invalid configuration error: `{0}`")]
    Config(String),
    ///
    #[error("Unknown error: `{0}`")]
    Unknown(String),
}

/// Public Result type for starsight
pub type Result<T> = std::result::Result<T, StarsightError>;
