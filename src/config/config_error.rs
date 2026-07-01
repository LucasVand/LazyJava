use std::{io, path::PathBuf};

use thiserror::Error;

use crate::{lock_file::LockFileError, maven_central::MavenError};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Faild to find config {0}")]
    NoConfig(PathBuf),

    #[error("Failed to parse config file")]
    ParseFailed(#[from] toml::de::Error),

    #[error("Failed to serialize config file")]
    SerializeFailed(#[from] toml::ser::Error),

    #[error("Failed to serialize config file")]
    SerializeFailedTomlEdit(#[from] toml_edit::ser::Error),

    #[error("Permission denied when trying to read/write config file at {0}")]
    PermissionDenied(String),

    // internal error
    #[error("Not Found")]
    NoFound,

    #[error("Error when interacting with file system")]
    IoError(#[from] io::Error),

    #[error("Error when operating on the lock file")]
    LockFileErrro(#[from] LockFileError),

    #[error("Maven error occured")]
    MavenError(#[from] MavenError),

    #[error("Could not remove because package is not included in config file")]
    PackageNotFound,
}
