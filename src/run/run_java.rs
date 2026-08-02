use std::process::{Command, Stdio};

use colored::Colorize;

use crate::{
    Context,
    args::RunArgs,
    lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
    run::{RunError, interactive_run::interactive_find_main},
    utils::{GlobalContext, IOError, processes::execute_java},
};

impl LazyJava {
    pub fn run(args: &RunArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Starting run operation");

        if args.jar {
            return Self::run_jar(args, ctx);
        }

        if !args.no_build {
            log::debug!("Building before run");
            Self::build_java(&args.build_args, ctx)?;
        }

        let class = match &args.class {
            Some(class) => class,
            None => &interactive_find_main(ctx)?,
        };
        println!("{} {}", "Running".bold().green(), class);

        execute_java(class, &ctx.bin, &ctx.lib, &args.args)
            .map_err(|_e| RunError::InvalidMainClass(class.to_string()))?;

        log::info!("Java execution completed successfully");
        Ok(())
    }

    fn run_jar(args: &RunArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        let jar_path = ctx.target.join("build.jar");

        if !jar_path.exists() {
            return Err(RunError::JarNotFound(jar_path))?;
        }

        println!("{} {}", "Running".bold().green(), jar_path.display());
        log::info!("Running jar: {:?}", jar_path);

        if GlobalContext::is_dry_run() {
            return Ok(());
        }

        let status = Command::new("java")
            .arg("-jar")
            .arg(&jar_path)
            .args(&args.args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .status()
            .map_err(|e| RunError::IoError(IOError::new("executing jar", &jar_path, e)))?;

        if !status.success() {
            log::warn!("jar exited with code: {:?}", status.code());
        }

        Ok(())
    }
}
