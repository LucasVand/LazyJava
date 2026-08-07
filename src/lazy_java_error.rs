use crate::utils::ContextError;
use std::io;

use thiserror::Error;
use toml_edit_derive::TomlError;

use crate::utils::{Diagnostic, DiagnosticProvider};
use crate::{
    build::BuildError, config::ConfigError, create::CreateError, generate::GenerateError,
    import::ImportError, lock_file::LockFileError, lsp::classpath_error::ClasspathError,
    maven_central::MavenError, run::RunError,
};

#[derive(Error, Debug)]
pub enum LazyJavaError {
    #[error("Toml Edit Error")]
    TomlEditError(#[from] TomlError),

    #[error("Could not generate value")]
    GenerateError(#[from] GenerateError),

    #[error("Could not import value")]
    ImportError(#[from] ImportError),

    #[error("Could not create project")]
    CreateError(#[from] CreateError),

    #[error("Build error occurred")]
    BuildError(#[from] BuildError),

    #[error("Context error occurred")]
    ContextError(#[from] ContextError),

    #[error("Unable to find main classes, {0}")]
    CouldNotFindMains(io::Error),

    #[error("Unable to remove build directory when cleaning, {0}")]
    NoRemoveBuild(io::Error),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Run error occurred")]
    RunError(#[from] RunError),

    #[error("Classpath error occurred")]
    ClasspathError(#[from] ClasspathError),

    #[error("Maven error occurred")]
    MavenError(#[from] MavenError),

    #[error("Error when operating on lock file")]
    LockFileError(#[from] LockFileError),

    #[error("Error when operating on config file")]
    ConfigFileError(#[from] ConfigError),
}

impl DiagnosticProvider for LazyJavaError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            LazyJavaError::TomlEditError(err) => {
                Diagnostic::new("Toml edit error").note(err.to_string())
            }
            LazyJavaError::GenerateError(err) => err.diagnostic(),
            LazyJavaError::ImportError(err) => err.diagnostic(),
            LazyJavaError::CreateError(err) => err.diagnostic(),
            LazyJavaError::BuildError(err) => err.diagnostic(),
            LazyJavaError::ContextError(err) => err.diagnostic(),
            LazyJavaError::CouldNotFindMains(err) => Diagnostic::new("Unable to find main classes")
                .message("Could not locate any main classes in the project.")
                .help("Create a main class, then try running again.")
                .note(err.to_string()),
            LazyJavaError::NoRemoveBuild(err) => {
                Diagnostic::new("Unable to remove build directory")
                    .message("Could not remove the build directory while cleaning.")
                    .help("Check that the build directory is not in use.")
                    .note(err.to_string())
            }
            LazyJavaError::IoError(err) => Diagnostic::new("IO error").note(err.to_string()),
            LazyJavaError::RunError(err) => err.diagnostic(),
            LazyJavaError::ClasspathError(err) => {
                Diagnostic::new("Classpath error").note(err.to_string())
            }
            LazyJavaError::MavenError(err) => err.diagnostic(),
            LazyJavaError::LockFileError(err) => err.diagnostic(),
            LazyJavaError::ConfigFileError(err) => err.diagnostic(),
        }
    }
}
