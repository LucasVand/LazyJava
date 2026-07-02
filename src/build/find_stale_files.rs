use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
};

use crate::{
    build::metadata::BuildMetadata, dependancy_graph::graph::DependancyGraph,
    lazy_java_error::LazyJavaError, utils::find_main::find_java_files,
};

pub fn find_modified_files(build: &BuildMetadata, src: &Path) -> Result<Vec<PathBuf>, io::Error> {
    log::debug!("Finding modified files in src: {:?}", src);

    let last_build_time = build.time_stamp;
    log::debug!("Last build time: {:?}", last_build_time);

    let java_files = find_java_files(src)?;
    log::debug!("Found {} Java files to check", java_files.len());

    let mut stale_files = Vec::new();
    for file in java_files.into_iter() {
        let meta = file.metadata()?;

        let modification_time = meta.modified()?;

        if modification_time > last_build_time {
            log::debug!("File modified: {:?}", file);
            stale_files.push(file);
        }
    }

    log::debug!("Found {} modified files", stale_files.len());
    Ok(stale_files)
}
pub fn files_to_recompile(
    graph: DependancyGraph,
    stale_files: Vec<PathBuf>,
) -> Result<Vec<PathBuf>, LazyJavaError> {
    log::debug!(
        "Calculating files to recompile from {} stale files",
        stale_files.len()
    );
    let mut recompile_files: Vec<PathBuf> = Vec::new();
    for file in stale_files {
        let mut deps = graph.dependancy_list_from_path(&file)?;
        log::debug!("File {:?} has {} dependencies", file, deps.len());

        recompile_files.append(&mut deps);
        recompile_files.push(file);
    }

    let recompile_hash: HashSet<_> = recompile_files.into_iter().collect();
    let unique_recompile: Vec<PathBuf> = recompile_hash.into_iter().collect();

    log::debug!(
        "Total unique files to recompile: {}",
        unique_recompile.len()
    );
    Ok(unique_recompile)
}
