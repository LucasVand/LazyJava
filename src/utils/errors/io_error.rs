use std::{io, path::PathBuf};

use thiserror::Error;

use crate::utils::{Diagnostic, DiagnosticProvider};

#[derive(Error, Debug)]
#[error("IO error occurred while {action} {path}: {source}")]
pub struct IOError {
    action: &'static str,
    path: PathBuf,
    #[source]
    source: io::Error,
}

impl IOError {
    pub fn new<P: Into<PathBuf>>(action: &'static str, path: P, source: io::Error) -> Self {
        Self {
            action,
            path: path.into(),
            source,
        }
    }
}

impl DiagnosticProvider for IOError {
    fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new("An IO error occurred")
            .message(format!(
                "Failed while {} {}",
                self.action,
                self.path.display()
            ))
            .help(format!("Ensure permissions allow for {}.", self.action))
            .note(self.source.to_string())
    }
}
