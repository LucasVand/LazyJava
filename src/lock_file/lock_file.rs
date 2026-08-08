use std::{
    collections::{HashMap, HashSet},
    io::{self, ErrorKind},
    mem,
    path::{Path, PathBuf},
};

use colored::Colorize;
use log::info;
use maven_version::Maven3ArtifactVersion;
use serde::{Deserialize, Serialize};

use crate::{
    ContextNoConfig, LOCK_FILE_NAME,
    lock_file::{LockFileError, fetch_packages::process_annotations},
    maven_central::{
        MavenId, MavenIdBuf, PartialMavenIdBuf,
        pom::{DependencyType, MavenDependencyList, Scope},
    },
    utils::{IOError, TomlDeserializeError, TomlSerializeError, fs},
};

#[derive(Serialize, Deserialize)]
pub struct LockFile {
    #[serde(default, rename = "package")]
    pub packages: Vec<LockFilePackageRemote>,

    #[serde(default, rename = "package-local")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub local_packages: Vec<LockFilePackageLocal>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct LockFilePackageRemote {
    #[serde(flatten)]
    pub id: MavenIdBuf,

    pub file_name: String,

    pub url: String,

    pub dependencies: Vec<MavenIdBuf>,

    pub packaging: DependencyType,

    pub root: bool,
    pub scope: Scope,

    pub path: Option<PathBuf>,

    #[serde(default)]
    pub annotations: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct LockFilePackageLocal {
    pub file_name: String,
    pub packaging: DependencyType,

    pub root: bool,
    pub scope: Scope,

    pub path: PathBuf,

    #[serde(default)]
    pub annotations: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct RootPackage {
    pub scope: Option<Scope>,
    pub group: String,
    pub version: String,
}
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct LocalRootPackage {
    pub scope: Option<Scope>,
    pub path: PathBuf,
}

impl LockFile {
    fn new() -> Self {
        LockFile {
            packages: Vec::new(),
            local_packages: Vec::new(),
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
            pkg.dependencies.sort();
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
        let list: Vec<LockFilePackageRemote> = MavenDependencyList::new(id, scope)?
            .into_iter()
            .map(|m| m.into())
            .collect();
        let list_len = list.len();

        let mut map: HashMap<u64, LockFilePackageRemote> = mem::take(&mut self.packages)
            .into_iter()
            .map(|p| {
                (
                    MavenDependencyList::hash_maven_bom_id(&p.id.group, &p.id.artifact),
                    p,
                )
            })
            .collect();

        for package in list {
            let hash =
                MavenDependencyList::hash_maven_bom_id(&package.id.group, &package.id.artifact);
            if let Some(old) = map.remove(&hash) {
                let old_version = Maven3ArtifactVersion::new(&old.id.version);
                let new_version = Maven3ArtifactVersion::new(&package.id.version);

                let replaces = if old.root {
                    package.root && new_version > old_version
                } else {
                    package.root || new_version > old_version
                };

                if replaces {
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
        map: &mut HashMap<String, &mut LockFilePackageRemote>,
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
        println!("{} dependencies", "Syncing".green().bold(),);
        let mut added: isize = 0;
        let mut removed: isize = 0;
        let mut map: HashMap<String, &mut LockFilePackageRemote> = self
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
                "dependencies"
            } else {
                "dependency"
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
        root_packages: &HashMap<PartialMavenIdBuf, RootPackage>,
        local_root_packages: &HashMap<PathBuf, LocalRootPackage>,
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

        let mut local_map: HashMap<&PathBuf, &LocalRootPackage> =
            local_root_packages.iter().collect();

        self.local_packages = mem::take(&mut self.local_packages)
            .into_iter()
            .filter(|local| local_map.remove(&local.path).is_some())
            .collect();

        for (_, local) in local_map {
            let package = Self::validate_local_package(local)?;
            self.local_packages.push(package);
        }

        Ok(())
    }
    fn remote_packages_with_scopes(
        &self,
        scopes: &[Scope],
    ) -> impl Iterator<Item = &LockFilePackageRemote> {
        self.packages
            .iter()
            .filter(move |package| scopes.contains(&package.scope))
    }
    fn local_packages_with_scopes(
        &self,
        scopes: &[Scope],
    ) -> impl Iterator<Item = &LockFilePackageLocal> {
        self.local_packages
            .iter()
            .filter(move |package| scopes.contains(&package.scope))
    }

    pub fn package_paths_with_scopes(&self, scopes: &[Scope]) -> Vec<PathBuf> {
        self.remote_packages_with_scopes(scopes)
            .filter_map(|package| package.path.as_ref().map(|p| p.to_path_buf()))
            .chain(
                self.local_packages_with_scopes(scopes)
                    .map(|package| package.path.to_path_buf()),
            )
            .collect()
    }
    pub fn processor_package_paths_with_scopes(&self, scopes: &[Scope]) -> Vec<PathBuf> {
        self.remote_packages_with_scopes(scopes)
            .filter(|p| !p.annotations.is_empty())
            .filter_map(|package| package.path.as_ref().map(|p| p.to_path_buf()))
            .chain(
                self.local_packages_with_scopes(scopes)
                    .filter(|p| !p.annotations.is_empty())
                    .map(|package| package.path.to_path_buf()),
            )
            .collect()
    }

    pub fn compile_time_packages(&self) -> Vec<PathBuf> {
        self.package_paths_with_scopes(&[Scope::System, Scope::Provided, Scope::Compile])
    }
    pub fn runtime_packages(&self) -> Vec<PathBuf> {
        self.package_paths_with_scopes(&[Scope::System, Scope::Runtime, Scope::Compile])
    }
    pub fn processor_compile_time_packages(&self) -> Vec<PathBuf> {
        self.processor_package_paths_with_scopes(&[Scope::System, Scope::Provided, Scope::Compile])
    }

    fn validate_local_package(
        package: &LocalRootPackage,
    ) -> Result<LockFilePackageLocal, LockFileError> {
        let Some(name) = package.path.file_name() else {
            return Err(IOError::new(
                "resolving filename of local package",
                &package.path,
                io::Error::new(io::ErrorKind::NotFound, "could not resolve filename"),
            ))?;
        };

        let bin = fs::read(&package.path)
            .map_err(|e| IOError::new("reading local jar", &package.path, e))?;
        let annots = process_annotations(&bin)?;

        Ok(LockFilePackageLocal {
            file_name: name.to_string_lossy().to_string(),
            packaging: DependencyType::Jar,
            root: true,
            scope: package.scope.unwrap_or(Scope::default()),
            path: package.path.clone(),
            annotations: annots,
        })
    }
}
