use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::{Context, build::BuildError, utils::IOError};

pub fn copy_resources(ctx: &Context) -> Result<(), BuildError> {
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
) -> Result<isize, BuildError> {
    let mut change = 0;
    for dir in fs::read_dir(current).map_err(|s| IOError::new("reading", current, s))? {
        if let Ok(dir) = dir {
            if dir.file_type().unwrap().is_dir() {
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
fn remove_file(resources: &HashSet<PathBuf>, dir: &fs::DirEntry) -> Result<isize, BuildError> {
    let path = dir.path();

    if !resources.contains(&path) {
        log::info!("removing resource at {}", path.display());
        fs::remove_file(&path).map_err(|e| IOError::new("removing resource", &path, e))?;
        return Ok(1);
    }

    Ok(0)
}
fn build_globset(ctx: &Context) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    if let Some(r) = ctx.config.resources()
        && let Some(exclude) = r.exclude()
    {
        for rule in exclude {
            if let Ok(glob_rule) = Glob::new(&rule) {
                builder.add(glob_rule);
            } else {
                log::warn!("Invalid glob rule, \"{}\" is not a valid rule", rule);
            }
        }
    }

    return builder.build().unwrap();
}

fn add_resources(
    path: &Path,
    relative: &Path,
    glob_set: &GlobSet,
    ctx: &Context,
) -> Result<HashSet<PathBuf>, BuildError> {
    let mut resources = HashSet::new();
    for dir in fs::read_dir(path).map_err(|e| IOError::new("reading source directory", path, e))? {
        if let Ok(dir) = dir {
            if is_excluded(&dir, glob_set) {
                continue;
            }

            if dir
                .file_type()
                .map_err(|e| IOError::new("reading source entry", path, e))?
                .is_dir()
            {
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
fn copy_file(file: &fs::DirEntry, relative: &Path, ctx: &Context) -> Result<PathBuf, BuildError> {
    let dest_path = ctx.bin.join(relative).join(file.file_name());

    let src = file.path();
    if !dest_path.exists() {
        log::info!("Copying {} to {}", src.display(), dest_path.display());
        fs::copy(&src, &dest_path).map_err(|e| IOError::new("copying resource", &dest_path, e))?;
    } else {
        let meta_dest = fs::metadata(&dest_path)
            .map_err(|e| IOError::new("reading resource metadata", &dest_path, e))?;
        let meta_src =
            fs::metadata(&src).map_err(|e| IOError::new("reading resource metadata", &src, e))?;

        if meta_src
            .modified()
            .map_err(|e| IOError::new("reading resource metadata", &src, e))?
            > meta_dest
                .modified()
                .map_err(|e| IOError::new("reading resource metadata", &dest_path, e))?
        {
            fs::copy(&src, &dest_path)
                .map_err(|e| IOError::new("copying resource", &dest_path, e))?;
        }
    }
    Ok(dest_path)
}
fn is_excluded(file: &fs::DirEntry, glob_set: &GlobSet) -> bool {
    let path = file.path();
    glob_set.is_match(path) || glob_set.is_match(file.file_name())
}
