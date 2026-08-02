use thiserror::Error;

use crate::utils::{Diagnostic, DiagnosticProvider, IOError};

#[derive(Error, Debug)]
pub enum CreateError {
    #[error("Couldnt prompt user for project name")]
    ProjectNameError,

    #[error("Couldnt create project directory")]
    CreateDirectoryError,

    #[error(transparent)]
    IoError(#[from] IOError),

    #[error("git is not install or included in path")]
    NoGit,
}

impl DiagnosticProvider for CreateError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            CreateError::ProjectNameError => Diagnostic::new("Failed to prompt user")
                .message("Could not get the project name from the user."),
            CreateError::CreateDirectoryError => Diagnostic::new("Failed to create directory")
                .message("Could not create the project directory."),
            CreateError::IoError(err) => err.diagnostic(),
            CreateError::NoGit => Diagnostic::new("Git not found")
                .message("Git is not installed or not available in your PATH.")
                .help("Install git and ensure it is accessible from the command line."),
        }
    }
}
