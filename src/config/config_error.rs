use std::path::PathBuf;

use thiserror::Error;
use toml_edit_derive::TomlError;

use crate::{
    lock_file::LockFileError,
    maven_central::MavenError,
    utils::{Diagnostic, DiagnosticProvider, IOError},
};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not parse ConfigDependancy missing field '{0}'")]
    MissingValue(&'static str),

    #[error("Failed to parse")]
    ParseError(#[from] TomlError),

    #[error("Failed to find config {0}")]
    NoConfig(PathBuf),

    #[error("Error when operating on the lock file")]
    LockFileError(#[from] LockFileError),

    #[error("Maven error occured")]
    MavenError(#[from] MavenError),

    #[error(transparent)]
    IoError(#[from] IOError),

    #[error("Could not remove because package is not included in config file")]
    PackageNotFound,
}

impl DiagnosticProvider for ConfigError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            ConfigError::MissingValue(field) => Diagnostic::new("Missing lazy-java.toml value")
                .message(format!(
                    "Could not parse lazy-java.toml, missing field '{field}'."
                )),
            ConfigError::ParseError(err) => {
                Diagnostic::new("Failed to parse lazy-java.toml").note(err.to_string())
            }
            ConfigError::NoConfig(path) => Diagnostic::new("lazy-java.toml file not found")
                .message(format!(
                    "Could not find lazy-java.toml file at {}.",
                    path.display()
                ))
                .help("Create a lazy-java.toml file or specify a valid project root."),
            ConfigError::LockFileError(err) => err.diagnostic(),
            ConfigError::MavenError(err) => err.diagnostic(),
            ConfigError::IoError(err) => err.diagnostic(),
            ConfigError::PackageNotFound => Diagnostic::new("Package not found").message(
                "Could not remove package because it is not included in lazy-java.toml file.",
            ),
        }
    }
}
