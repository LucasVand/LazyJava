use crate::args::BuildArgs;
use crate::args::LazyJavaGlobalArgs;
use crate::utils::println_verbose;
use std::path::PathBuf;

use crate::{build::build_error::BuildError, find_root::find_file_in_dir, processes::compile_java};

pub fn build_java(
    root: &PathBuf,
    args: BuildArgs,
    global: &LazyJavaGlobalArgs,
) -> Result<(), BuildError> {
    println_verbose("Finding source directory", &global);
    let src = find_file_in_dir(&root, &args.source).map_err(|e| BuildError::NoSrc {
        path: root.to_str().unwrap().to_string(),
        os_error: e,
    })?;
    println_verbose(
        &format!("Found source directory ({})", src.path().to_str().unwrap()),
        &global,
    );

    println_verbose("Finding build directory", &global);
    let build = find_file_in_dir(&root, &args.build).map_err(|e| BuildError::NoBuild {
        path: root.to_str().unwrap().to_string(),
        os_error: e,
    })?;
    println_verbose(
        &format!("Found build directory ({})", build.path().to_str().unwrap()),
        &global,
    );

    println_verbose("Compiling java files", &global);
    let compile_res =
        compile_java(&src.path(), &build.path()).map_err(|e| BuildError::IOError(e))?;

    if compile_res.success() {
        return Ok(());
    } else {
        return Err(BuildError::BuildErrors);
    }
}
