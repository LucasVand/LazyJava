use std::fs;

use crate::{lazy_java::LazyJava, lazy_java_error::LazyJavaError};

impl LazyJava {
    pub fn clean(&self) -> Result<(), LazyJavaError> {
        fs::remove_dir_all(&self.build).map_err(|e| return LazyJavaError::NoRemoveBuild(e))?;

        fs::create_dir(&self.build).map_err(|e| return LazyJavaError::NoCreateBuild(e))?;

        return Ok(());
    }
}
