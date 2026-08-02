use std::path::PathBuf;

use thiserror::Error;
use toml::ser;

use crate::utils::{Diagnostic, DiagnosticProvider};
use toml::de;

#[derive(Error, Debug)]
#[error("failed to serialize TOML while {action} '{path}': {source}")]
pub struct TomlSerializeError {
    action: &'static str,
    path: PathBuf,

    #[source]
    source: ser::Error,
}

impl TomlSerializeError {
    pub fn new<P: Into<PathBuf>>(action: &'static str, path: P, source: ser::Error) -> Self {
        Self {
            action,
            path: path.into(),
            source,
        }
    }
}

impl DiagnosticProvider for TomlSerializeError {
    fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new("Failed to serialize TOML")
            .message(format!(
                "Unable to {} '{}'.",
                self.action,
                self.path.display()
            ))
            .help("Ensure the data being written can be represented as TOML.")
            .note(self.source.to_string())
    }
}

#[derive(Error, Debug)]
#[error("failed to deserialize TOML while {action} '{path}': {source}")]
pub struct TomlDeserializeError {
    action: &'static str,
    path: PathBuf,

    #[source]
    source: de::Error,
}

impl TomlDeserializeError {
    pub fn new<P: Into<PathBuf>>(action: &'static str, path: P, source: de::Error) -> Self {
        Self {
            action,
            path: path.into(),
            source,
        }
    }
}

impl DiagnosticProvider for TomlDeserializeError {
    fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new("Failed to parse TOML")
            .message(format!(
                "Unable to {} '{}'.",
                self.action,
                self.path.display()
            ))
            .help("Ensure the TOML file is valid and follows the expected format.")
            .help("Check for missing quotes, commas, brackets, or required fields.")
            .note(self.source.to_string())
    }
}
