use std::io;

use thiserror::Error;

use crate::{
    build::dependancy_graph::GraphError, config::ConfigError, lsp::classpath_error::ClasspathError,
};

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Errors while compiling java files")]
    CompilationErrors,

    #[error("Unable to run commands to compile java, {0}")]
    UnableToCompile(io::Error),

    #[error("Unable to find stale files")]
    NoStaleFilesError(io::Error),

    #[error("Graph error occured")]
    GraphError(#[from] GraphError),

    #[error("Classpath error occured")]
    ClasspathError(#[from] ClasspathError),

    #[error("Config error occured")]
    ConfigError(#[from] ConfigError),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Error serializing")]
    SeError(#[from] toml::ser::Error),

    #[error(
        "No main class specified, provide one in the command line or insert into the lazy-java.toml file"
    )]
    NoMainClass,
}
