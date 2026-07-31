use std::process::ExitStatus;

use colored::Colorize;
use log::debug;

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
            return Ok(());
        }
    }
    pub fn build_jar(args: &JarArgs, ctx: &Context) -> Result<(), BuildError> {
        Self::build_java(&args.build_args, ctx)?;
        build_jar(args, ctx)?;
        return Ok(());
    }

    pub fn build_java(args: &BuildArgs, ctx: &Context) -> Result<ExitStatus, BuildError> {
        if ctx.dry_run {
            println!("{} java sources", "Compiling".bold().green());
            return Ok(ExitStatus::default());
        }

        let build_data = BuildMetadata::fetch(&ctx.target);
        if build_data.is_none() {
            debug!("Could not find build data, full rebuild");
            if !build_processors(args, ctx)? {
                return Err(BuildError::CompilationErrors);
            }

            let status = Self::rebuild(args, ctx)?;
            copy_resources(ctx)?;
            save_metadata(ctx, status, None)?;

            return Ok(status);
        }
        let build_data = build_data.unwrap();
        debug!("Found build data ");

        // before the processors that modify the build dir
        let current_bin_hash = hash_directory(&ctx.bin);

        let current_lib_hash = hash_directory(&ctx.lib);
        log::debug!(
            "bin_hash: stored={}, current={}, match={}",
            build_data.bin_hash,
            current_bin_hash,
            current_bin_hash == build_data.bin_hash
        );
        log::debug!(
            "lib_hash: stored={}, current={}, match={}",
            build_data.lib_hash,
            current_lib_hash,
            current_lib_hash == build_data.lib_hash
        );
        let bin_hash_match = current_bin_hash == build_data.bin_hash;
        let lib_hash_match = current_lib_hash == build_data.lib_hash;
        debug!("Lib Hash Match: {}", lib_hash_match);
        debug!("Bin Hash Match: {}", bin_hash_match);

        if !build_processors(args, ctx)? {
            return Err(BuildError::CompilationErrors);
        }

        // Cannot hash bin because lsp will overwrite it and change the hash
        let status = if args.build_all || !lib_hash_match {
            Self::rebuild(args, ctx)
        } else {
            Self::incrimental_build(args, ctx, &build_data)
        }?;
        copy_resources(ctx)?;
        save_metadata(ctx, status, Some(build_data))?;

        Ok(status)
    }
    fn incrimental_build(
        args: &BuildArgs,
        ctx: &Context,
        build_data: &BuildMetadata,
    ) -> Result<ExitStatus, BuildError> {
        log::info!("Starting incremental build");
        Classpath::generate_if_stale(ctx)?;

        let graph = DependancyGraph::create(&ctx.src)?;
        log::debug!("Created dependency graph");

        let modified_files =
            find_modified_files(build_data, &ctx.src).map_err(BuildError::NoStaleFilesError)?;
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

        log::debug!("Found {} modified files", modified_files.len());
        let recompile = files_to_recompile(graph, modified_files)?;

        println!(
            "{} {} source file{}",
            "Compiling".bold().green(),
            recompile.len(),
            if recompile.len() == 1 { "" } else { "s" }
        );
        log::debug!("Need to recompile {} files", recompile.len());

        let status = compile_java(recompile, &ctx.bin, ctx, &args.javac_args, true)
            .map_err(BuildError::UnableToCompile)?;

        log::debug!("Java compilation completed status {}", status);
        return Ok(status);
    }

    fn rebuild(args: &BuildArgs, ctx: &Context) -> Result<ExitStatus, BuildError> {
        println!("{} using full rebuild", "Compiling".bold().green());
        log::info!("Starting full rebuild");
        Classpath::generate(ctx)?;

        let files = find_java_files(&ctx.src);

        let status = compile_java(files, &ctx.bin, ctx, &args.javac_args, true)
            .map_err(BuildError::UnableToCompile)?;
        log::debug!("Java compilation completed status {}", status);

        Ok(status)
    }
    fn show_dependancy_graph(ctx: &Context) -> Result<(), BuildError> {
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
    fn show_modified_files(ctx: &Context) -> Result<(), BuildError> {
        let build_data = BuildMetadata::fetch(&ctx.target).unwrap_or(BuildMetadata::new());

        log::info!("Displaying modified files");
        let stale_files =
            find_modified_files(&build_data, &ctx.src).map_err(BuildError::NoStaleFilesError)?;

        for file in stale_files {
            println!("{}", file.to_string_lossy());
        }

        Ok(())
    }
    fn show_rebuild_files(ctx: &Context) -> Result<(), BuildError> {
        let build_data = BuildMetadata::fetch(&ctx.target).unwrap_or(BuildMetadata::new());

        log::info!("Displaying files to rebuild");
        let graph = DependancyGraph::create(&ctx.src)?;

        let stale_files =
            find_modified_files(&build_data, &ctx.src).map_err(BuildError::NoStaleFilesError)?;

        let recompile = files_to_recompile(graph, stale_files)?;

        for file in recompile {
            println!("{}", file.to_string_lossy());
        }

        Ok(())
    }
    fn show_depentants_graph(ctx: &Context) -> Result<(), BuildError> {
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
    fn rebuild_classpath(ctx: &Context) -> Result<(), BuildError> {
        Ok(Classpath::generate(ctx)?)
    }
}
