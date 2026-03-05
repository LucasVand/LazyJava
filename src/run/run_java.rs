use crate::{
    args::RunArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError, logger::logger::Logger,
    utils::processes::execute_java,
};

impl LazyJava {
    pub fn run(&self, args: &RunArgs) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        if !args.no_build {
            self.build_java(&args.build_args)?;
        }

        let class = match &args.class {
            Some(class) => class,
            None => &self.interactive_find_main()?,
        };
        Logger::verbose_elog(&format!("Found a Class to Run {}", class));

        execute_java(class, &self.build, &self.lib, &args.args)
            .map_err(|_e| return LazyJavaError::InvalidMainClass(class.to_string()))?;

        Logger::verbose_elog("Successfully Ran Java");
        return Ok(());
    }
}
