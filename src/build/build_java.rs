use crate::args::{BuildArgs, BuildCommand, BuildSubCommand};
use crate::build::find_stale_files::{files_to_recompile, find_modified_files};
use crate::dependancy_graph::graph::DependancyGraph;
use crate::lazy_java::LazyJava;

use crate::lazy_java_error::LazyJavaError;
use crate::logger::logger::Logger;
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
            }
        } else {
            return self.build_java(&args.args);
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
        let graph = DependancyGraph::create(&self.src)?;

        let modified_files = find_modified_files(&self.build, &self.src)
            .map_err(|e| return LazyJavaError::NoStaleFilesError(e))?;

        if modified_files.is_empty() {
            return Ok(());
        }

        let recompile = files_to_recompile(graph, modified_files)?;

        let status = compile_java_files(&self.build, &self.lib, &args.javac_args, recompile)
            .map_err(|e| return LazyJavaError::UnableToCompile(e))?;

        Logger::verbose_elog("Compiled Java");
        if status.success() {
            Logger::verbose_elog("Compilation Exit Code Successful");

            let file_time = filetime::FileTime::now();
            filetime::set_file_mtime(&self.build, file_time)
                .map_err(|e| return LazyJavaError::NoBuildModificationTime(e))?;

            return Ok(());
        } else {
            Logger::verbose_elog("Compilation Exit Code Not Successful");
            return Err(LazyJavaError::CompilationErrors);
        }
    }

    fn rebuild(&self, args: &BuildArgs) -> Result<(), LazyJavaError> {
        let status = compile_java(&self.src, &self.build, &self.lib, &args.javac_args)
            .map_err(|e| return LazyJavaError::UnableToCompile(e))?;
        Logger::verbose_elog("Compiled Java");

        if status.success() {
            Logger::verbose_elog("Compilation Exit Code Successful");

            let file_time = filetime::FileTime::now();
            filetime::set_file_mtime(&self.build, file_time)
                .map_err(|e| return LazyJavaError::NoBuildModificationTime(e))?;

            return Ok(());
        } else {
            Logger::verbose_elog("Compilation Exit Code Not Successful");
            return Err(LazyJavaError::CompilationErrors);
        }
    }
    fn show_dependancy_graph(&self) -> Result<(), LazyJavaError> {
        let graph = DependancyGraph::create(&self.src)?;

        for (_key, entry) in graph.nodes.iter() {
            println!(" {}", entry.file_name,);
            for dep in &entry.dependancies {
                println!("  - {}", dep);
            }
            println!("");
        }
        return Ok(());
    }
    fn show_modified_files(&self) -> Result<(), LazyJavaError> {
        let stale_files = find_modified_files(&self.build, &self.src)
            .map_err(|e| return LazyJavaError::NoStaleFilesError(e))?;

        for file in stale_files {
            println!("{}", file.to_string_lossy());
        }

        return Ok(());
    }
    fn show_rebuild_files(&self) -> Result<(), LazyJavaError> {
        let graph = DependancyGraph::create(&self.src)?;

        let stale_files = find_modified_files(&self.build, &self.src)
            .map_err(|e| return LazyJavaError::NoStaleFilesError(e))?;

        let recompile = files_to_recompile(graph, stale_files)?;

        for file in recompile {
            println!("{}", file.to_string_lossy());
        }

        return Ok(());
    }
    fn show_depentants_graph(&self) -> Result<(), LazyJavaError> {
        let graph = DependancyGraph::create(&self.src)?;
        for (_key, entry) in graph.nodes.iter() {
            println!(" {}", entry.file_name,);
            for dep in &entry.dependants {
                println!("  - {}", dep);
            }
            println!("");
        }

        return Ok(());
    }
}
