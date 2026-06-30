use std::{fs, io::ErrorKind, path::Path};

use colored::Colorize;
use toml_edit::{DocumentMut, Item};

use crate::{
    CONFIG_FILE_NAME,
    args::{AddArgs, RemoveArgs},
    config::{config_error::ConfigError, config_structs::Config},
    context::ContextNoConfig,
    lock_file::LockFile,
    maven_central::{MavenError, MavenIdBuf, PartialMavenIdBuf, fetch_artifact_metadata},
};

impl Config {
    fn new() -> Self {
        Self::default()
    }
    pub fn fetch(root: &Path) -> Result<Config, ConfigError> {
        let fs_file = Self::fetch_from_fs(root);

        if let Ok(lockfile) = fs_file {
            Ok(lockfile)
        } else {
            match fs_file.err().unwrap() {
                ConfigError::NoFound => Ok(Config::new()),
                err => Err(err),
            }
        }
    }
    fn fetch_from_fs(root: &Path) -> Result<Config, ConfigError> {
        let file = fs::read_to_string(root.join(CONFIG_FILE_NAME)).map_err(|e| match e.kind() {
            ErrorKind::NotFound => ConfigError::NoFound,
            ErrorKind::PermissionDenied => {
                ConfigError::PermissionDenied(root.to_string_lossy().into())
            }
            _ => ConfigError::NoFound,
        })?;

        let lockfile: Config = toml::from_str(&file)?;

        Ok(lockfile)
    }
    pub fn write(&self, root: &Path) -> Result<(), ConfigError> {
        let str = toml::to_string_pretty(self)?;

        let mut doc: DocumentMut = str.parse().unwrap();

        if let Some(Item::Table(table)) = doc.get_mut("dependancies") {
            for (_key, item) in table.iter_mut() {
                if let Item::Table(sub_table) = item {
                    let inline_version = std::mem::take(sub_table).into_inline_table();

                    *item = toml_edit::value(inline_version);
                }
            }
        }

        let res = fs::write(root.join(CONFIG_FILE_NAME), doc.to_string());
        if let Err(err) = res {
            return match err.kind() {
                ErrorKind::PermissionDenied => {
                    Err(ConfigError::PermissionDenied(root.to_string_lossy().into()))
                }
                _ => Err(ConfigError::IoError(err)),
            };
        }

        Ok(())
    }
    pub fn add_package(
        &mut self,
        add_args: &AddArgs,
        ctx: &ContextNoConfig,
    ) -> Result<(), ConfigError> {
        let mut lockfile = LockFile::fetch(&ctx.root)?;

        let version: Result<String, MavenError> = match &add_args.artifact_version {
            Some(version) => Ok(version.to_string()),
            None => {
                let meta = fetch_artifact_metadata(&add_args.group, &add_args.artifact)?;
                Ok(meta.versioning.release)
            }
        };

        let version = version?;

        let id = MavenIdBuf::new(add_args.group.clone(), add_args.artifact.clone(), version);
        println!("{} {} to dependency list", "Adding".green().bold(), id);

        if !ctx.dry_run {
            self.dependancies
                .insert(id.clone().into(), id.clone().into());
            lockfile.add_package(id)?;
        }

        self.sync_lock_file(&mut lockfile, ctx)?;
        Ok(())
    }
    pub fn remove_package(
        &mut self,
        remove_args: &RemoveArgs,
        ctx: &ContextNoConfig,
    ) -> Result<(), ConfigError> {
        let mut lockfile = LockFile::fetch(&ctx.root)?;

        let partial_id =
            PartialMavenIdBuf::new(remove_args.group.clone(), remove_args.artifact.clone());

        let package = self.dependancies.get(&partial_id);
        if package.is_none() {
            return Err(ConfigError::PackageNotFound);
        }

        let package = package.unwrap();

        println!(
            "{} {} from dependency list",
            "Removing".green().bold(),
            package.id,
        );

        if !ctx.dry_run {
            self.dependancies.remove(&partial_id);
        }

        self.sync_lock_file(&mut lockfile, ctx)?;

        Ok(())
    }
    pub fn sync_lock_file(
        &self,
        lockfile: &mut LockFile,
        ctx: &ContextNoConfig,
    ) -> Result<(), ConfigError> {
        lockfile.sync_with_root_packages(&self.dependancies)?;

        if !ctx.dry_run {
            lockfile.write(&ctx.root)?;
        }

        lockfile.validate_current_packages(ctx)?;
        Ok(())
    }
}
