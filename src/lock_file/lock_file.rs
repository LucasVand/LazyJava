use std::{
    collections::{HashMap, HashSet},
    fs,
    io::ErrorKind,
    mem,
    path::Path,
};

use colored::Colorize;
use maven_version::Maven3ArtifactVersion;
use serde::{Deserialize, Serialize};

use crate::{
    LOCK_FILE_NAME,
    config::ConfigDependancy,
    context::ContextNoConfig,
    lock_file::LockFileError,
    maven_central::{
        MavenId, MavenIdBuf, PartialMavenIdBuf,
        pom::{DependancyType, MavenDependancyList, Scope},
    },
};

#[derive(Serialize, Deserialize)]
pub struct LockFile {
    #[serde(default, rename = "package")]
    pub packages: Vec<LockFilePackage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LockFilePackage {
    #[serde(flatten)]
    pub id: MavenIdBuf,

    pub file_name: String,

    pub url: String,

    pub dependancies: Vec<MavenIdBuf>,

    pub packaging: DependancyType,

    pub root: bool,
    pub scope: Scope,

    #[serde(default)]
    pub annotations: Vec<String>,
}

impl LockFile {
    fn new() -> Self {
        LockFile {
            packages: Vec::new(),
        }
    }
    pub fn fetch(root: &Path) -> Result<LockFile, LockFileError> {
        let fs_file = Self::fetch_from_fs(root);

        if let Ok(lockfile) = fs_file {
            Ok(lockfile)
        } else {
            match fs_file.err().unwrap() {
                LockFileError::NoFound => Ok(LockFile::new()),
                err => Err(err),
            }
        }
    }
    fn fetch_from_fs(root: &Path) -> Result<LockFile, LockFileError> {
        let file = fs::read_to_string(root.join(LOCK_FILE_NAME)).map_err(|e| match e.kind() {
            ErrorKind::NotFound => LockFileError::NoFound,
            ErrorKind::PermissionDenied => {
                LockFileError::PermissionDenied(root.to_string_lossy().into())
            }
            _ => LockFileError::NoFound,
        })?;

        let lockfile: LockFile = toml::from_str(&file)?;

        Ok(lockfile)
    }
    pub fn write(&self, root: &Path) -> Result<(), LockFileError> {
        let str = toml::to_string_pretty(self)?;

        let res = fs::write(root.join(LOCK_FILE_NAME), str);
        if let Err(err) = res {
            return match err.kind() {
                ErrorKind::PermissionDenied => Err(LockFileError::PermissionDenied(
                    root.to_string_lossy().into(),
                )),
                _ => Err(LockFileError::IoError(err)),
            };
        }

        Ok(())
    }
    pub fn add_package(&mut self, id: MavenIdBuf) -> Result<isize, LockFileError> {
        let list: Vec<LockFilePackage> = MavenDependancyList::new(id)?
            .into_iter()
            .map(|m| m.into())
            .collect();
        let list_len = list.len();

        let mut map: HashMap<u64, LockFilePackage> = mem::take(&mut self.packages)
            .into_iter()
            .map(|p| {
                (
                    MavenDependancyList::hash_maven_bom_id(&p.id.group, &p.id.artifact),
                    p,
                )
            })
            .collect();

        for package in list {
            let hash =
                MavenDependancyList::hash_maven_bom_id(&package.id.group, &package.id.artifact);
            if let Some(old) = map.remove(&hash) {
                let old_version = Maven3ArtifactVersion::new(&old.id.version);
                let new_version = Maven3ArtifactVersion::new(&package.id.version);

                if new_version > old_version {
                    map.insert(hash, package);
                } else {
                    map.insert(hash, old);
                }
            } else {
                map.insert(hash, package);
            }
        }

        self.packages = map.into_values().collect();
        Ok(list_len as isize)
    }
    fn validate_dir(
        path: &Path,
        map: &mut HashMap<String, &mut LockFilePackage>,
        dry_run: bool,
    ) -> Result<isize, LockFileError> {
        println!(
            "    {} {}",
            "Validating".green().bold(),
            path.file_stem().unwrap().display()
        );
        let mut removed = 0;
        for file in walkdir::WalkDir::new(path)
            .into_iter()
            .filter(|p| p.is_ok())
            .map(|p| p.unwrap())
            .filter(|p| !p.file_type().is_dir())
        {
            if let Some(name) = file.path().file_name() {
                let name = name.to_string_lossy().to_string();

                match map.remove(name.as_str()) {
                    Some(_pack) => {
                        // println!("    {} {}", "Found".green().bold(), pack.id);
                    }
                    None => {
                        let path = file.path();
                        let stem = path
                            .file_name()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or_default();
                        println!("        {} {}", "Removed".green().bold(), stem);
                        if !dry_run {
                            fs::remove_file(path)?;
                        }
                        removed += 1;
                    }
                }
            }
        }

        return Ok(removed);
    }
    pub fn validate_current_packages(
        &mut self,
        ctx: &ContextNoConfig,
    ) -> Result<isize, LockFileError> {
        println!("{} dependancies", "Syncing".green().bold(),);
        let mut added: isize = 0;
        let mut removed: isize = 0;
        let mut map: HashMap<String, &mut LockFilePackage> = self
            .packages
            .iter_mut()
            .map(|p| (p.file_name.as_str().to_string(), p))
            .collect();

        removed += Self::validate_dir(&ctx.lib, &mut map, ctx.dry_run)?;
        removed += Self::validate_dir(&ctx.lib_annotations, &mut map, ctx.dry_run)?;

        let download_change =
            Self::fetch_packages(ctx, map.into_iter().map(|(_k, v)| v).collect())?;
        added += download_change;

        let plural = |change: isize| {
            if change.abs() != 1 {
                "dependancies"
            } else {
                "dependancy"
            }
        };
        if added != 0 {
            println!("{} {} {}", "Added".green().bold(), added, plural(added));
        }
        if removed != 0 {
            println!(
                "{} {} {}",
                "Removed".green().bold(),
                removed,
                plural(removed)
            );
        }
        Ok(added - removed)
    }
    pub fn contains_package(&self, id: &MavenId) -> bool {
        return self.packages.iter().any(|p| p.id == *id);
    }
    pub fn sync_with_root_packages(
        &mut self,
        root_packages: &HashMap<PartialMavenIdBuf, ConfigDependancy>,
    ) -> Result<(), LockFileError> {
        let mut map: HashSet<PartialMavenIdBuf> = self
            .packages
            .iter()
            .filter(|p| p.root)
            .map(|p| p.id.clone().to_partial_buf())
            .collect();

        for (key, package) in root_packages {
            if !map.contains(key) {
                self.add_package(package.id.clone())?;
            }
        }

        for (key, _root_package) in root_packages {
            map.remove(key);
        }

        for key in map {
            self.remove_package(&key.group, &key.artifact)?;
        }

        self.remove_unneed_packages();
        Ok(())
    }
}
