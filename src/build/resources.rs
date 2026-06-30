use std::{
    collections::HashSet,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::{Context, args::BuildArgs, lazy_java_error::LazyJavaError};

pub fn copy_resources(_args: &BuildArgs, ctx: &Context) -> Result<(), LazyJavaError> {
    let glob_set = build_globset(ctx);
    let resource_paths = add_resources(&ctx.src, Path::new(""), &glob_set, ctx)?;
    log::info!("Found {} resources", resource_paths.len());

    let removed = remove_unknown_resources(&resource_paths, &ctx.bin, ctx)?;
    log::info!("Removed {} resources", removed);
    Ok(())
}
fn remove_unknown_resources(
    resources: &HashSet<PathBuf>,
    current: &Path,
    ctx: &Context,
) -> Result<isize, LazyJavaError> {
    let mut change = 0;
    for dir in fs::read_dir(current)? {
        if let Ok(dir) = dir {
            if dir.file_type()?.is_dir() {
                change += remove_unknown_resources(resources, &dir.path(), ctx)?;
            } else {
                if let Some(ext) = dir.path().extension() {
                    if ext != "class" {
                        change += remove_file(resources, &dir)?;
                    }
                } else {
                    change += remove_file(resources, &dir)?;
                }
            }
        }
    }

    Ok(change)
}
fn remove_file(resources: &HashSet<PathBuf>, dir: &DirEntry) -> Result<isize, LazyJavaError> {
    let path = dir.path();

    if !resources.contains(&path) {
        log::info!("removing resource at {}", path.display());
        fs::remove_file(path)?;
        return Ok(1);
    }

    Ok(0)
}
fn build_globset(ctx: &Context) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for rule in &ctx.config.resources.exclude {
        if let Ok(glob_rule) = Glob::new(rule) {
            builder.add(glob_rule);
        } else {
            log::warn!("Invalid glob rule, \"{}\" is not a valid rule", rule);
        }
    }

    return builder.build().unwrap();
}

fn add_resources(
    path: &Path,
    relative: &Path,
    glob_set: &GlobSet,
    ctx: &Context,
) -> Result<HashSet<PathBuf>, LazyJavaError> {
    let mut resources = HashSet::new();
    for dir in fs::read_dir(path)? {
        if let Ok(dir) = dir {
            if is_excluded(&dir, glob_set) {
                continue;
            }

            if dir.file_type()?.is_dir() {
                resources.extend(add_resources(
                    &dir.path(),
                    &relative.join(dir.file_name()),
                    glob_set,
                    ctx,
                )?);
            } else {
                if let Some(ext) = dir.path().extension() {
                    if ext != "java" {
                        resources.insert(copy_file(&dir, relative, ctx)?);
                    }
                } else {
                    resources.insert(copy_file(&dir, relative, ctx)?);
                }
            }
        }
    }

    Ok(resources)
}
fn copy_file(file: &DirEntry, relative: &Path, ctx: &Context) -> Result<PathBuf, LazyJavaError> {
    let dest_path = ctx.bin.join(relative).join(file.file_name());

    if !dest_path.exists() {
        let src = file.path();

        log::info!("Copying {} to {}", src.display(), dest_path.display());
        fs::copy(src, &dest_path)?;
    }
    Ok(dest_path)
}
fn is_excluded(file: &DirEntry, glob_set: &GlobSet) -> bool {
    let path = file.path();
    glob_set.is_match(path) || glob_set.is_match(file.file_name())
}
