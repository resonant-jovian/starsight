

///
pub enum Error {
    /// Rendering backend failures
    Render(String),
    /// Data shape/type mismatches
    Data(String),
    /// File I/O errors
    Io(std::io::Error),
    /// Scale domain/range errors
    Scale(String),
    /// Export format errors
    Export(String),
    /// Invalid configuration
    Config(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {}
