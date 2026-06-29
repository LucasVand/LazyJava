use crate::{
    Context, args::RunArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    run::interactive_run::interactive_find_main, utils::processes::execute_java,
};

impl LazyJava {
    pub fn run(args: &RunArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Starting run operation");

        if !args.no_build {
            log::debug!("Building before run");
            Self::build_java(&args.build_args, ctx)?;
        }

        let class = match &args.class {
            Some(class) => class,
            None => &interactive_find_main(ctx)?,
        };
        log::debug!("Running class: {}", class);

        execute_java(class, &ctx.bin, &ctx.lib, &args.args)
            .map_err(|_e| LazyJavaError::InvalidMainClass(class.to_string()))?;

        log::info!("Java execution completed successfully");
        Ok(())
    }
}
