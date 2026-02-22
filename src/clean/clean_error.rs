use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CleanError {
    #[error("Could not find build folder")]
    NoBuildFolder { path: String, os_error: io::Error },

    #[error("Could not remove build folder")]
    IOError(io::Error),

    #[error("Could not remove build directory, {path}, reason: {os_error}")]
    NoRemove { path: String, os_error: io::Error },

    #[error("Could not create build directory, {path}, reason: {os_error}")]
    NoCreate { path: String, os_error: io::Error },
}
