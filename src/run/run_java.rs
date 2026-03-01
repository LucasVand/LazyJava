use crate::{
    args::RunArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    utils::processes::execute_java,
};

impl LazyJava {
    pub fn run(&self, args: &RunArgs) -> Result<(), LazyJavaError> {
        let class = match &args.class {
            Some(class) => class,
            None => &self.interactive_find_main()?,
        };
        execute_java(class, &self.build, &args.args)
            .map_err(|_e| return LazyJavaError::InvalidMainClass(class.to_string()))?;

        return Ok(());
    }
}
