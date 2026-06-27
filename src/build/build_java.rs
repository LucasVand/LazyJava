use crate::args::{BuildArgs, BuildCommand, BuildSubCommand};
use crate::build::find_stale_files::{files_to_recompile, find_modified_files};
use crate::dependancy_graph::graph::DependancyGraph;
use crate::lazy_java::LazyJava;

use crate::lazy_java_error::LazyJavaError;
use crate::lsp::classpath::Classpath;
use crate::utils::processes::{compile_java, compile_java_files};

impl LazyJava {
    pub fn build(&self, args: &BuildCommand) -> Result<(), LazyJavaError> {
        self.assert_build_lib_src()?;

        if let Some(build_command) = &args.command {
            match build_command {
                BuildSubCommand::Modified {} => self.show_modified_files(),
                BuildSubCommand::Dependancies {} => self.show_dependancy_graph(),
                BuildSubCommand::Dependants {} => self.show_depentants_graph(),
                BuildSubCommand::Stale {} => self.show_rebuild_files(),
                BuildSubCommand::Classpath {} => self.rebuild_classpath(),
            }
        } else {
            self.build_java(&args.args)
        }
    }
    pub fn build_java(&self, args: &BuildArgs) -> Result<(), LazyJavaError> {
        if args.build_all {
            self.rebuild(args)
        } else {
            self.incrimental_build(args)
        }
    }
    fn incrimental_build(&self, args: &BuildArgs) -> Result<(), LazyJavaError> {
        log::info!("Starting incremental build");
        Classpath::generate_if_stale(self)?;

        let graph = DependancyGraph::create(&self.src)?;
        log::debug!("Created dependency graph");

        let modified_files = find_modified_files(&self.build, &self.src)
            .map_err(LazyJavaError::NoStaleFilesError)?;

        if modified_files.is_empty() {
            log::info!("No modified files, skipping compilation");
            return Ok(());
        }

        log::debug!("Found {} modified files", modified_files.len());
        let recompile = files_to_recompile(graph, modified_files)?;
        log::debug!("Need to recompile {} files", recompile.len());

        let status = compile_java_files(&self.build, &self.lib, &args.javac_args, recompile)
            .map_err(LazyJavaError::UnableToCompile)?;

        log::debug!("Java compilation completed");
        if status.success() {
            log::info!("Compilation successful");

            let file_time = filetime::FileTime::now();
            filetime::set_file_mtime(&self.build, file_time)
                .map_err(LazyJavaError::NoBuildModificationTime)?;

            Ok(())
        } else {
            log::error!("Compilation failed with non-zero exit code");
            Err(LazyJavaError::CompilationErrors)
        }
    }

    fn rebuild(&self, args: &BuildArgs) -> Result<(), LazyJavaError> {
        log::info!("Starting full rebuild");
        Classpath::generate(self)?;

        let status = compile_java(&self.src, &self.build, &self.lib, &args.javac_args)
            .map_err(LazyJavaError::UnableToCompile)?;
        log::debug!("Java compilation completed");

        if status.success() {
            log::info!("Build successful");

            let file_time = filetime::FileTime::now();
            filetime::set_file_mtime(&self.build, file_time)
                .map_err(LazyJavaError::NoBuildModificationTime)?;

            Ok(())
        } else {
            log::error!("Build failed with non-zero exit code");
            Err(LazyJavaError::CompilationErrors)
        }
    }
    fn show_dependancy_graph(&self) -> Result<(), LazyJavaError> {
        log::info!("Displaying dependency graph");
        let graph = DependancyGraph::create(&self.src)?;

        for (_key, entry) in graph.nodes.iter() {
            println!(" {}", entry.file_name,);
            for dep in &entry.dependancies {
                println!("  - {}", dep);
            }
            println!();
        }
        Ok(())
    }
    fn show_modified_files(&self) -> Result<(), LazyJavaError> {
        log::info!("Displaying modified files");
        let stale_files = find_modified_files(&self.build, &self.src)
            .map_err(LazyJavaError::NoStaleFilesError)?;

        for file in stale_files {
            println!("{}", file.to_string_lossy());
        }

        Ok(())
    }
    fn show_rebuild_files(&self) -> Result<(), LazyJavaError> {
        log::info!("Displaying files to rebuild");
        let graph = DependancyGraph::create(&self.src)?;

        let stale_files = find_modified_files(&self.build, &self.src)
            .map_err(LazyJavaError::NoStaleFilesError)?;

        let recompile = files_to_recompile(graph, stale_files)?;

        for file in recompile {
            println!("{}", file.to_string_lossy());
        }

        Ok(())
    }
    fn show_depentants_graph(&self) -> Result<(), LazyJavaError> {
        log::info!("Displaying dependants graph");
        let graph = DependancyGraph::create(&self.src)?;
        for (_key, entry) in graph.nodes.iter() {
            println!(" {}", entry.file_name,);
            for dep in &entry.dependants {
                println!("  - {}", dep);
            }
            println!();
        }

        Ok(())
    }
    fn rebuild_classpath(&self) -> Result<(), LazyJavaError> {
        Ok(Classpath::generate(self)?)
    }
}
