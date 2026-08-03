use colored::Colorize;
use std::process::ExitStatus;

use crate::Context;
use crate::args::{BuildArgs, BuildCommand, BuildSubCommand, JarArgs};
use crate::build::build_jar::build_jar;
use crate::build::compile::compile_java;
use crate::build::dependancy_graph::DependancyGraph;
use crate::build::find_stale_files::{files_to_recompile, find_modified_files};
use crate::build::metadata::{BuildMetadata, hash_directory, save_metadata};
use crate::build::processors::build_processors;
use crate::build::resources::copy_resources;
use crate::lazy_java::LazyJava;

use crate::build::BuildError;
use crate::lsp::classpath::Classpath;
use crate::utils::find_main::find_java_files;
use crate::utils::{GlobalContext, IOError, Timings};

impl LazyJava {
    pub fn build(args: &BuildCommand, ctx: &Context) -> Result<(), BuildError> {
        if let Some(build_command) = &args.command {
            match build_command {
                BuildSubCommand::Modified {} => Self::show_modified_files(ctx),
                BuildSubCommand::Dependancies {} => Self::show_dependancy_graph(ctx),
                BuildSubCommand::Dependants {} => Self::show_depentants_graph(ctx),
                BuildSubCommand::Stale {} => Self::show_rebuild_files(ctx),
                BuildSubCommand::Classpath {} => Self::rebuild_classpath(ctx),
                BuildSubCommand::Jar { args } => Self::build_jar(args, ctx),
            }
        } else {
            Self::build_java(&args.args, ctx)?;
            Ok(())
        }
    }
    pub fn build_jar(args: &JarArgs, ctx: &Context) -> Result<(), BuildError> {
        Self::build_java(&args.build_args, ctx)?;
        build_jar(args, ctx)?;
        Ok(())
    }

    pub fn build_java(args: &BuildArgs, ctx: &Context) -> Result<(), BuildError> {
        if GlobalContext::is_dry_run() {
            println!("{} java sources", "Compiling".bold().green());
            return Ok(());
        }

        let mut timings = Timings::start();
        let build_data = BuildMetadata::fetch(&ctx.target);

        let current_lib_hash = hash_directory(&ctx.lib);

        let lib_hash_match = build_data
            .as_ref()
            .is_some_and(|t| current_lib_hash == t.lib_hash);
        timings.record_current("Metadata parse");

        build_processors(args, ctx)?;
        timings.record_current("Processor compile");

        let status = if args.build_all || !lib_hash_match || build_data.is_none() {
            let r = Self::rebuild(args, ctx);
            timings.record_current("Compile");
            r
        } else {
            Self::incrimental_build(args, ctx, build_data.as_ref().unwrap(), &mut timings)
        }?;
        copy_resources(ctx)?;
        timings.record_current("Copy resources");

        save_metadata(ctx, status, build_data)?;

        if !status.success() {
            return Err(BuildError::MainCompilationErrors);
        }

        if args.timings {
            println!("{}", timings);
        } else {
            println!(
                "{} {:.2}s",
                "Compiled in".bold().green(),
                timings.total.elapsed().as_secs_f64()
            );
        }
        Ok(())
    }
    fn incrimental_build(
        args: &BuildArgs,
        ctx: &Context,
        build_data: &BuildMetadata,
        timings: &mut Timings,
    ) -> Result<ExitStatus, BuildError> {
        let exclude = if let Some(s) = ctx.config.setup()
            && let Some(list) = s.exclude()
        {
            list
        } else {
            Vec::new()
        };

        let graph = DependancyGraph::create(&ctx.src, &exclude)?;

        let modified_files = find_modified_files(build_data, &ctx.src, &exclude)
            .map_err(|e| IOError::new("finding modified files", &ctx.src, e))?;
        println!(
            "{} using incrimental build ({} stale file{})",
            "Compiling".bold().green(),
            modified_files.len(),
            if modified_files.len() == 1 { "" } else { "s" }
        );

        if modified_files.is_empty() {
            log::info!("No modified files, skipping compilation");
            return Ok(ExitStatus::default());
        }

        let recompile = files_to_recompile(graph, modified_files)?;

        println!(
            "{} {} source file{}",
            "Compiling".bold().green(),
            recompile.len(),
            if recompile.len() == 1 { "" } else { "s" }
        );
        timings.record_current("Incrimental processing");

        let status = compile_java(recompile, &ctx.bin, ctx, &args.javac_args, true)
            .map_err(|e| IOError::new("compiling java files", &ctx.bin, e))?;

        timings.record_current("Incrimental compile");

        Ok(status)
    }

