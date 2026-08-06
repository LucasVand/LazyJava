use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    process::ExitStatus,
    time::SystemTime,
};

use same_file::is_same_file;
use serde::{Deserialize, Serialize};
use walkdir::DirEntry;

use crate::{
    BUILD_METADATA_NAME, Context,
    args::BuildArgs,
    build::BuildError,
    utils::{IOError, TomlSerializeError, fs, jdk_version::desired_jdk_version},
};

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub struct BuildMetadata {
    pub time_stamp: SystemTime,
    pub java_version: String,
    pub src: PathBuf,
    pub lib_hash: u64,
    pub lib_annotations_hash: u64,
    pub bin_hash: u64,
    pub local_libs_hash: u64,
    pub build_passed: bool,
}

impl BuildMetadata {
    pub fn new() -> BuildMetadata {
        BuildMetadata {
            time_stamp: SystemTime::now(),
            java_version: "".into(),
            src: "".into(),
            lib_hash: 0,
            lib_annotations_hash: 0,
            bin_hash: 0,
            local_libs_hash: 0,
            build_passed: false,
        }
    }
    pub fn fetch(target: &Path) -> Option<BuildMetadata> {
        let Ok(build_str) = fs::read_to_string(target.join(BUILD_METADATA_NAME)) else {
            return None;
        };
        let Ok(metadata) = toml::from_str(&build_str) else {
            return None;
        };
        Some(metadata)
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
pub fn required_full_build(
    args: &BuildArgs,
    ctx: &Context,
    meta: Option<&BuildMetadata>,
) -> Result<bool, BuildError> {
    let Some(meta) = meta else {
        return Ok(true);
    };

    let current_lib_hash = hash_directory(&ctx.lib);
    let lib_hash_match = current_lib_hash == meta.lib_hash;

    let current_lib_annotations_hash = hash_directory(&ctx.lib_annotations);
    let lib_annotations_hash_match =
        current_lib_annotations_hash == meta.lib_annotations_hash;

    let current_local_hash = hash_local_libs(ctx)?;
    let local_hash_match = current_local_hash == meta.local_libs_hash;

    let jdk = desired_jdk_version(Some(args), Some(ctx));
    let version_match = meta.java_version == jdk;

    let src_match = is_same_file(&meta.src, &ctx.src).unwrap_or(false);

    Ok(args.build_all
        || !lib_hash_match
        || !lib_annotations_hash_match
        || !version_match
        || !src_match
        || !local_hash_match)
}

pub fn save_metadata(
    ctx: &Context,
    status: ExitStatus,
    meta: Option<BuildMetadata>,
    jdk: &str,
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
        java_version: jdk.to_string(),
        lib_hash: hash_directory(&ctx.lib),
        lib_annotations_hash: hash_directory(&ctx.lib_annotations),
        bin_hash: hash_directory(&ctx.bin),
        local_libs_hash: hash_local_libs(ctx)?,
        build_passed: status.success(),
        src: ctx.src.clone(),
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

/// Hash the local dependency jars referenced by the project's config so a full
/// rebuild is triggered whenever one of them changes.
pub fn hash_local_libs(ctx: &Context) -> Result<u64, BuildError> {
    let deps = ctx.config.local_package_list()?;
    let paths: Vec<PathBuf> = deps.into_iter().map(|dep| dep.path).collect();

    let hash = hash_files(&paths);
    log::debug!("hash_local_libs = {}", hash);
    Ok(hash)
}

/// Hash a set of files by their paths and byte contents.
pub fn hash_files(paths: &[PathBuf]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for path in paths {
        path.hash(&mut hasher);
        if let Ok(bytes) = fs::read(path) {
            bytes.hash(&mut hasher);
        }
    }
    hasher.finish()
}
