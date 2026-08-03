use colored::Colorize;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::PathBuf;
use std::process::ExitStatus;

use crate::Context;
use crate::args::{BuildArgs, BuildCommand, BuildSubCommand, JarArgs};
use crate::build::build_jar::build_jar;
use crate::build::compile::compile_java;
use crate::build::graph::Graph;
use crate::build::metadata::{BuildMetadata, hash_directory, save_metadata};
use crate::build::processors::build_processors;
use crate::build::resources::copy_resources;
use crate::lazy_java::LazyJava;

use crate::build::BuildError;
use crate::lsp::classpath::Classpath;
use crate::utils::find_main::find_java_files_glob;
use crate::utils::{GlobalContext, IOError, Timings};

impl LazyJava {
    pub fn build(args: &BuildCommand, ctx: &Context) -> Result<(), BuildError> {
        if let Some(build_command) = &args.command {
            match build_command {
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

        let mut timings = Timings::start("Build");
        let build_data = BuildMetadata::fetch(&ctx.target);

        let current_lib_hash = hash_directory(&ctx.lib);

        let lib_hash_match = build_data
            .as_ref()
            .is_some_and(|t| current_lib_hash == t.lib_hash);
        timings.record_current("Metadata parse");

        build_processors(args, ctx)?;
        timings.record_current("Processor compile");

        let glob = build_globset(ctx);

        let files: Result<Vec<PathBuf>, BuildError> =
            if args.build_all || !lib_hash_match || build_data.is_none() {
                let files = find_java_files_glob(&ctx.src, &glob);
                println!(
                    "{} using full build ({} file{})",
                    "Compiling".bold().green(),
                    files.len(),
                    if files.len() == 1 { "" } else { "s" }
                );
                Ok(files)
            } else {
                let graph = Graph::from_path(&ctx.src, &glob)
                    .map_err(|e| IOError::new("finding modified files", &ctx.src, e))?;

                let stale = graph.stale_files(build_data.as_ref().unwrap().time_stamp);
                println!(
                    "{} using incrimental build ({} stale file{})",
                    "Compiling".bold().green(),
                    stale.len(),
                    if stale.len() == 1 { "" } else { "s" }
                );

                Ok(stale)
            };
        let files = files?;
        if args.show_compiled {
            for f in &files {
                println!("  {}", f.display());
            }
        }

        timings.record_current("Incrimental prcoessing");
        let mut status: Option<ExitStatus> = None;
        if !files.is_empty() {
            let e_status = compile_java(files, &ctx.bin, ctx, &args.javac_args, true)
                .map_err(|e| IOError::new("compiling java files", &ctx.bin, e))?;
            timings.record_current("Compile");
            status = Some(e_status);
        }

        copy_resources(ctx)?;
        timings.record_current("Copy resources");

        if let Some(status) = status {
            save_metadata(ctx, status, build_data)?;

            if !status.success() {
                return Err(BuildError::MainCompilationErrors);
            }
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

    fn show_dependancy_graph(ctx: &Context) -> Result<(), BuildError> {
        let glob = build_globset(ctx);

        let graph = Graph::from_path(&ctx.src, &glob)
            .map_err(|e| IOError::new("finding modified files", &ctx.src, e))?;

        let src = &ctx.src;
        println!("{}", "Dependancy graph".bold().green());
        for (key, entry) in graph.dependencies.iter() {
            if let Some(name) = key.file_name()
                && let Ok(rel) = key.strip_prefix(src)
            {
                let f = format!("({})", rel.to_string_lossy().dimmed());
                println!("{} {}", name.display(), f.dimmed());
                for dep in entry {
                    if let Some(n) = dep.file_name()
                        && let Ok(rel) = dep.strip_prefix(src)
                    {
                        let f = format!("({})", rel.to_string_lossy().dimmed());
                        println!("  - {} {}", n.display(), f.dimmed());
                    }
                }
                println!();
            }
        }
        Ok(())
    }

    fn show_rebuild_files(ctx: &Context) -> Result<(), BuildError> {
        let build_data = BuildMetadata::fetch(&ctx.target).unwrap_or_default();

        let glob = build_globset(ctx);
        let graph = Graph::from_path(&ctx.src, &glob)
            .map_err(|e| IOError::new("finding modified files", &ctx.src, e))?;

        let stale = graph.stale_files(build_data.time_stamp);

        println!(
            "{} {} file{} to recompile",
            "Stale".bold().green(),
            stale.len(),
            if stale.len() == 1 { "" } else { "s" }
        );
        for file in stale {
            println!("  {}", file.to_string_lossy());
        }

        Ok(())
    }
    fn show_depentants_graph(ctx: &Context) -> Result<(), BuildError> {
        let glob = build_globset(ctx);

        let graph = Graph::from_path(&ctx.src, &glob)
            .map_err(|e| IOError::new("finding modified files", &ctx.src, e))?;

        let src = &ctx.src;
        println!("{}", "Dependants graph".bold().green());
        for (key, entry) in graph.dependents.iter() {
            if let Some(name) = key.file_name()
                && let Ok(rel) = key.strip_prefix(src)
            {
                let f = format!("({})", rel.to_string_lossy().dimmed());
                println!("{} {}", name.display(), f.dimmed());
                for dep in entry {
                    if let Some(n) = dep.file_name()
                        && let Ok(rel) = dep.strip_prefix(src)
                    {
                        let f = format!("({})", rel.to_string_lossy().dimmed());
                        println!("  - {} {}", n.display(), f.dimmed());
                    }
                }
                println!();
            }
        }

        Ok(())
    }
    fn rebuild_classpath(ctx: &Context) -> Result<(), BuildError> {
        Ok(Classpath::generate(ctx)?)
    }
}
fn build_globset(ctx: &Context) -> GlobSet {
    let exclude = if let Some(s) = ctx.config.setup()
        && let Some(list) = s.exclude()
    {
        list
    } else {
        Vec::new()
    };
    let mut builder = GlobSetBuilder::new();
    for rule in exclude {
        if let Ok(glob_rule) = Glob::new(&rule) {
            builder.add(glob_rule);
        } else {
            log::warn!("Invalid glob rule, \"{}\" is not a valid rule", rule);
        }
    }

    builder.build().unwrap()
}
