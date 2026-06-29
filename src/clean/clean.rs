use std::fs;

use crate::{Context, lazy_java::LazyJava, lazy_java_error::LazyJavaError};

impl LazyJava {
    pub fn clean(ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Starting clean operation");

        let classpath = ctx.root.join(".classpath");

        let _ = fs::remove_file(classpath);
        log::debug!("Removed .classpath file");

        fs::remove_dir_all(&ctx.bin).map_err(LazyJavaError::NoRemoveBuild)?;
        log::debug!("Removed build directory: {:?}", ctx.bin);

        fs::create_dir(&ctx.bin).map_err(LazyJavaError::NoCreateBuild)?;
        log::debug!("Created new build directory: {:?}", ctx.bin);

        log::info!("Clean operation completed successfully");
        Ok(())
    }
}
