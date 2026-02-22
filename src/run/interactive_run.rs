use std::path::PathBuf;

use inquire::Select;

use crate::{
    args::{LazyJavaGlobalArgs, RunArgs},
    find_main::find_main_classes,
    find_root::find_file_in_dir,
    run::run_error::RunError,
    utils::println_verbose,
};

pub fn interactive_find_main(
    root: &PathBuf,
    args: &RunArgs,
    global: &LazyJavaGlobalArgs,
) -> Result<String, RunError> {
    println_verbose("Finding source directory", &global);
    let src = find_file_in_dir(&root, &args.build_args.source).map_err(|e| RunError::NoSrc {
        path: root.to_str().unwrap().to_string(),
        os_error: e,
    })?;
    println_verbose(
        &format!("Found source directory ({})", src.path().to_str().unwrap()),
        &global,
    );

    let options = find_main_classes(root).map_err(|e| RunError::MainClassSearchError(e))?;

    let configured_options: Vec<String> = options
        .into_iter()
        .map(|op| {
            return op.classname;
        })
        .collect();

    let res = Select::new("Select a Main Class to Run: ", configured_options)
        .without_help_message()
        .without_filtering()
        .prompt()
        .map_err(|_e| RunError::PromptError)?;

    return Ok(res);
}
