use std::{
    env,
    path::{self, Path, PathBuf},
};

use colored::Colorize;
use decompose::decompose;

use crate::{
    SRC_FOLDER, TARGET_FOLDER,
    args::LazyJavaArgs,
    config::ConfigTomlEdit,
    lazy_java_error::LazyJavaError,
    lock_file::LockFile,
    lsp::sync_lsp_config,
    utils::{GlobalContext, IOError, find_root::find_root, fs},
};

use super::ContextError;

#[decompose(ContextNoConfig, exclude(config))]
pub struct Context {
    pub src: PathBuf,
    pub bin: PathBuf,
    pub bin_processors: PathBuf,
    pub lib: PathBuf,
    pub lib_annotations: PathBuf,
    pub src_generated: PathBuf,
    pub root: PathBuf,
    pub current: PathBuf,
    pub target: PathBuf,

    pub relative_src: String,
    pub relative_bin: String,
    pub relative_lib: String,
    pub relative_lib_annotations: String,
    pub relative_src_generated: String,
    pub relative_target: String,
    pub relative_bin_processors: String,

    pub config: ConfigTomlEdit,
}

impl Context {
    pub fn new(args: &LazyJavaArgs) -> Result<Context, ContextError> {
        Self::new_internal(Some(args), None)
    }
    pub fn new_options(
        args: Option<&LazyJavaArgs>,
        config: Option<ConfigTomlEdit>,
    ) -> Result<Context, ContextError> {
        Self::new_internal(args, config)
    }
    fn new_internal(
        args: Option<&LazyJavaArgs>,
        config: Option<ConfigTomlEdit>,
    ) -> Result<Context, ContextError> {
        let current = env::current_dir().map_err(ContextError::NoCurrentDir)?;
        log::debug!("Current directory: {:?}", current);

        let root = find_root(&current).map_err(|e| {
            log::error!("Could not locate project root");
            ContextError::NoRoot(e)
        })?;

        let root = root.unwrap_or(env::current_dir().map_err(ContextError::NoRoot)?);

        // Anchor the process to the project root so any relative paths resolved
        // from here on (e.g. local dependency `path` values, `canonicalize`)
        // are interpreted relative to the project rather than the invocation
        // directory.
        env::set_current_dir(&root).map_err(ContextError::NoDir)?;

        let config = match config {
            Some(config) => Ok(config),
            None => ConfigTomlEdit::fetch(&root),
        }?;

        Self::assert_project_name(&config)?;

        log::info!("Project root: {:?}", root);

        let relative_target = Self::target_path(args, &config).to_string();

        let relative_src = Self::src_path(args, &config).to_string();
        let relative_lib: String = "lib".into();
        let relative_bin: String = "bin".into();
        let relative_lib_annotations = format!("{}-annotations", relative_lib);
        let relative_src_generated: String = "generated-source".into();
        let relative_bin_processors: String = "processor-bin".into();

        let target = root.join(&relative_target);

        let lib = target.join(&relative_lib);
        let src = root.join(&relative_src);
        let bin = target.join(&relative_bin);
        let lib_annotations = target.join(&relative_lib_annotations);
        let src_generated = target.join(&relative_src_generated);
        let bin_processors = target.join(&relative_bin_processors);

        log::debug!("Source directory: {:?}", src);
        log::debug!("Build directory: {:?}", bin);
        log::debug!("Library directory: {:?}", lib);

        let ctx = Context {
            relative_target,
            target,
            relative_bin,
            relative_lib,
            relative_src,
            src,
            bin,
            lib,
            root,
            current,
            config,
            lib_annotations,
            relative_lib_annotations,
            src_generated,
            relative_src_generated,
            bin_processors,
            relative_bin_processors,
        };

        Ok(ctx)
    }

    fn assert_project_name(config: &ConfigTomlEdit) -> Result<(), ContextError> {
        if config.project().and_then(|p| p.name()).is_none() {
            return Err(ContextError::MissingProjectName);
        }
        Ok(())
    }

    pub fn name(&self) -> String {
        self.config
            .project()
            .and_then(|p| p.name())
            .expect("project name is asserted during context construction")
    }

