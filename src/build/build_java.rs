use std::path::PathBuf;

use crate::{
    BUILD_FOLDER, SRC_FOLDER, build::build_error::BuildError, find_root::find_file_in_dir,
    processes::compile_java,
};

pub fn build_java(root: &PathBuf) -> Result<(), BuildError> {
    let src = find_file_in_dir(&root, SRC_FOLDER).map_err(|e| BuildError::NoSrc {
        path: String::new(),
        os_error: e,
    })?;

    let build = find_file_in_dir(&root, BUILD_FOLDER).map_err(|e| BuildError::NoBuild {
        path: String::new(),
        os_error: e,
    })?;

    let compile_res =
        compile_java(&src.path(), &build.path()).map_err(|e| BuildError::IOError(e))?;

    if compile_res.success() {
        return Ok(());
    } else {
        return Err(BuildError::BuildErrors);
    }
}
