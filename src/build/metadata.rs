use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
    process::ExitStatus,
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use walkdir::DirEntry;

use crate::{
    BUILD_METADATA_NAME, Context,
    build::BuildError,
    utils::{IOError, TomlSerializeError, fs},
};

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct BuildMetadata {
    pub time_stamp: SystemTime,
    pub java_version: String,
    pub lib_hash: u64,
    pub bin_hash: u64,
    pub build_passed: bool,
}

impl BuildMetadata {
    pub fn new() -> BuildMetadata {
        BuildMetadata {
            time_stamp: SystemTime::now(),
            java_version: "".into(),
            lib_hash: 0,
            bin_hash: 0,
            build_passed: false,
        }
    }
    pub fn fetch(target: &Path) -> Option<BuildMetadata> {
        let build_str = fs::read_to_string(target.join(BUILD_METADATA_NAME));
        if build_str.is_err() {
            return None;
        }
        let build_str = build_str.unwrap();

        let metadata: Result<BuildMetadata, toml::de::Error> = toml::from_str(&build_str);

        if metadata.is_err() {
            return None;
        }

        Some(metadata.unwrap())
    }
    pub fn write(&self, target: &Path) -> Result<(), BuildError> {
        let path = target.join(BUILD_METADATA_NAME);
        let ser = toml::to_string_pretty(self)
            .map_err(|e| TomlSerializeError::new("serializing build metadata", &path, e))?;

        fs::write(&path, ser).map_err(|e| IOError::new("writing build metadata", &path, e))?;
        Ok(())
    }
}

impl Default for BuildMetadata {
    fn default() -> Self {
        Self::new()
    }
}

pub fn save_metadata(
    ctx: &Context,
    status: ExitStatus,
    meta: Option<BuildMetadata>,
) -> Result<(), BuildError> {
    let time = if !status.success() {
        match meta {
            Some(meta) => meta.time_stamp,
            None => SystemTime::UNIX_EPOCH,
        }
    } else {
        SystemTime::now()
    };
    let meta = BuildMetadata {
        time_stamp: time,
        java_version: "25".to_string(),
        lib_hash: hash_directory(&ctx.lib),
        bin_hash: hash_directory(&ctx.bin),
        build_passed: status.success(),
    };

    meta.write(&ctx.target)?;
    Ok(())
}

pub fn hash_directory(path: &Path) -> u64 {
    let dirs: Vec<DirEntry> = walkdir::WalkDir::new(path)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    let mut hasher = DefaultHasher::new();
    for dir in &dirs {
        let meta = dir.metadata();
        if let Ok(meta) = meta {
            meta.len().hash(&mut hasher);
        }
        let file_path = dir.path().strip_prefix(path);
        if let Ok(relative_path) = file_path {
            relative_path.hash(&mut hasher);
        }
    }

    let hash = hasher.finish();
    log::debug!("hash_directory({:?}) = {}", path, hash);
    hash
}
