use std::fs;

use colored::Colorize;

use crate::{Context, lazy_java::LazyJava, lazy_java_error::LazyJavaError};

impl LazyJava {
    pub fn clean(ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Starting clean operation");

        let classpath = ctx.root.join(".classpath");

        let _ = fs::remove_file(classpath);
        log::debug!("Removed .classpath file");

        if ctx.target.exists() {
            fs::remove_dir_all(&ctx.target).map_err(LazyJavaError::NoRemoveBuild)?;
            log::debug!("Removed target directory: {:?}", ctx.target);
        }
        println!(
            "{} project (removed /{})",
            "Cleaned".bold().green(),
            ctx.relative_target
        );
        log::info!("Clean operation completed successfully");
        Ok(())
    }
}