    /// Resolve a possibly-relative path against the project root. Absolute
    /// paths are returned unchanged, and relative paths (e.g. a local
    /// dependency's `path` value) are anchored at [`Context::root`] rather than
    /// the process working directory, keeping the config portable.
    pub fn resolve(&self, path: &Path) -> PathBuf {
        if path.is_relative() {
            self.root.join(path)
        } else {
            path.to_path_buf()
        }
    }

    pub fn assert_src_exists(&self) -> Result<(), ContextError> {
        if !self.src.exists() {
            let path = path::absolute(self.src.clone()).unwrap();
            return Err(ContextError::NoSource(path.to_string_lossy().into()));
        }
        Ok(())
    }

    pub fn ensure_bin_exists(&self) -> Result<(), ContextError> {
        if !self.bin.exists() {
            println!(
                "{} {}",
                "Creating".green().bold(),
                format!("build directory: {}", self.bin.display())
            );
            fs::create_dir_all(&self.bin)
                .map_err(|e| IOError::new("creating build directory", &self.bin, e))?;
        }
        if !self.bin_processors.exists() {
            println!(
                "{} {}",
                "Creating".green().bold(),
                format!(
                    "build processor directory: {}",
                    self.bin_processors.display()
                )
            );
            fs::create_dir_all(&self.bin_processors).map_err(|e| {
                IOError::new(
                    "creating build processor directory",
                    &self.bin_processors,
                    e,
                )
            })?;
        }
        Ok(())
    }

    pub fn ensure_lib_exists(&self) -> Result<(), ContextError> {
        if !self.lib.exists() {
            println!(
                "{} {}",
                "Creating".green().bold(),
                format!("library directory: {}", self.lib.display())
            );
            fs::create_dir_all(&self.lib)
                .map_err(|e| IOError::new("creating library directory", &self.lib, e))?;
        }
        if !self.lib_annotations.exists() {
            println!(
                "{} {}",
                "Creating".green().bold(),
                format!(
                    "annotation library directory: {}",
                    self.lib_annotations.display()
                )
            );
            fs::create_dir_all(&self.lib_annotations).map_err(|e| {
                IOError::new(
                    "creating annotation library directory",
                    &self.lib_annotations,
                    e,
                )
            })?;
        }
        if !self.src_generated.exists() {
            println!(
                "{} {}",
                "Creating".green().bold(),
                format!("generated src directory: {}", self.src_generated.display())
            );
            fs::create_dir_all(&self.src_generated).map_err(|e| {
                IOError::new("creating generated src directory", &self.src_generated, e)
            })?;
        }
        Ok(())
    }

    pub fn ensure_target_exists(&self) -> Result<(), ContextError> {
        if !self.target.exists() {
            println!(
                "{} {}",
                "Creating".green().bold(),
                format!("target directory: {}", self.target.display())
            );
            fs::create_dir_all(&self.target)
                .map_err(|e| IOError::new("creating target directory", &self.target, e))?;
        }
        Ok(())
    }

    pub fn assert_all(&self) -> Result<(), ContextError> {
        self.assert_src_exists()?;
        self.ensure_bin_exists()?;
        self.ensure_lib_exists()?;
        self.ensure_target_exists()?;
        Ok(())
    }
    pub fn assert_packages(self) -> Result<Context, LazyJavaError> {
        let (inc, exc) = self.decompose();

        let mut lockfile = LockFile::fetch(&inc.root)?;

        exc.config.sync_lock_file(&mut lockfile, &inc)?;

        let ctx = Context::compose(inc, exc);

        if !GlobalContext::is_dry_run() {
            sync_lsp_config(&ctx)?;
        }

        Ok(ctx)
    }
    fn target_path<'a>(args: Option<&'a LazyJavaArgs>, config: &'a ConfigTomlEdit) -> String {
        if args.is_some()
            && let Some(target) = &args.unwrap().global_args.target
        {
            target.to_string()
        } else if let Some(setup) = &config.setup()
            && let Some(target) = setup.target()
        {
            target
        } else {
            TARGET_FOLDER.to_string()
        }
    }

    fn src_path<'a>(args: Option<&'a LazyJavaArgs>, config: &'a ConfigTomlEdit) -> String {
        if args.is_some()
            && let Some(src) = &args.unwrap().global_args.source
        {
            src.to_string()
        } else if let Some(setup) = &config.setup()
            && let Some(src) = setup.src()
        {
            src
        } else {
            SRC_FOLDER.to_string()
        }
    }
}
