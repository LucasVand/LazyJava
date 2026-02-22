use std::io;

use thiserror::Error;

use crate::build::build_error::BuildError;

#[derive(Error, Debug)]
pub enum RunError {
    #[error("Errors when building the project")]
    BuildError(BuildError),

    #[error("Could not find main class, {0}")]
    NoMainClass(String),

    #[error("Could not find build folder")]
    NoBuildFolder { path: String, os_error: io::Error },
}
