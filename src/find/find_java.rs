use crate::{
    args::FindArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    utils::find_main::find_main_classes,
};

impl LazyJava {
    pub fn find(&self, _args: &FindArgs) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;
        log::info!("Starting find operation");

        let mains =
            find_main_classes(&self.src).map_err(LazyJavaError::CouldntFindMains)?;
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
