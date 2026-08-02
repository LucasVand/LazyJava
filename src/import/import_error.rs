use thiserror::Error;
use toml_edit_derive::TomlError;

use crate::utils::{Diagnostic, DiagnosticProvider, IOError, XmlDeserializeError};

#[derive(Error, Debug)]
pub enum ImportError {
    #[error(transparent)]
    IOError(#[from] IOError),

    #[error(transparent)]
    ParseError(#[from] XmlDeserializeError),

    #[error("Could not parse or serialize config")]
    TomlEditError(#[from] TomlError),
}

impl DiagnosticProvider for ImportError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            ImportError::IOError(err) => err.diagnostic(),
            ImportError::ParseError(err) => err.diagnostic(),
            ImportError::TomlEditError(err) => {
                Diagnostic::new("Could not parse or serialize config").note(err.to_string())
            }
        }
    }
}
