use reqwest::StatusCode;
use thiserror::Error;
use zip::result::ZipError;

use crate::maven_central::MavenError;
use crate::utils::{
    Diagnostic, DiagnosticProvider, IOError, TomlDeserializeError, TomlSerializeError,
};

#[derive(Error, Debug)]
pub enum LockFileError {
    #[error(transparent)]
    ParseFailed(#[from] TomlDeserializeError),

    #[error(transparent)]
    SerializeFailed(#[from] TomlSerializeError),

    #[error("Could not download package")]
    DownloadFailed(StatusCode),

    #[error(transparent)]
    IoError(#[from] IOError),

    #[error("Error fetching file from maven")]
    RequestError(#[from] reqwest::Error),

    #[error("Unable to remove package because it is not included in this project")]
    PackageNotFound,

    #[error("Encountered errors when fetching packages")]
    FetchError(Vec<LockFileError>),

    #[error("Encountered an error when fetching maven")]
    MavenError(#[from] MavenError),

    #[error("Error when processing downloaded file")]
    RemoteProcesserParsingError(#[from] ZipError),

    #[error("Error processing local dependency")]
    LocalProcessorParsingError(String),
}

impl DiagnosticProvider for LockFileError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            LockFileError::LocalProcessorParsingError(name) => {
                Diagnostic::new("Failed to process local jar")
                    .message(format!("Processing local dependency `{}` failed", name))
                    .help("Ensure that the file is a valid jar")
            }
            LockFileError::DownloadFailed(code) => Diagnostic::new("Downloading dependency failed")
                .message(format!("Downloading failed with code {}", code)),
            LockFileError::ParseFailed(err) => err.diagnostic(),
            LockFileError::SerializeFailed(err) => err.diagnostic(),
            LockFileError::IoError(err) => err.diagnostic(),
            LockFileError::RequestError(err) => Diagnostic::new("Failed to fetch from Maven")
                .message("An error occurred while fetching a file from Maven.")
                .help("Check your network connection and try again.")
                .note(err.to_string()),
            LockFileError::PackageNotFound => Diagnostic::new("Package not found")
                .message("This package is not included in the project.")
                .help("Check that the package is declared in your dependencies."),
            LockFileError::FetchError(errors) => {
                let mut diag = Diagnostic::new("Failed to fetch packages").message(format!(
                    "Encountered {} errors while fetching packages.",
                    errors.len()
                ));
                for error in errors {
                    diag = diag.note(error.to_string());
                }
                diag
            }
            LockFileError::MavenError(err) => err.diagnostic(),
            LockFileError::RemoteProcesserParsingError(err) => {
                Diagnostic::new("Failed to process downloaded file")
                    .message("An error occurred while processing the downloaded archive.")
                    .help("The downloaded file may be corrupt or incomplete.")
                    .note(err.to_string())
            }
        }
    }
}
