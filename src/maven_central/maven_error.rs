use thiserror::Error;

use crate::utils::{Diagnostic, DiagnosticProvider, IOError, XmlDeserializeError};

#[derive(Error, Debug)]
pub enum MavenError {
    #[error("Unable to fetch from maven, {0}")]
    UnableToFetch(#[from] reqwest::Error),

    #[error(transparent)]
    UnableToDeserialize(#[from] XmlDeserializeError),

    #[error("Server responded with error: {0}")]
    ErrorResponse(reqwest::Error),

    #[error(transparent)]
    UnableToWrite(#[from] IOError),

    #[error("Async task failed: {0}")]
    JoinError(#[from] tokio::task::JoinError),
}

impl DiagnosticProvider for MavenError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            MavenError::UnableToFetch(err) => Diagnostic::new("Unable to fetch from Maven")
                .message("An error occurred while fetching from Maven.")
                .help("Check your network connection and try again.")
                .note(err.to_string()),
            MavenError::UnableToDeserialize(err) => err.diagnostic(),
            MavenError::ErrorResponse(err) => Diagnostic::new("Maven server error")
                .message("The Maven server responded with an error.")
                .note(err.to_string()),
            MavenError::UnableToWrite(err) => err.diagnostic(),
            MavenError::JoinError(err) => Diagnostic::new("Async task failed")
                .message("An async task failed while fetching dependencies.")
                .note(err.to_string()),
        }
    }
}
