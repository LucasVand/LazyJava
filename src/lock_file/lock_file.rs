use std::{collections::HashMap, fs, io::ErrorKind, mem, path::Path};

use maven_version::Maven3ArtifactVersion;
use serde::{Deserialize, Serialize};

use crate::{
    LOCK_FILE_NAME,
    lock_file::LockFileError,
    maven_central::{MavenIdBuf, pom::MavenDependancyList},
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
    pub fn validate_current_packages(&self, lib: &Path) -> Result<(), LockFileError> {
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

                if map.remove(name.as_str()).is_none() {
                    log::debug!("Validation removed {}", name);
                    fs::remove_file(file.path())?;
                }
            }
        }

        // the packages that do not exist currently
        for (key, value) in map {
            log::debug!("Validation added {}", value.id);
            let bin = fetch_bin(&value.url)?;

            fs::write(lib.join(key), bin)?;
        }

        Ok(())
    }
}

fn fetch_bin(url: &str) -> Result<Vec<u8>, LockFileError> {
    let res = reqwest::blocking::get(url)?;

    match res.error_for_status() {
        Err(err) => {
            log::warn!("Failed to fetch Maven artifact: {}", err);
            Err(LockFileError::RequestError(err))
        }
        Ok(res) => Ok(res.bytes()?.to_vec()),
    }
}
