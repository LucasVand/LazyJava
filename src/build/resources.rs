use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use crate::{
    Context,
    build::BuildError,
    utils::{IOError, fs},
};
use globset::{Glob, GlobSet, GlobSetBuilder};
use same_file::is_same_file;

pub fn copy_resources(ctx: &Context) -> Result<(), BuildError> {
    let list = if let Some(r) = ctx.config.resources()
        && let Some(l) = r.exclude()
    {
        l
    } else {
        Vec::new()
    };
    let glob_set = build_globset(&list);

    let mut resource_paths = add_resources(&ctx.src, Path::new(""), &glob_set, ctx)?;
    log::info!("Found {} resources", resource_paths.len());

    let external = if let Some(r) = ctx.config.resources()
        && let Some(list) = r.external()
    {
        list
    } else {
        Vec::new()
    };

    if !external.is_empty() {
        let glob_external = build_globset(&external);
        let external_paths = add_external_resources(&ctx.root, &glob_external, ctx)?;
        resource_paths.extend(external_paths);
    }

    let removed = remove_unknown_resources(&resource_paths, &ctx.bin, ctx)?;
    log::info!("Removed {} resources", removed);
    Ok(())
}
fn add_external_resources(
    path: &Path,
    glob_set: &GlobSet,
    ctx: &Context,
) -> Result<HashSet<PathBuf>, BuildError> {
    let mut res = HashSet::new();
    for entry in fs::read_dir(path)
        .map_err(|s| IOError::new("reading project directory", path, s))?
        .flatten()
    {
        let p = entry.path();

        if is_same_file(&p, &ctx.src).unwrap_or(false)
            || is_same_file(&p, &ctx.target).unwrap_or(false)
            || is_same_file(&p, &ctx.bin).unwrap_or(false)
        {
            continue;
        }
        if entry.file_type().is_ok_and(|t| t.is_dir()) {
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }
            res.extend(add_external_resources(&p, glob_set, ctx)?);
        } else {
            let relative = p.strip_prefix(&ctx.root).unwrap_or(&p);
            if glob_set.is_match(relative) {
                res.insert(copy_file(&entry, Path::new("."), ctx)?);
            }
        }
    }

    Ok(res)
}
fn remove_unknown_resources(
    resources: &HashSet<PathBuf>,
    current: &Path,
    ctx: &Context,
) -> Result<isize, BuildError> {
    let mut change = 0;
    for dir in fs::read_dir(current)
        .map_err(|s| IOError::new("reading build directory", current, s))?
        .flatten()
    {
        if dir.file_type().is_ok_and(|t| t.is_dir()) {
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
fn build_globset(list: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for rule in list {
        if let Ok(glob_rule) = Glob::new(rule) {
            builder.add(glob_rule);
        } else {
            log::warn!("Invalid glob rule, \"{}\" is not a valid rule", rule);
        }
    }

    builder.build().unwrap()
}

fn add_resources(
    path: &Path,
    relative: &Path,
    glob_set: &GlobSet,
    ctx: &Context,
) -> Result<HashSet<PathBuf>, BuildError> {
    let mut resources = HashSet::new();
    for dir in fs::read_dir(path)
        .map_err(|e| IOError::new("reading source directory", path, e))?
        .flatten()
    {
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

    Ok(resources)
}
fn copy_file(file: &fs::DirEntry, relative: &Path, ctx: &Context) -> Result<PathBuf, BuildError> {
    let dest_path = ctx.bin.join(relative).join(file.file_name());

    let src = file.path();
    if !dest_path.exists() {
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| IOError::new("creating resource directory", parent, e))?;
        }
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
