use std::{fs, path::PathBuf};

use crate::{
    BUILD_FOLDER, args::LazyJavaGlobalArgs, clean::clean_error::CleanError,
    find_root::find_file_in_dir, utils::println_verbose,
};

pub fn clean(root: &PathBuf, global: &LazyJavaGlobalArgs) -> Result<(), CleanError> {
    println_verbose("Finding source directory", &global);
    let build = find_file_in_dir(root, BUILD_FOLDER).map_err(|e| CleanError::NoBuildFolder {
        path: root.to_str().unwrap().to_string(),
        os_error: e,
    })?;
    println_verbose(
        &format!(
            "Found source directory ({})",
            build.path().to_str().unwrap()
        ),
        &global,
    );

    fs::remove_dir_all(&build.path()).map_err(|e| CleanError::NoRemove {
        path: root.to_str().unwrap().to_string(),
        os_error: e,
    })?;
    println_verbose("Removed build folder", global);

    fs::create_dir(&build.path()).map_err(|e| CleanError::NoCreate {
        path: root.to_str().unwrap().to_string(),
        os_error: e,
    })?;
    println_verbose("Created new build folder", global);

    return Ok(());
}
