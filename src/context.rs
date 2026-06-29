use std::{
    env,
    path::{self, PathBuf},
};

use crate::{
    BUILD_FOLDER, LIB_FOLDER, SRC_FOLDER, args::LazyJavaArgs, config::Config,
    lazy_java_error::LazyJavaError, utils::find_root::find_root,
};

pub struct Context {
    pub src: PathBuf,
    pub bin: PathBuf,
    pub lib: PathBuf,
    pub root: PathBuf,
    pub current: PathBuf,

    pub relative_src: String,
    pub relative_bin: String,
    pub relative_lib: String,

    pub config: Config,
}
impl Context {
    pub fn new(args: &LazyJavaArgs) -> Result<Context, LazyJavaError> {
        let current = env::current_dir().map_err(LazyJavaError::NoCurrentDir)?;
        log::debug!("Current directory: {:?}", current);

        let root = find_root(&current).map_err(|e| {
            log::error!("Could not locate project root");
            LazyJavaError::NoRoot(e)
        })?;

        let root = root.unwrap_or(env::current_dir().map_err(|e| LazyJavaError::NoRoot(e))?);

        let config = Config::fetch(&root)?;

        log::info!("Project root: {:?}", root);

        let relative_src = Self::src_path(args, &config).to_string();
        let relative_lib = Self::lib_path(args, &config).to_string();
        let relative_bin = Self::bin_path(args, &config).to_string();

        let lib = root.join(&relative_lib);
        let src = root.join(&relative_src);
        let bin = root.join(&relative_bin);

        log::debug!("Source directory: {:?}", src);
        log::debug!("Build directory: {:?}", bin);
        log::debug!("Library directory: {:?}", lib);

        let ctx = Context {
            relative_bin,
            relative_lib,
            relative_src,
            src,
            bin,
            lib,
            root,
            current,
            config,
        };

        Ok(ctx)
    }

    pub fn assert_src_exists(&self) -> Result<(), LazyJavaError> {
        if !self.src.exists() {
            let path = path::absolute(self.src.clone()).unwrap();
            return Err(LazyJavaError::NoSource(path.to_string_lossy().into()));
        }
        Ok(())
    }

    pub fn assert_bin_exists(&self) -> Result<(), LazyJavaError> {
        if !self.bin.exists() {
            let path = path::absolute(self.bin.clone()).unwrap();
            return Err(LazyJavaError::NoBuild(path.to_string_lossy().into()));
        }
        Ok(())
    }

    pub fn assert_lib_exists(&self) -> Result<(), LazyJavaError> {
        if !self.lib.exists() {
            let path = path::absolute(self.lib.clone()).unwrap();
            return Err(LazyJavaError::NoLib(path.to_string_lossy().into()));
        }
        Ok(())
    }

    pub fn assert_build_lib_src(&self) -> Result<(), LazyJavaError> {
        self.assert_src_exists()?;
        self.assert_bin_exists()?;
        self.assert_lib_exists()?;
        Ok(())
    }

    fn src_path<'a>(args: &'a LazyJavaArgs, config: &'a Config) -> &'a str {
        if let Some(src) = &args.global_args.source {
            return src;
        } else if let Some(src) = &config.setup.src {
            return src;
        } else {
            return SRC_FOLDER;
        }
    }

    fn bin_path<'a>(args: &'a LazyJavaArgs, config: &'a Config) -> &'a str {
        if let Some(bin) = &args.global_args.build {
            return bin;
        } else if let Some(bin) = &config.setup.bin {
            return bin;
        } else {
            return BUILD_FOLDER;
        }
    }
    fn lib_path<'a>(args: &'a LazyJavaArgs, config: &'a Config) -> &'a str {
        if let Some(lib) = &args.global_args.lib {
            return lib;
        } else if let Some(lib) = &config.setup.lib {
            return lib;
        } else {
            return LIB_FOLDER;
        }
    }
}
