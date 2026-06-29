use crate::{
    Context, args::FindArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    utils::find_main::find_main_classes,
};

impl LazyJava {
    pub fn find(_args: &FindArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Starting find operation");

        let mains = find_main_classes(&ctx.src).map_err(LazyJavaError::CouldntFindMains)?;
        log::debug!("Found {} main classes", mains.len());

        for main in mains {
            println!(
                "- {}, Package: {}, File: {}",
                main.classname,
                main.full_package_name,
                main.path.to_str().unwrap()
            );
        }
        log::info!("Find operation completed successfully");

        Ok(())
    }
}
