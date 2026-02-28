use std::fs;

use crate::{clean::clean_error::CleanError, lazy_java::LazyJava};

impl LazyJava {
    pub fn clean(&self) -> Result<(), CleanError> {
        fs::remove_dir_all(&self.build).map_err(|e| CleanError::NoRemove {
            path: self.root.to_str().unwrap().to_string(),
            os_error: e,
        })?;

        fs::create_dir(&self.build).map_err(|e| CleanError::NoCreate {
            path: self.root.to_str().unwrap().to_string(),
            os_error: e,
        })?;

        return Ok(());
    }
}
