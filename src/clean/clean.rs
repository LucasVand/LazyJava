use std::fs;

use crate::{lazy_java::LazyJava, lazy_java_error::LazyJavaError, logger::logger::Logger};

impl LazyJava {
    pub fn clean(&self) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        fs::remove_dir_all(&self.build).map_err(|e| return LazyJavaError::NoRemoveBuild(e))?;
        Logger::verbose_elog("Removed Build Directory");

        fs::create_dir(&self.build).map_err(|e| return LazyJavaError::NoCreateBuild(e))?;
        Logger::verbose_elog("Created New Build Directory");

        return Ok(());
    }
}
