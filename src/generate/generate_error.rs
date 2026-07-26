use std::io;

use quick_xml::SeError;
use thiserror::Error;

use crate::lock_file::LockFileError;

#[derive(Error, Debug)]
pub enum GenerateError {
    #[error("Field {value_name} required to generate {generated_value}")]
    MissingValue {
        value_name: &'static str,
        generated_value: &'static str,
    },

    #[error("Error serializing")]
    SerialError(#[from] SeError),

    #[error("Error when operating on lock file")]
    LockFileError(#[from] LockFileError),

    #[error("An IO error occured")]
    IOError(#[from] io::Error),
}
