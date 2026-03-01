use crate::{
    args::FindArgs, find_main::find_main_classes, lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
};

impl LazyJava {
    pub fn find(&self, _args: &FindArgs) -> Result<(), LazyJavaError> {
        let mains =
            find_main_classes(&self.src).map_err(|e| return LazyJavaError::CouldntFindMains(e))?;

        for main in mains {
            println!(
                "- {}, Package: {}, File: {}",
                main.classname,
                main.full_package_name,
                main.path.to_str().unwrap()
            );
        }

        return Ok(());
    }
}
