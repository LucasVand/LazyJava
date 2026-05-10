use std::fs;

use crate::{lazy_java::LazyJava, lazy_java_error::LazyJavaError};

impl LazyJava {
    pub fn clean(&self) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;
        log::info!("Starting clean operation");

        let classpath = &self.root.join(".classpath");

        let _ = fs::remove_file(classpath);
        log::debug!("Removed .classpath file");

        fs::remove_dir_all(&self.build).map_err(|e| return LazyJavaError::NoRemoveBuild(e))?;
        log::debug!("Removed build directory: {:?}", self.build);

        fs::create_dir(&self.build).map_err(|e| return LazyJavaError::NoCreateBuild(e))?;
        log::debug!("Created new build directory: {:?}", self.build);

        log::info!("Clean operation completed successfully");
        return Ok(());
    }
}
