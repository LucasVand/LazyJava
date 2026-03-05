use std::io;

use thiserror::Error;

use crate::dependancy_graph::graph_error::GraphError;

#[derive(Error, Debug)]
pub enum LazyJavaError {
    #[error(r#"Could not find build directory {0}, try changing the build location, or add the directory"#)]
    NoBuild(String),

    #[error(r#"Could not find source directory {0}, try changing the source location, or add the directory"#)]
    NoSource(String),

    #[error(
        r#"Could not find lib directory {0}, try changing the lib location, or add the directory"#
    )]
    NoLib(String),

    #[error(r#"Could not find main class {0}, try changing the specified main class, or create a new one with name {0}"#)]
    InvalidMainClass(String),

    #[error("Could not read current directory, {0}")]
    NoCurrentDir(io::Error),

    #[error(
        "Could not locate root, no root markers were found, try adding in a root marker or manually specify a root"
    )]
    NoRoot,

    #[error("Unable to find main classes, {0}")]
    CouldntFindMains(io::Error),

    #[error("Unable to remove build directory when cleaning, {0}")]
    NoRemoveBuild(io::Error),

    #[error("Unable to create new build directory when cleaning, {0}")]
    NoCreateBuild(io::Error),

    #[error("Errors while compiling java files")]
    CompilationErrors,

    #[error("Unable to run commands to compile java, {0}")]
    UnableToCompile(io::Error),

    #[error("Unable to prompt user to select main class")]
    PromptError,

    #[error("Unable to find stale files")]
    NoStaleFilesError(io::Error),

    #[error("Unable to set file modification time for build directory")]
    NoBuildModificationTime(io::Error),

    #[error("Graph error occured, {0}")]
    GraphError(#[from] GraphError),
}
