use std::path::PathBuf;

use crate::{
    BUILD_FOLDER,
    args::{LazyJavaGlobalArgs, RunArgs},
    build::build_java::build_java,
    find_root::find_file_in_dir,
    processes::execute_java,
    run::{interactive_run::interactive_find_main, run_error::RunError},
    utils::println_verbose,
};

pub fn run_java(
    root: &PathBuf,
    args: RunArgs,
    global: &LazyJavaGlobalArgs,
) -> Result<(), RunError> {
    if !args.no_build {
        println_verbose("Building Java", global);
        build_java(root, args.build_args.clone(), global).map_err(|e| RunError::BuildError(e))?;
    }

    println_verbose("Finding build folder", global);
    let build = find_file_in_dir(root, BUILD_FOLDER).map_err(|e| RunError::NoBuildFolder {
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

    println_verbose("Executing java", global);
    let class = match args.class {
        Some(class) => class,
        None => interactive_find_main(root, &args, global)?,
    };

    execute_java(&class, &build.path(), &args.args).map_err(|_e| RunError::NoMainClass(class))?;

    return Ok(());
}
