use std::io;

use thiserror::Error;

use crate::{
    config::ConfigError,
    lock_file::LockFileError,
    utils::{Diagnostic, DiagnosticProvider, IOError},
};

#[derive(Error, Debug)]
pub enum ContextError {
    #[error("Could not read current directory, {0}")]
    NoCurrentDir(io::Error),

    #[error(
        "Could not locate root, no root markers were found, try adding in a root marker or manually specify a root"
    )]
    NoRoot(io::Error),

    #[error(r#"Could not find source directory {0}, try changing the source location, or add the directory"#)]
    NoSource(String),

    #[error("Error when operating on config file")]
    ConfigError(#[from] ConfigError),

    #[error("Error when operating on lock file")]
    LockFileError(#[from] LockFileError),

    #[error(transparent)]
    IoError(#[from] IOError),
}

impl DiagnosticProvider for ContextError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            ContextError::NoCurrentDir(err) => Diagnostic::new("Could not read current directory")
                .message("Could not determine the current working directory.")
                .note(err.to_string()),
            ContextError::NoRoot(err) => Diagnostic::new("Could not locate project root")
                .message("No root markers were found while locating the project.")
                .help("Add a root marker (e.g. .git, pom.xml, lazy-java.toml) or manually specify a root.")
                .note(err.to_string()),
            ContextError::NoSource(path) => Diagnostic::new("Source directory not found")
                .message(format!("Could not find source directory {path}."))
                .help("Change the source location, or add the directory."),
            ContextError::ConfigError(err) => err.diagnostic(),
            ContextError::LockFileError(err) => err.diagnostic(),
            ContextError::IoError(err) => err.diagnostic(),
        }
    }
}
