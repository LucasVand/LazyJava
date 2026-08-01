use std::path::PathBuf;

use thiserror::Error;

use crate::{
    build::BuildError,
    utils::{Diagnostic, DiagnosticProvider, IOError},
};

#[derive(Error, Debug)]
pub enum RunError {
    #[error("Unable to prompt user to select main class")]
    PromptError,

    #[error(r#"No main classes to run, try creating some"#)]
    NoMainClasses,

    #[error(r#"Could not find main class {0}, try changing the specified main class, or create a new one with name {0}"#)]
    InvalidMainClass(String),

    #[error("Jar not found at {0}, build the jar first with `lazy-java build jar`")]
    JarNotFound(PathBuf),

    #[error("Build error occured while building before running")]
    BuildError(#[from] BuildError),

    #[error(transparent)]
    IoError(#[from] IOError),
}

impl DiagnosticProvider for RunError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            RunError::PromptError => Diagnostic::new("Failed to prompt user")
                .message("Could not get the main class selection from the user."),
            RunError::NoMainClasses => Diagnostic::new("No main classes found")
                .message("There are no main classes to run.")
                .help("Create a main class, then try running again."),
            RunError::InvalidMainClass(class) => Diagnostic::new("Invalid main class")
                .message(format!("Could not find or run the main class {class}."))
                .help(format!(
                    "Try changing the specified main class or create a new one named {class}."
                )),
            RunError::JarNotFound(path) => Diagnostic::new("Jar not found")
                .message(format!("No jar found at {}.", path.display()))
                .help("Build the jar first with `lazy-java build jar`."),
            RunError::BuildError(err) => err.diagnostic(),
            RunError::IoError(err) => err.diagnostic(),
        }
    }
}
