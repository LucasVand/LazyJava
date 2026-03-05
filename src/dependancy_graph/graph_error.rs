use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("File {0}, does not exist while resolving file to id")]
    NotFound(PathBuf),

    #[error("Io Error occured in the graph")]
    IOError(#[from] io::Error),

    #[error("Error creating dependancy graph, {0}")]
    CreationError(io::Error),
}
