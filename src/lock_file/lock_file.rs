use std::{
    collections::{HashMap, HashSet},
    io::ErrorKind,
    mem,
    path::Path,
};

use colored::Colorize;
use log::info;
use maven_version::Maven3ArtifactVersion;
use serde::{Deserialize, Serialize};

use crate::{
    ContextNoConfig, LOCK_FILE_NAME,
    config::ConfigDependancy,
    lock_file::LockFileError,
    maven_central::{
        MavenId, MavenIdBuf, PartialMavenIdBuf,
        pom::{DependancyType, MavenDependancyList, Scope},
    },
    utils::{IOError, TomlDeserializeError, TomlSerializeError, fs},
};

#[derive(Serialize, Deserialize)]
pub struct LockFile {
    #[serde(default, rename = "package")]
    pub packages: Vec<LockFilePackage>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
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
        let fs_file = Self::fetch_from_fs(root)?;
        let lock = match fs_file {
            None => {
                log::info!("Creating lock file from new");
                LockFile::new()
            }
            Some(l) => l,
        };
        Ok(lock)
    }
    fn fetch_from_fs(root: &Path) -> Result<Option<LockFile>, LockFileError> {
        let p = root.join(LOCK_FILE_NAME);
        let file = fs::read_to_string(&p);
        if let Err(e) = file {
            if e.kind() == ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(IOError::new("reading lazy-java.lock", &p, e))?;
        }
        let file = file.unwrap();

        let lockfile: LockFile = toml::from_str(&file)
            .map_err(|s| TomlDeserializeError::new("reading lazy-java.lock", &p, s))?;

        Ok(Some(lockfile))
    }
    pub fn write(&mut self, root: &Path) -> Result<(), LockFileError> {
        self.packages.sort();
        for pkg in &mut self.packages {
            pkg.dependancies.sort();
            pkg.annotations.sort();
        }

        let p = root.join(LOCK_FILE_NAME);
        let str = toml::to_string_pretty(self)
            .map_err(|s| TomlSerializeError::new("writing lazy-java.lock", &p, s))?;

        fs::write(&p, str).map_err(|s| IOError::new("writing lazy-java.lock", p, s))?;

        Ok(())
    }
    pub fn add_package(
        &mut self,
        id: MavenIdBuf,
        scope: Option<Scope>,
    ) -> Result<isize, LockFileError> {
        let list: Vec<LockFilePackage> = MavenDependancyList::new(id, scope)?
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

                if (new_version > old_version || package.root) && !old.root {
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
    ) -> Result<isize, LockFileError> {
        println!(
            "    {} {}",
            "Validating".green().bold(),
            path.file_stem().unwrap().display()
        );
        let mut removed = 0;
        for file in walkdir::WalkDir::new(path)
            .into_iter()
            .flatten()
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
                        fs::remove_file(path)
                            .map_err(|s| IOError::new("removing stale package", path, s))?;
                        println!("        {} {}", "Removed".green().bold(), stem);
                        removed += 1;
                    }
                }
            }
        }

        Ok(removed)
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

        removed += Self::validate_dir(&ctx.lib, &mut map)?;
        removed += Self::validate_dir(&ctx.lib_annotations, &mut map)?;

        let download_change = Self::fetch_packages(ctx, map.into_values().collect())?;
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
        self.packages.iter().any(|p| p.id == *id)
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
                let id = key.clone().to_full_buf(package.version.clone());
                info!("Adding package '{}'", id);
                self.add_package(id, package.scope)?;
            }
        }

        for key in root_packages.keys() {
            map.remove(key);
        }

        for key in map {
            self.remove_package(&key.group, &key.artifact)?;
        }

        self.remove_unneed_packages();
        Ok(())
    }
}
