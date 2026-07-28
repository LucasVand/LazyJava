use std::{collections::HashMap, fs, io::ErrorKind, path::Path};

use colored::Colorize;
use log::debug;

use crate::{
    CONFIG_FILE_NAME,
    args::{AddArgs, RemoveArgs},
    config::{ConfigDependancy, ConfigTomlEdit, config_error::ConfigError},
    context::ContextNoConfig,
    lock_file::LockFile,
    maven_central::{MavenError, MavenIdBuf, PartialMavenIdBuf, fetch_artifact_metadata},
};

impl ConfigTomlEdit {
    pub fn assert_config_file_exists(root: &Path) -> Result<(), ConfigError> {
        let config_path = root.join(CONFIG_FILE_NAME);
        if !config_path.exists() {
            return Err(ConfigError::NoConfig(config_path));
        }
        Ok(())
    }
    pub fn fetch(root: &Path) -> Result<ConfigTomlEdit, ConfigError> {
        Self::fetch_from_fs(root)
    }
    fn fetch_from_fs(root: &Path) -> Result<ConfigTomlEdit, ConfigError> {
        let file = fs::read_to_string(root.join(CONFIG_FILE_NAME)).map_err(|e| match e.kind() {
            ErrorKind::NotFound => ConfigError::NoConfig(root.join(CONFIG_FILE_NAME)),
            ErrorKind::PermissionDenied => {
                ConfigError::PermissionDenied(root.to_string_lossy().into())
            }
            _ => ConfigError::NoConfig(root.join(CONFIG_FILE_NAME)),
        })?;

        let config: ConfigTomlEdit = ConfigTomlEdit::parse(&file)?;

        Ok(config)
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
            let mut deps = self.dependancies_mut().get_or_insert(HashMap::new());
            let mut value = deps.insert_empty(&id.artifact);

            value.version_mut().replace(id.version.clone());
            value.group_mut().replace(id.group.clone());

            lockfile.add_package(id)?;
            self.write(&ctx.root)?;
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

        let mut version: Option<String> = None;
        if let Some(deps) = self.dependancies() {
            if let Some(d) = deps.get(&partial_id.artifact) {
                if d.group().as_ref() == Some(&partial_id.group) {
                    version = Some(d.version().unwrap().to_string());
                }
            }
        }

        if version.is_none() {
            debug!("Could not find version");
            return Err(ConfigError::PackageNotFound);
        }

        let version = version.unwrap();

        println!(
            "{} {}:{}:{} from dependency list",
            "Removing".green().bold(),
            partial_id.group,
            partial_id.artifact,
            version
        );

        if !ctx.dry_run {
            if let Some(mut deps) = self.dependancies_mut().get_mut() {
                deps.remove(&partial_id.artifact);
            }
            self.write(&ctx.root)?;
        }

        self.sync_lock_file(&mut lockfile, ctx)?;

        Ok(())
    }
    pub fn sync_lock_file(
        &self,
        lockfile: &mut LockFile,
        ctx: &ContextNoConfig,
    ) -> Result<(), ConfigError> {
        let dep_list = self.dependancy_list()?;
        lockfile.sync_with_root_packages(&dep_list)?;

        lockfile.validate_current_packages(ctx)?;

        if !ctx.dry_run {
            lockfile.write(&ctx.root)?;
        }
        Ok(())
    }
    pub fn dependancy_list(
        &self,
    ) -> Result<HashMap<PartialMavenIdBuf, ConfigDependancy>, ConfigError> {
        if let Some(deps) = self.dependancies() {
            let mut map = HashMap::new();
            for (k, dep) in deps {
                let de: Result<ConfigDependancy, ConfigError> = dep.to_config_dependancy();
                let de = de?;

                let id = PartialMavenIdBuf::new(&de.group, k);

                map.insert(id, de);
            }

            Ok(map)
        } else {
            Ok(HashMap::new())
        }
    }

    pub fn write(&self, root: &Path) -> Result<(), ConfigError> {
        let doc = self.to_toml_string();
        let res = fs::write(root.join(CONFIG_FILE_NAME), doc);
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
}
