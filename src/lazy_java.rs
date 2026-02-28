use std::{env, error::Error, io, path::PathBuf};

use clap::Parser;

use crate::{
    args::{LazyJavaArgs, LazyJavaCommand},
    find_root::{find_file_in_dir, find_root},
    lazy_java_error::LazyJavaError,
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
    pub fn new() -> Result<LazyJava, io::Error> {
        let current = env::current_dir()?;
        let args = LazyJavaArgs::parse();
        let root = find_root(&current)?;

        let src = find_file_in_dir(&root, &args.global_args.source)?;
        let build = find_file_in_dir(&root, &args.global_args.build)?;

        let lib = find_file_in_dir(&root, &args.global_args.lib)?;

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
