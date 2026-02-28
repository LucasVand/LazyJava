use std::io;

use thiserror::Error;

use crate::{
    build::build_error::BuildError, clean::clean_error::CleanError, run::run_error::RunError,
};

#[derive(Error, Debug)]
pub enum LazyJavaError {
    #[error("Error Running: {0}")]
    RunError(#[from] RunError),
    #[error("Error Building: {0}")]
    BuildError(#[from] BuildError),
    #[error("Error Cleaning: {0}")]
    CleanError(#[from] CleanError),
    #[error("Error Finding: {0}")]
    FindError(#[from] io::Error),
}
