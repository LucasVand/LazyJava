use std::{collections::HashMap, io::ErrorKind, path::Path, str::FromStr};

use colored::Colorize;
use log::debug;

use crate::{
    CONFIG_FILE_NAME, ContextNoConfig,
    args::{AddArgs, RemoveArgs},
    config::{ConfigTomlEdit, LocalDependency, RemoteDependency, config_error::ConfigError},
    lock_file::{LockFile, RootPackage},
    maven_central::{
        MavenError, MavenIdBuf, PartialMavenIdBuf, fetch_artifact_metadata, pom::Scope,
    },
    utils::{IOError, fs},
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
            _ => ConfigError::IoError(IOError::new(
                "reading config file",
                root.join(CONFIG_FILE_NAME),
                e,
            )),
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

        let mut deps = self.dependancies_mut().get_or_insert(HashMap::new());
        let mut value = deps.insert_empty(&id.artifact);

        value.version_mut().replace(id.version.clone());
        value.group_mut().replace(id.group.clone());

        let scope = Scope::from_str(add_args.scope.as_ref().map_or("", |s| s.as_str())).ok();
        if let Some(s) = &add_args.scope
            && scope.is_none()
        {
            println!("{}: unknown scope `{}`", "Warning".yellow().bold(), s)
        }

        lockfile.add_package(id, scope)?;

        self.sync_lock_file(&mut lockfile, ctx)?;

        self.write(&ctx.root)?;
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
        if let Some(deps) = self.dependancies()
            && let Some(d) = deps.get(&partial_id.artifact)
            && d.group().as_ref() == Some(&partial_id.group)
            && let Some(v) = d.version()
        {
            version = Some(v.to_string());
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

        let mut deps_guard = self.dependancies_mut();
        if let Some(mut deps) = deps_guard.get_mut() {
            deps.remove(&partial_id.artifact);
        }

        self.sync_lock_file(&mut lockfile, ctx)?;

        self.write(&ctx.root)?;
        Ok(())
    }
    pub fn sync_lock_file(
        &self,
        lockfile: &mut LockFile,
        ctx: &ContextNoConfig,
    ) -> Result<(), ConfigError> {
        let dep_list = self.root_package_list()?;
        lockfile.sync_with_root_packages(&dep_list)?;

        lockfile.validate_current_packages(ctx)?;

        lockfile.write(&ctx.root)?;
        Ok(())
    }
    pub fn root_package_list(
        &self,
    ) -> Result<HashMap<PartialMavenIdBuf, RootPackage>, ConfigError> {
        if let Some(deps) = self.dependancies() {
            let mut map = HashMap::new();
            for (k, dep) in deps {
                let de: Option<RemoteDependency> = dep.to_remote_dependency()?;

                if let Some(dep) = de {
                    let id = PartialMavenIdBuf::new(&dep.group, k);

                    map.insert(id, dep.into());
                }
            }

            Ok(map)
        } else {
            Ok(HashMap::new())
        }
    }
    pub fn local_package_list(&self) -> Result<Vec<LocalDependency>, ConfigError> {
        if let Some(deps) = self.dependancies() {
            let mut v = Vec::new();
            for (_key, dep) in deps {
                let de: Option<LocalDependency> = dep.to_local_dependency()?;
                if let Some(dep) = de {
                    v.push(dep);
                }
            }
            Ok(v)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn write(&self, root: &Path) -> Result<(), ConfigError> {
        let doc = self.to_toml_string();
        fs::write(root.join(CONFIG_FILE_NAME), doc).map_err(|err| {
            ConfigError::IoError(IOError::new(
                "writing config file",
                root.join(CONFIG_FILE_NAME),
                err,
            ))
        })?;

        Ok(())
    }
}
