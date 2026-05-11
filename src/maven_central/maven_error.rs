use quick_xml::DeError;
use reqwest::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MavenError {
    #[error("Unable to fetch from maven, {0}")]
    UnableToFetch(#[from] reqwest::Error),

    #[error("Unable to deserialze metadata error: {0}")]
    UnableToDeserialize(#[from] DeError),

    #[error("Server responded with error code: {0}")]
    ErrorResponse(StatusCode),
}
