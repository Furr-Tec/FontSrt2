use std::fmt;
use std::io;
use std::path::PathBuf;

/// Custom error type for the FontSrt application
#[derive(Debug)]
pub enum Error {
    /// IO operations errors
    Io(io::Error),
    /// Font parsing or processing errors
    Font(String),
    /// Invalid file or directory path
    InvalidPath(PathBuf),
    /// Configuration errors
    Config(String),
    /// Batch processing errors
    #[allow(dead_code)]
    Batch(String),
    /// Font metadata extraction errors
    #[allow(dead_code)]
    Metadata(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Font(msg) => write!(f, "Font error: {}", msg),
            Error::InvalidPath(path) => write!(f, "Invalid path: {}", path.display()),
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Batch(msg) => write!(f, "Batch processing error: {}", msg),
            Error::Metadata(msg) => write!(f, "Metadata extraction error: {}", msg),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

/// Result type alias for FontSrt operations
pub type Result<T> = std::result::Result<T, Error>;

