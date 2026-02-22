use std::io;

use thiserror::Error;

use crate::build::build_error::BuildError;

#[derive(Error, Debug)]
pub enum RunError {
    #[error("Errors when building the project")]
    BuildError(BuildError),

    #[error("Could not find main class, {0}")]
    NoMainClass(String),

    #[error("Could not find build folder, {path}, {os_error}")]
    NoBuildFolder { path: String, os_error: io::Error },

    #[error("Could not search for main classes")]
    MainClassSearchError(io::Error),

    #[error("Could not prompt user for main class selection")]
    PromptError,

    #[error("Could not locate src folder at {path}, error: {os_error}")]
    NoSrc { path: String, os_error: io::Error },
}
