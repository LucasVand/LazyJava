use thiserror::Error;

use crate::lock_file::LockFileError;
use crate::utils::{Diagnostic, DiagnosticProvider, IOError, XmlSerializeError};

#[derive(Error, Debug)]
pub enum GenerateError {
    #[error("Field {value_name} required to generate {generated_value}")]
    MissingValue {
        value_name: &'static str,
        generated_value: &'static str,
    },

    #[error(transparent)]
    SerialError(#[from] XmlSerializeError),

    #[error("Error when operating on lock file")]
    LockFileError(#[from] LockFileError),

    #[error(transparent)]
    IOError(#[from] IOError),
}

impl DiagnosticProvider for GenerateError {
    fn diagnostic(&self) -> Diagnostic {
        match self {
            GenerateError::MissingValue {
                value_name,
                generated_value,
            } => Diagnostic::new("Missing required value")
                .message(format!(
                    "Field {value_name} is required to generate {generated_value}"
                ))
                .help(format!(
                    "Set the {value_name} field in the config to generate {generated_value}"
                )),
            GenerateError::SerialError(err) => err.diagnostic(),

            GenerateError::LockFileError(err) => {
                Diagnostic::new("Failed to operate on the lock file").message(err.to_string())
            }
            GenerateError::IOError(err) => err.diagnostic(),
        }
    }
}
