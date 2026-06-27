use std::io;

use quick_xml::DeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MavenError {
    #[error("Unable to fetch from maven, {0}")]
    UnableToFetch(#[from] reqwest::Error),

    #[error("Unable to deserialze metadata error: {0}")]
    UnableToDeserialize(#[from] DeError),

    #[error("Server responded with error: {0}")]
    ErrorResponse(reqwest::Error),

    #[error("Unable to write .jar to lib folder, error: {0}")]
    UnableToWrite(io::Error),

    #[error("Circular dependancies found")]
    CircularDependancy,
}
