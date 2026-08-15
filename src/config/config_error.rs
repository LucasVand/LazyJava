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
    #[error("Duplicate local dependencies")]
    DuplicateLocalDependencies(String),

    #[error("Could not parse ConfigDependency missing field '{0}'")]
    MissingValue(&'static str),

    #[error("Failed to parse")]
    ParseError(#[from] TomlError),

    #[error("Failed to find config {0}")]
    NoConfig(PathBuf),

    #[error("Error when operating on the lock file")]
    LockFileError(#[from] LockFileError),

    #[error("Maven error occurred")]
    MavenError(#[from] MavenError),

    #[error(transparent)]
    IoError(#[from] IOError),

    #[error("Could not remove because package is not included in config file")]
    PackageNotFound,

    #[error("Could not parse ConfigDependency unexpected field '{0}'")]
    UnexpectedValue(&'static str),

    #[error("Local dependency path does not exist")]
    LocalDependencyNotFound(PathBuf),

    #[error("Local dependency is not a jar file")]
    LocalDependencyNotJar(PathBuf),
}

impl DiagnosticProvider for ConfigError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            ConfigError::DuplicateLocalDependencies(name) => {
                Diagnostic::new("Duplicate local dependencies found")
                    .message(format!(
                        "Found more then one local dependency tagged `{}`",
                        name
                    ))
                    .help("Remove one dependency or rename it")
            }
            ConfigError::LocalDependencyNotFound(path) => {
                Diagnostic::new("Local dependency path does not exist")
                    .message(format!(
                        "Could not find the local dependency {}",
                        path.display()
                    ))
                    .help("Ensure that the path given exists and is a jar file")
            }
            ConfigError::LocalDependencyNotJar(path) => {
                Diagnostic::new("Local dependency is not a jar file")
                    .message(format!(
                        "The local dependency {} does not have a .jar extension",
                        path.display()
                    ))
                    .note("Currently only .jar files are supported as local dependencies")
                    .help("Point the dependency at a valid .jar file")
            }

            ConfigError::MissingValue(field) => Diagnostic::new("Missing lazy-java.toml value")
                .message(format!(
                    "Could not parse lazy-java.toml, missing field '{field}'."
                )),
            ConfigError::UnexpectedValue(field) => {
                Diagnostic::new("Unexpected value in lazy-java.toml ").message(format!(
                    "Could not parse lazy-java.toml, unexpected field '{field}'."
                ))
            }

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
