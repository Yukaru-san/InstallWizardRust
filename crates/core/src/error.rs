use std::io;

/// Failures shared by native and browser package operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O: {0}")]
    Io(#[from] io::Error),
    #[error("ZIP: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Manifest: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
    #[error("Operation cancelled")]
    Cancelled,
}

pub type Result<T> = std::result::Result<T, Error>;
