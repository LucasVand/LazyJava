use std::io;

use quick_xml::{DeError, SeError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClasspathError {
    #[error("Could not read .classpath, does not exist")]
    NoClasspathFile,

    #[error("Could not read .classpath, OS Error: {0}")]
    OSErrorClasspath(io::Error),

    #[error("Could not parse .classpath file, {0}")]
    ParsingError(#[from] DeError),

    #[error("Could not read lib folder at {0}, OS Error: {1}")]
    OSErrorLib(String, io::Error),

    #[error("Could not write classpath file, attemping to write it at {0}, OS Error: {1}")]
    ClasspathWrite(String, io::Error),

    #[error("Could not serialize classpath, Error: {0}")]
    SerializeError(#[from] SeError),
}