    fn rebuild(args: &BuildArgs, ctx: &Context) -> Result<ExitStatus, BuildError> {
        println!("{} using full rebuild", "Compiling".bold().green());
        Classpath::generate(ctx)?;
        let exclude = if let Some(s) = ctx.config.setup()
            && let Some(list) = s.exclude()
        {
            list
        } else {
            Vec::new()
        };

        let files = find_java_files(&ctx.src, &exclude);

        let status = compile_java(files, &ctx.bin, ctx, &args.javac_args, true)
            .map_err(|e| IOError::new("compiling java files", &ctx.bin, e))?;
        log::debug!("Java compilation completed status {}", status);

        Ok(status)
    }
    fn show_dependancy_graph(ctx: &Context) -> Result<(), BuildError> {
        let exclude = if let Some(s) = ctx.config.setup()
            && let Some(list) = s.exclude()
        {
            list
        } else {
            Vec::new()
        };
        let graph = DependancyGraph::create(&ctx.src, &exclude)?;

        println!("{}", "Dependancy graph".bold().green());
        for (_key, entry) in graph.nodes.iter() {
            println!(" {}", entry.file_name.bold());
            for dep in &entry.dependancies {
                println!("  - {}", dep);
            }
            println!();
        }
        Ok(())
    }
    fn show_modified_files(ctx: &Context) -> Result<(), BuildError> {
        let build_data = BuildMetadata::fetch(&ctx.target).unwrap_or_default();

        let exclude = if let Some(s) = ctx.config.setup()
            && let Some(list) = s.exclude()
        {
            list
        } else {
            Vec::new()
        };
        let stale_files = find_modified_files(&build_data, &ctx.src, &exclude)
            .map_err(|e| IOError::new("finding modified files", &ctx.src, e))?;

        println!(
            "{} {} file{} since last build",
            "Modified".bold().green(),
            stale_files.len(),
            if stale_files.len() == 1 { "" } else { "s" }
        );
        for file in stale_files {
            println!("  {}", file.to_string_lossy());
        }

        Ok(())
    }
    fn show_rebuild_files(ctx: &Context) -> Result<(), BuildError> {
        let build_data = BuildMetadata::fetch(&ctx.target).unwrap_or_default();

        let exclude = if let Some(s) = ctx.config.setup()
            && let Some(list) = s.exclude()
        {
            list
        } else {
            Vec::new()
        };
        let graph = DependancyGraph::create(&ctx.src, &exclude)?;

        let stale_files = find_modified_files(&build_data, &ctx.src, &exclude)
            .map_err(|e| IOError::new("finding modified files", &ctx.src, e))?;

        let recompile = files_to_recompile(graph, stale_files)?;

        println!(
            "{} {} file{} to recompile",
            "Stale".bold().green(),
            recompile.len(),
            if recompile.len() == 1 { "" } else { "s" }
        );
        for file in recompile {
            println!("  {}", file.to_string_lossy());
        }

        Ok(())
    }
    fn show_depentants_graph(ctx: &Context) -> Result<(), BuildError> {
        let exclude = if let Some(s) = ctx.config.setup()
            && let Some(list) = s.exclude()
        {
            list
        } else {
            Vec::new()
        };
        let graph = DependancyGraph::create(&ctx.src, &exclude)?;
        println!("{}", "Dependants graph".bold().green());
        for (_key, entry) in graph.nodes.iter() {
            println!(" {}", entry.file_name.bold());
            for dep in &entry.dependants {
                println!("  - {}", dep);
            }
            println!();
        }

        Ok(())
    }
    fn rebuild_classpath(ctx: &Context) -> Result<(), BuildError> {
        Ok(Classpath::generate(ctx)?)
    }
}
