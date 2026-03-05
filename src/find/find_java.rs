use crate::{
    args::FindArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError, logger::logger::Logger,
    utils::find_main::find_main_classes,
};

impl LazyJava {
    pub fn find(&self, _args: &FindArgs) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        let mains =
            find_main_classes(&self.src).map_err(|e| return LazyJavaError::CouldntFindMains(e))?;
        Logger::verbose_elog("Found Main Classes");

        for main in mains {
            println!(
                "- {}, Package: {}, File: {}",
                main.classname,
                main.full_package_name,
                main.path.to_str().unwrap()
            );
        }
        Logger::verbose_elog("Printed Main Classes");

        return Ok(());
    }
}
