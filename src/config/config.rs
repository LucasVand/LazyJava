use std::{fs, io::ErrorKind, path::Path};

use colored::Colorize;

use crate::{
    CONFIG_FILE_NAME,
    args::{AddArgs, RemoveArgs},
    config::{config_error::ConfigError, config_structs::Config},
    lock_file::LockFile,
    maven_central::{MavenError, MavenIdBuf, fetch_artifact_metadata, pom::MavenDependancyList},
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

        let res = fs::write(root.join(CONFIG_FILE_NAME), str);
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
        root: &Path,
        lib: &Path,
    ) -> Result<(), ConfigError> {
        let mut lockfile = LockFile::fetch(root)?;

        let version: Result<String, MavenError> = match &add_args.artifact_version {
            Some(version) => Ok(version.to_string()),
            None => {
                let meta = fetch_artifact_metadata(&add_args.group, &add_args.artifact)?;
                Ok(meta.versioning.release)
            }
        };

        let version = version?;

        let id = MavenIdBuf::new(&add_args.group, &add_args.artifact, &version);
        let list = MavenDependancyList::new(id.clone())?;
        println!("{} {} to dependency list", "Adding".green().bold(), id);

        if !add_args.dry_run {
            self.dependancies.push(id.clone().into());
            lockfile.add_packages(list.into_iter().map(|p| p.into()).collect());
        }

        self.sync_lock_file(&mut lockfile, root, lib, add_args.dry_run)?;
        Ok(())
    }
    pub fn remove_package(
        &mut self,
        remove_args: &RemoveArgs,
        root: &Path,
        lib: &Path,
    ) -> Result<(), ConfigError> {
        let mut lockfile = LockFile::fetch(root)?;
        let pos = self
            .dependancies
            .iter()
            .position(|d| d.id.artifact == remove_args.artifact && d.id.group == remove_args.group);

        if pos.is_none() {
            return Err(ConfigError::PackageNotFound);
        }
        let pos = pos.unwrap();

        println!(
            "{} {} from dependency list",
            "Removing".green().bold(),
            self.dependancies[pos].id
        );

        if !remove_args.dry_run {
            self.dependancies.remove(pos);
        }

        self.sync_lock_file(&mut lockfile, root, lib, remove_args.dry_run)?;

        Ok(())
    }
    pub fn sync_lock_file(
        &self,
        lockfile: &mut LockFile,
        root: &Path,
        lib: &Path,
        dry_run: bool,
    ) -> Result<(), ConfigError> {
        lockfile.sync_with_root_packages(&self.dependancies)?;

        if !dry_run {
            lockfile.write(root)?;
        }

        lockfile.validate_current_packages(lib, dry_run)?;
        Ok(())
    }
}
