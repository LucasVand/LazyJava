use std::process::{Command, Stdio};

use colored::Colorize;

use crate::{
    Context, args::RunArgs, lazy_java::LazyJava, lazy_java_error::LazyJavaError,
    run::interactive_run::interactive_find_main, utils::processes::execute_java,
};

impl LazyJava {
    pub fn run(args: &RunArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Starting run operation");

        if args.jar {
            return Self::run_jar(args, ctx);
        }

        if ctx.dry_run {
            let class = match &args.class {
                Some(class) => class.clone(),
                None => "<interactive selection>".into(),
            };
            println!("{} {}", "Running".bold().green(), class);
            return Ok(());
        }

        if !args.no_build {
            log::debug!("Building before run");
            let status = Self::build_java(&args.build_args, ctx)?;
            if !status.success() {
                return Ok(());
            }
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

    fn run_jar(args: &RunArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        let jar_path = ctx.target.join("build.jar");

        if !jar_path.exists() {
            return Err(LazyJavaError::JarNotFound(jar_path));
        }

        if ctx.dry_run {
            println!("{} {:?}", "Running".bold().green(), jar_path);
            return Ok(());
        }

        println!("{} {}", "Running".bold().green(), jar_path.display());
        log::info!("Running jar: {:?}", jar_path);

        let status = Command::new("java")
            .arg("-jar")
            .arg(&jar_path)
            .args(&args.args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .status()
            .map_err(LazyJavaError::JarExecutionFailed)?;

        if !status.success() {
            log::warn!("jar exited with code: {:?}", status.code());
        }

        Ok(())
    }
}
