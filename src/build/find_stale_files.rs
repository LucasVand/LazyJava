use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{
    dependancy_graph::graph::DependancyGraph, lazy_java_error::LazyJavaError,
    utils::find_main::find_java_files,
};

pub fn find_modified_files(build: &Path, src: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let build_meta = fs::metadata(build)?;

    let last_build_time = build_meta.modified()?;

    let java_files = find_java_files(src)?;

    let mut stale_files = Vec::new();
    for file in java_files.into_iter() {
        let meta = file.metadata()?;

        let modification_time = meta.modified()?;

        if modification_time > last_build_time {
            stale_files.push(file);
        }
    }

    return Ok(stale_files);
}
pub fn files_to_recompile(
    graph: DependancyGraph,
    stale_files: Vec<PathBuf>,
) -> Result<Vec<PathBuf>, LazyJavaError> {
    let mut recompile_files: Vec<PathBuf> = Vec::new();
    for file in stale_files {
        let mut deps = graph.dependancy_list_from_path(&file)?;

        recompile_files.append(&mut deps);
        recompile_files.push(file);
    }

    let recompile_hash: HashSet<_> = recompile_files.into_iter().collect();
    let unique_recompile: Vec<PathBuf> = recompile_hash.into_iter().collect();

    return Ok(unique_recompile);
}
