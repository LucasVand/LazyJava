use std::io;

use thiserror::Error;
use toml_edit_derive::TomlError;

use crate::{
    build::BuildError, config::ConfigError, create::create_project::CreateProjectError,
    generate::GenerateError, import::ImportError, lock_file::LockFileError,
    lsp::classpath_error::ClasspathError, maven_central::MavenError,
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
    CreateError(#[from] CreateProjectError),

    #[error("Build error occured")]
    BuildError(#[from] BuildError),

    #[error(r#"Could not find source directory {0}, try changing the source location, or add the directory"#)]
    NoSource(String),

    #[error(r#"Could not find main class {0}, try changing the specified main class, or create a new one with name {0}"#)]
    InvalidMainClass(String),

    #[error(r#"No main classes to run, try creating some"#)]
    NoMainClasses,

    #[error("Could not read current directory, {0}")]
    NoCurrentDir(io::Error),

    #[error(
        "Could not locate root, no root markers were found, try adding in a root marker or manually specify a root"
    )]
    NoRoot(io::Error),

    #[error("Unable to find main classes, {0}")]
    CouldntFindMains(io::Error),

    #[error("Unable to remove build directory when cleaning, {0}")]
    NoRemoveBuild(io::Error),

    #[error("Unable to prompt user to select main class")]
    PromptError,

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Classpath error occured")]
    ClasspathError(#[from] ClasspathError),

    #[error("Maven error occured")]
    MavenError(#[from] MavenError),

    #[error("Error when operating on lock file")]
    LockFileError(#[from] LockFileError),

    #[error("Error when operating on config file")]
    ConfigFileError(#[from] ConfigError),

    #[error("Error serializing")]
    SeError(#[from] toml::ser::Error),

    #[error("Jar not found at {0}, build the jar first with `lazy-java build jar`")]
    JarNotFound(std::path::PathBuf),

    #[error("Failed to execute jar: {0}")]
    JarExecutionFailed(io::Error),
}
