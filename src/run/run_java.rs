use crate::{
    args::RunArgs, lazy_java::LazyJava, processes::execute_java, run::run_error::RunError,
};

impl LazyJava {
    pub fn run(&self, args: &RunArgs) -> Result<(), RunError> {
        let class = match &args.class {
            Some(class) => class,
            None => &self.interactive_find_main()?,
        };
        execute_java(class, &self.build, &args.args)
            .map_err(|_e| RunError::NoMainClass(class.to_string()))?;

        return Ok(());
    }
}
