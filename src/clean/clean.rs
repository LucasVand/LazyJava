use std::{fs, path::PathBuf};

use crate::{
    BUILD_FOLDER, args::LazyJavaGlobalArgs, clean::clean_error::CleanError,
    find_root::find_file_in_dir,
};

pub fn clean(root: &PathBuf, global: &LazyJavaGlobalArgs) -> Result<(), CleanError> {
    let build = find_file_in_dir(root, BUILD_FOLDER).map_err(|e| CleanError::NoBuildFolder {
        path: root.to_str().unwrap().to_string(),
        os_error: e,
    })?;

    fs::remove_dir_all(&build.path()).map_err(|e| CleanError::NoRemove {
        path: root.to_str().unwrap().to_string(),
        os_error: e,
    })?;

    fs::create_dir(&build.path()).map_err(|e| CleanError::NoCreate {
        path: root.to_str().unwrap().to_string(),
        os_error: e,
    })?;

    return Ok(());
}
