use crate::{
    args::RunArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    utils::processes::execute_java,
};

impl LazyJava {
    pub fn run(&self, args: &RunArgs) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;
        log::info!("Starting run operation");

        if !args.no_build {
            log::debug!("Building before run");
            self.build_java(&args.build_args)?;
        }

        let class = match &args.class {
            Some(class) => class,
            None => &self.interactive_find_main()?,
        };
        log::debug!("Running class: {}", class);

        execute_java(class, &self.build, &self.lib, &args.args)
            .map_err(|_e| LazyJavaError::InvalidMainClass(class.to_string()))?;

        log::info!("Java execution completed successfully");
        Ok(())
    }
}
