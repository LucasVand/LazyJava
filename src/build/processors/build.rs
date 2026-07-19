use std::{ffi::OsStr, path::PathBuf, process::ExitStatus};

use crate::{
    Context,
    lazy_java_error::LazyJavaError,
    utils::{find_main::find_java_files, processes::compile_java_files},
};

pub fn build_processors(ctx: &Context) -> Result<ExitStatus, LazyJavaError> {
    let files = find_java_files(&ctx.src);

    let os_str: Vec<&OsStr> = ctx
        .config
        .processers
        .processers
        .iter()
        .map(|c| OsStr::new(&c.class_name))
        .collect();

    let mut full_paths: Vec<PathBuf> = Vec::new();
    for file in files {
        if let Some(file_name) = file.file_name()
            && os_str.contains(&file_name)
        {
            full_paths.push(file);
        }
    }

    return Ok(compile_java_files(
        full_paths,
        &ctx.bin_processors,
        &ctx.lib,
        &ctx.lib_annotations,
        &ctx.src_generated,
        &Vec::new(),
    )?);
}
