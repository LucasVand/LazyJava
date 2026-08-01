use std::path::PathBuf;

use quick_xml::{DeError, se::SeError};
use thiserror::Error;

use crate::utils::{Diagnostic, DiagnosticProvider};

#[derive(Error, Debug)]
#[error("failed to serialize XML while {action} '{path}': {source}")]
pub struct XmlSerializeError {
    action: &'static str,
    path: PathBuf,

    #[source]
    source: SeError,
}

impl XmlSerializeError {
    pub fn new<P: Into<PathBuf>>(action: &'static str, path: P, source: SeError) -> Self {
        Self {
            action,
            path: path.into(),
            source,
        }
    }
}

impl DiagnosticProvider for XmlSerializeError {
    fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new("Failed to serialize XML")
            .message(format!(
                "Unable to {} '{}'.",
                self.action,
                self.path.display()
            ))
            .help("Ensure the XML data is valid and all required fields are present.")
            .note(self.source.to_string())
    }
}

#[derive(Error, Debug)]
#[error("failed to deserialize XML while {action} '{path}': {source}")]
pub struct XmlDeserializeError {
    action: &'static str,
    path: PathBuf,

    #[source]
    source: DeError,
}

impl XmlDeserializeError {
    pub fn new<P: Into<PathBuf>>(action: &'static str, path: P, source: DeError) -> Self {
        Self {
            action,
            path: path.into(),
            source,
        }
    }
}

impl DiagnosticProvider for XmlDeserializeError {
    fn diagnostic(&self) -> Diagnostic {
        Diagnostic::new("Failed to read XML")
            .message(format!(
                "Unable to {} '{}'.",
                self.action,
                self.path.display()
            ))
            .help("Ensure the XML file is well-formed and matches the expected format.")
            .help("If the file was edited manually, verify that all required elements and attributes are present.")
            .note(self.source.to_string())
    }
}
