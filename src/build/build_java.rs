use crate::Context;
use crate::args::{BuildArgs, BuildCommand, BuildSubCommand};
use crate::build::find_stale_files::{files_to_recompile, find_modified_files};
use crate::build::resources::copy_resources;
use crate::dependancy_graph::graph::DependancyGraph;
use crate::lazy_java::LazyJava;

use crate::lazy_java_error::LazyJavaError;
use crate::lsp::classpath::Classpath;
use crate::utils::processes::{compile_java, compile_java_files};

impl LazyJava {
    pub fn build(args: &BuildCommand, ctx: &Context) -> Result<(), LazyJavaError> {
        if let Some(build_command) = &args.command {
            match build_command {
                BuildSubCommand::Modified {} => Self::show_modified_files(ctx),
                BuildSubCommand::Dependancies {} => Self::show_dependancy_graph(ctx),
                BuildSubCommand::Dependants {} => Self::show_depentants_graph(ctx),
                BuildSubCommand::Stale {} => Self::show_rebuild_files(ctx),
                BuildSubCommand::Classpath {} => Self::rebuild_classpath(ctx),
            }
        } else {
            Self::build_java(&args.args, ctx)
        }
    }
    pub fn build_java(args: &BuildArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        if args.build_all {
            Self::rebuild(args, ctx)?;
        } else {
            Self::incrimental_build(args, ctx)?;
        }
        copy_resources(args, ctx)?;
        Ok(())
    }
    fn incrimental_build(args: &BuildArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Starting incremental build");
        Classpath::generate_if_stale(ctx)?;

        let graph = DependancyGraph::create(&ctx.src)?;
        log::debug!("Created dependency graph");

        let modified_files =
            find_modified_files(&ctx.bin, &ctx.src).map_err(LazyJavaError::NoStaleFilesError)?;

        if modified_files.is_empty() {
            log::info!("No modified files, skipping compilation");
            return Ok(());
        }

        log::debug!("Found {} modified files", modified_files.len());
        let recompile = files_to_recompile(graph, modified_files)?;
        log::debug!("Need to recompile {} files", recompile.len());

        let status = compile_java_files(&ctx.bin, &ctx.lib, &args.javac_args, recompile)
            .map_err(LazyJavaError::UnableToCompile)?;

        log::debug!("Java compilation completed");
        if status.success() {
            log::info!("Compilation successful");

            let file_time = filetime::FileTime::now();
            filetime::set_file_mtime(&ctx.bin, file_time)
                .map_err(LazyJavaError::NoBuildModificationTime)?;

            Ok(())
        } else {
            log::error!("Compilation failed with non-zero exit code");
            Err(LazyJavaError::CompilationErrors)
        }
    }

    fn rebuild(args: &BuildArgs, ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Starting full rebuild");
        Classpath::generate(ctx)?;

        let status = compile_java(&ctx.src, &ctx.bin, &ctx.lib, &args.javac_args)
            .map_err(LazyJavaError::UnableToCompile)?;
        log::debug!("Java compilation completed");

        if status.success() {
            log::info!("Build successful");

            let file_time = filetime::FileTime::now();
            filetime::set_file_mtime(&ctx.bin, file_time)
                .map_err(LazyJavaError::NoBuildModificationTime)?;

            Ok(())
        } else {
            log::error!("Build failed with non-zero exit code");
            Err(LazyJavaError::CompilationErrors)
        }
    }
    fn show_dependancy_graph(ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Displaying dependency graph");
        let graph = DependancyGraph::create(&ctx.src)?;

        for (_key, entry) in graph.nodes.iter() {
            println!(" {}", entry.file_name,);
            for dep in &entry.dependancies {
                println!("  - {}", dep);
            }
            println!();
        }
        Ok(())
    }
    fn show_modified_files(ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Displaying modified files");
        let stale_files =
            find_modified_files(&ctx.bin, &ctx.src).map_err(LazyJavaError::NoStaleFilesError)?;

        for file in stale_files {
            println!("{}", file.to_string_lossy());
        }

        Ok(())
    }
    fn show_rebuild_files(ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Displaying files to rebuild");
        let graph = DependancyGraph::create(&ctx.src)?;

        let stale_files =
            find_modified_files(&ctx.bin, &ctx.src).map_err(LazyJavaError::NoStaleFilesError)?;

        let recompile = files_to_recompile(graph, stale_files)?;

        for file in recompile {
            println!("{}", file.to_string_lossy());
        }

        Ok(())
    }
    fn show_depentants_graph(ctx: &Context) -> Result<(), LazyJavaError> {
        log::info!("Displaying dependants graph");
        let graph = DependancyGraph::create(&ctx.src)?;
        for (_key, entry) in graph.nodes.iter() {
            println!(" {}", entry.file_name,);
            for dep in &entry.dependants {
                println!("  - {}", dep);
            }
            println!();
        }

        Ok(())
    }
    fn rebuild_classpath(ctx: &Context) -> Result<(), LazyJavaError> {
        Ok(Classpath::generate(ctx)?)
    }
}
