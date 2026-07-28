use std::io;

use thiserror::Error;
use toml_edit_derive::TomlError;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Could not read pom.xml")]
    IOError(#[from] io::Error),

    #[error("Could not parse pom.xml")]
    ParseError(#[from] quick_xml::DeError),

    #[error("Could not parse or serialize config")]
    TomlEditError(#[from] TomlError),
}
