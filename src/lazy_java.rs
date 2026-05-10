use std::{
    env, fs,
    path::{self, PathBuf},
};

use crate::{
    args::{LazyJavaArgs, LazyJavaCommand},
    lazy_java_error::LazyJavaError,
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
        let current = env::current_dir().map_err(|e| LazyJavaError::NoCurrentDir(e))?;
        log::debug!("Current directory: {:?}", current);

        let root = find_root(&current).map_err(|_e| {
            log::error!("Could not locate project root");
            return LazyJavaError::NoRoot;
        })?;
        let root = root.unwrap_or(env::current_dir().map_err(|_e| return LazyJavaError::NoRoot)?);
        log::info!("Project root: {:?}", root);

        let mut lib = root.clone();
        lib.push(args.global_args.lib.clone());
        let mut src = root.clone();
        src.push(args.global_args.source.clone());
        let mut build = root.clone();
        build.push(args.global_args.build.clone());

        log::debug!("Source directory: {:?}", src);
        log::debug!("Build directory: {:?}", build);
        log::debug!("Library directory: {:?}", lib);

        let lazy_java = LazyJava {
            src: src,
            build: build,
            lib: lib,
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
            log::info!("Build directory not found, creating: {:?}", self.build);
            fs::create_dir_all(&self.build)
                .map_err(|e| LazyJavaError::NoCreateBuildDirectory(e))?;
            log::info!("Created build directory: {:?}", self.build);
        }
        if !self.lib.exists() {
            log::info!("Lib directory not found, creating: {:?}", self.lib);
            fs::create_dir_all(&self.lib)
                .map_err(|e| LazyJavaError::NoCreateLibDirectory(e))?;
            log::info!("Created lib directory: {:?}", self.lib);
        }
        Ok(())
    }
}
