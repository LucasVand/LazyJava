use std::{
    env,
    path::{self, PathBuf},
};

use crate::{
    args::{LazyJavaArgs, LazyJavaCommand},
    lazy_java,
    lazy_java_error::LazyJavaError,
    logger::logger::Logger,
    lsp::classpath,
    utils::find_root::find_root,
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
    pub fn new(args: LazyJavaArgs) -> Result<LazyJava, LazyJavaError> {
        let current = env::current_dir().map_err(|e| return LazyJavaError::NoCurrentDir(e))?;
        let root = find_root(&current).map_err(|_e| {
            Logger::verbose_elog("Could not locate root");
            return LazyJavaError::NoRoot;
        })?;
        let root = root.unwrap_or(env::current_dir().map_err(|_e| return LazyJavaError::NoRoot)?);

        let mut lib = root.clone();
        lib.push(args.global_args.lib.clone());
        let mut src = root.clone();
        src.push(args.global_args.source.clone());
        let mut build = root.clone();
        build.push(args.global_args.build.clone());

        Logger::verbose(args.global_args.verbose);

        let lazy_java = LazyJava {
            src: src,
            build: build,
            lib: lib,
            root: root,
            args: args,
        };

        println!("{:?}", classpath::Classpath::generate(&lazy_java));

        return Ok(lazy_java);
    }

    pub fn execute(&self) -> Result<(), LazyJavaError> {
        match &self.args.command {
            LazyJavaCommand::Run { args } => self.run(args)?,
            LazyJavaCommand::Build { args } => self.build(args)?,
            LazyJavaCommand::Clean {} => self.clean()?,
            LazyJavaCommand::Find { args } => self.find(args)?,
            LazyJavaCommand::Create { args } => self.create(args)?,
        };
        return Ok(());
    }

    pub fn assert_build_lib_src(&self) -> Result<(), LazyJavaError> {
        if !self.src.exists() {
            let path = path::absolute(self.src.clone()).unwrap();
            return Err(LazyJavaError::NoSource(path.to_string_lossy().into()));
        }

        if !self.build.exists() {
            let path = path::absolute(self.build.clone()).unwrap();
            return Err(LazyJavaError::NoBuild(path.to_string_lossy().into()));
        }
        if !self.lib.exists() {
            let path = path::absolute(self.lib.clone()).unwrap();
            return Err(LazyJavaError::NoLib(path.to_string_lossy().into()));
        }
        return Ok(());
    }
}
