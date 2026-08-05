use std::io;
use std::process::Stdio;

use colored::Colorize;

use crate::{
    Context,
    args::RunArgs,
    build::BuildError,
    build::metadata::BuildMetadata,
    lazy_java::LazyJava,
    lazy_java_error::LazyJavaError,
    run::{RunError, interactive_run::interactive_find_main},
    utils::{
        GlobalContext, IOError,
        jdk_version::warn_runtime_mismatch,
        processes::{execute_java, java_tool_command},
    },
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

        if let Some(meta) = BuildMetadata::fetch(&ctx.target)
            && !meta.java_version.is_empty()
        {
            warn_runtime_mismatch(&meta.java_version);
        }

        execute_java(class, &ctx.bin, &ctx.lib, &args.args)?;

        log::info!("Java execution completed successfully");
        Ok(())
    }

    fn run_jar(args: &RunArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        let jar_path = ctx.target.join("build.jar");

        if !jar_path.exists() {
            return Err(RunError::JarNotFound(jar_path))?;
        }

        println!("{} {}", "Running".bold().green(), jar_path.display());

        if GlobalContext::is_dry_run() {
            return Ok(());
        }

        let status = java_tool_command("java")
            .arg("-jar")
            .arg(&jar_path)
            .args(&args.args)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit())
            .status()
            .map_err(|e| {
                if e.kind() == io::ErrorKind::NotFound {
                    RunError::BuildError(BuildError::JavaNotFound)
                } else {
                    RunError::IoError(IOError::new("executing jar", &jar_path, e))
                }
            })?;

        if !status.success() {
            log::warn!("jar exited with code: {:?}", status.code());
        }

        Ok(())
    }
}
