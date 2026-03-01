use std::{env, path::PathBuf};

use clap::Parser;

use crate::{
    args::{LazyJavaArgs, LazyJavaCommand},
    lazy_java_error::LazyJavaError,
    utils::find_root::{find_file_in_dir, find_root},
};

#[derive(Debug, Clone)]
pub struct LazyJava {
    pub src: PathBuf,
    pub build: PathBuf,
    pub lib: PathBuf,
    pub root: PathBuf,
    pub args: LazyJavaArgs,
}

impl LazyJava {
    pub fn new() -> Result<LazyJava, LazyJavaError> {
        let current = env::current_dir().map_err(|e| return LazyJavaError::NoCurrentDir(e))?;
        let args = LazyJavaArgs::parse();
        let root = find_root(&current).map_err(|_e| return LazyJavaError::NoRoot)?;

        let src = find_file_in_dir(&root, &args.global_args.source)
            .map_err(|_e| return LazyJavaError::NoSource(args.global_args.source.clone()))?;
        let build = find_file_in_dir(&root, &args.global_args.build)
            .map_err(|_e| return LazyJavaError::NoBuild(args.global_args.build.clone()))?;

        let lib = find_file_in_dir(&root, &args.global_args.lib)
            .map_err(|_e| return LazyJavaError::NoLib(args.global_args.lib.clone()))?;

        let lazy_java = LazyJava {
            src: src.path(),
            build: build.path(),
            lib: lib.path(),
            root: root,
            args: args,
        };

        return Ok(lazy_java);
    }

    pub fn execute(&self) -> Result<(), LazyJavaError> {
        match &self.args.command {
            LazyJavaCommand::Run { args } => self.run(args)?,
            LazyJavaCommand::Build { args } => self.build(args)?,
            LazyJavaCommand::Clean {} => self.clean()?,
            LazyJavaCommand::Find { args } => self.find(args)?,
        };
        return Ok(());
    }
}
