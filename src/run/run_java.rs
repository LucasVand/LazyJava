use std::path::PathBuf;

use crate::{
    BUILD_FOLDER,
    args::{LazyJavaGlobalArgs, RunArgs},
    build::build_java::build_java,
    find_root::find_file_in_dir,
    processes::execute_java,
    run::run_error::RunError,
};

pub fn run_java(
    root: &PathBuf,
    args: RunArgs,
    global: &LazyJavaGlobalArgs,
) -> Result<(), RunError> {
    println!("{:?}", args.args);
    if !args.no_build {
        println_verbose("Building Java", global);
        build_java(root).map_err(|e| RunError::BuildError(e))?;
    }

    println_verbose("Finding build folder", global);
    let build = find_file_in_dir(root, BUILD_FOLDER).map_err(|e| RunError::NoBuildFolder {
        path: root.to_str().unwrap().to_string(),
        os_error: e,
    })?;

    println_verbose("Executing java", global);
    execute_java(&args.class, &build.path(), &args.args)
        .map_err(|_e| RunError::NoMainClass(args.class))?;

    return Ok(());
}

fn println_verbose(msg: &str, global: &LazyJavaGlobalArgs) {
    if !global.verbose {
        return;
    }
    println!("{}", msg);
}
