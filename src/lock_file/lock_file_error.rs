use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LockFileError {
    #[error("Failed to parse lockfile")]
    ParseFailed(#[from] toml::de::Error),

    #[error("Failed to serialize lockfile")]
    SerializeFailed(#[from] toml::ser::Error),

    #[error("Permission denied when trying to read/write lock file at {0}")]
    PermissionDenied(String),

    // internal error
    #[error("Not Found")]
    NoFound,

    #[error("Error when interacting with file system")]
    IoError(#[from] io::Error),

    #[error("Error fetching file from maven")]
    RequestError(#[from] reqwest::Error),

    #[error("Unable to remove package because it is not included in this project")]
    PackageNotFound,
}
