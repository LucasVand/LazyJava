use std::{collections::HashMap, fs, io::ErrorKind, mem, path::Path};

use colored::Colorize;
use maven_version::Maven3ArtifactVersion;
use serde::{Deserialize, Serialize};

use crate::{
    LOCK_FILE_NAME,
    lock_file::LockFileError,
    maven_central::{
        MavenIdBuf,
        pom::{DependancyType, MavenDependancyList},
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
    pub fn add_packages(&mut self, packages: Vec<LockFilePackage>) {
        let mut map: HashMap<u64, LockFilePackage> = mem::take(&mut self.packages)
            .into_iter()
            .map(|p| {
                (
                    MavenDependancyList::hash_maven_bom_id(&p.id.group, &p.id.artifact),
                    p,
                )
            })
            .collect();

        for package in packages {
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
    }
    pub fn validate_current_packages(
        &self,
        lib: &Path,
        dry_run: bool,
    ) -> Result<isize, LockFileError> {
        println!(
            "{} dependancies with /{}",
            "Syncing".green().bold(),
            lib.file_name().unwrap().to_string_lossy()
        );
        let mut added: isize = 0;
        let mut removed: isize = 0;
        let mut map: HashMap<&str, &LockFilePackage> = self
            .packages
            .iter()
            .map(|p| (p.file_name.as_str(), p))
            .collect();

        let dir = fs::read_dir(lib)?;

        for file in dir {
            if let Ok(file) = file
                && let Some(name) = file.path().file_name()
            {
                let name = name.to_string_lossy().to_string();

                match map.remove(name.as_str()) {
                    Some(pack) => {
                        println!("    {} {}", "Found".green().bold(), pack.id);
                    }
                    None => {
                        let path = file.path();
                        let stem = path
                            .file_name()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or_default();
                        println!("    {} {}", "Removed".green().bold(), stem);
                        if !dry_run {
                            fs::remove_file(path)?;
                        }
                        removed += 1;
                    }
                }
            }
        }

        let download_change =
            Self::fetch_packages(lib, map.into_iter().map(|(_k, v)| v).collect(), dry_run)?;
        added += download_change;

        let plural = |change: isize| {
            if change.abs() != 1 {
                "dependancies"
            } else {
                "dependancy"
            }
        };
        if added != 0 {
            println!("    {} {} {}", "Added".green().bold(), added, plural(added));
        }
        if removed != 0 {
            println!(
                "    {} {} {}",
                "Removed".green().bold(),
                removed,
                plural(removed)
            );
        }
        Ok(added - removed)
    }
}
