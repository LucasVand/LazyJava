use thiserror::Error;

use crate::{
    config::ConfigError,
    lsp::classpath_error::ClasspathError,
    utils::{Diagnostic, DiagnosticProvider, IOError, TomlSerializeError},
};

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Errors while compiling annotation processors")]
    ProcessorCompilationErrors,

    #[error("Errors while compiling java files")]
    MainCompilationErrors,

    #[error("Errors while creating the jar")]
    JarCreationError,

    #[error("Classpath error occurred")]
    ClasspathError(#[from] ClasspathError),

    #[error("Config error occurred")]
    ConfigError(#[from] ConfigError),

    #[error("Error serializing build metadata")]
    MetadataSerializeError(#[from] TomlSerializeError),

    #[error(transparent)]
    IoError(#[from] IOError),

    #[error(
        "No main class specified, provide one in the command line or insert into the lazy-java.toml file"
    )]
    NoMainClass,

    #[error("`javac` was not found on the PATH, ensure a JDK is installed and on the PATH")]
    JavacNotFound,

    #[error("`jar` was not found on the PATH, ensure a JDK is installed and on the PATH")]
    JarNotFound,

    #[error("`java` was not found on the PATH, ensure a JDK is installed and on the PATH")]
    JavaNotFound,
}

impl DiagnosticProvider for BuildError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            BuildError::ProcessorCompilationErrors => {
                Diagnostic::new("Annotation processor compilation failed")
                    .message("Errors occurred while compiling the annotation processors.")
                    .help("Check the annotation processor sources for compile errors.")
            }
            BuildError::MainCompilationErrors => Diagnostic::new("Compilation failed")
                .message("Errors occurred while compiling the java sources.")
                .help("Fix the compile errors and try again."),
            BuildError::JarCreationError => Diagnostic::new("Jar creation failed")
                .message("An error occurred while creating the jar file.")
                .help("Check that the jar tool is available and the target directory is writable."),
            BuildError::ClasspathError(err) => Diagnostic::new("Classpath error")
                .message("An error occurred while generating the classpath.")
                .note(err.to_string()),
            BuildError::ConfigError(err) => err.diagnostic(),
            BuildError::MetadataSerializeError(err) => err.diagnostic(),
            BuildError::IoError(err) => err.diagnostic(),
            BuildError::NoMainClass => Diagnostic::new("No main class specified")
                .message("No main class was specified for the jar.")
                .help("Provide one in the command line or insert into the lazy-java.toml file."),
            BuildError::JavacNotFound => Diagnostic::new("javac not found")
                .message("The `javac` compiler could not be found on the PATH.")
                .help("Ensure a JDK is installed and its bin directory is on the PATH."),
            BuildError::JarNotFound => Diagnostic::new("jar not found")
                .message("The `jar` tool could not be found on the PATH.")
                .help("Ensure a JDK is installed and its bin directory is on the PATH."),
            BuildError::JavaNotFound => Diagnostic::new("java not found")
                .message("The `java` command could not be found on the PATH.")
                .help("Ensure a JDK is installed and its bin directory is on the PATH."),
        }
    }
}
