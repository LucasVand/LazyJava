use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("Could not locate build folder at {path}, error: {os_error}")]
    NoBuild { path: String, os_error: io::Error },

    #[error("Java compiler reported build errors")]
    BuildErrors,

    #[error("Could not locate src folder at {path}, error: {os_error}")]
    NoSrc { path: String, os_error: io::Error },

    #[error("Other OS Error: {0}")]
    IOError(io::Error),
}
