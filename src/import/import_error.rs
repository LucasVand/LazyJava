use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("Could not read pom.xml")]
    IOError(#[from] io::Error),

    #[error("Could not parse pom.xml")]
    ParseError(#[from] quick_xml::DeError),

    #[error("Could not serialize config")]
    SerializeError(#[from] toml::ser::Error),
}
